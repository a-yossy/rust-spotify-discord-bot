use std::env;

use serenity::{
    Client,
    all::{EventHandler, GatewayIntents},
};

pub async fn get_client<H: EventHandler + 'static>(handler: H) -> Client {
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKENの取得でエラーが発生しました");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Discordクライアントの作成でエラーが発生しました")
}
