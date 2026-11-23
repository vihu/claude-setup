# Two-stage build: static musl binary with assets embedded, scratch runtime.
# Run `just css` before building so assets/dist/app.css exists.

FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=build /src/target/x86_64-unknown-linux-musl/release/hurs-app /app
ENV BIND=0.0.0.0:8080 DATABASE_URL=sqlite:///data/app.db RUST_LOG=info
VOLUME /data
EXPOSE 8080
ENTRYPOINT ["/app"]
