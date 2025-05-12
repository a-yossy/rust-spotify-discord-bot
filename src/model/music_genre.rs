use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::MySqlPool;

pub struct MusicGenre {
    pub id: u32,
    pub name: String,
    pub search_key: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl MusicGenre {
    pub async fn find_all(db_pool: &MySqlPool) -> Result<Vec<Self>> {
        let genres = sqlx::query_as!(
            MusicGenre,
            r#"
                SELECT
                  id, name, search_key, created_at, updated_at
                FROM
                  music_genres
                ORDER BY
                  id ASC
            "#
        )
        .fetch_all(db_pool)
        .await?;

        Ok(genres)
    }
}
