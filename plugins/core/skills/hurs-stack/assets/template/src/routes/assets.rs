//! Static assets. Debug builds read `assets/dist/` from disk on every request (CSS watcher output shows up
//! without a recompile); release builds embed the directory into the binary.

use axum::{
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets/dist/"]
struct Assets;

pub async fn serve(Path(path): Path<String>) -> Response {
    match Assets::get(&path) {
        Some(file) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            let cache = if cfg!(debug_assertions) {
                "no-cache"
            } else {
                "public, max-age=31536000, immutable"
            };
            (
                [
                    (header::CONTENT_TYPE, mime.as_ref().to_owned()),
                    (header::CACHE_CONTROL, cache.to_owned()),
                ],
                file.data.into_owned(),
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
