use anyhow::Result;
use ofdb_boundary::{Entry, NewPlace, PlaceSearchResult};
use reqwest::blocking::Client;
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
    log::debug!("Read {} places", uuids.len());
    let uuids = uuids
        .into_iter()
        .map(Uuid::to_simple)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let url = format!("{}/entries/{}", api, uuids);
    let client = reqwest::blocking::Client::new();
    let res = client.get(&url).send()?;

    // assert the response can be deserialized
    let res: Vec<Entry> = res.json()?;

    // and serialize it again ;-)
    println!("{}", serde_json::to_string(&res)?);

    Ok(())
}

fn update(api: &str, path: PathBuf) -> Result<()> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let places: Vec<Entry> = serde_json::from_reader(reader)?;
    log::debug!("Read {} places from JSON file", places.len());
    let client = reqwest::blocking::Client::new();
    for entry in &places {
        match update_place(api, &client, &entry) {
            Ok(id) => {
                debug_assert!(id == entry.id);
                log::debug!("Successfully updated '{}' with ID={}", entry.title, id);
            }
            Err(err) => {
                log::warn!("Could not update '{}': {}", entry.title, err);
            }
        }
    }
    Ok(())
}

fn search_duplicates(
    api: &str,
    client: &Client,
    new_place: &NewPlace,
) -> Result<Option<Vec<PlaceSearchResult>>> {
    let url = format!("{}/search/duplicates", api);
    let res = client.post(&url).json(&new_place).send()?;
    let res: Vec<PlaceSearchResult> = res.json()?;
    Ok(if res.is_empty() { None } else { Some(res) })
}

fn create_new_place(api: &str, client: &Client, new_place: &NewPlace) -> Result<String> {
    let url = format!("{}/entries", api);
    let res = client.post(&url).json(&new_place).send()?;
    Ok(res.json()?)
}

fn update_place(api: &str, client: &Client, entry: &Entry) -> Result<String> {
    let mut entry = entry.clone();
    entry.version += 1;
    let url = format!("{}/entries/{}", api, entry.id);
    let res = client.put(&url).json(&entry).send()?;
    Ok(res.json()?)
}
