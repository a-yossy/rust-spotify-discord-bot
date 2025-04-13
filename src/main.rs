use base64::prelude::*;
use reqwest::Client;
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct SpotifyTokenResponse {
    access_token: String,
}

// fn generate_random_string(len: usize) -> String {
//     rand::rng()
//         .sample_iter(rand::distr::Alphanumeric)
//         .take(len)
//         .map(char::from)
//         .collect()
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let refresh_token = env::var("SPOTIFY_REFRESH_TOKEN")?;
    let client = Client::new();
    let url = "https://accounts.spotify.com/api/token";
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", &refresh_token),
    ];
    let client_id = env::var("SPOTIFY_CLIENT_ID").unwrap();
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET").unwrap();
    let authorization = BASE64_STANDARD.encode(format!("{}:{}", client_id, client_secret));
    let response = client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", format!("Basic {}", authorization))
        .form(&params)
        .send()
        .await?
        .json::<SpotifyTokenResponse>()
        .await?;
    let access_token = response.access_token;

    let response = client
        .get("https://api.spotify.com/v1/me/player/recently-played")
        .query(&[("limit", 10)])
        .bearer_auth(access_token)
        .send()
        .await?;

    if response.status().is_success() {
        println!("{}", response.text().await?);
    } else {
        eprintln!("Error: {}", response.status());
    }

    Ok(())
}
