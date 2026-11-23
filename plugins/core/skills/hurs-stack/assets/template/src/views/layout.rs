use maud::{DOCTYPE, Markup, html};

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
                main .container.mx-auto.max-w-3xl.p-4 { (body) }
            }
        }
    }
}

fn nav() -> Markup {
    html! {
        div .navbar.bg-base-100.shadow-sm {
            div .flex-1 { a .btn.btn-ghost.text-xl href="/" { "HURS" } }
            div .flex-none {
                // ponytail: theme toggle is client-only state; persist server-side if accounts appear.
                label .swap.swap-rotate x-data="{ dark: false }"
                    "@click"="document.documentElement.dataset.theme = dark ? 'light' : 'dark'; dark = !dark" {
                    input type="checkbox";
                    span .swap-off { "☀" }
                    span .swap-on { "☾" }
                }
            }
        }
    }
}
