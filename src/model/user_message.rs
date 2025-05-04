use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::MySqlPool;

pub struct UserMessage {
    pub id: u64,
    pub thread_id: u64,
    pub message_id: u64,
    pub user_id: u64,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct InsertInput {
    pub thread_id: u64,
    pub message_id: u64,
    pub user_id: u64,
    pub content: String,
}

impl InsertInput {
    pub fn new(thread_id: u64, message_id: u64, user_id: u64, content: String) -> Self {
        Self {
            thread_id,
            message_id,
            user_id,
            content,
        }
    }
}

impl UserMessage {
    pub async fn insert(db_pool: &MySqlPool, input: &InsertInput) -> Result<Self> {
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
        .execute(db_pool)
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
        .fetch_one(db_pool)
        .await?;

        Ok(user_message)
    }
}
