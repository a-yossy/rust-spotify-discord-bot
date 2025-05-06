use anyhow::Result;
use chrono::NaiveDateTime;
use rig::OneOrMany;
use rig::message::{AssistantContent, Message as RigMessage, Text, UserContent};
use sqlx::{MySql, MySqlPool, Transaction};

#[derive(Debug)]
pub struct Thread {
    pub id: u64,
    pub guild_id: u64,
    pub channel_id: u64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct InsertInput {
    guild_id: u64,
    channel_id: u64,
}

impl InsertInput {
    pub fn new(guild_id: u64, channel_id: u64) -> Self {
        Self {
            guild_id,
            channel_id,
        }
    }
}

#[derive(sqlx::Type, Debug)]
#[sqlx(rename_all = "lowercase")]
pub enum MessageSender {
    User,
    Agent,
}

#[derive(Debug)]
pub struct Message {
    pub sender: MessageSender,
    pub content: String,
    pub created_at: NaiveDateTime,
}

impl From<Message> for RigMessage {
    fn from(message: Message) -> Self {
        match message.sender {
            MessageSender::Agent => RigMessage::Assistant {
                content: OneOrMany::one(AssistantContent::Text(Text {
                    text: message.content,
                })),
            },
            MessageSender::User => RigMessage::User {
                content: OneOrMany::one(UserContent::Text(Text {
                    text: message.content,
                })),
            },
        }
    }
}

impl Thread {
    pub async fn insert(tx: &mut Transaction<'_, MySql>, input: &InsertInput) -> Result<Self> {
        let last_insert_id = sqlx::query!(
            r#"
                INSERT INTO
                    threads (guild_id, channel_id)
                VALUES
                    (?, ?)
            "#,
            input.guild_id,
            input.channel_id
        )
        .execute(&mut **tx)
        .await?
        .last_insert_id();

        let thread = sqlx::query_as!(
            Self,
            r#"
                SELECT
                    id, guild_id, channel_id, created_at
                FROM
                    threads
                WHERE
                    id = ?
            "#,
            last_insert_id
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(thread)
    }

    pub async fn find_by_channel_id(db_pool: &MySqlPool, channel_id: u64) -> Result<Option<Self>> {
        let thread = sqlx::query_as!(
            Self,
            r#"
                SELECT
                    id, guild_id, channel_id, created_at
                FROM
                    threads
                WHERE
                    channel_id = ?
                LIMIT 1
            "#,
            channel_id
        )
        .fetch_optional(db_pool)
        .await?;

        Ok(thread)
    }

    pub async fn find_messages_by_thread_id(
        db_pool: &MySqlPool,
        thread_id: u64,
    ) -> Result<Vec<Message>> {
        let messages = sqlx::query_as!(
            Message,
            r#"
                SELECT
                    'user' as "sender: MessageSender", content, created_at
                FROM
                    user_messages
                WHERE
                    thread_id = ?
                UNION ALL
                    SELECT
                        'agent' as "sender: MessageSender", agent_messages.content, agent_messages.created_at
                    FROM
                        agent_messages
                    JOIN
                        user_messages ON agent_messages.user_message_id = user_messages.id
                ORDER BY
                    created_at ASC
            "#,
            thread_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(messages)
    }
}
