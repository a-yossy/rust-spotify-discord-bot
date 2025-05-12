use rand::seq::IndexedRandom;

use crate::{client::spotify, types::discord::framework::Error};

use super::Context;

pub async fn handle(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let response = spotify::api::token::post().await?;
    let artists = spotify::v1::me::following::get(&response.access_token).await?;
    let artist_id = match { artists.choose(&mut rand::rng()) } {
        Some(artist) => &artist.id,
        None => {
            ctx.say("アーティストが見つかりません").await?;

            return Ok(());
        }
    };
    let tracks = spotify::v1::artists::top_tracks::get(&artist_id, &response.access_token).await?;
    let track_url = match { tracks.choose(&mut rand::rng()) } {
        Some(track) => &track.external_urls.spotify,
        None => {
            ctx.say("曲が見つかりません").await?;

            return Ok(());
        }
    };
    ctx.say(track_url).await?;

    Ok(())
}
