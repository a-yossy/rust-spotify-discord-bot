use futures::StreamExt;
use once_cell::sync::Lazy;
use poise::FrameworkContext;
use rig::OneOrMany;
use rig::message::{AssistantContent, Message as RigMessage, Text, UserContent};
use rig::streaming::StreamingChat;
use serenity::all::{Channel, EditMessage, ThreadMember};
use serenity::all::{FullEvent, UserId};
use serenity::prelude::*;
use std::collections::HashMap;
use tokio::sync::Mutex;

use crate::model::agent_message::{self, AgentMessage};
use crate::model::thread::Thread;
use crate::model::user_message::{self, UserMessage};
use crate::types::discord::framework::{Data, Error};

static CONVERSATIONS: Lazy<Mutex<HashMap<u64, Vec<RigMessage>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub struct Messages;

impl Messages {
    pub async fn handle(
        ctx: &Context,
        _event: &FullEvent,
        framework: FrameworkContext<'_, Data, Error>,
        data: &Data,
        new_message: &serenity::all::Message,
    ) -> Result<(), Error> {
        let bot_id = framework.bot_id;
        if is_message_from_bot(&bot_id, new_message) {
            return Ok(());
        }

        let channel = new_message.channel(&ctx.http).await?;
        if !is_thread(&channel) {
            return Ok(());
        }

        let thread_members = channel.id().get_thread_members(&ctx.http).await?;
        if !is_bot_in_thread(&bot_id, &thread_members) {
            return Ok(());
        }

        let db_pool = &data.db_pool;
        let channel_id = channel.id().get();
        let thread = match Thread::find_by_channel_id(db_pool, channel_id).await? {
            Some(thread) => thread,
            None => {
                return Ok(());
            }
        };

        let mut conversations = CONVERSATIONS.lock().await;
        let history = conversations.entry(0).or_insert_with(Vec::new);

        let llm_agent = &data.llm_agent;

        let mut response_stream = llm_agent
            .stream_chat(&new_message.content, history.to_vec())
            .await?;

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
                        let message = channel.id().say(ctx.http(), &assistant_text).await?;
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

        let user_contest = UserContent::Text(Text {
            text: new_message.content.clone(),
        });
        let user_message = RigMessage::User {
            content: OneOrMany::one(user_contest),
        };
        history.push(user_message);
        let assistant_content = AssistantContent::Text(Text {
            text: assistant_text.clone(),
        });
        let assistant_message = RigMessage::Assistant {
            content: OneOrMany::one(assistant_content),
        };
        history.push(assistant_message);

        let user_message_input = user_message::InsertInput::new(
            thread.id,
            new_message.id.get(),
            new_message.author.id.get(),
            new_message.content.clone(),
        );
        let user_message = UserMessage::insert(db_pool, &user_message_input).await?;
        let agent_message_input = agent_message::InsertInput::new(user_message.id, assistant_text);
        AgentMessage::insert(db_pool, &agent_message_input).await?;

        Ok(())
    }
}

fn is_message_from_bot(bot_id: &UserId, new_message: &serenity::all::Message) -> bool {
    *bot_id == new_message.author.id
}

fn is_thread(thread: &Channel) -> bool {
    matches!(thread,
        serenity::model::channel::Channel::Guild(guild_channel) if matches!(guild_channel.kind,
            serenity::model::channel::ChannelType::PublicThread |
            serenity::model::channel::ChannelType::PrivateThread |
            serenity::model::channel::ChannelType::NewsThread
        )
    )
}

fn is_bot_in_thread(bot_id: &UserId, thread_members: &[ThreadMember]) -> bool {
    thread_members
        .iter()
        .any(|thread_member| thread_member.user_id == *bot_id)
}
