use crate::import::*;
use anyhow::Result;
use ofdb_boundary::{Entry, NewPlace, UpdatePlace};
use ofdb_cli::*;
use std::{fs::File, io, path::PathBuf};
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Debug, StructOpt)]
#[structopt(name = "ofdb", about = "CLI for OpenFairDB", author)]
struct Opt {
    #[structopt(long = "api-url", help = "The URL of the JSON API")]
    api: String,
    #[structopt(subcommand)]
    cmd: SubCommand,
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    #[structopt(about = "Import new entries")]
    Import {
        #[structopt(parse(from_os_str), help = "JSON file")]
        file: PathBuf,
    },
    #[structopt(about = "Read entry")]
    Read {
        #[structopt(required = true, min_values = 1, help = "UUID")]
        uuids: Vec<Uuid>,
    },
    #[structopt(about = "Update entries")]
    Update {
        #[structopt(parse(from_os_str), help = "JSON file")]
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    match opt.cmd {
        SubCommand::Import { file } => import(&opt.api, file),
        SubCommand::Read { uuids } => read(&opt.api, uuids),
        SubCommand::Update { file } => update(&opt.api, file),
    }
}

fn read(api: &str, uuids: Vec<Uuid>) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let entries = read_entries(api, &client, uuids)?;
    println!("{}", serde_json::to_string(&entries)?);
    Ok(())
}

fn update(api: &str, path: PathBuf) -> Result<()> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let places: Vec<Entry> = serde_json::from_reader(reader)?;
    log::debug!("Read {} places from JSON file", places.len());
    let client = reqwest::blocking::Client::new();
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

fn import(api: &str, path: PathBuf) -> Result<()> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let places: Vec<NewPlace> = serde_json::from_reader(reader)?;
    log::debug!("Read {} places from JSON file", places.len());
    let client = reqwest::blocking::Client::new();
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
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}
