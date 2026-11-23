//! Boots the real app on a random port with a throwaway SQLite file, and launches headless Chrome.

use std::time::Duration;

use chromiumoxide::{Browser, BrowserConfig, Page};
use color_eyre::eyre::{Result, eyre};
use futures::StreamExt;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

pub struct TestServer {
    pub url: String,
    _dir: tempfile::TempDir,
}

pub async fn spawn_app() -> Result<TestServer> {
    let dir = tempfile::tempdir()?;
    let opts = SqliteConnectOptions::new()
        .filename(dir.path().join("test.db"))
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(opts)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(axum::serve(listener, hurs_app::build_app(pool)).into_future());
    Ok(TestServer {
        url: format!("http://{addr}"),
        _dir: dir,
    })
}

pub async fn browser(url: &str) -> Result<(Browser, Page)> {
    let config = BrowserConfig::builder()
        .no_sandbox()
        .build()
        .map_err(|e| eyre!("browser config: {e}"))?;
    let (browser, mut handler) = Browser::launch(config).await?;
    tokio::spawn(async move { while handler.next().await.is_some() {} });
    let page = browser.new_page(url).await?;
    page.wait_for_navigation().await?;
    Ok((browser, page))
}

/// Poll until `selector` matches `n` elements and no view transition is running, or fail after two seconds.
/// htmx swaps with `transitions:true`; clicks during the transition hit its overlay and are lost.
pub async fn wait_for_count(page: &Page, selector: &str, n: usize) -> Result<()> {
    for _ in 0..40 {
        let count = page
            .find_elements(selector)
            .await
            .map(|v| v.len())
            .unwrap_or(0);
        let settled = page
            .evaluate("!document.activeViewTransition")
            .await
            .ok()
            .and_then(|v| v.into_value::<bool>().ok())
            .unwrap_or(false);
        if count == n && settled {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Err(eyre!("expected {n} elements matching {selector}"))
}
