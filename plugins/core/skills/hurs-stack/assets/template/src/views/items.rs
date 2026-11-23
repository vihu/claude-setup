use maud::{Markup, html};

use crate::db::Item;

pub fn list(items: &[Item]) -> Markup {
    html! {
        h1 .text-2xl.font-semibold.mb-4 { "Items" }
        div .card.bg-base-100.shadow {
            div .card-body {
                (form(None, None))
                table .table {
                    tbody #items {
                        @for item in items { (row(item)) }
                    }
                }
            }
        }
    }
}

pub fn row(item: &Item) -> Markup {
    html! {
        tr {
            td { (item.title) }
            td .text-right {
                button .btn.btn-ghost.btn-xs
                    hx-delete=(format!("/items/{}", item.id))
                    hx-target="closest tr" hx-swap="delete"
                    hx-confirm="Delete this item?" { "Delete" }
            }
        }
    }
}

/// `prev` re-fills the input after a validation failure. `error` renders inline.
pub fn form(prev: Option<&str>, error: Option<&str>) -> Markup {
    html! {
        form #item-form .flex.gap-2.items-start.mb-4
            hx-post="/items" hx-swap="outerHTML" hx-disable="find button" {
            div .flex-1 {
                input .input.w-full .input-error[error.is_some()]
                    name="title" placeholder="What needs doing?" value=[prev] autofocus
                    aria-invalid=[error.map(|_| "true")];
                @if let Some(e) = error { p .text-error.text-sm.mt-1 { (e) } }
            }
            button .btn.btn-primary type="submit" {
                "Add"
                span .loading.loading-spinner.loading-xs.htmx-indicator {}
            }
        }
    }
}
