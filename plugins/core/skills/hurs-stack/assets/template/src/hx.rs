//! htmx 4 request extractor and response builder.
//! Header names follow htmx 4 (`HX-Source`, `HX-Request-Type`); `axum-htmx` on crates.io still speaks htmx 2.

use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use maud::Markup;

/// What htmx told us about the request. Absent headers mean a plain browser navigation.
#[derive(Debug, Clone, Default)]
pub struct HxRequest {
    pub is_htmx: bool,
    pub boosted: bool,
    pub history_restore: bool,
    /// `"partial"` or `"full"`. `full` when the target is `body` or `hx-select` is used.
    pub request_type: Option<String>,
    /// `tag#id` of the element that fired the request.
    pub source: Option<String>,
    /// `tag#id` of the swap target.
    pub target: Option<String>,
    pub current_url: Option<String>,
}

impl HxRequest {
    /// True when only a fragment should be rendered (no layout).
    pub fn is_partial(&self) -> bool {
        self.is_htmx && self.request_type.as_deref() != Some("full")
    }

    pub fn source_id(&self) -> Option<&str> {
        self.source
            .as_deref()
            .and_then(|s| s.split_once('#'))
            .map(|(_, id)| id)
    }
}

impl<S: Send + Sync> FromRequestParts<S> for HxRequest {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let h = &parts.headers;
        let get = |n: &str| h.get(n).and_then(|v| v.to_str().ok()).map(str::to_owned);
        let flag = |n: &str| get(n).as_deref() == Some("true");
        Ok(Self {
            is_htmx: flag("hx-request"),
            boosted: flag("hx-boosted"),
            history_restore: flag("hx-history-restore-request"),
            request_type: get("hx-request-type"),
            source: get("hx-source"),
            target: get("hx-target"),
            current_url: get("hx-current-url"),
        })
    }
}

/// Builder for htmx response headers. `Hx::new().trigger("saved").render(markup)`.
#[derive(Debug, Default)]
pub struct Hx {
    headers: HeaderMap,
    status: Option<StatusCode>,
}

impl Hx {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::VARY,
            HeaderValue::from_static("HX-Request, HX-Request-Type"),
        );
        Self {
            headers,
            status: None,
        }
    }

    fn set(mut self, name: &'static str, value: impl AsRef<str>) -> Self {
        if let Ok(v) = HeaderValue::from_str(value.as_ref()) {
            self.headers.insert(HeaderName::from_static(name), v);
        }
        self
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = Some(status);
        self
    }

    /// Fire a client-side event. Pass a bare name, or a JSON object for events with detail.
    pub fn trigger(self, event: impl AsRef<str>) -> Self {
        self.set("hx-trigger", event)
    }
    pub fn push_url(self, url: impl AsRef<str>) -> Self {
        self.set("hx-push-url", url)
    }
    pub fn replace_url(self, url: impl AsRef<str>) -> Self {
        self.set("hx-replace-url", url)
    }
    /// Ajax navigation that keeps the page shell.
    pub fn location(self, url: impl AsRef<str>) -> Self {
        self.set("hx-location", url)
    }
    /// Full page redirect.
    pub fn redirect(self, url: impl AsRef<str>) -> Self {
        self.set("hx-redirect", url)
    }
    pub fn refresh(self) -> Self {
        self.set("hx-refresh", "true")
    }
    pub fn retarget(self, selector: impl AsRef<str>) -> Self {
        self.set("hx-retarget", selector)
    }
    pub fn reswap(self, swap: impl AsRef<str>) -> Self {
        self.set("hx-reswap", swap)
    }
    pub fn reselect(self, selector: impl AsRef<str>) -> Self {
        self.set("hx-reselect", selector)
    }

    pub fn render(self, body: Markup) -> Response {
        let status = self.status.unwrap_or(StatusCode::OK);
        (status, self.headers, body).into_response()
    }

    /// Headers only, empty body. Use with `hx-swap="none"` or `delete`.
    pub fn empty(self) -> Response {
        let status = self.status.unwrap_or(StatusCode::OK);
        (status, self.headers).into_response()
    }
}

/// Redirect that works for both htmx and plain requests: `HX-Location` for htmx, 303 otherwise.
pub fn redirect_to(hx: &HxRequest, url: &str) -> Response {
    if hx.is_htmx {
        Hx::new().location(url).empty()
    } else {
        (StatusCode::SEE_OTHER, [(header::LOCATION, url.to_owned())]).into_response()
    }
}
