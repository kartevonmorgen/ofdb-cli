use anyhow::Result;
use ofdb_boundary::{NewPlace, PlaceSearchResult};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, result};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Found possible duplicates")]
    Duplicates(Vec<PlaceSearchResult>),
    #[error("Could not import place: {0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum CsvImportError {
    #[error("Could not read CSV record: {0}")]
    InvalidRecord(String),
    #[error("Invalid address or geo coordinates: {0}")]
    InvalidAddressOrGeoCoordinates(String),
}

type PlaceId = String;

#[derive(Debug)]
pub struct ImportResult<'a> {
    pub new_place: &'a NewPlace,
    pub import_id: Option<String>,
    pub result: result::Result<PlaceId, Error>,
}

#[derive(Debug)]
pub struct CsvImportResult {
    pub record_nr: usize,
    pub result: result::Result<NewPlace, CsvImportError>,
}

impl ImportResult<'_> {
    fn place(&self) -> &NewPlace {
        self.new_place
    }
    fn err(&self) -> Option<&Error> {
        self.result.as_ref().err()
    }
    fn id(&self) -> Option<&str> {
        self.result.as_ref().ok().map(|x| x.as_str())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FailureReport {
    pub new_place: NewPlace,
    pub import_id: Option<String>,
    pub error: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DuplicateReport {
    pub new_place: NewPlace,
    pub import_id: Option<String>,
    pub duplicates: Vec<PlaceSearchResult>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SuccessReport {
    pub new_place: NewPlace,
    pub import_id: Option<String>,
    pub uuid: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CsvImportSuccessReport {
    pub record_nr: usize,
    pub new_place: NewPlace,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CsvImportFailureReport {
    pub record_nr: usize,
    pub error: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Report {
    pub duplicates: Vec<DuplicateReport>,
    pub failures: Vec<FailureReport>,
    pub successes: Vec<SuccessReport>,
    pub csv_import_successes: Vec<CsvImportSuccessReport>,
    pub csv_import_failures: Vec<CsvImportFailureReport>,
}

impl TryFrom<&ImportResult<'_>> for FailureReport {
    type Error = ();
    fn try_from(res: &ImportResult) -> Result<Self, Self::Error> {
        res.err()
            .and_then(|e| match e {
                Error::Other(msg) => Some(msg),
                _ => None,
            })
            .map(|e| FailureReport {
                new_place: res.place().to_owned(),
                import_id: res.import_id.clone(),
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
                Error::Duplicates(dups) => Some(dups),
                _ => None,
            })
            .map(|dups| DuplicateReport {
                new_place: res.place().to_owned(),
                import_id: res.import_id.clone(),
                duplicates: dups.to_vec(),
            })
            .ok_or(())
    }
}

impl TryFrom<&ImportResult<'_>> for SuccessReport {
    type Error = ();
    fn try_from(res: &ImportResult) -> Result<Self, Self::Error> {
        res.id()
            .map(|id| SuccessReport {
                new_place: res.place().to_owned(),
                import_id: res.import_id.clone(),
                uuid: id.to_owned(),
            })
            .ok_or(())
    }
}

impl TryFrom<&CsvImportResult> for CsvImportSuccessReport {
    type Error = ();
    fn try_from(res: &CsvImportResult) -> Result<Self, Self::Error> {
        let CsvImportResult { record_nr, result } = res;
        result
            .as_ref()
            .map(|new_place| CsvImportSuccessReport {
                record_nr: *record_nr,
                new_place: new_place.clone(),
            })
            .map_err(|_| ())
    }
}

impl TryFrom<&CsvImportResult> for CsvImportFailureReport {
    type Error = ();
    fn try_from(res: &CsvImportResult) -> Result<Self, Self::Error> {
        let CsvImportResult { record_nr, result } = res;
        result
            .as_ref()
            .err()
            .map(|err| CsvImportFailureReport {
                record_nr: *record_nr,
                error: err.to_string(),
            })
            .ok_or(())
    }
}

impl From<Vec<ImportResult<'_>>> for Report {
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
            .map(SuccessReport::try_from)
            .filter_map(Result::ok)
            .collect();

        Self {
            duplicates,
            failures,
            successes,
            ..Default::default()
        }
    }
}

impl From<Vec<CsvImportResult>> for Report {
    fn from(results: Vec<CsvImportResult>) -> Self {
        let csv_import_failures = results
            .iter()
            .map(CsvImportFailureReport::try_from)
            .filter_map(Result::ok)
            .collect();

        let csv_import_successes = results
            .iter()
            .map(CsvImportSuccessReport::try_from)
            .filter_map(Result::ok)
            .collect();

        Self {
            csv_import_failures,
            csv_import_successes,
            ..Default::default()
        }
    }
}
