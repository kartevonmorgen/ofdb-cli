use anyhow::Result;
use csv::ReaderBuilder;
use ofdb_boundary as json;
use ofdb_cli as ofdb;
use ofdb_core::gateways::geocode::GeoCodingGateway;
use ofdb_entities::{address, geo};
use ofdb_gateways::opencage::*;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct Record {
    ID: usize,
    Titel: String,
    Beschreibung: String,
    Hashtags: String,
    Link: String,
    Ort: String,
    Herausgeber: String,
}

pub fn main() -> Result<()> {
    env_logger::init();

    let geo_coding = {
        let key = match env::var("OPENCAGE_API_KEY") {
            Ok(key) => Some(key),
            Err(_) => {
                log::warn!("No OpenCage API key found");
                None
            }
        };
        OpenCage::new(key)
    };

    let bbox_germany = geo::MapBbox::new(
        geo::MapPoint::new(
            geo::LatCoord::from_deg(47.15),
            geo::LngCoord::from_deg(5.72),
        ),
        geo::MapPoint::new(
            geo::LatCoord::from_deg(55.11),
            geo::LngCoord::from_deg(15.09),
        ),
    );

    let api = "https://dev.ofdb.io/v0/";
    let client = reqwest::blocking::Client::new();

    let mut rdr = ReaderBuilder::new().from_path("./entries.csv")?;

    let mut new_places = vec![];
    let mut no_new_places = vec![];
    let mut results = vec![];

    for result in rdr.deserialize() {
        let record: Record = result?;

        let city = record
            .Ort
            .replace("\n", " ")
            .replace("  ", " ")
            .trim()
            .to_string();
        let title = record
            .Titel
            .replace("\n", " ")
            .replace("  ", " ")
            .trim()
            .to_string();

        log::info!("Try to find geo location for entry '{}' ({})", title, city);

        let mut addr = address::Address::default();
        addr.country = Some("Deutschland".into());
        addr.city = Some(city.clone());

        match geo_coding.resolve_address_lat_lng(&addr) {
            Some((lat, lng)) => {
                let center =
                    geo::MapPoint::new(geo::LatCoord::from_deg(lat), geo::LngCoord::from_deg(lng));
                if !bbox_germany.contains_point(center) {
                    log::warn!("Entry '{}' ({}) is not in Germany", title, city);
                } else {
                    let new_place = json::NewPlace {
                        title: title.clone(),
                        description: record.Beschreibung,
                        lat,
                        lng,
                        city: addr.city,
                        country: addr.country,
                        state: None,
                        street: None,
                        zip: None,
                        contact_name: Some(record.Herausgeber),
                        email: None,
                        founded_on: None,
                        homepage: Some(record.Link),
                        image_link_url: None,
                        categories: vec![],
                        license: "CC0-1.0".to_string(),
                        links: vec![],
                        opening_hours: None,
                        tags: record
                            .Hashtags
                            .split(',')
                            .map(ToString::to_string)
                            .collect(),
                        telephone: None,
                        image_url: None,
                    };

                    // Workaround:
                    // Because the duplicates API has no search distance yet we do a usual search
                    // and look for title equality.
                    // TODO: either expose the duplicate checking algorithm in or extend the API
                    let search_distance = geo::Distance::from_meters(50_000.0);
                    let search_bbox =
                        geo::MapBbox::centered_around(center, search_distance, search_distance);
                    let json_bbox = json::MapBbox::from(search_bbox);
                    let entries = ofdb::search(api, &client, &title, &json_bbox)?;
                    if let Some(e) = entries.visible.into_iter().find(|e| e.title == title) {
                        log::warn!(
                            "Entry '{}' ({}) with import ID = {} already exists: UUID = {}",
                            title,
                            city,
                            record.ID,
                            e.id
                        );
                        no_new_places.push((record.ID, new_place, vec![e]));
                    } else {
                        new_places.push((record.ID, new_place));
                    }
                }
            }
            None => {
                log::warn!("Could not find geo location for '{}'", record.Ort);
            }
        }
    }
    for (record_id, new_place, dups) in &no_new_places {
        results.push(ofdb::import::ImportResult {
            new_place,
            import_id: Some(record_id.to_string()),
            result: Err(ofdb::import::Error::Duplicates(dups.to_vec())),
        });
    }
    for (record_id, p) in &new_places {
        log::debug!(
            "Try to create a new entry '{}' ({:?}) from import ID = {}",
            p.title,
            p.city,
            record_id
        );
        let import_id = Some(record_id.to_string());
        match ofdb::create_new_place(api, &client, &p) {
            Ok(id) => {
                log::info!("Successfully imported '{}' with ID={}", p.title, id);
                results.push(ofdb::import::ImportResult {
                    new_place: &p,
                    import_id,
                    result: Ok(id),
                });
            }
            Err(err) => {
                log::warn!("Could not import '{}': {}", p.title, err);
                results.push(ofdb::import::ImportResult {
                    new_place: &p,
                    import_id,
                    result: Err(ofdb::import::Error::Other(err.to_string())),
                });
            }
        }
    }
    let report = ofdb::import::Report::from(results);
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
