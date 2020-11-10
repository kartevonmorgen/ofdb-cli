use anyhow::Result;
use csv::ReaderBuilder;
use ofdb_boundary::{CustomLink, UpdatePlace};
use ofdb_cli as ofdb;
use serde::Deserialize;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Record {
    url: Url,
    kvm_id: Uuid,
}

pub fn main() -> Result<()> {
    env_logger::init();

    let title = "Profil im Werkzeugkasten des Wandels";

    let api = "https://dev.ofdb.io/v0/";
    let client = reqwest::blocking::Client::new();

    let mut rdr = ReaderBuilder::new().from_path("./urls.csv")?;

    for result in rdr.deserialize() {
        let record: Record = result?;
        let entry = ofdb::read_entries(api, &client, vec![record.kvm_id])?[0].clone();
        let id = entry.id.clone();
        let mut update = UpdatePlace::from(entry);
        if update.links.is_empty()
            || !update
                .links
                .iter()
                .any(|l| l.url.parse::<Url>().unwrap() == record.url)
        {
            update.links.push(CustomLink {
                url: record.url.to_string(),
                title: Some(title.to_string()),
                description: None,
            });
            match ofdb::update_place(api, &client, &id, &update) {
                Ok(_) => {
                    log::info!("Successfully updated entry '{}'.", update.title);
                }
                Err(err) => {
                    log::error!("Could not update entry '{}': {}", update.title, err);
                }
            }
        } else {
            log::warn!("Entry '{}' is already up to date", update.title);
        }
    }
    Ok(())
}
