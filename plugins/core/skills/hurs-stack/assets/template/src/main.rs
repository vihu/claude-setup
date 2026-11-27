use std::{net::SocketAddr, str::FromStr, time::Duration};

use color_eyre::eyre::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=info".into()))
        .with(fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/app.db".into());
    let pool = connect(&db_url).await?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .wrap_err("migrations")?;

    let app = hurs_app::build_app(pool);
    let addr: SocketAddr = std::env::var("BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn connect(url: &str) -> Result<sqlx::SqlitePool> {
    let opts = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5))
        .foreign_keys(true);
    // Single pool, one process per volume. Move to a read pool + write pool if writes start contending.
    SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(opts)
        .await
        .wrap_err("connect sqlite")
}
