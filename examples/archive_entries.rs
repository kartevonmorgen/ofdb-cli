use anyhow::Result;
use csv::ReaderBuilder;
use ofdb_boundary as json;
use ofdb_cli as ofdb;

pub fn main() -> Result<()> {
    env_logger::init();
    let api = "https://dev.ofdb.io/v0/";
    let client = reqwest::blocking::ClientBuilder::new()
        .cookie_store(true)
        .build()?;
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 4 {
        println!("Usage: archive_entries <email> <password> <csv-file>");
        return Ok(());
    }
    let email = args[1].to_string();
    let password = args[2].to_string();
    let mut rdr = ReaderBuilder::new().from_path(&args[3])?;
    ofdb::login(api, &client, &json::Credentials { email, password })?;
    let mut uuids = vec![];
    for result in rdr.records() {
        let record = result?;
        let uuid = record[0].parse()?;
        uuids.push(uuid);
    }
    let comment = Some("Pleite".to_string());
    let status = json::ReviewStatus::Archived;
    match ofdb::review_places(api, &client, uuids, json::Review { status, comment }) {
        Ok(_) => {
            log::info!("Successfully archived entries");
        }
        Err(err) => {
            log::warn!("Could archive entries: {}", err);
        }
    }
    Ok(())
}
