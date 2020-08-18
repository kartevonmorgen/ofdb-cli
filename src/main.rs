use anyhow::Result;
use ofdb_boundary::{NewPlace, PlaceSearchResult};
use reqwest::blocking::Client;
use std::{collections::HashMap, fs::File, io, path::PathBuf};
use structopt::StructOpt;

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
}

fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    match opt.cmd {
        SubCommand::Import { file } => import(&opt.api, file),
    }
}

fn import(api: &str, path: PathBuf) -> Result<()> {
    let err_file_name = path.with_file_name("_place_imports_with_errors.json");
    let ok_file_name = path.with_file_name("_place_imports_with_success.json");
    let duplicates_file_name = path.with_file_name("_place_imports_with_duplicates.json");

    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let places: Vec<NewPlace> = serde_json::from_reader(reader)?;
    log::debug!("Read {} places from JSON file", places.len());
    let client = reqwest::blocking::Client::new();
    let mut places_with_duplicates = vec![];
    let mut places_with_errors = vec![];
    let mut places_with_success = HashMap::new();
    for new_place in &places {
        if let Some(possible_duplicates) = search_duplicates(api, &client, new_place)? {
            log::warn!(
                "Found {} possible duplicates for '{}':",
                possible_duplicates.len(),
                new_place.title
            );
            for p in possible_duplicates {
                log::warn!(" - {} (id: {})", p.title, p.id);
            }
            places_with_duplicates.push(new_place);
        } else {
            match create_new_place(api, &client, new_place) {
                Ok(id) => {
                    log::debug!("Successfully imported '{}' with ID={}", new_place.title, id);
                    places_with_success.insert(id, new_place);
                }
                Err(err) => {
                    log::warn!("Could not import '{}': {}", new_place.title, err);
                    places_with_errors.push(new_place);
                }
            }
        }
    }
    if !places_with_success.is_empty() {
        let file = File::create(ok_file_name)?;
        serde_json::to_writer(file, &places_with_success)?;
        log::info!("Successfully imported {} places", places_with_success.len());
    }
    if !places_with_duplicates.is_empty() {
        let file = File::create(duplicates_file_name)?;
        serde_json::to_writer(file, &places_with_duplicates)?;
        log::warn!(
            "Found {} places with possible duplicates",
            places_with_duplicates.len()
        );
    }
    if !places_with_errors.is_empty() {
        let file = File::create(err_file_name)?;
        serde_json::to_writer(file, &places_with_errors)?;
        log::warn!("{} places contain errors ", places_with_errors.len());
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
