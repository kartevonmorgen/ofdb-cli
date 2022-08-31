use anyhow::Result;
use ofdb_boundary::{
    Credentials, Entry, Error, MapBbox, NewPlace, PlaceSearchResult, Review, SearchResponse,
    UpdatePlace,
};
use reqwest::blocking::{Client, Response};
use uuid::Uuid;

pub mod csv;
pub mod import;
pub mod review;

pub fn create_new_place(api: &str, client: &Client, new_place: &NewPlace) -> Result<String> {
    let url = format!("{}/entries", api);
    let res = client.post(&url).json(&new_place).send()?;
    handle_response(res)
}

pub fn update_place(api: &str, client: &Client, id: &str, place: &UpdatePlace) -> Result<String> {
    let mut place = place.clone();
    place.version += 1;
    let url = format!("{}/entries/{}", api, id);
    let res = client.put(&url).json(&place).send()?;
    handle_response(res)
}

pub fn read_entries(api: &str, client: &Client, uuids: Vec<Uuid>) -> Result<Vec<Entry>> {
    log::debug!("Read {} places", uuids.len());
    let uuids = uuids
        .into_iter()
        .map(Uuid::simple)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let url = format!("{}/entries/{}", api, uuids);
    let res = client.get(&url).send()?;
    handle_response(res)
}

/// Login
///
/// Important:
/// The
/// [cookie store](https://docs.rs/reqwest/0.11.1/reqwest/struct.ClientBuilder.html#method.cookie_store)
/// should be enabled.  
pub fn login(api: &str, client: &Client, req: &Credentials) -> Result<()> {
    let url = format!("{}/login", api);
    log::info!("Try to login with '{}' ", req.email);
    let res = client
        .post(&url)
        .header("Access-Control-Allow-Credentials", "true")
        .json(&req)
        .send()?;
    handle_response(res)
}

pub fn review_places(api: &str, client: &Client, uuids: Vec<Uuid>, review: Review) -> Result<()> {
    let url = format!(
        "{}/places/{}/review",
        api,
        uuids
            .into_iter()
            .map(Uuid::simple)
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    let json_string = serde_json::to_string(&review).unwrap();
    log::debug!("Send review {json_string} to {url}");
    let res = client.post(&url).json(&review).send()?;
    handle_response(res)
}

pub fn search(api: &str, client: &Client, txt: &str, bbox: &MapBbox) -> Result<SearchResponse> {
    let url = format!("{}/search", api);
    let MapBbox { sw, ne } = bbox;
    let bbox_string = format!("{},{},{},{}", sw.lat, sw.lng, ne.lat, ne.lng);
    let res = client
        .get(&url)
        .query(&[("text", txt), ("bbox", &bbox_string)])
        .send()?;
    handle_response(res)
}

pub fn search_duplicates(
    api: &str,
    client: &Client,
    new_place: &NewPlace,
) -> Result<Option<Vec<PlaceSearchResult>>> {
    let url = format!("{}/search/duplicates", api);
    let res = client.post(&url).json(&new_place).send()?;
    let res: Vec<PlaceSearchResult> = handle_response(res)?;
    Ok(if res.is_empty() { None } else { Some(res) })
}

fn handle_response<T>(res: Response) -> Result<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    if res.status().is_success() {
        Ok(res.json::<T>()?)
    } else {
        let err: Error = res.json()?;
        Err(anyhow::anyhow!(err.message))
    }
}
