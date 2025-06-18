use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use anyhow::Result;

pub async fn create_pool(database_url: &str) -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    
    // Optional: Run migrations
    // sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
} 