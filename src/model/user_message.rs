use anyhow::Result;
use chrono::NaiveDateTime;
use rig::{
    OneOrMany,
    message::{Message, Text, UserContent},
};
use sqlx::{MySql, Transaction};

#[derive(Debug, Clone)]
pub struct UserMessage {
    pub id: u64,
    pub thread_id: u64,
    pub message_id: u64,
    pub user_id: u64,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct InsertInput<'a> {
    pub thread_id: u64,
    pub message_id: u64,
    pub user_id: u64,
    pub content: &'a str,
}

impl<'a> InsertInput<'a> {
    pub fn new(thread_id: u64, message_id: u64, user_id: u64, content: &'a str) -> Self {
        Self {
            thread_id,
            message_id,
            user_id,
            content,
        }
    }
}

impl From<UserMessage> for Message {
    fn from(message: UserMessage) -> Self {
        Message::User {
            content: OneOrMany::one(UserContent::Text(Text {
                text: message.content,
            })),
        }
    }
}

impl UserMessage {
    pub async fn insert(tx: &mut Transaction<'_, MySql>, input: &InsertInput<'_>) -> Result<Self> {
        let last_insert_id = sqlx::query!(
            r#"
                INSERT INTO
                    user_messages (thread_id, message_id, user_id, content)
                VALUES
                    (?, ?, ?, ?)
            "#,
            input.thread_id,
            input.message_id,
            input.user_id,
            input.content
        )
        .execute(&mut **tx)
        .await?
        .last_insert_id();

        let user_message = sqlx::query_as!(
            Self,
            r#"
                SELECT
                    id, thread_id, message_id, user_id, content, created_at, updated_at
                FROM
                    user_messages
                WHERE
                    id = ?
            "#,
            last_insert_id
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(user_message)
    }
}
