use crate::constant::spotify::API_BASE_URL;
use anyhow::Result;
use reqwest::Client;

type GetResponse = Vec<bool>;
pub enum Type {
    Artist,
    User,
}

pub async fn get(access_token: &str, r#type: Type, ids: &[String]) -> Result<GetResponse> {
    let client = Client::new();
    let query = [
        (
            "type",
            match r#type {
                Type::Artist => "artist",
                Type::User => "user",
            },
        ),
        ("ids", &ids.join(",")),
    ];

    Ok(client
        .get(&format!("{}/v1/me/following/contains", API_BASE_URL))
        .query(&query)
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<GetResponse>()
        .await?)
}
