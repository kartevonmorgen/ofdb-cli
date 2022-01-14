use crate::import::*;
use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use ofdb_boundary::{Entry, NewPlace, UpdatePlace};
use ofdb_cli::*;
use reqwest::blocking::Client;
use std::{
    env,
    fs::File,
    io,
    path::{Path, PathBuf},
    str::FromStr,
};
use uuid::Uuid;

#[derive(Parser)]
#[clap(name = "ofdb", about = "CLI for OpenFairDB", author)]
struct Cli {
    #[clap(flatten)]
    opt: Opt,
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Args)]
struct Opt {
    #[clap(long = "api-url", help = "The URL of the JSON API")]
    api: String,
}

#[derive(Subcommand)]
enum SubCommand {
    #[clap(about = "Import new entries")]
    Import {
        #[clap(parse(from_os_str), help = "JSON or CSV file with entries")]
        file: PathBuf,
        #[clap(
            parse(from_os_str),
            long = "report-file",
            help = "File with the import report",
            default_value = "import-report.json"
        )]
        report_file: PathBuf,
        #[clap(long = "opencage-api-key", help = "OpenCage API key")]
        opencage_api_key: Option<String>,
    },
    #[clap(about = "Read entry")]
    Read {
        #[clap(required = true, min_values = 1, help = "UUID")]
        uuids: Vec<Uuid>,
    },
    #[clap(about = "Update entries")]
    Update {
        #[clap(parse(from_os_str), help = "JSON file")]
        file: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FileType {
    Json,
    Csv,
}

impl FromStr for FileType {
    type Err = anyhow::Error;
    fn from_str(t: &str) -> Result<Self, Self::Err> {
        match &*t.to_lowercase() {
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            _ => Err(anyhow::anyhow!("Unsupported file type")),
        }
    }
}

fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    let args = Cli::parse();
    match args.cmd {
        SubCommand::Import {
            file,
            report_file,
            opencage_api_key,
        } => import(&args.opt.api, file, report_file, opencage_api_key),
        SubCommand::Read { uuids } => read(&args.opt.api, uuids),
        SubCommand::Update { file } => update(&args.opt.api, file),
    }
}

fn read(api: &str, uuids: Vec<Uuid>) -> Result<()> {
    let client = new_client()?;
    let entries = read_entries(api, &client, uuids)?;
    println!("{}", serde_json::to_string(&entries)?);
    Ok(())
}

fn update(api: &str, path: PathBuf) -> Result<()> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let places: Vec<Entry> = serde_json::from_reader(reader)?;
    log::debug!("Read {} places from JSON file", places.len());
    let client = new_client()?;
    for entry in places {
        let id = entry.id.clone();
        let update = UpdatePlace::from(entry);
        match update_place(api, &client, &id, &update) {
            Ok(updated_id) => {
                debug_assert!(updated_id == id);
                log::debug!("Successfully updated '{}' with ID={}", update.title, id);
            }
            Err(err) => {
                log::warn!("Could not update '{}': {}", update.title, err);
            }
        }
    }
    Ok(())
}

fn import(
    api: &str,
    path: PathBuf,
    report_file_path: PathBuf,
    opencage_api_key: Option<String>,
) -> Result<()> {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| anyhow::anyhow!("Unsupported file extension"))?;
    let file_type = ext.parse()?;
    log::info!(
        "Import entries from file ({}): {}",
        format!("{:?}", file_type).to_uppercase(),
        path.display()
    );
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let places = match file_type {
        FileType::Json => {
            let places: Vec<NewPlace> = serde_json::from_reader(reader)?;
            log::debug!("Import {} places from JSON file", places.len());
            places
        }
        FileType::Csv => {
            let csv_results = csv::from_reader(reader, opencage_api_key)?;
            if csv_results.iter().any(|r| r.result.is_err()) {
                let report = Report::from(csv_results);
                log::warn!(
                    "{} csv records contain errors ",
                    report.csv_import_failures.len()
                );
                write_import_report(report, report_file_path)?;
                return Ok(());
            } else {
                let places: Vec<NewPlace> =
                    csv_results.into_iter().map(|r| r.result.unwrap()).collect();
                log::debug!("Import {} places from CSV file", places.len());
                places
            }
        }
    };
    let client = new_client()?;
    let mut results = vec![];
    for (i, new_place) in places.iter().enumerate() {
        let import_id = Some(i.to_string());
        if let Some(possible_duplicates) = search_duplicates(api, &client, new_place)? {
            log::warn!(
                "Found {} possible duplicates for '{}':",
                possible_duplicates.len(),
                new_place.title
            );
            for p in &possible_duplicates {
                log::warn!(" - {} (id: {})", p.title, p.id);
            }
            results.push(ImportResult {
                new_place,
                import_id,
                result: Err(Error::Duplicates(possible_duplicates)),
            });
        } else {
            match create_new_place(api, &client, new_place) {
                Ok(id) => {
                    log::debug!("Successfully imported '{}' with ID={}", new_place.title, id);
                    results.push(ImportResult {
                        new_place,
                        import_id,
                        result: Ok(id),
                    });
                }
                Err(err) => {
                    log::warn!("Could not import '{}': {}", new_place.title, err);
                    results.push(ImportResult {
                        new_place,
                        import_id,
                        result: Err(Error::Other(err.to_string())),
                    });
                }
            }
        }
    }
    let report = Report::from(results);
    if !report.successes.is_empty() {
        log::info!("Successfully imported {} places", report.successes.len());
    }
    if !report.duplicates.is_empty() {
        log::warn!(
            "Found {} places with possible duplicates",
            report.duplicates.len()
        );
    }
    if !report.failures.is_empty() {
        log::warn!("{} places contain errors ", report.failures.len());
    }
    write_import_report(report, report_file_path)?;
    Ok(())
}

fn write_import_report<P: AsRef<Path>>(report: Report, path: P) -> Result<()> {
    let file = File::create(path)?;
    let writer = io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &report)?;
    Ok(())
}

fn new_client() -> Result<Client> {
    let client = Client::builder()
        // Disable idle pool:
        // see https://github.com/hyperium/hyper/issues/2136#issuecomment-861826148
        .pool_max_idle_per_host(0)
        .build()?;
    Ok(client)
}
