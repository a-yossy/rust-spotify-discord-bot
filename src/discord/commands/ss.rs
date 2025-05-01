use crate::types::discord::framework::Error;
use once_cell::sync::Lazy;
use rig::OneOrMany;
use rig::message::{AssistantContent, Message, Text, UserContent};
use serenity::all::EditMessage;
use std::collections::HashMap;
use tokio::sync::Mutex;

use futures::StreamExt;
use rig::streaming::StreamingChat;

use super::Context;

pub async fn handle(ctx: Context<'_>, prompt: String) -> Result<(), Error> {
    ctx.defer().await?;
    static CONVERSATIONS: Lazy<Mutex<HashMap<u64, Vec<Message>>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    let mut conversations = CONVERSATIONS.lock().await;
    let history = conversations.entry(0).or_insert_with(Vec::new);

    let llm_agent = &ctx.data().llm_agent;

    let mut response_stream = llm_agent
        .stream_chat(&prompt, history.to_vec())
        .await
        .expect("プロンプトの読み込みに失敗しました");
    let user_contest = UserContent::Text(Text { text: prompt });
    let user_message = Message::User {
        content: OneOrMany::one(user_contest),
    };
    history.push(user_message);

    let mut assistant_text = String::new();
    let mut sent_message: Option<serenity::model::channel::Message> = None;
    while let Some(chunk) = response_stream.next().await {
        let _ = ctx.channel_id().broadcast_typing(&ctx.http()).await;
        match chunk {
            Ok(rig::streaming::StreamingChoice::Message(text)) => {
                assistant_text.push_str(&text);
                if let Some(ref mut msg_obj) = sent_message {
                    let builder = EditMessage::new().content(&assistant_text);
                    let _ = msg_obj.edit(&ctx.http(), builder).await;
                } else {
                    match ctx.channel_id().say(&ctx.http(), &assistant_text).await {
                        Ok(m) => sent_message = Some(m),
                        Err(e) => {
                            println!("Error sending message: {e:?}");
                            break;
                        }
                    }
                }
            }
            Ok(rig::streaming::StreamingChoice::ToolCall(..)) => {
                todo!()
            }
            // Ok(rig::streaming::StreamingChoice::ToolCall(name, _, param)) => {
            //     let _ = msg
            //         .channel_id
            //         .say(
            //             &ctx.http,
            //             format!(
            //                 "🛠️ **ツール呼び出し**: `{}` \n```json\n{}\n```",
            //                 name, param
            //             ),
            //         )
            //         .await;

            //     if let Ok(tool_result) = claude.tools.call(&name, param.to_string()).await {
            //         let _ = msg
            //             .channel_id
            //             .say(
            //                 &ctx.http,
            //                 format!("🔍 **ツール結果**:\n```json\n{}\n```", tool_result),
            //             )
            //             .await;

            //         assistant_text.push_str(&format!(
            //             "\n\n【ツール `{}` の結果】\n{}",
            //             name, tool_result
            //         ));
            //     }
            // }
            Err(e) => {
                let _ = ctx
                    .channel_id()
                    .say(&ctx.http(), format!("[エラー: {e}]"))
                    .await;
                break;
            }
        }
    }
    let assistant_content = AssistantContent::Text(Text {
        text: assistant_text,
    });
    let assistant_message = Message::Assistant {
        content: OneOrMany::one(assistant_content),
    };
    history.push(assistant_message);

    Ok(())
}
