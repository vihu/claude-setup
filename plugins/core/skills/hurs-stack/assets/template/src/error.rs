use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use maud::html;

/// Anything that bubbles out of a handler. Logged with its backtrace, rendered as a plain 500 fragment.
pub struct AppError(pub color_eyre::Report);

impl<E: Into<color_eyre::Report>> From<E> for AppError {
    fn from(e: E) -> Self {
        Self(e.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = ?self.0, "handler failed");
        let body = html! {
            div .alert.alert-error role="alert" { "Something went wrong. Please try again." }
        };
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
