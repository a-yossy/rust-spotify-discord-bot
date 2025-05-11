use anyhow::Result;
use chrono::NaiveDateTime;
use rig::{
    OneOrMany,
    message::{AssistantContent, Message, Text},
};
use sqlx::{MySql, Transaction};

#[derive(Debug)]
pub struct AgentMessage {
    pub id: u64,
    pub message_id: u64,
    pub user_message_id: u64,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct InsertInput<'a> {
    pub user_message_id: u64,
    pub message_id: u64,
    pub content: &'a str,
}

impl<'a> InsertInput<'a> {
    pub fn new(user_message_id: u64, message_id: u64, content: &'a str) -> Self {
        Self {
            user_message_id,
            message_id,
            content,
        }
    }
}

impl From<AgentMessage> for Message {
    fn from(message: AgentMessage) -> Self {
        Message::Assistant {
            content: OneOrMany::one(AssistantContent::Text(Text {
                text: message.content,
            })),
        }
    }
}

impl AgentMessage {
    pub async fn insert(tx: &mut Transaction<'_, MySql>, input: &InsertInput<'_>) -> Result<Self> {
        let last_insert_id = sqlx::query!(
            r#"
                INSERT INTO
                    agent_messages (user_message_id, message_id, content)
                VALUES
                    (?, ?, ?)
            "#,
            input.user_message_id,
            input.message_id,
            input.content
        )
        .execute(&mut **tx)
        .await?
        .last_insert_id();

        let agent_message = sqlx::query_as!(
            Self,
            r#"
                SELECT
                    id, message_id, user_message_id, content, created_at, updated_at
                FROM
                    agent_messages
                WHERE
                    id = ?
            "#,
            last_insert_id
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(agent_message)
    }
}
