# hurs-app

Hypermedia, Unix, Rust, SQLite. axum + maud + htmx 4 + Alpine + Tailwind/daisyUI + sqlx. One static binary.

```
just setup     # once per machine: tailwind binary, daisyUI, htmx, Alpine
just dev       # server with reload + CSS watcher, http://localhost:8080
just check     # fmt, clippy, tests
just test-browser
just release   # static musl binary
```

Needs: Rust 1.94+, `just`, `curl`, `watchexec` (`brew install watchexec` or `cargo install watchexec-cli`), `sqlx-cli` for `just prepare`
(`cargo install sqlx-cli --no-default-features --features sqlite`), Chrome for browser tests.
