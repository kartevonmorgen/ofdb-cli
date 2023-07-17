use crate::import::{CsvImportError, CsvImportResult};
use anyhow::Result;
use csv::ReaderBuilder;
use ofdb_boundary::{Address, CustomLink, Entry, NewPlace, Review, ReviewStatus};
use ofdb_core::gateways::geocode::GeoCodingGateway;
use ofdb_gateways::opencage::*;
use serde::Deserialize;
use std::io::Read;
use time::Date;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct NewPlaceRecord {
    title: String,
    description: String,
    lat: Option<f64>,
    lng: Option<f64>,
    street: Option<String>,
    zip: Option<String>,
    city: Option<String>,
    country: Option<String>,
    state: Option<String>,
    contact_name: Option<String>,
    contact_email: Option<String>,
    contact_phone: Option<String>,
    opening_hours: Option<String>,
    founded_on: Option<Date>,
    tags: String,
    homepage: Option<String>,
    license: String,
    image_url: Option<String>,
    image_link_url: Option<String>,
}

pub fn new_places_from_reader<R: Read>(
    r: R,
    opencage_api_key: Option<String>,
) -> Result<Vec<CsvImportResult<NewPlace>>> {
    log::info!("Read entries form CSV");
    let mut rdr = ReaderBuilder::new().from_reader(r);

    if opencage_api_key.is_none() {
        log::warn!("No OpenCage API provided");
    }

    let geo_coding = OpenCage::new(opencage_api_key);

    let mut results = vec![];

    for (record_nr, result) in rdr.deserialize().enumerate() {
        match result {
            Err(err) => {
                results.push(CsvImportResult {
                    record_nr,
                    result: Err(CsvImportError::InvalidRecord(err.to_string())),
                });
            }
            Ok(r) => {
                let NewPlaceRecord {
                    title,
                    street,
                    zip,
                    city,
                    country,
                    state,
                    lat,
                    lng,
                    ..
                } = r;

                log::info!(
                    "Check address and geo location for entry '{}' ({:?})",
                    title,
                    city
                );
                let addr = Address {
                    street,
                    zip,
                    city,
                    country,
                    state,
                };
                match check_address_and_geo_coordinates(&geo_coding, addr, lat, lng) {
                    Ok((addr, (lat, lng))) => {
                        let new_place = NewPlace {
                            title,
                            description: r.description,
                            lat,
                            lng,
                            city: addr.city,
                            country: addr.country,
                            state: addr.state,
                            street: addr.street,
                            zip: addr.zip,
                            contact_name: r.contact_name,
                            email: r.contact_email,
                            founded_on: r.founded_on,
                            homepage: r.homepage,
                            categories: vec![],
                            license: r.license,
                            links: vec![],
                            opening_hours: r.opening_hours,
                            tags: r.tags.split(',').map(ToString::to_string).collect(),
                            telephone: r.contact_phone,
                            image_url: r.image_url,
                            image_link_url: r.image_link_url,
                        };
                        results.push(CsvImportResult {
                            record_nr,
                            result: Ok(new_place),
                        });
                    }
                    Err(err) => {
                        results.push(CsvImportResult {
                            record_nr,
                            result: Err(CsvImportError::InvalidAddressOrGeoCoordinates(
                                err.to_string(),
                            )),
                        });
                    }
                }
            }
        }
    }
    Ok(results)
}

#[derive(Debug, Deserialize)]
struct PlaceRecord {
    id: String,
    created: i64,
    version: u64,
    title: String,
    description: String,
    lat: f64,
    lng: f64,
    street: Option<String>,
    zip: Option<String>,
    city: Option<String>,
    country: Option<String>,
    state: Option<String>,
    contact_name: Option<String>,
    contact_email: Option<String>,
    contact_phone: Option<String>,
    opening_hours: Option<String>,
    founded_on: Option<Date>,
    tags: String,
    ratings: Vec<String>,
    homepage: Option<String>,
    license: String,
    image_url: Option<String>,
    image_link_url: Option<String>,
    custom_link_title_0: Option<String>,
    custom_link_title_1: Option<String>,
    custom_link_title_2: Option<String>,
    custom_link_title_3: Option<String>,
    custom_link_title_4: Option<String>,
    custom_link_description_0: Option<String>,
    custom_link_description_1: Option<String>,
    custom_link_description_2: Option<String>,
    custom_link_description_3: Option<String>,
    custom_link_description_4: Option<String>,
    custom_link_url_0: Option<String>,
    custom_link_url_1: Option<String>,
    custom_link_url_2: Option<String>,
    custom_link_url_3: Option<String>,
    custom_link_url_4: Option<String>,
}

pub fn places_from_reader<R: Read>(r: R) -> Result<Vec<CsvImportResult<Entry>>> {
    log::info!("Read entries form CSV");
    let mut rdr = ReaderBuilder::new().from_reader(r);
    let mut results = vec![];

    for (record_nr, result) in rdr.deserialize().enumerate() {
        match result {
            Err(err) => {
                log::warn!("Invalid CSV entry: {err}");
                results.push(CsvImportResult {
                    record_nr,
                    result: Err(CsvImportError::InvalidRecord(err.to_string())),
                });
            }
            Ok(r) => {
                let PlaceRecord {
                    id,
                    created,
                    version,
                    title,
                    description,
                    lat,
                    lng,
                    street,
                    zip,
                    city,
                    country,
                    state,
                    contact_name,
                    homepage,
                    opening_hours,
                    founded_on,
                    image_url,
                    image_link_url,
                    ratings,
                    custom_link_title_0,
                    custom_link_title_1,
                    custom_link_title_2,
                    custom_link_title_3,
                    custom_link_title_4,
                    custom_link_description_0,
                    custom_link_description_1,
                    custom_link_description_2,
                    custom_link_description_3,
                    custom_link_description_4,
                    custom_link_url_0,
                    custom_link_url_1,
                    custom_link_url_2,
                    custom_link_url_3,
                    custom_link_url_4,
                    ..
                } = r;

                let license = Some(r.license);
                let categories = vec![];
                let telephone = r.contact_phone;
                let email = r.contact_email;
                let tags = r.tags.split(',').map(ToString::to_string).collect();

                let custom_links = vec![
                    construct_custom_link(
                        custom_link_url_0,
                        custom_link_title_0,
                        custom_link_description_0,
                    ),
                    construct_custom_link(
                        custom_link_url_1,
                        custom_link_title_1,
                        custom_link_description_1,
                    ),
                    construct_custom_link(
                        custom_link_url_2,
                        custom_link_title_2,
                        custom_link_description_2,
                    ),
                    construct_custom_link(
                        custom_link_url_3,
                        custom_link_title_3,
                        custom_link_description_3,
                    ),
                    construct_custom_link(
                        custom_link_url_4,
                        custom_link_title_4,
                        custom_link_description_4,
                    ),
                ]
                .into_iter()
                .flatten()
                .collect();

                let place = Entry {
                    id,
                    created,
                    version,
                    title,
                    description,
                    lat,
                    lng,
                    city,
                    country,
                    state,
                    street,
                    zip,
                    contact_name,
                    email,
                    founded_on,
                    homepage,
                    categories,
                    license,
                    custom_links,
                    opening_hours,
                    tags,
                    telephone,
                    image_url,
                    image_link_url,
                    ratings,
                };
                results.push(CsvImportResult {
                    record_nr,
                    result: Ok(place),
                });
            }
        }
    }
    Ok(results)
}

fn construct_custom_link(
    url: Option<String>,
    title: Option<String>,
    description: Option<String>,
) -> Option<CustomLink> {
    url.map(|url| CustomLink {
        url,
        title,
        description,
    })
}

fn check_address_and_geo_coordinates(
    geo_coding: &dyn GeoCodingGateway,
    addr: Address,
    lat: Option<f64>,
    lng: Option<f64>,
) -> Result<(Address, (f64, f64))> {
    use ofdb_entities::address;

    match (addr.is_empty(), lat.zip(lng)) {
        (false, None) => {
            let addr = address::Address::from(addr);
            log::info!("Try to resolve lat/lang from address ({:?})", addr);
            match geo_coding.resolve_address_lat_lng(&addr) {
                Some((lat, lng)) => Ok((Address::from(addr), (lat, lng))),
                None => Err(anyhow::anyhow!("Unable to find geo coordinates")),
            }
        }
        (true, Some(coordinates)) => {
            log::warn!("Found entry without address");
            // TODO: look up address
            Ok((addr, coordinates))
        }
        (false, Some(coordinates)) => {
            // nothing to to
            Ok((addr, coordinates))
        }
        (true, None) => Err(anyhow::anyhow!(
            "An address or geo coordinates (lat/lng) are required"
        )),
    }
}

#[derive(Debug, Deserialize)]
struct ReviewRecord {
    id: String,
    status: String,
    comment: Option<String>,
}

pub fn reviews_from_reader<R: Read>(r: R) -> Result<Vec<(Uuid, Review)>> {
    log::info!("Read reviews form CSV");
    let mut rdr = ReaderBuilder::new().from_reader(r);
    let mut results = vec![];

    for (record_nr, result) in rdr.deserialize().enumerate() {
        match result {
            Err(err) => {
                log::warn!("Unable to read record nr {record_nr}): {}", err);
                continue;
            }
            Ok(r) => {
                let ReviewRecord {
                    id,
                    status,
                    comment,
                } = r;
                if let Ok(id) = id.parse::<Uuid>() {
                    let status = match &*status.trim().to_lowercase() {
                        "archived" => ReviewStatus::Archived,
                        "confirmed" => ReviewStatus::Confirmed,
                        "created" => ReviewStatus::Created,
                        "rejected" => ReviewStatus::Rejected,
                        _ => {
                            log::warn!("Invalid status '{status}' in record {record_nr}");
                            continue;
                        }
                    };
                    let review = Review { status, comment };
                    results.push((id, review));
                } else {
                    log::warn!("Invalid ID '{}' in record {record_nr})", id);
                    continue;
                }
            }
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn read_reviews_from_csv_file() {
        let file = File::open("tests/review-example.csv").unwrap();
        let reviews = reviews_from_reader(file).unwrap();
        assert_eq!(reviews.len(), 3);
    }

    #[test]
    fn read_places_from_csv_file() {
        let file = File::open("tests/import-example.csv").unwrap();
        let import = new_places_from_reader(file, None).unwrap();
        assert_eq!(import.len(), 1);
        let new_place = import[0].result.as_ref().unwrap();
        assert_eq!(new_place.title, "GLS Bank");
        assert_eq!(new_place.tags, vec!["bank", "geld", "commercial"]);
    }

    #[test]
    fn read_updates_from_csv_file() {
        let file = File::open("tests/update-example.csv").unwrap();
        let updates = places_from_reader(file).unwrap();
        assert!(updates[0].result.is_ok());
    }
}
