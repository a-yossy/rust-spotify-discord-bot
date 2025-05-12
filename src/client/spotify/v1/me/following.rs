pub mod contains;

use crate::constant::spotify::API_BASE_URL;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GetResponse {
    artists: Artists,
}

#[derive(Debug, Deserialize)]
struct Artists {
    cursors: Cursors,
    items: Vec<Artist>,
}

#[derive(Debug, Deserialize)]
struct Cursors {
    after: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub id: String,
}

pub async fn get(access_token: &str) -> Result<Vec<Artist>> {
    let mut artists = Vec::new();
    let mut after = Some(String::new());
    let client = Client::new();
    while let Some(now_after) = after {
        let query = [("type", "artist"), ("after", &now_after)];
        let response = client
            .get(&format!("{}/v1/me/following", API_BASE_URL))
            .query(&query)
            .bearer_auth(access_token)
            .send()
            .await?
            .json::<GetResponse>()
            .await?;
        after = response.artists.cursors.after;
        response
            .artists
            .items
            .into_iter()
            .for_each(|artist| artists.push(artist));
    }

    Ok(artists)
}

pub enum PutType {
    Artist,
    User,
}

pub async fn put(access_token: &str, r#type: PutType, ids: &[String]) -> Result<()> {
    let client = Client::new();
    let query = &[(
        "type",
        match r#type {
            PutType::Artist => "artist",
            PutType::User => "user",
        },
    )];
    let body = serde_json::json!({
        "ids": ids
    });
    client
        .put(&format!("{}/v1/me/following", API_BASE_URL))
        .bearer_auth(access_token)
        .header("Content-Type", "application/json")
        .query(&query)
        .json(&body)
        .send()
        .await?;

    Ok(())
}
