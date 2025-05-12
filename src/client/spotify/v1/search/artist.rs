use crate::constant::spotify::API_BASE_URL;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetResponse {
    pub artists: ArtistsPage,
}

#[derive(Debug, Deserialize)]
pub struct ArtistsPage {
    pub href: String,
    pub limit: u32,
    pub next: Option<String>,
    pub offset: u32,
    pub previous: Option<String>,
    pub total: u32,
    pub items: Vec<Artist>,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub external_urls: ExternalUrls,
    pub followers: Followers,
    pub genres: Vec<String>,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub name: String,
    pub popularity: u32,

    #[serde(rename = "type")]
    pub artist_type: String,
    pub uri: String,
}

#[derive(Debug, Deserialize)]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Debug, Deserialize)]
pub struct Followers {
    pub href: Option<String>,
    pub total: u32,
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub url: String,
    pub height: u32,
    pub width: u32,
}

pub struct GetQuery {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub genre: Option<String>,
}

pub async fn get(access_token: &str, query: &GetQuery) -> Result<GetResponse> {
    let client = Client::new();
    let query = &[
        ("type", "artist"),
        (
            "q",
            &format!("genre:{}", query.genre.as_deref().unwrap_or(""),),
        ),
        (
            "offset",
            &query
                .offset
                .map(|offset| offset.to_string())
                .unwrap_or_else(|| String::new()),
        ),
        (
            "limit",
            &query
                .limit
                .map(|limit| limit.to_string())
                .unwrap_or_else(|| String::new()),
        ),
    ];

    Ok(client
        .get(&format!("{}/v1/search", API_BASE_URL))
        .bearer_auth(access_token)
        .query(&query)
        .send()
        .await?
        .json::<GetResponse>()
        .await?)
}
