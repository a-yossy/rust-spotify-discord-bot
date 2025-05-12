use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::MySqlPool;

pub struct MusicSearchProgress {
    pub id: u32,
    pub music_genre_id: u32,
    pub position: u32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct UpsertInput {
    pub position: u32,
}

impl UpsertInput {
    pub fn new(position: u32) -> Self {
        Self { position }
    }
}

impl MusicSearchProgress {
    pub async fn find_by_music_genre_id(
        db_pool: &MySqlPool,
        music_genre_id: u32,
    ) -> Result<Option<Self>> {
        let result = sqlx::query_as!(
            MusicSearchProgress,
            r#"
                SELECT
                  id, music_genre_id, position, created_at, updated_at
                FROM
                  music_search_progresses
                WHERE
                  music_genre_id = ?
            "#,
            music_genre_id
        )
        .fetch_optional(db_pool)
        .await?;

        Ok(result)
    }

    pub async fn upsert(
        db_pool: &MySqlPool,
        music_genre_id: u32,
        input: &UpsertInput,
    ) -> Result<Self> {
        let id = sqlx::query!(
            r#"
                INSERT INTO
                  music_search_progresses (music_genre_id, position)
                VALUES
                  (?, ?)
                ON DUPLICATE KEY UPDATE
                  position = ?
            "#,
            music_genre_id,
            input.position,
            input.position
        )
        .execute(db_pool)
        .await?
        .last_insert_id();

        let music_search_progress = sqlx::query_as!(
            MusicSearchProgress,
            r#"
                SELECT
                  id, music_genre_id, position, created_at, updated_at
                FROM
                  music_search_progresses
                WHERE
                  id = ?
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;

        Ok(music_search_progress)
    }
}
