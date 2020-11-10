use anyhow::Result;
use ofdb_boundary::{Entry, UpdatePlace};
use ofdb_cli::*;
use std::{fs::File, io, path::PathBuf};
use structopt::StructOpt;
use uuid::Uuid;

mod import;
use self::import::*;

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
