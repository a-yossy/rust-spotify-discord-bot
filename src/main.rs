use std::env;

use rand::rng;
use rand::seq::IndexedRandom;
use rig::completion::Prompt;
use rig::providers::gemini;
use rig::providers::gemini::completion::GEMINI_2_0_FLASH;
use rig::tool::Tool as RigTool;
use rmcp::model::{CallToolRequestParam, CallToolResult, Tool as McpTool};
use rmcp::service::ServerSink;
use rmcp::{Error, ServiceExt};
use serde::{Deserialize, Serialize};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use ss_discord_bot::client::spotify;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.mentions_me(&ctx.http).await.unwrap_or(false) {
            return;
        }

        let msg_content = strip_mentions_msg_content(&msg);
        if msg_content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        } else if msg_content == "spotify" {
            let access_token = match spotify::api::token::post().await {
                Ok(token) => token,
                Err(e) => {
                    eprintln!("{:?}", e);
                    return;
                }
            };

            let artists = match spotify::v1::me::following::get(&access_token).await {
                Ok(artists) => artists,
                Err(e) => {
                    eprintln!("{:?}", e);
                    return;
                }
            };
            let artist_id = match { artists.choose(&mut rng()) } {
                Some(artist) => &artist.id,
                None => {
                    if let Err(why) = msg
                        .channel_id
                        .say(&ctx.http, "アーティストが見つかりません")
                        .await
                    {
                        println!("Error sending message: {why:?}");
                    }
                    return;
                }
            };

            let tracks = match spotify::v1::artists::top_tracks::get(artist_id, &access_token).await
            {
                Ok(tracks) => tracks,
                Err(e) => {
                    eprintln!("{:?}", e);
                    return;
                }
            };
            let track_url = match { tracks.choose(&mut rng()) } {
                Some(track) => &track.external_urls.spotify,
                None => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, "曲が見つかりません").await
                    {
                        println!("Error sending message: {why:?}");
                    }

                    return;
                }
            };

            if let Err(why) = msg.channel_id.say(&ctx.http, track_url).await {
                println!("Error sending message: {why:?}");
            }
        } else {
            #[derive(Debug, Deserialize)]
            struct McpConfig {
                name: String,
                protocol: String,
                command: String,
                args: Vec<String>,
            }

            #[derive(Debug, Deserialize)]
            struct Config {
                mcp: McpConfig,
            }

            let content = tokio::fs::read_to_string("config.toml").await.unwrap();
            let config: Config = toml::from_str(&content).unwrap();
            let transport = rmcp::transport::TokioChildProcess::new(
                tokio::process::Command::new(config.mcp.command).args(config.mcp.args),
            )
            .unwrap();
            let mcp_manager = ().serve(transport).await.unwrap();
            struct McpToolAdaptor {
                tool: McpTool,
                server: ServerSink,
            }
            #[derive(serde::Deserialize, Serialize)]
            struct SearchArgs {
                genre: String,
            }

            impl RigTool for McpToolAdaptor {
                const NAME: &'static str = "spotify";

                type Error = Error;
                type Args = SearchArgs;
                type Output = CallToolResult;
                fn name(&self) -> String {
                    self.tool.name.to_string()
                }

                async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
                    rig::completion::ToolDefinition {
                        name: self.name(),
                        description: "ジャンルに基づいて音楽を探します".to_string(),
                        parameters: self.tool.schema_as_json_value(),
                    }
                }
                async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
                    let server = self.server.clone();
                    println!(
                        "{:?}",
                        serde_json::to_value(&args)
                            .unwrap()
                            .as_object()
                            .cloned()
                            .unwrap()
                    );
                    let call_mcp_tool_result = server
                        .call_tool(CallToolRequestParam {
                            name: self.tool.name.clone(),
                            arguments: Some(
                                serde_json::to_value(&args)
                                    .unwrap()
                                    .as_object()
                                    .cloned()
                                    .unwrap(),
                            ),
                        })
                        .await
                        .unwrap();

                    Ok(call_mcp_tool_result)
                }
            }

            let adaptor = McpToolAdaptor {
                tool: mcp_manager.list_all_tools().await.unwrap()[0].clone(),
                server: mcp_manager.peer().clone(),
            };

            let client = gemini::Client::from_env();
            let gemini = client.agent(GEMINI_2_0_FLASH).tool(adaptor).build();
            let response = gemini
                .prompt(msg_content)
                .await
                .expect("プロンプトの読み込みに失敗しました");

            if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
                println!("Error sending message: {why:?}");
            }
        }
    }
}

fn strip_mentions_msg_content(msg: &Message) -> String {
    let mut content = msg.content.clone();
    for user in &msg.mentions {
        let user_id = format!("<@{}>", user.id);
        content = content.replace(&user_id, "");
    }

    content.trim().to_string()
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKENの取得でエラーが発生しました");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Discordクライアントの作成でエラーが発生しました");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
