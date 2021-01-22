use anyhow::Result;
use ofdb_boundary::{Entry, MapBbox, NewPlace, PlaceSearchResult, SearchResponse, UpdatePlace};
use reqwest::blocking::Client;
use uuid::Uuid;

pub mod import;

pub fn create_new_place(api: &str, client: &Client, new_place: &NewPlace) -> Result<String> {
    let url = format!("{}/entries", api);
    let res = client.post(&url).json(&new_place).send()?;
    Ok(res.json()?)
}

pub fn update_place(api: &str, client: &Client, id: &str, place: &UpdatePlace) -> Result<String> {
    let mut place = place.clone();
    place.version += 1;
    let url = format!("{}/entries/{}", api, id);
    let res = client.put(&url).json(&place).send()?;
    Ok(res.json()?)
}

pub fn read_entries(api: &str, client: &Client, uuids: Vec<Uuid>) -> Result<Vec<Entry>> {
    log::debug!("Read {} places", uuids.len());
    let uuids = uuids
        .into_iter()
        .map(Uuid::to_simple)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let url = format!("{}/entries/{}", api, uuids);
    let res = client.get(&url).send()?;
    let res = res.json()?;
    Ok(res)
}

pub fn search(api: &str, client: &Client, txt: &str, bbox: &MapBbox) -> Result<SearchResponse> {
    let url = format!("{}/search", api);
    let MapBbox { sw, ne } = bbox;
    let bbox_string = format!("{},{},{},{}", sw.lat, sw.lng, ne.lat, ne.lng);
    let res = client
        .get(&url)
        .query(&[("text", txt), ("bbox", &bbox_string)])
        .send()?;
    Ok(res.json()?)
}

pub fn search_duplicates(
    api: &str,
    client: &Client,
    new_place: &NewPlace,
) -> Result<Option<Vec<PlaceSearchResult>>> {
    let url = format!("{}/search/duplicates", api);
    let res = client.post(&url).json(&new_place).send()?;
    let res: Vec<PlaceSearchResult> = res.json()?;
    Ok(if res.is_empty() { None } else { Some(res) })
}