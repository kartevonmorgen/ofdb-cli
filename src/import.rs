use anyhow::Result;
use ofdb_boundary::{Entry, NewPlace, PlaceSearchResult};
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

#[derive(Debug, Clone, Error)]
pub enum CsvImportError {
    #[error("Could not read CSV record: {0}")]
    Record(String),
    #[error("Invalid address or geo coordinates: {0}")]
    AddressOrGeoCoordinates(String),
    #[error("Invalid patch request: {0}")]
    PatchRequest(String),
}

type PlaceId = String;

#[derive(Debug)]
pub struct ImportResult<'a> {
    pub new_place: &'a NewPlace,
    pub import_id: Option<String>,
    pub result: result::Result<PlaceId, Error>,
}

#[derive(Debug)]
pub struct UpdateResult<'a> {
    pub place: &'a Entry,
    pub import_id: Option<String>,
    pub result: result::Result<PlaceId, Error>,
}

#[derive(Debug, Clone)]
pub struct CsvImportResult<T> {
    pub record_nr: usize,
    pub result: result::Result<T, CsvImportError>,
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
pub struct FailureReport<T> {
    pub place: T,
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
pub struct SuccessReport<T> {
    pub place: T,
    pub import_id: Option<String>,
    pub uuid: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CsvImportSuccessReport<T> {
    pub record_nr: usize,
    pub place: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CsvImportFailureReport {
    pub record_nr: usize,
    pub error: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Report<T, S> {
    pub duplicates: Vec<DuplicateReport>,
    pub failures: Vec<FailureReport<T>>,
    pub successes: Vec<S>,
    pub csv_import_successes: Vec<CsvImportSuccessReport<T>>,
    pub csv_import_failures: Vec<CsvImportFailureReport>,
}

impl TryFrom<&ImportResult<'_>> for FailureReport<NewPlace> {
    type Error = ();
    fn try_from(res: &ImportResult) -> Result<Self, Self::Error> {
        res.err()
            .and_then(|e| match e {
                Error::Other(msg) => Some(msg),
                _ => None,
            })
            .map(|e| FailureReport {
                place: res.place().to_owned(),
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

impl TryFrom<&ImportResult<'_>> for SuccessReport<NewPlace> {
    type Error = ();
    fn try_from(res: &ImportResult) -> Result<Self, Self::Error> {
        res.id()
            .map(|id| Self {
                place: res.place().to_owned(),
                import_id: res.import_id.clone(),
                uuid: id.to_owned(),
            })
            .ok_or(())
    }
}

impl<T> TryFrom<&CsvImportResult<T>> for CsvImportSuccessReport<T>
where
    T: Clone,
{
    type Error = ();
    fn try_from(res: &CsvImportResult<T>) -> Result<Self, Self::Error> {
        let CsvImportResult { record_nr, result } = res;
        result
            .as_ref()
            .map(|place| CsvImportSuccessReport {
                record_nr: *record_nr,
                place: place.clone(),
            })
            .map_err(|_| ())
    }
}

impl<T> TryFrom<&CsvImportResult<T>> for CsvImportFailureReport {
    type Error = ();
    fn try_from(res: &CsvImportResult<T>) -> Result<Self, Self::Error> {
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

impl From<Vec<ImportResult<'_>>> for Report<NewPlace, SuccessReport<NewPlace>> {
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
            csv_import_failures: Default::default(),
            csv_import_successes: Default::default(),
        }
    }
}

impl From<Vec<CsvImportResult<NewPlace>>> for Report<NewPlace, SuccessReport<NewPlace>> {
    fn from(results: Vec<CsvImportResult<NewPlace>>) -> Self {
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
            duplicates: Default::default(),
            failures: Default::default(),
            successes: Default::default(),
        }
    }
}

impl From<Vec<CsvImportResult<Entry>>> for Report<Entry, SuccessReport<Entry>> {
    fn from(results: Vec<CsvImportResult<Entry>>) -> Self {
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
            duplicates: Default::default(),
            failures: Default::default(),
            successes: Default::default(),
        }
    }
}
