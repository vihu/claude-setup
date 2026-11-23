# Handler and view patterns

All examples assume the template's `App` state, `AppError`, `HxRequest`, `Hx` response builder, and `views::layout::page`.

## 1. Page that is also a partial

One route serves both the full page (direct navigation, refresh, `hx-boost`) and the fragment (htmx request).

```rust
pub async fn index(State(app): State<App>, hx: HxRequest) -> Result<Response, AppError> {
    let items = db::list_items(&app.pool).await?;
    let body = views::items::list(&items);
    Ok(if hx.is_partial() {
        body.into_response()
    } else {
        layout::page("Items", body).into_response()
    })
}
```

`is_partial()` reads `HX-Request-Type: partial`. A boosted navigation sends `full`, so it gets the layout. Set
`Vary: HX-Request-Type` (the `Hx` builder does this) so caches do not mix the two.

## 2. Create with validation (422)

```rust
#[derive(Deserialize)]
pub struct NewItem { pub title: String }

pub async fn create(State(app): State<App>, Form(f): Form<NewItem>) -> Result<Response, AppError> {
    let title = f.title.trim();
    if title.is_empty() {
        let form = views::items::form(Some(&f), Some("Title is required"));
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, form).into_response());
    }
    let item = db::insert_item(&app.pool, title).await?;
    Ok(Hx::new()
        .trigger("itemCreated")
        .render(html! {
            (views::items::form(None, None))
            hx-partial hx-target="#items" hx-swap="beforeend" { (views::items::row(&item)) }
        }))
}
```

Markup side:

```rust
pub fn form(prev: Option<&NewItem>, error: Option<&str>) -> Markup {
    html! {
        form #item-form hx-post="/items" hx-swap="outerHTML" hx-disable="find button" {
            fieldset .fieldset {
                label .label for="title" { "Title" }
                input #title .input .w-full .input-error[error.is_some()] name="title"
                    value=[prev.map(|p| p.title.as_str())] autofocus;
                @if let Some(e) = error { p .text-error.text-sm { (e) } }
            }
            button .btn.btn-primary type="submit" { "Add" span .loading.loading-spinner.htmx-indicator {} }
        }
    }
}
```

The form targets itself with `outerHTML`, so a 422 replaces it with the error version and a 200 replaces it with a clean
form plus an `<hx-partial>` that appends the new row. No `hx-status` needed because the target is the same either way.
Use `hx-status:422="target:#errors"` only when errors should land somewhere other than the main target.

Custom element tags in maud: `hx-partial` is written as a plain tag name; maud allows hyphenated element names.

## 3. Edit in place

Display view carries `hx-get="/items/{id}/edit" hx-swap="outerHTML" hx-trigger="click"`. The edit handler returns a form
with `hx-put="/items/{id}" hx-swap="outerHTML"` and a cancel button with `hx-get="/items/{id}" hx-swap="outerHTML"`.
The update handler returns the display view. Three handlers, one row id, no page reload.

## 4. Delete with confirm

```rust
button .btn.btn-ghost.btn-xs hx-delete=(format!("/items/{}", item.id))
    hx-target="closest tr" hx-swap="delete" hx-confirm="Delete this item?" { "Delete" }
```

Handler returns `StatusCode::OK` with an empty body; `hx-swap="delete"` removes the row regardless of response.

## 5. Server-pushed refresh

When an action elsewhere should refresh a list, return `HX-Trigger: itemsChanged` from that action's handler and give the
list `hx-get="/items" hx-trigger="itemsChanged from:body"`. The list re-fetches itself. Use this over returning the whole
list from unrelated handlers.

## 6. Redirect after POST

For "create then go to detail page": `Hx::new().location("/items/42")` sends `HX-Location` (ajax navigation, keeps layout)
for htmx requests, and the `Hx` builder falls back to a 303 `Location` when the request was not from htmx.

## 7. Alpine inside htmx content

```rust
div x-data="{ open: false }" {
    button .btn @click="open = !open" { "Options" }
    div x-show="open" x-cloak { ... }
}
```

Rules: `x-data` on the smallest wrapper. `x-cloak` on anything initially hidden, with `[x-cloak]{display:none}` in CSS
(the template has it). After an htmx swap, `hx-alpine-compat` re-initializes Alpine on the new subtree; do not call
`Alpine.initTree` yourself. Morph swaps keep Alpine state; `innerHTML` swaps reset it, which is sometimes what you want.

## 8. Loading indicators

`hx-indicator="#spin"` plus an element with class `htmx-indicator`. daisyUI: `span .loading.loading-spinner.htmx-indicator`.
Inside a button, no `hx-indicator` is needed: the button gets `htmx-request` while in flight, and the nested indicator
shows. Add the `transition-delay` CSS so fast requests never flash the spinner (template `input.css` has it).

## 9. Layout

```rust
pub fn page(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" data-theme="light" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="htmx-config" content="defaultSwap:innerHTML, transitions:true";
                title { (title) }
                link rel="stylesheet" href="/static/app.css";
                script src="/static/js/htmx.min.js" {}
                script src="/static/js/hx-alpine-compat.js" {}
                script src="/static/js/hx-live.js" {}
                script defer src="/static/js/alpine.min.js" {}
            }
            body .min-h-screen.bg-base-200 hx-boost="true" {
                (nav())
                main .container.mx-auto.p-4 { (body) }
            }
        }
    }
}
```

`hx-boost` on body turns ordinary links and forms into partial navigations. Handlers still render full pages for those
because `HX-Request-Type` is `full`.

## 10. Browser test

```rust
#[tokio::test]
#[ignore = "needs Chrome; run with `just test-browser`"]
async fn create_item_happy_path() -> color_eyre::Result<()> {
    let srv = common::spawn_app().await?;              // boots build_app on 127.0.0.1:0 with a temp db
    let (browser, page) = common::browser(&srv.url).await?;
    page.find_element("input[name=title]").await?.click().await?.type_str("Milk").await?;
    page.find_element("button[type=submit]").await?.click().await?;
    page.wait_for_navigation().await?;                 // or poll until the row exists
    let rows = page.find_elements("#items tr").await?;
    assert_eq!(rows.len(), 1);
    browser.close().await?;
    Ok(())
}
```

Prefer asserting DOM state after the happy path over screenshot diffs. Screenshots are for the agent to look at
(`page.save_screenshot(...)`), not for assertions.

## 11. Database access

```rust
pub async fn list_items(pool: &SqlitePool) -> sqlx::Result<Vec<Item>> {
    sqlx::query_as!(Item, "SELECT id, title, created_at FROM items ORDER BY id DESC").fetch_all(pool).await
}

pub async fn insert_item(pool: &SqlitePool, title: &str) -> sqlx::Result<Item> {
    sqlx::query_as!(Item, "INSERT INTO items (title) VALUES (?) RETURNING id, title, created_at", title)
        .fetch_one(pool).await
}
```

`query_as!` needs `DATABASE_URL` at compile time or a committed `.sqlx/` directory. `just prepare` regenerates it.
SQLite integer primary keys come back as `i64`; timestamps stored as `TEXT` in RFC 3339 map to `String` unless the
`chrono` feature is on.
