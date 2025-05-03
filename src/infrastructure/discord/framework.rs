use futures::StreamExt;
use once_cell::sync::Lazy;
use poise::{Framework, FrameworkContext};
use rig::OneOrMany;
use rig::message::{AssistantContent, Message, Text, UserContent};
use rig::streaming::StreamingChat;
use serenity::all::EditMessage;
use serenity::all::FullEvent;
use serenity::prelude::*;
use std::collections::HashMap;
use tokio::sync::Mutex;

use crate::{
    discord::commands::Commands,
    infrastructure::llm,
    types::discord::framework::{Data, Error},
};

static CONVERSATIONS: Lazy<Mutex<HashMap<u64, Vec<Message>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

async fn event_handler(
    ctx: &Context,
    event: &FullEvent,
    framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Message { new_message } => {
            if new_message.author.id == framework.bot_id {
                return Ok(());
            }

            let thread = new_message.channel(&ctx.http).await?;
            let is_in_thread = matches!(&thread,
                serenity::model::channel::Channel::Guild(guild_channel) if matches!(guild_channel.kind,
                    serenity::model::channel::ChannelType::PublicThread |
                    serenity::model::channel::ChannelType::PrivateThread |
                    serenity::model::channel::ChannelType::NewsThread
                )
            );
            if is_in_thread {
                let thread_members = thread.id().get_thread_members(&ctx.http).await?;
                if !thread_members
                    .iter()
                    .any(|thread_member| thread_member.user_id == framework.bot_id)
                {
                    return Ok(());
                }

                let mut conversations = CONVERSATIONS.lock().await;
                let history = conversations.entry(0).or_insert_with(Vec::new);

                let llm_agent = &data.llm_agent;

                let mut response_stream = llm_agent
                    .stream_chat(&new_message.content, history.to_vec())
                    .await?;
                let user_contest = UserContent::Text(Text {
                    text: new_message.content.clone(),
                });
                let user_message = Message::User {
                    content: OneOrMany::one(user_contest),
                };
                history.push(user_message);

                let mut assistant_text = String::new();
                let mut sent_message: Option<serenity::all::Message> = None;
                while let Some(chunk) = response_stream.next().await {
                    match chunk? {
                        rig::streaming::StreamingChoice::Message(text) => {
                            assistant_text.push_str(&text);
                            if let Some(ref mut msg_obj) = sent_message {
                                let builder = EditMessage::new().content(&assistant_text);
                                msg_obj.edit(ctx, builder).await?;
                            } else {
                                let message = thread.id().say(ctx.http(), &assistant_text).await?;
                                sent_message = Some(message);
                            }
                        }
                        rig::streaming::StreamingChoice::ToolCall(..) => {
                            todo!()
                        } // Ok(rig::streaming::StreamingChoice::ToolCall(name, _, param)) => {
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
                    }
                }
                let assistant_content = AssistantContent::Text(Text {
                    text: assistant_text,
                });
                let assistant_message = Message::Assistant {
                    content: OneOrMany::one(assistant_content),
                };
                history.push(assistant_message);
            }
        }
        _ => {}
    }

    Ok(())
}

pub fn get() -> Framework<Data, Error> {
    let options = poise::FrameworkOptions {
        commands: vec![
            Commands::ping(),
            Commands::random_music(),
            Commands::thread(),
        ],
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };
    let llm_agent = llm::client::get();

    poise::Framework::builder()
        .options(options)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data::new(llm_agent))
            })
        })
        .build()
}
