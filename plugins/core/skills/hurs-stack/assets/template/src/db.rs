//! All SQL lives here. One function per query, checked at compile time by sqlx.
//! After editing a query: `just prepare` to refresh `.sqlx/` for offline builds.

use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Item {
    pub id: i64,
    pub title: String,
    pub created_at: String,
}

pub async fn list_items(pool: &SqlitePool) -> sqlx::Result<Vec<Item>> {
    sqlx::query_as!(
        Item,
        "SELECT id, title, created_at FROM items ORDER BY id DESC"
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_item(pool: &SqlitePool, title: &str) -> sqlx::Result<Item> {
    sqlx::query_as!(
        Item,
        "INSERT INTO items (title) VALUES (?) RETURNING id, title, created_at",
        title
    )
    .fetch_one(pool)
    .await
}

pub async fn delete_item(pool: &SqlitePool, id: i64) -> sqlx::Result<u64> {
    let r = sqlx::query!("DELETE FROM items WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected())
}
