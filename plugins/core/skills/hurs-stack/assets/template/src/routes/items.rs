use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use maud::html;
use serde::Deserialize;

use crate::{
    App, db,
    error::AppError,
    hx::{Hx, HxRequest},
    views,
};

pub async fn index(State(app): State<App>, hx: HxRequest) -> Result<Response, AppError> {
    let items = db::list_items(&app.pool).await?;
    let body = views::items::list(&items);
    Ok(if hx.is_partial() {
        body.into_response()
    } else {
        views::layout::page("Items", body).into_response()
    })
}

#[derive(Deserialize)]
pub struct NewItem {
    pub title: String,
}

pub async fn create(State(app): State<App>, Form(f): Form<NewItem>) -> Result<Response, AppError> {
    let title = f.title.trim();
    if title.is_empty() {
        let form = views::items::form(Some(&f.title), Some("Title is required"));
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, form).into_response());
    }
    let item = db::insert_item(&app.pool, title).await?;
    Ok(Hx::new().trigger("itemCreated").render(html! {
        (views::items::form(None, None))
        hx-partial hx-target="#items" hx-swap="afterbegin" { (views::items::row(&item)) }
    }))
}

pub async fn delete(State(app): State<App>, Path(id): Path<i64>) -> Result<Response, AppError> {
    db::delete_item(&app.pool, id).await?;
    Ok(Hx::new().trigger("itemDeleted").empty())
}
