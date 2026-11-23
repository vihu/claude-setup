# Crates and tooling

Versions verified with `cargo search` and upstream docs on 2026-10-18. Re-verify before writing `Cargo.toml`:
`cargo search <crate> --limit 1`. Do not pin to memory.

## Core

| Need | Crate | Version | Why this one |
|---|---|---|---|
| HTTP | `axum` | 0.8 | Tower-native, `/{id}` path syntax, extractors without `async_trait` |
| Runtime | `tokio` | 1 | `features = ["full"]` |
| Middleware | `tower-http` | 0.7 | `trace`, `compression-gzip`, `timeout`, `fs` only if serving a dir |
| Templates | `maud` | 0.27 | Compile-time HTML in Rust, `features = ["axum"]` gives `IntoResponse`. Closest thing to Go's `templ` |
| DB | `sqlx` | 0.9 | `features = ["runtime-tokio", "sqlite", "migrate", "macros"]`. `query!` is the `sqlc` substitute |
| Errors (app) | `color-eyre` | 0.6 | Backtraces and span traces, like `cockroachdb/errors` |
| Errors (typed) | `thiserror` | 2 | For domain errors that handlers match on |
| Logging | `tracing`, `tracing-subscriber` | 0.1 / 0.3 | `env-filter` feature |
| Assets | `rust-embed` | 8 | Disk in debug, embedded in release |
| Forms | `serde` | 1 | `derive` feature, for `axum::Form` |

## Testing

| Need | Crate | Version | Notes |
|---|---|---|---|
| Browser | `chromiumoxide` | 0.9 | `default-features = false, features = ["tokio-runtime"]`. Needs `futures` for the handler stream. Replaces `go-rod` |
| Runner | `cargo-nextest` | tool | Optional. `just check` falls back to `cargo test` |
| Watcher | `watchexec` | tool (2.8) | `just dev` restarts `cargo run` on change. Optional: `bacon` (3.26) for a check/clippy loop in its own terminal |
| One-off eyes | `rodney` | tool | simonw's CDP CLI. Language-agnostic, keep using it |

## Optional

| Need | Crate | Version | Notes |
|---|---|---|---|
| Background jobs | `apalis` + `apalis-sql` | 1.0 / 1.0-rc | SQLite backend. Nearest thing to DBOS; no durable-workflow engine exists for Rust |
| OpenAPI | `utoipa` + `utoipa-axum` | 6 | Only for real JSON endpoints |
| Pure-Rust SQLite | `turso` | 0.9-pre | Experimental. `sqlx` + bundled `libsqlite3-sys` is the default |
| htmx helpers | `axum-htmx` | 0.8 | Targets htmx 2 header names (`HX-Trigger-Name`, `HX-Trigger-After-Swap`). Do not use; `src/hx.rs` in the template covers htmx 4 |

## sqlx 0.9 specifics

- MSRV 1.94.
- Dynamic SQL strings must be wrapped in `sqlx::AssertSqlSafe(...)`. Static `&'static str` works as before. Prefer `query!` and never `format!` a query.
- Combined runtime+TLS features are gone; SQLite needs no TLS feature at all.
- SQLite extension loading is `unsafe` and behind `sqlite-load-extension`.
- `sqlx-cli` for `cargo sqlx prepare`: `cargo install sqlx-cli --no-default-features --features sqlite`.
- Offline mode: commit `.sqlx/`. CI sets `SQLX_OFFLINE=true`.

## Front-end tooling (no Node)

- Tailwind v4 standalone binary from `github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-<os>-<arch>`.
- daisyUI 5 bundles: `github.com/saadeghi/daisyui/releases/latest/download/daisyui.mjs` and `daisyui-theme.mjs`, placed next to the binary. CSS loads them with `@plugin "./daisyui.mjs";`. Exclude them from scanning with `@source not`.
- htmx 4: `https://cdn.jsdelivr.net/npm/htmx.org@4.0.0/dist/htmx.min.js`, extensions under `dist/ext/`.
- Alpine 3: `https://cdn.jsdelivr.net/npm/alpinejs@3/dist/cdn.min.js`, loaded with `defer` after `hx-alpine-compat.js`.
- `just setup` downloads all of these into `tools/` and `assets/dist/js/` so builds never hit the network.

## Linkers

Rebuild time is the main cost versus Go. `.cargo/config.toml` enables `mold` on Linux and `lld` on macOS when installed.
Install: `apt install mold` / `brew install llvm` (for `ld64.lld`). Builds still work without them.
