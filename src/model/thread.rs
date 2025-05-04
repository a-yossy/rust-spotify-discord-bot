use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::MySqlPool;

#[derive(Debug)]
pub struct Thread {
    pub id: u64,
    pub guild_id: u64,
    pub channel_id: u64,
    pub created_at: NaiveDateTime,
}

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

impl Thread {
    pub async fn insert(db_pool: &MySqlPool, input: &InsertInput) -> Result<Self> {
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
        .execute(db_pool)
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
        .fetch_one(db_pool)
        .await?;

        Ok(thread)
    }
}
