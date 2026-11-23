//! Happy-path browser tests. Ignored by default because they need Chrome. Run with `just test-browser`.

mod common;

use color_eyre::eyre::Result;

#[tokio::test]
#[ignore = "needs Chrome on PATH; run with `just test-browser`"]
async fn create_and_delete_item() -> Result<()> {
    let srv = common::spawn_app().await?;
    let (mut browser, page) = common::browser(&srv.url).await?;

    // Empty submit renders the inline error (422 swapped into the form).
    page.find_element("button[type=submit]")
        .await?
        .click()
        .await?;
    common::wait_for_count(&page, "input[name=title].input-error", 1).await?;

    // Valid submit appends a row via <hx-partial> and resets the form.
    page.find_element("input[name=title]")
        .await?
        .click()
        .await?
        .type_str("Milk")
        .await?;
    page.find_element("button[type=submit]")
        .await?
        .click()
        .await?;
    common::wait_for_count(&page, "#items tr", 1).await?;
    common::wait_for_count(&page, "input[name=title].input-error", 0).await?;

    // Delete removes the row. hx-confirm opens a dialog; auto-accept it.
    page.evaluate("window.confirm = () => true").await?;
    page.find_element("#items button").await?.click().await?;
    common::wait_for_count(&page, "#items tr", 0).await?;

    browser.close().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "needs Chrome on PATH; run with `just test-browser`"]
async fn screenshot_for_review() -> Result<()> {
    // Not an assertion. Produces target/screenshot.png for a human or agent to look at.
    let srv = common::spawn_app().await?;
    let (mut browser, page) = common::browser(&srv.url).await?;
    page.save_screenshot(
        chromiumoxide::page::ScreenshotParams::builder()
            .full_page(true)
            .build(),
        "target/screenshot.png",
    )
    .await?;
    browser.close().await?;
    Ok(())
}
