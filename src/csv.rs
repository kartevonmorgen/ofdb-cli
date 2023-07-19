use std::io::Read;

use anyhow::{anyhow, Result};
use csv::ReaderBuilder;
use serde::Deserialize;
use thiserror::Error;
use time::Date;
use uuid::Uuid;

use ofdb_boundary::{Address, CustomLink, Entry, NewPlace, Review, ReviewStatus};
use ofdb_core::gateways::geocode::GeoCodingGateway;
use ofdb_gateways::opencage::*;

use crate::{
    import::{CsvImportError, CsvImportResult},
    read_entries, Client,
};

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
                    result: Err(CsvImportError::Record(err.to_string())),
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
                            result: Err(CsvImportError::AddressOrGeoCoordinates(err.to_string())),
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
    custom_link_title_5: Option<String>,
    custom_link_description_0: Option<String>,
    custom_link_description_1: Option<String>,
    custom_link_description_2: Option<String>,
    custom_link_description_3: Option<String>,
    custom_link_description_4: Option<String>,
    custom_link_description_5: Option<String>,
    custom_link_url_0: Option<String>,
    custom_link_url_1: Option<String>,
    custom_link_url_2: Option<String>,
    custom_link_url_3: Option<String>,
    custom_link_url_4: Option<String>,
    custom_link_url_5: Option<String>,
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
                    result: Err(CsvImportError::Record(err.to_string())),
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
                    custom_link_title_5,
                    custom_link_description_0,
                    custom_link_description_1,
                    custom_link_description_2,
                    custom_link_description_3,
                    custom_link_description_4,
                    custom_link_description_5,
                    custom_link_url_0,
                    custom_link_url_1,
                    custom_link_url_2,
                    custom_link_url_3,
                    custom_link_url_4,
                    custom_link_url_5,
                    ..
                } = r;

                let license = Some(r.license);
                let categories = vec![];
                let telephone = r.contact_phone;
                let email = r.contact_email;
                let tags = r.tags.split(',').map(ToString::to_string).collect();

                if custom_link_url_5.is_some()
                    || custom_link_title_5.is_some()
                    || custom_link_description_5.is_some()
                {
                    log::warn!("At the moment a max. of 5 custom links are supported!");
                }

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

pub fn patch_places_with_reader<R: Read>(
    r: R,
    api: &str,
    client: &Client,
) -> Result<Vec<CsvImportResult<Entry>>> {
    log::info!("Read entries form CSV");
    let mut rdr = ReaderBuilder::new().from_reader(r);
    let mut results = vec![];

    let mut patch_place_records = vec![];

    for (record_nr, result) in rdr.deserialize::<PatchPlaceRecord>().enumerate() {
        match result {
            Err(err) => {
                log::warn!("Invalid CSV entry: {err}");
                results.push(CsvImportResult {
                    record_nr,
                    result: Err(CsvImportError::Record(err.to_string())),
                });
            }
            Ok(record) => match record.id.parse::<Uuid>() {
                Ok(uuid) => {
                    patch_place_records.push((uuid, record_nr, record));
                }
                Err(err) => {
                    let err_msg = format!("Invalid entry ID: {err}");
                    results.push(CsvImportResult {
                        record_nr,
                        result: Err(CsvImportError::Record(err_msg)),
                    });
                }
            },
        }
    }
    let uuids: Vec<_> = patch_place_records
        .iter()
        .map(|(uuid, _, _)| *uuid)
        .collect();
    let mut original_entries = read_entries(api, client, uuids)?;

    for (_, record_nr, record) in patch_place_records {
        let index = original_entries
            .iter()
            .position(|x| x.id == record.id)
            .unwrap();
        let original = original_entries.remove(index);
        match patch_place(original, record) {
            Ok(place) => {
                results.push(CsvImportResult {
                    record_nr,
                    result: Ok(place),
                });
            }
            Err(err) => {
                results.push(CsvImportResult {
                    record_nr,
                    result: Err(CsvImportError::PatchRequest(err.to_string())),
                });
            }
        }
    }
    Ok(results)
}

const OP_APPEND: &str = "++";
const OP_DELETE: &str = "--";
const OP_REPLACE: &str = "==";

const APPEND_SEPERATOR: &str = " ";

fn patch_place(mut original: Entry, record: PatchPlaceRecord) -> Result<Entry> {
    let PatchPlaceRecord {
        id,
        created,
        version,
        license,
        ratings,
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
        contact_email,
        contact_phone,
        tags,
        homepage,
        opening_hours,
        founded_on,
        image_url,
        image_link_url,
        // TODO custom_link_title_0,
        // TODO custom_link_title_1,
        // TODO custom_link_title_2,
        // TODO custom_link_title_3,
        // TODO custom_link_title_4,
        // TODO custom_link_title_5,
        // TODO custom_link_description_0,
        // TODO custom_link_description_1,
        // TODO custom_link_description_2,
        // TODO custom_link_description_3,
        // TODO custom_link_description_4,
        // TODO custom_link_description_5,
        // TODO custom_link_url_0,
        // TODO custom_link_url_1,
        // TODO custom_link_url_2,
        // TODO custom_link_url_3,
        // TODO custom_link_url_4,
        // TODO custom_link_url_5,
        ..
    } = record;

    assert_eq!(original.id, id);

    if original.version + 1 != version {
        return Err(anyhow!("Invalid entry version"));
    }
    original.version = version;

    if created.is_some() {
        log::warn!("The field 'created' can't be modified.");
    }

    if license.is_some() {
        log::warn!("The license can't be modified.");
    }

    if ratings.is_some() {
        log::warn!("The ratings can't be modified.");
    }

    patch_string_field("title", &mut original.title, title)?;
    patch_string_field("description", &mut original.description, description)?;
    patch_float_field("lat", &mut original.lat, lat)?;
    patch_float_field("lng", &mut original.lng, lng)?;
    patch_optional_string_field("street", &mut original.street, street)?;
    patch_optional_string_field("zip", &mut original.zip, zip)?;
    patch_optional_string_field("city", &mut original.city, city)?;
    patch_optional_string_field("country", &mut original.country, country)?;
    patch_optional_string_field("state", &mut original.state, state)?;
    patch_optional_string_field("contact_name", &mut original.contact_name, contact_name)?;
    patch_optional_string_field("contact_email", &mut original.email, contact_email)?;
    patch_optional_string_field("contact_phone", &mut original.telephone, contact_phone)?;
    patch_optional_string_field("homepage", &mut original.homepage, homepage)?;
    patch_optional_string_field("opening_hours", &mut original.opening_hours, opening_hours)?;
    patch_optional_date_field("founded_on", &mut original.founded_on, founded_on)?;
    patch_optional_string_field("image_url", &mut original.image_url, image_url)?;
    patch_optional_string_field(
        "image_link_url",
        &mut original.image_link_url,
        image_link_url,
    )?;

    if let Some(tags) = tags {
        for tag in tags.split(',') {
            match patch_op(tag) {
                Ok(PatchOp::Append(new_tag)) => {
                    original.tags.push(new_tag.to_string());
                }
                Ok(PatchOp::Delete(remove_tag)) => {
                    original.tags.retain(|t| t != remove_tag);
                }
                Ok(PatchOp::Replace(_)) => {
                    log::warn!("Tags can't be replaced, only removed or added");
                }
                Ok(PatchOp::DeleteAll) => {
                    log::warn!("You must not remove all tags at once");
                }
                Err(err) => {
                    log::warn!("Invalid tag patch operation: {err}");
                }
            }
        }
    }

    Ok(original)
}

#[derive(Debug, PartialEq)]
enum PatchOp<'a> {
    Append(&'a str),
    Replace(&'a str),
    Delete(&'a str),
    DeleteAll,
}

#[derive(Debug, PartialEq, Error)]
enum PatchOpError {
    #[error("No patch operation found")]
    NoOp,
    #[error("Empty string")]
    EmptyString,
}

fn patch_string_field(
    field_name: &str,
    field: &mut String,
    patch: Option<String>,
) -> anyhow::Result<()> {
    if let Some(patch) = patch {
        let op = patch_op(&patch)?;
        match op {
            PatchOp::Replace(replace) => {
                *field = replace.to_string();
            }
            PatchOp::Append(append) => {
                field.push_str(APPEND_SEPERATOR);
                field.push_str(append);
            }
            PatchOp::Delete(_) | PatchOp::DeleteAll => {
                return Err(anyhow!("The field '{field_name}' can't be deleted."));
            }
        }
    }
    Ok(())
}

fn patch_optional_string_field(
    field_name: &str,
    field: &mut Option<String>,
    patch: Option<String>,
) -> anyhow::Result<()> {
    if let Some(patch) = patch {
        let op = patch_op(&patch)?;
        match op {
            PatchOp::Replace(replace) => {
                *field = Some(replace.to_string());
            }
            PatchOp::Append(append) => match field {
                Some(field) => {
                    field.push_str(APPEND_SEPERATOR);
                    field.push_str(append);
                }
                None => {
                    *field = Some(append.to_string());
                }
            },
            PatchOp::Delete(_) => {
                return Err(anyhow!("You can't delete only parts of '{field_name}'"));
            }
            PatchOp::DeleteAll => {
                *field = None;
            }
        }
    }
    Ok(())
}

fn patch_optional_date_field(
    field_name: &str,
    field: &mut Option<Date>,
    patch: Option<String>,
) -> anyhow::Result<()> {
    if let Some(patch) = patch {
        let op = patch_op(&patch)?;
        match op {
            PatchOp::Replace(replace) => {
                let date: Date = serde_json::from_str(replace)?;
                *field = Some(date);
            }
            PatchOp::Append(_) => {
                return Err(anyhow!(
                    "'{field_name}' can't be extended, replace or remove it"
                ));
            }
            PatchOp::Delete(_) => {
                return Err(anyhow!(
                    "You can't delete only parts of '{field_name}', replace or remove it"
                ));
            }
            PatchOp::DeleteAll => {
                *field = None;
            }
        }
    }
    Ok(())
}

fn patch_float_field(
    field_name: &str,
    field: &mut f64,
    patch: Option<String>,
) -> anyhow::Result<()> {
    if let Some(patch) = patch {
        let op = patch_op(&patch)?;
        match op {
            PatchOp::Replace(replace) => {
                *field = replace.parse()?;
            }
            _ => {
                return Err(anyhow!("You can only replace '{field_name}'"));
            }
        }
    }
    Ok(())
}

fn patch_op(s: &str) -> Result<PatchOp<'_>, PatchOpError> {
    let trimmed = s.trim();

    if trimmed.starts_with(OP_DELETE) {
        let delete = trimmed.split(OP_DELETE).nth(1).unwrap().trim();
        return Ok(if delete.is_empty() {
            PatchOp::DeleteAll
        } else {
            PatchOp::Delete(delete)
        });
    }

    if trimmed.starts_with(OP_APPEND) {
        let append = trimmed.split(OP_APPEND).nth(1).unwrap().trim();
        if append.is_empty() {
            return Err(PatchOpError::EmptyString);
        }
        return Ok(PatchOp::Append(append.trim()));
    }

    if trimmed.starts_with(OP_REPLACE) {
        let replace = trimmed.split(OP_REPLACE).nth(1).unwrap();
        if replace.is_empty() {
            return Err(PatchOpError::EmptyString);
        }
        return Ok(PatchOp::Replace(replace.trim()));
    }

    Err(PatchOpError::NoOp)
}

#[derive(Debug, Default, Deserialize)]
struct PatchPlaceRecord {
    id: String,
    version: u64,
    created: Option<String>,
    title: Option<String>,
    description: Option<String>,
    lat: Option<String>,
    lng: Option<String>,
    street: Option<String>,
    zip: Option<String>,
    city: Option<String>,
    country: Option<String>,
    state: Option<String>,
    contact_name: Option<String>,
    contact_email: Option<String>,
    contact_phone: Option<String>,
    opening_hours: Option<String>,
    founded_on: Option<String>,
    tags: Option<String>,
    ratings: Option<String>,
    homepage: Option<String>,
    license: Option<String>,
    image_url: Option<String>,
    image_link_url: Option<String>,
    // TODO custom_link_title_0: Option<String>,
    // TODO custom_link_title_1: Option<String>,
    // TODO custom_link_title_2: Option<String>,
    // TODO custom_link_title_3: Option<String>,
    // TODO custom_link_title_4: Option<String>,
    // TODO custom_link_title_5: Option<String>,
    // TODO custom_link_description_0: Option<String>,
    // TODO custom_link_description_1: Option<String>,
    // TODO custom_link_description_2: Option<String>,
    // TODO custom_link_description_3: Option<String>,
    // TODO custom_link_description_4: Option<String>,
    // TODO custom_link_description_5: Option<String>,
    // TODO custom_link_url_0: Option<String>,
    // TODO custom_link_url_1: Option<String>,
    // TODO custom_link_url_2: Option<String>,
    // TODO custom_link_url_3: Option<String>,
    // TODO custom_link_url_4: Option<String>,
    // TODO custom_link_url_5: Option<String>,
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
                None => Err(anyhow!("Unable to find geo coordinates")),
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
        (true, None) => Err(anyhow!(
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

    mod patch {

        use super::*;

        fn default_entry() -> Entry {
            Entry {
                id: Default::default(),
                created: Default::default(),
                version: Default::default(),
                title: Default::default(),
                description: Default::default(),
                lat: Default::default(),
                lng: Default::default(),
                street: Default::default(),
                zip: Default::default(),
                city: Default::default(),
                country: Default::default(),
                state: Default::default(),
                contact_name: Default::default(),
                email: Default::default(),
                telephone: Default::default(),
                homepage: Default::default(),
                opening_hours: Default::default(),
                founded_on: Default::default(),
                categories: Default::default(),
                tags: Default::default(),
                ratings: Default::default(),
                license: Default::default(),
                image_url: Default::default(),
                image_link_url: Default::default(),
                custom_links: Default::default(),
            }
        }

        #[test]
        fn append() {
            assert_eq!(patch_op("++foo"), Ok(PatchOp::Append("foo")));
            assert_eq!(patch_op("  ++foo"), Ok(PatchOp::Append("foo")));
            assert_eq!(patch_op("++   foo"), Ok(PatchOp::Append("foo")));
            assert_eq!(patch_op("foo++"), Err(PatchOpError::NoOp));
            assert_eq!(patch_op("++"), Err(PatchOpError::EmptyString));
        }

        #[test]
        fn replace() {
            assert_eq!(patch_op("==foo"), Ok(PatchOp::Replace("foo")));
            assert_eq!(patch_op("  ==foo"), Ok(PatchOp::Replace("foo")));
            assert_eq!(patch_op("==   foo"), Ok(PatchOp::Replace("foo")));
            assert_eq!(patch_op("foo=="), Err(PatchOpError::NoOp));
            assert_eq!(patch_op("=="), Err(PatchOpError::EmptyString));
        }

        #[test]
        fn delete() {
            assert_eq!(patch_op("--"), Ok(PatchOp::DeleteAll));
            assert_eq!(patch_op("  --"), Ok(PatchOp::DeleteAll));
            assert_eq!(patch_op("Foo bar --"), Err(PatchOpError::NoOp));
            assert_eq!(patch_op("-- some text"), Ok(PatchOp::Delete("some text")));
        }

        #[test]
        fn append_title() {
            let original = Entry {
                title: "Foo bar".to_string(),
                ..default_entry()
            };
            let record = PatchPlaceRecord {
                version: original.version + 1,
                title: Some("++baz".to_string()),
                ..Default::default()
            };
            let patched = patch_place(original, record).unwrap();
            assert_eq!(patched.title, "Foo bar baz");
        }

        #[test]
        fn replace_title() {
            let original = Entry {
                title: "Foo bar".to_string(),
                ..default_entry()
            };
            let record = PatchPlaceRecord {
                version: original.version + 1,
                title: Some("==Baz".to_string()),
                ..Default::default()
            };
            let patched = patch_place(original, record).unwrap();
            assert_eq!(patched.title, "Baz");
        }

        #[test]
        fn remove_title() {
            let original = Entry {
                title: "Foo bar".to_string(),
                ..default_entry()
            };
            let record = PatchPlaceRecord {
                version: original.version + 1,
                title: Some("--".to_string()),
                ..Default::default()
            };
            assert!(patch_place(original, record).is_err());
        }

        #[test]
        fn append_tags() {
            let original = Entry {
                tags: vec!["foo".to_string(), "bar".to_string()],
                ..default_entry()
            };
            let record = PatchPlaceRecord {
                version: original.version + 1,
                tags: Some("++baz,++boing".to_string()),
                ..Default::default()
            };
            let patched = patch_place(original, record).unwrap();
            assert_eq!(patched.tags, vec!["foo", "bar", "baz", "boing"]);
        }

        #[test]
        fn remove_tags() {
            let original = Entry {
                tags: vec!["foo".to_string(), "bar".to_string()],
                ..default_entry()
            };
            let record = PatchPlaceRecord {
                version: original.version + 1,
                tags: Some("--foo".to_string()),
                ..Default::default()
            };
            let patched = patch_place(original, record).unwrap();
            assert_eq!(patched.tags, vec!["bar"]);
        }

        #[test]
        fn remove_and_append_tags() {
            let original = Entry {
                tags: vec!["foo".to_string(), "bar".to_string()],
                ..default_entry()
            };
            let record = PatchPlaceRecord {
                version: original.version + 1,
                tags: Some("--bar, ++baz".to_string()),
                ..Default::default()
            };
            let patched = patch_place(original, record).unwrap();
            assert_eq!(patched.tags, vec!["foo", "baz"]);
        }
    }
}
