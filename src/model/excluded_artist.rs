use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::MySqlPool;

pub struct InsertInput {
    id: String,
    name: String,
}

impl InsertInput {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }
}

pub struct ExcludedArtist {
    pub id: String,
    pub name: String,
    pub created_at: NaiveDateTime,
}

impl ExcludedArtist {
    pub async fn find_by_ids(db_pool: &MySqlPool, ids: &[String]) -> Result<Vec<Self>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let excluded_artists = sqlx::query_as!(
            Self,
            r#"
              SELECT
                id,
                name,
                created_at
              FROM
                excluded_artists
              WHERE
                id IN ( ? )
              ORDER BY
                id DESC
            "#,
            &ids.join(",")
        )
        .fetch_all(db_pool)
        .await?;

        Ok(excluded_artists)
    }

    pub async fn insert(db_pool: &MySqlPool, input: &InsertInput) -> Result<Self> {
        let id = sqlx::query!(
            r#"
              INSERT INTO
                excluded_artists (id, name)
              VALUES
                (?, ?)
            "#,
            input.id,
            input.name
        )
        .execute(db_pool)
        .await?
        .last_insert_id();

        let excluded_artist = sqlx::query_as!(
            Self,
            r#"
              SELECT
                id, name, created_at
              FROM
                excluded_artists
              WHERE
                id = ?
            "#,
            id
        )
        .fetch_one(db_pool)
        .await?;

        Ok(excluded_artist)
    }
}
