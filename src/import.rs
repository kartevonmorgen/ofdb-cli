use super::{create_new_place, search_duplicates};
use anyhow::Result;
use ofdb_boundary::{NewPlace, PlaceSearchResult};
use serde::Serialize;
use std::{convert::TryFrom, fs::File, io, path::PathBuf, result};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("Found possible duplicates")]
    Duplicates(Vec<PlaceSearchResult>),
    #[error("Could not import place: {0}")]
    Other(String),
}

type PlaceId = String;

#[derive(Debug)]
pub struct ImportResult<'a> {
    pub new_place: &'a NewPlace,
    pub result: result::Result<PlaceId, ImportError>,
}

impl ImportResult<'_> {
    fn place(&self) -> &NewPlace {
        self.new_place
    }
    fn err(&self) -> Option<&ImportError> {
        self.result.as_ref().err()
    }
    fn id(&self) -> Option<&str> {
        self.result.as_ref().ok().map(|x| x.as_str())
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ImportReport {
    pub duplicates: Vec<DuplicateReport>,
    pub failures: Vec<FailureReport>,
    pub successes: Vec<String>,
}

impl TryFrom<&ImportResult<'_>> for FailureReport {
    type Error = ();
    fn try_from(res: &ImportResult) -> Result<Self, Self::Error> {
        res.err()
            .and_then(|e| match e {
                ImportError::Other(msg) => Some(msg),
                _ => None,
            })
            .map(|e| FailureReport {
                new_place: res.place().to_owned(),
                error: e.to_string(),
            })
            .ok_or(())
    }
}

impl TryFrom<&ImportResult<'_>> for DuplicateReport {
    type Error = ();
    fn try_from(res: &ImportResult) -> Result<Self, Self::Error> {
        res.err()
            .and_then(|e| match e {
                ImportError::Duplicates(dups) => Some(dups),
                _ => None,
            })
            .map(|dups| DuplicateReport {
                new_place: res.place().to_owned(),
                duplicates: dups.to_vec(),
            })
            .ok_or(())
    }
}

impl From<Vec<ImportResult<'_>>> for ImportReport {
    fn from(results: Vec<ImportResult>) -> Self {
        let failures = results
            .iter()
            .map(FailureReport::try_from)
            .filter_map(Result::ok)
            .collect();

        let duplicates = results
            .iter()
            .map(DuplicateReport::try_from)
            .filter_map(Result::ok)
            .collect();

        let successes = results
            .iter()
            .filter_map(ImportResult::id)
            .map(ToString::to_string)
            .collect();

        Self {
            duplicates,
            failures,
            successes,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FailureReport {
    pub new_place: NewPlace,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct DuplicateReport {
    pub new_place: NewPlace,
    pub duplicates: Vec<PlaceSearchResult>,
}

pub fn import(api: &str, path: PathBuf) -> Result<()> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let places: Vec<NewPlace> = serde_json::from_reader(reader)?;
    log::debug!("Read {} places from JSON file", places.len());
    let client = reqwest::blocking::Client::new();
    let mut results = vec![];
    for new_place in &places {
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
                result: Err(ImportError::Duplicates(possible_duplicates)),
            });
        } else {
            match create_new_place(api, &client, new_place) {
                Ok(id) => {
                    log::debug!("Successfully imported '{}' with ID={}", new_place.title, id);
                    results.push(ImportResult {
                        new_place,
                        result: Ok(id),
                    });
                }
                Err(err) => {
                    log::warn!("Could not import '{}': {}", new_place.title, err);
                    results.push(ImportResult {
                        new_place,
                        result: Err(ImportError::Other(err.to_string())),
                    });
                }
            }
        }
    }
    let report = ImportReport::from(results);
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
