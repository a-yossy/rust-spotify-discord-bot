use crate::model::agent_message::{self, AgentMessage};
use crate::model::thread::{MessageSender, Thread};
use crate::model::user_message::{self, UserMessage};
use crate::types::discord::framework::{Data, Error};
use futures::StreamExt;
use poise::FrameworkContext;
use rig::OneOrMany;
use rig::message::{AssistantContent, Message as RigMessage, Text, UserContent};
use rig::streaming::StreamingChat;
use serenity::all::{Channel, EditMessage, ThreadMember};
use serenity::all::{FullEvent, UserId};
use serenity::prelude::*;
use sqlx::MySqlPool;

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

        let llm_agent = &data.llm_agent;
        let chat_history = get_chat_history(db_pool, thread.id).await?;
        let mut response_stream = llm_agent
            .stream_chat(&new_message.content, chat_history)
            .await?;

        let mut agent_text = String::new();
        let mut agent_message = channel.id().say(ctx.http(), "生成中...").await?;
        while let Some(chunk) = response_stream.next().await {
            match chunk? {
                rig::streaming::StreamingChoice::Message(text) => {
                    if agent_text.len() + text.len() < 4000 {
                        agent_text.push_str(&text);
                        let builder = EditMessage::new().content(&agent_text);
                        agent_message.edit(ctx, builder).await?;
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

        let user_message_input = user_message::InsertInput::new(
            thread.id,
            new_message.id.get(),
            new_message.author.id.get(),
            new_message.content.clone(),
        );
        let user_message = UserMessage::insert(db_pool, &user_message_input).await?;
        let agent_message_input =
            agent_message::InsertInput::new(user_message.id, agent_message.id.get(), agent_text);
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

async fn get_chat_history(db_pool: &MySqlPool, thread_id: u64) -> Result<Vec<RigMessage>, Error> {
    Ok(Thread::find_messages_by_thread_id(db_pool, thread_id)
        .await?
        .into_iter()
        .map(|message| match message.sender {
            MessageSender::Agent => RigMessage::Assistant {
                content: OneOrMany::one(AssistantContent::Text(Text {
                    text: message.content,
                })),
            },
            MessageSender::User => RigMessage::User {
                content: OneOrMany::one(UserContent::Text(Text {
                    text: message.content.clone(),
                })),
            },
        })
        .collect())
}
