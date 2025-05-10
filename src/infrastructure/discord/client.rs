use std::env;

use serenity::{
    Client as SerenityClient,
    all::{Framework, GatewayIntents},
};

pub struct Client;

impl Client {
    pub async fn get<F: Framework + 'static>(framework: F) -> SerenityClient {
        let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKENの取得でエラーが発生しました");
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MEMBERS;

        SerenityClient::builder(&token, intents)
            .framework(framework)
            .await
            .expect("Discordクライアントの作成でエラーが発生しました")
    }
}
