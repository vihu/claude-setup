---
name: hurs-stack
description: "Build and extend web apps on the HURS stack (Hypermedia, Unix, Rust, SQLite): axum + maud + htmx 4 + Alpine.js + Tailwind/daisyUI + sqlx/SQLite, shipped as one static binary. Use this whenever the user wants a new Rust web app, a server-rendered page, an htmx partial, a daisyUI form, a sqlx migration, a browser test, or a deploy for a small-to-medium Rust app. Also use it when they say 'scratch app', 'HUGS but in Rust', 'hypermedia app', 'no React', 'single binary', or mention htmx, maud, daisyUI, or SQLite in a Rust project, even if they do not name the stack."
---

# HURS stack

Server renders HTML. htmx swaps fragments. Alpine handles ephemeral UI state. SQLite on a persistent disk. One static binary.
The constraint is the point: one language, one build tool, no client framework, fast feedback for the agent.

## Pick the right move

| Situation | Do this |
|---|---|
| New project | Copy `assets/template/` into the target dir, rename `hurs-app` and `hurs_app` in every file `grep -rl "hurs.app"` lists, run `just setup`, then `just check`. |
| Adding a page or partial | Read `references/patterns.md` first. Follow the handler shapes there. |
| Writing any `hx-*` attribute | Read `references/htmx4.md`. htmx 4 differs from htmx 2 in ways that silently break (inheritance, headers, status codes). |
| Picking or upgrading a crate | Read `references/crates.md`. Versions there were verified on 2026-10-18; re-check with `cargo search <crate> --limit 1` before editing `Cargo.toml`. |
| Verifying a page looks right | Run `just test-browser` or write a `tests/browser.rs` case. Do not guess from the HTML; look at the DOM. |

## Non-negotiables

1. Endpoints return HTML fragments via `maud`, not JSON. A JSON endpoint is a deliberate exception and gets `utoipa` annotations.
2. htmx owns anything that touches the server. Alpine owns state that never leaves the browser (open/closed, selected tab, draft text). Never wire both to the same piece of data.
3. Validation errors return `422` with the re-rendered form. The form carries `hx-status:422="target:#form-errors"` or re-renders itself with `hx-swap="outerHTML"`.
4. Every multi-region update uses `<hx-partial>` tags, not `hx-swap-oob`. One response, several targets, explicit.
5. Parent attributes that children must see use `:inherited` (`hx-target:inherited="#out"`). Without it, htmx 4 children see nothing.
6. SQL lives in `src/db.rs` behind `sqlx::query!` / `query_as!` so it is checked at compile time. Run `cargo sqlx prepare` after changing queries so offline builds keep working.
7. Migrations are plain SQL in `migrations/`, embedded with `sqlx::migrate!()`, run at startup. Never edit a migration that has shipped; add a new one.
8. Assets (`app.css`, `htmx.min.js`, `alpine.min.js`, `hx-alpine-compat.js`) are embedded with `rust-embed`. In debug builds they load from disk, so CSS rebuilds show up without recompiling.

## Project layout

```
Cargo.toml          lib + bin: lib exposes build_app(pool) so tests can boot the server
Justfile            setup, css, dev, check, test-browser, release
src/main.rs         config, pool, migrations, serve
src/lib.rs          build_app(): router + middleware
src/hx.rs           htmx 4 extractors and response builder (HX-Source, HX-Request-Type, HX-Trigger, HX-Push-Url, HX-Retarget, HX-Reswap)
src/error.rs        AppError wrapping color_eyre::Report -> 500 with a traced backtrace
src/db.rs           all SQL, one fn per query
src/views/          maud: layout.rs (shell), partials per feature
src/routes/         axum handlers, one module per feature
migrations/         NNNN_name.sql
assets/input.css    Tailwind v4 + daisyUI entry
assets/dist/        built css + vendored js (gitignored except .gitkeep)
tools/              tailwindcss binary + daisyui.mjs (gitignored)
tests/browser.rs    chromiumoxide happy-path test against a real server
```

## Handler shape

A handler decides between full page and fragment with one extractor. See `references/patterns.md` for the complete versions of each pattern.

```rust
pub async fn index(State(app): State<App>, hx: HxRequest) -> Result<Response, AppError> {
    let items = db::list_items(&app.pool).await?;
    let body = views::items::list(&items);
    Ok(if hx.is_partial() { body.into_response() } else { layout::page("Items", body).into_response() })
}
```

## Dev loop

- `just dev` runs the Tailwind watcher and `watchexec -r ... -- cargo run` side by side. `cargo-watch` is unmaintained (its README points to watchexec and bacon); watchexec restarts a server cleanly next to the CSS watcher, where bacon's TUI would fight its output. First run after `just setup` takes a minute; later runs are incremental.
- `just check` = `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo nextest run` (falls back to `cargo test`). Run it before declaring anything done.
- `just test-browser` runs the ignored chromiumoxide tests. It needs Chrome or Chromium on `PATH`. On a fresh machine, say so and offer the install command instead of pretending it passed.
- Link time dominates rebuilds. `.cargo/config.toml` in the template turns on `mold` (Linux) or `lld` (macOS) when present; leave it.

## Deploy

`just release` builds `x86_64-unknown-linux-musl` with assets embedded. The template ships a `Dockerfile` (two-stage, `scratch` runtime) and a `fly.toml` that mounts a volume at `/data` and sets `DATABASE_URL=sqlite:///data/app.db`. The app sets WAL mode and a busy timeout at startup, so one process per volume is the only rule.

## Reactivity, in order of preference

1. Plain htmx: a request returns HTML. Most "reactivity" is this.
2. `hx-live` (ships with htmx 4): `take`, `toggle`, `attr`, `debounce`. Good for one-liners that would otherwise be an `hx-on:` handler.
3. Alpine.js with `hx-alpine-compat.js` loaded before Alpine: real client state (modals, multi-step forms, optimistic toggles). Keep `x-data` small and local to the component.

## UI

daisyUI 5 on Tailwind v4 via the standalone CLI, no Node. Use daisyUI semantic classes (`btn`, `card`, `modal`, `form-control`, `alert`) so maud markup stays short, and set `data-theme` on `<html>` for dark mode. Loading state: `hx-indicator` plus daisyUI `loading loading-spinner`. Validation display: daisyUI `input-error` + `label-text-alt text-error`.

## When the stack does not fit

Say so early. Real-time collaboration, heavy client-side computation, or offline-first apps want something else. Durable workflows have no DBOS equivalent in Rust; `apalis` with the SQLite backend covers background jobs, and a `workflow_runs` checkpoint table covers resumable multi-step work. Mention the gap when the user's app needs it.
