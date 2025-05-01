use std::env;

use serenity::{
    Client,
    all::{EventHandler, Framework, GatewayIntents},
};

pub async fn get<H: EventHandler + 'static, F: Framework + 'static>(
    handler: H,
    framework: F,
) -> Client {
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKENの取得でエラーが発生しました");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    Client::builder(&token, intents)
        .event_handler(handler)
        .framework(framework)
        .await
        .expect("Discordクライアントの作成でエラーが発生しました")
}
