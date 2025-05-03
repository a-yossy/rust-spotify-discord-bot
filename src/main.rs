use once_cell::sync::Lazy;
use rig::OneOrMany;
use rig::message::{AssistantContent, Message, Text, UserContent};
use rig::providers::gemini::completion::GEMINI_2_0_FLASH;
use serenity::all::EditMessage;
use ss_discord_bot::infrastructure::{discord, llm};
use std::clone;
use std::collections::HashMap;
use tokio::sync::Mutex;

use futures::StreamExt;
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

// pub fn convert_mcp_call_tool_result_to_string(result: CallToolResult) -> String {
//     serde_json::to_string(&result).unwrap()
// }

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: SerenityMessage) {
        let thread = msg
            .channel(&ctx.http)
            .await
            .ok()
            .unwrap()
            .id()
            .get_thread_members(&ctx.http)
            .await;
        println!("{:?}", thread);
        let is_in_thread = msg.thread.is_some()
            || msg
                .channel(&ctx.http)
                .await
                .ok()
                .and_then(|ch| match ch {
                    serenity::model::channel::Channel::Guild(guild_channel) => Some(
                        guild_channel.kind == serenity::model::channel::ChannelType::PublicThread
                            || guild_channel.kind
                                == serenity::model::channel::ChannelType::PrivateThread
                            || guild_channel.kind
                                == serenity::model::channel::ChannelType::NewsThread,
                    ),
                    _ => None,
                })
                .unwrap_or(false);
        if !is_in_thread {
            return;
        }

        let bot_id = ctx.cache.current_user().id;
        println!("{:?}", bot_id);
        // let is_bot_in_thread = if let Some(thread) = &msg.thread {
        //     thread.members(cache)
        // }
        return;

        // let msg_content = strip_mentions_msg_content(&msg);
        msg.channel_id.say(ctx.http, "こんにちは").await.unwrap();
        // #[derive(Debug, Deserialize)]
        // struct McpConfig {
        //     name: String,
        //     protocol: String,
        //     command: String,
        //     args: Vec<String>,
        // }

        // #[derive(Debug, Deserialize)]
        // struct Config {
        //     mcp: McpConfig,
        // }

        // let content = tokio::fs::read_to_string("config.toml").await.unwrap();
        // let config: Config = toml::from_str(&content).unwrap();
        // let transport = rmcp::transport::TokioChildProcess::new(
        //     tokio::process::Command::new(config.mcp.command).args(config.mcp.args),
        // )
        // .unwrap();
        // let mcp_manager = ().serve(transport).await.unwrap();
        // struct McpToolAdaptor {
        //     tool: McpTool,
        //     server: ServerSink,
        // }

        // impl RigTool for McpToolAdaptor {
        //     fn name(&self) -> String {
        //         self.tool.name.to_string()
        //     }

        //     fn definition(
        //         &self,
        //         _prompt: String,
        //     ) -> std::pin::Pin<
        //         Box<dyn Future<Output = rig::completion::ToolDefinition> + Send + Sync + '_>,
        //     > {
        //         Box::pin(std::future::ready(rig::completion::ToolDefinition {
        //             name: self.name(),
        //             description: self.tool.description.to_string(),
        //             parameters: self.tool.schema_as_json_value(),
        //         }))
        //     }

        //     fn call(
        //         &self,
        //         args: String,
        //     ) -> std::pin::Pin<
        //         Box<dyn Future<Output = Result<String, rig::tool::ToolError>> + Send + Sync + '_>,
        //     > {
        //         let server = self.server.clone();
        //         Box::pin(async move {
        //             let call_mcp_tool_result = server
        //                 .call_tool(CallToolRequestParam {
        //                     name: self.tool.name.clone(),
        //                     arguments: serde_json::from_str(&args)
        //                         .map_err(rig::tool::ToolError::JsonError)?,
        //                 })
        //                 .await
        //                 .map_err(|e| rig::tool::ToolError::ToolCallError(Box::new(e)))?;

        //             Ok(convert_mcp_call_tool_result_to_string(call_mcp_tool_result))
        //         })
        //     }
        // }

        // impl ToolEmbeddingDyn for McpToolAdaptor {
        //     fn context(&self) -> serde_json::Result<serde_json::Value> {
        //         serde_json::to_value(self.tool.clone())
        //     }

        //     fn embedding_docs(&self) -> Vec<String> {
        //         vec![self.tool.description.to_string()]
        //     }
        // }

        // let tools = mcp_manager.list_all_tools().await.unwrap();
        // let mut tool_builder = ToolSet::builder();
        // for tool in tools {
        //     let adaptor = McpToolAdaptor {
        //         tool: tool.clone(),
        //         server: mcp_manager.peer().clone(),
        //     };
        //     tool_builder = tool_builder.dynamic_tool(adaptor);
        // }
        // let tools = tool_builder.build();

        // let embedding_model = client.embedding_model(GEMINI_2_0_FLASH);
        // let cohere_client = cohere::Client::from_env();
        // let embedding_model =
        //     cohere_client.embedding_model(cohere::EMBED_MULTILINGUAL_V3, "classification");

        // let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        //     .documents(tools.schemas().unwrap())
        //     .unwrap()
        //     .build()
        //     .await
        //     .unwrap();
        // let store = InMemoryVectorStore::from_documents_with_id_f(embeddings, |f| f.name.clone());
        // let index = store.index(embedding_model);

        // static CONVERSATIONS: Lazy<Mutex<HashMap<u64, Vec<Message>>>> =
        //     Lazy::new(|| Mutex::new(HashMap::new()));

        // let mut conversations = CONVERSATIONS.lock().await;
        // let history = conversations.entry(0).or_insert_with(Vec::new);

        // let llm_agent = &ctx.data().llm_agent;

        // let mut response_stream = llm_agent.stream_chat(&prompt, history.to_vec()).await?;
        // let user_contest = UserContent::Text(Text { text: prompt });
        // let user_message = Message::User {
        //     content: OneOrMany::one(user_contest),
        // };
        // history.push(user_message);

        // let mut assistant_text = String::new();
        // let mut sent_message: Option<serenity::all::Message> = None;
        // while let Some(chunk) = response_stream.next().await {
        //     match chunk? {
        //         rig::streaming::StreamingChoice::Message(text) => {
        //             assistant_text.push_str(&text);
        //             if let Some(ref mut msg_obj) = sent_message {
        //                 let builder = EditMessage::new().content(&assistant_text);
        //                 msg_obj.edit(ctx, builder).await?;
        //             } else {
        //                 let message = thread.id.say(ctx.http(), &assistant_text).await?;
        //                 sent_message = Some(message);
        //             }
        //         }
        //         rig::streaming::StreamingChoice::ToolCall(..) => {
        //             todo!()
        //         } // Ok(rig::streaming::StreamingChoice::ToolCall(name, _, param)) => {
        //           //     let _ = msg
        //           //         .channel_id
        //           //         .say(
        //           //             &ctx.http,
        //           //             format!(
        //           //                 "🛠️ **ツール呼び出し**: `{}` \n```json\n{}\n```",
        //           //                 name, param
        //           //             ),
        //           //         )
        //           //         .await;

        //           //     if let Ok(tool_result) = claude.tools.call(&name, param.to_string()).await {
        //           //         let _ = msg
        //           //             .channel_id
        //           //             .say(
        //           //                 &ctx.http,
        //           //                 format!("🔍 **ツール結果**:\n```json\n{}\n```", tool_result),
        //           //             )
        //           //             .await;

        //           //         assistant_text.push_str(&format!(
        //           //             "\n\n【ツール `{}` の結果】\n{}",
        //           //             name, tool_result
        //           //         ));
        //           //     }
        //           // }
        //     }
        // }
        // let assistant_content = AssistantContent::Text(Text {
        //     text: assistant_text,
        // });
        // let assistant_message = Message::Assistant {
        //     content: OneOrMany::one(assistant_content),
        // };
        // history.push(assistant_message);
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let framework = discord::framework::get();
    let mut client = discord::client::get(Handler, framework).await;
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
