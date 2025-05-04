use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::MySqlPool;

#[derive(Debug)]
pub struct AgentMessage {
    pub id: u64,
    pub user_message_id: u64,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct InsertInput {
    pub user_message_id: u64,
    pub content: String,
}

impl InsertInput {
    pub fn new(user_message_id: u64, content: String) -> Self {
        Self {
            user_message_id,
            content,
        }
    }
}

impl AgentMessage {
    pub async fn insert(db_pool: &MySqlPool, input: &InsertInput) -> Result<Self> {
        let last_insert_id = sqlx::query!(
            r#"
                INSERT INTO
                    agent_messages (user_message_id, content)
                VALUES
                    (?, ?)
            "#,
            input.user_message_id,
            input.content
        )
        .execute(db_pool)
        .await?
        .last_insert_id();

        let agent_message = sqlx::query_as!(
            Self,
            r#"
                SELECT
                    id, user_message_id, content, created_at, updated_at
                FROM
                    agent_messages
                WHERE
                    id = ?
            "#,
            last_insert_id
        )
        .fetch_one(db_pool)
        .await?;

        Ok(agent_message)
    }
}
