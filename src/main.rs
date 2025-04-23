use once_cell::sync::Lazy;
use rig::OneOrMany;
use rig::message::{AssistantContent, Message, Text, UserContent};
use rig::providers::anthropic::CLAUDE_3_7_SONNET;
use serenity::all::EditMessage;
use std::collections::HashMap;
use std::env;
use tokio::sync::Mutex;

use futures::StreamExt; // トレイトをスコープに入れる
use rig::embeddings::EmbeddingsBuilder;
use rig::providers::{anthropic, cohere};
use rig::streaming::StreamingChat;
use rig::tool::{ToolDyn as RigTool, ToolEmbeddingDyn, ToolSet};
use rig::vector_store::in_memory_store::InMemoryVectorStore;
use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParam, CallToolResult, Tool as McpTool};
use rmcp::service::ServerSink;
use serde::Deserialize;
use serenity::async_trait;
use serenity::model::channel::Message as SerenityMessage;
use serenity::prelude::*;

// 不要なimportを削除

pub fn convert_mcp_call_tool_result_to_string(result: CallToolResult) -> String {
    serde_json::to_string(&result).unwrap()
}

// グローバルな会話履歴（ユーザーIDごと）
static CONVERSATIONS: Lazy<Mutex<HashMap<u64, Vec<Message>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: SerenityMessage) {
        if !msg.mentions_me(&ctx.http).await.unwrap_or(false) {
            return;
        }

        let msg_content = strip_mentions_msg_content(&msg);

        let mut conversations = CONVERSATIONS.lock().await;
        let history = conversations.entry(0).or_insert_with(Vec::new);

        if msg_content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        } else if msg_content == "spotify" {
            // let access_token = match spotify::api::token::post().await {
            //     Ok(token) => token,
            //     Err(e) => {
            //         eprintln!("{:?}", e);
            //         return;
            //     }
            // };

            // let artists = match spotify::v1::me::following::get(&access_token).await {
            //     Ok(artists) => artists,
            //     Err(e) => {
            //         eprintln!("{:?}", e);
            //         return;
            //     }
            // };
            // let artist_id = match { artists.choose(&mut thread_rng()) } {
            //     Some(artist) => &artist.id,
            //     None => {
            //         if let Err(why) = msg
            //             .channel_id
            //             .say(&ctx.http, "アーティストが見つかりません")
            //             .await
            //         {
            //             println!("Error sending message: {why:?}");
            //         }
            //         return;
            //     }
            // };

            // let tracks = match spotify::v1::artists::top_tracks::get(artist_id, &access_token).await
            // {
            //     Ok(tracks) => tracks,
            //     Err(e) => {
            //         eprintln!("{:?}", e);
            //         return;
            //     }
            // };
            // let track_url = match { tracks.choose(&mut thread_rng()) } {
            //     Some(track) => &track.external_urls.spotify,
            //     None => {
            //         if let Err(why) = msg.channel_id.say(&ctx.http, "曲が見つかりません").await
            //         {
            //             println!("Error sending message: {why:?}");
            //         }

            //         return;
            //     }
            // };

            // if let Err(why) = msg.channel_id.say(&ctx.http, track_url).await {
            //     println!("Error sending message: {why:?}");
            // }
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

            impl RigTool for McpToolAdaptor {
                fn name(&self) -> String {
                    self.tool.name.to_string()
                }

                fn definition(
                    &self,
                    _prompt: String,
                ) -> std::pin::Pin<
                    Box<dyn Future<Output = rig::completion::ToolDefinition> + Send + Sync + '_>,
                > {
                    Box::pin(std::future::ready(rig::completion::ToolDefinition {
                        name: self.name(),
                        description: self.tool.description.to_string(),
                        parameters: self.tool.schema_as_json_value(),
                    }))
                }

                fn call(
                    &self,
                    args: String,
                ) -> std::pin::Pin<
                    Box<
                        dyn Future<Output = Result<String, rig::tool::ToolError>>
                            + Send
                            + Sync
                            + '_,
                    >,
                > {
                    let server = self.server.clone();
                    Box::pin(async move {
                        let call_mcp_tool_result = server
                            .call_tool(CallToolRequestParam {
                                name: self.tool.name.clone(),
                                arguments: serde_json::from_str(&args)
                                    .map_err(rig::tool::ToolError::JsonError)?,
                            })
                            .await
                            .map_err(|e| rig::tool::ToolError::ToolCallError(Box::new(e)))?;

                        Ok(convert_mcp_call_tool_result_to_string(call_mcp_tool_result))
                    })
                }
            }

            impl ToolEmbeddingDyn for McpToolAdaptor {
                fn context(&self) -> serde_json::Result<serde_json::Value> {
                    serde_json::to_value(self.tool.clone())
                }

                fn embedding_docs(&self) -> Vec<String> {
                    vec![self.tool.description.to_string()]
                }
            }

            let tools = mcp_manager.list_all_tools().await.unwrap();
            let mut tool_builder = ToolSet::builder();
            for tool in tools {
                let adaptor = McpToolAdaptor {
                    tool: tool.clone(),
                    server: mcp_manager.peer().clone(),
                };
                tool_builder = tool_builder.dynamic_tool(adaptor);
            }
            let tools = tool_builder.build();

            let client = anthropic::Client::from_env();
            // let embedding_model = client.embedding_model(GEMINI_2_0_FLASH);
            let cohere_client = cohere::Client::from_env();
            let embedding_model =
                cohere_client.embedding_model(cohere::EMBED_MULTILINGUAL_V3, "classification");

            let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
                .documents(tools.schemas().unwrap())
                .unwrap()
                .build()
                .await
                .unwrap();
            let store =
                InMemoryVectorStore::from_documents_with_id_f(embeddings, |f| f.name.clone());
            let index = store.index(embedding_model);
            let claude = client
                .agent(CLAUDE_3_7_SONNET)
                .max_tokens(25000)
                .preamble(
                    "あなたは音楽検索エージェントです。
                    次のことを考慮してください:
                        1. 会話の履歴に基づいて応答してください。
                        2. ユーザーの質問に答えるために、ツールを使用してください。
                        4. ツールから取得してきた情報を表示して会話を終えることを禁止します、ユーザーに次のアクションを促してください。
                ",
                )
                .dynamic_tools(20, index, tools)
                .build();

            // ストリーミング応答に変更
            let mut response_stream = claude
                .stream_chat(&msg_content, history.to_vec())
                .await
                .expect("プロンプトの読み込みに失敗しました");
            let user_contest = UserContent::Text(Text {
                text: msg_content.clone(),
            });
            let user_message = Message::User {
                content: OneOrMany::one(user_contest),
            };
            history.push(user_message);

            // Discordにストリーミングで送信
            let mut assistant_text = String::new();
            let mut sent_message: Option<serenity::model::channel::Message> = None;
            while let Some(chunk) = response_stream.next().await {
                // StreamExtトレイトのnextメソッドを直接使用
                match chunk {
                    Ok(rig::streaming::StreamingChoice::Message(text)) => {
                        assistant_text.push_str(&text);
                        // 1回目は新規送信、それ以降は編集
                        if let Some(ref mut msg_obj) = sent_message {
                            let builder = EditMessage::new().content(&assistant_text);
                            let _ = msg_obj.edit(&ctx.http, builder).await;
                        } else {
                            match msg.channel_id.say(&ctx.http, &assistant_text).await {
                                Ok(m) => sent_message = Some(m),
                                Err(e) => {
                                    println!("Error sending message: {e:?}");
                                    break;
                                }
                            }
                        }
                    }
                    Ok(rig::streaming::StreamingChoice::ToolCall(name, _, param)) => {
                        // ツールコールの通知をより視覚的にわかりやすく改善
                        let _ = msg
                            .channel_id
                            .say(
                                &ctx.http,
                                format!(
                                    "🛠️ **ツール呼び出し**: `{}` \n```json\n{}\n```",
                                    name, param
                                ),
                            )
                            .await;

                        // ツール結果も表示する
                        if let Ok(tool_result) = claude.tools.call(&name, param.to_string()).await {
                            let _ = msg
                                .channel_id
                                .say(
                                    &ctx.http,
                                    format!("🔍 **ツール結果**:\n```json\n{}\n```", tool_result),
                                )
                                .await;

                            // 後続の応答にツール結果を含める
                            assistant_text.push_str(&format!(
                                "\n\n【ツール `{}` の結果】\n{}",
                                name, tool_result
                            ));
                        }
                    }
                    Err(e) => {
                        let _ = msg
                            .channel_id
                            .say(&ctx.http, format!("[エラー: {e}]"))
                            .await;
                        break;
                    }
                }
            }
            // assistant_textを履歴に追加
            let assistant_content = AssistantContent::Text(Text {
                text: assistant_text.clone(),
            });
            let assistant_message = Message::Assistant {
                content: OneOrMany::one(assistant_content),
            };
            history.push(assistant_message);
            println!("{:?}", history);
        }
    }
}

fn strip_mentions_msg_content(msg: &SerenityMessage) -> String {
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
