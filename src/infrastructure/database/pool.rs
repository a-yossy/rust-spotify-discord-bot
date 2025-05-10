use std::env;

use anyhow::Result;
use sqlx::{MySql, Pool as SqlxPool, mysql::MySqlPoolOptions};

pub struct Pool;

impl Pool {
    pub async fn get() -> Result<SqlxPool<MySql>> {
        let database_url = env::var("DATABASE_URL")?;
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        Ok(pool)
    }
}
