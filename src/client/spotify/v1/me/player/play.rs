use std::collections::HashMap;

use crate::constant::spotify::API_BASE_URL;
use anyhow::Result;
use reqwest::Client;

pub async fn put(access_token: &str, context_uri: &str) -> Result<()> {
    let client = Client::new();
    let mut body = HashMap::new();
    body.insert("context_uri", context_uri);
    let _ = client
        .put(&format!("{}/v1/me/player/play", API_BASE_URL))
        .bearer_auth(access_token)
        .header("Content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    Ok(())
}
