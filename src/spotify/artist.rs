use anyhow::{Context, Result};
use rand::{rng, seq::IndexedRandom};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct FollowingResponse {
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

pub async fn get_following(access_token: &str) -> Result<Vec<Artist>> {
    let mut artists = Vec::new();
    let mut after = Some(String::new());
    let client = Client::new();
    while let Some(now_after) = after {
        let response = client
            .get("https://api.spotify.com/v1/me/following")
            .query(&[("type", "artist"), ("after", &now_after)])
            .bearer_auth(access_token)
            .send()
            .await?
            .json::<FollowingResponse>()
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
