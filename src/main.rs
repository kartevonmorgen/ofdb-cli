use anyhow::Result;
use ofdb_boundary::{NewPlace, PlaceSearchResult};
use reqwest::blocking::Client;
use std::{fs::File, io, path::PathBuf};
use structopt::StructOpt;

const API_URL: &str = "https://api.ofdb.io/v0";

#[derive(Debug, StructOpt)]
#[structopt(name = "ofdb", about = "CLI for OpenFairDB", author)]
enum Opt {
    #[structopt(about = "Import new entries")]
    Import {
        #[structopt(parse(from_os_str), help = "JSON file")]
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::from_args();
    match opt {
        Opt::Import { file } => import(file),
    }
}

fn import(path: PathBuf) -> Result<()> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let places: Vec<NewPlace> = serde_json::from_reader(reader)?;
    log::debug!("Read {} places from JSON file", places.len());
    let client = reqwest::blocking::Client::new();
    for new_place in &places {
        if let Some(possible_duplicates) = search_duplicates(&client, new_place)? {
            println!(
                "Found {} possible duplicates for '{}':",
                possible_duplicates.len(),
                new_place.title
            );
            for p in possible_duplicates {
                println!(" - {} (id: {})", p.title, p.id);
            }
        }
    }
    println!("The import feature is not fully implemented yet"); // TODO
    Ok(())
}

fn search_duplicates(
    client: &Client,
    new_place: &NewPlace,
) -> Result<Option<Vec<PlaceSearchResult>>> {
    let url = format!("{}/search/duplicates", API_URL);
    let res = client.post(&url).json(&new_place).send()?;
    let res: Vec<PlaceSearchResult> = res.json()?;
    Ok(if res.is_empty() { None } else { Some(res) })
}
