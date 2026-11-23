# htmx 4 cheat sheet

Condensed from the official `htmx-guidance.md` shipped in htmx v4.0.0 (`dist/skills/`). If anything here conflicts with the
installed version, fetch `https://raw.githubusercontent.com/bigskysoftware/htmx/v4.0.0/dist/skills/htmx-guidance.md`.

Model: an element issues an HTTP request, the server returns an HTML fragment, htmx swaps it into the DOM. htmx 4 uses
`fetch()`.

## Requests

`hx-get`, `hx-post`, `hx-put`, `hx-patch`, `hx-delete`, `hx-query`. Each takes a URL. Or `hx-action` + `hx-method`.

Default triggers: `input`/`textarea`/`select` on `change`, `form` on `submit`, everything else on `click`.

`hx-trigger` modifiers: `once`, `changed`, `delay:500ms` (debounce), `throttle:1s`, `from:<sel>` (`document`, `window`,
`closest`, `find`, `next`, `previous`), `target:<sel>`, `prevent`, `stop`, `halt`, `capture`, `passive`. Filters in brackets:
`click[ctrlKey]`. Special events: `load`, `revealed`, `intersect`. Polling: `every 2s`. Server-driven refresh:
`hx-trigger="refreshTable from:body"` paired with an `HX-Trigger: refreshTable` response header.

Form data: non-GET/DELETE requests include the enclosing form. GET and DELETE do not; add `hx-include="closest form"`.
`hx-vals`, `hx-headers`, `hx-config` take HCON (JSON works too). `js:` prefix for computed values.

## Targets and swaps

`hx-target`: CSS selector, or `this`, `closest <sel>`, `find <sel>`, `next [sel]`, `previous [sel]`. Default: the element
itself. Relative selectors avoid ids in loops.

`hx-swap` values: `innerHTML` (default), `outerHTML`, `outerSync`, `innerMorph`, `outerMorph`, `textContent`, `before`,
`prepend`, `append`, `after`, `delete`, `none`. Modifiers: `swap:100ms`, `settle:200ms`, `transition:true`, `scroll:top`,
`show:bottom`, `strip:true`, `target:<sel>`, `ignoreTitle:true`.

Morph swaps preserve focus, scroll, input values, listeners. They cannot reset a form; use `innerHTML`/`outerHTML` for that.
Freeze an element from morphing with `hx-morph-skip`; freeze only its children with `hx-morph-skip-children`.

## Inheritance is explicit (biggest htmx 2 difference)

Children do not inherit parent attributes. Mark the parent: `hx-target:inherited="#out"`, `hx-include:inherited="..."`.
`:append` appends to an inherited value. `htmx.config.implicitInheritance = true` restores htmx 2 behaviour; do not use it.

## Status codes

All statuses swap except 204 and 304 (htmx 2 skipped 4xx/5xx). Per-status overrides:

```html
<form hx-post="/register" hx-target="#result"
      hx-status:422="target:#errors"
      hx-status:5xx="swap:none">
```

Options inside the value: `swap:`, `target:`, `select:`, `push:`, `replace:`, `transition:`. Wildcards `5xx`, `50x`.

## Multi-region updates

Prefer `<hx-partial>` over `hx-swap-oob`:

```html
<hx-partial hx-target="#messages" hx-swap="beforeend"><div>New message</div></hx-partial>
<hx-partial hx-target="#badge" hx-swap="innerHTML"><span class="badge">5</span></hx-partial>
```

Each partial names its own target and swap. The main response body (outside partials) still goes to the normal target.
OOB swaps, if used, run after the main swap in htmx 4.

## Headers

Request (htmx sends): `HX-Request: true`, `HX-Source: tag#id` (replaces `HX-Trigger`/`HX-Trigger-Name`), `HX-Target: tag#id`,
`HX-Current-URL`, `HX-Request-Type: partial|full` (`full` when targeting body or using `hx-select`), `HX-Boosted`,
`HX-History-Restore-Request`.

Response (server sends): `HX-Trigger` (event name or JSON), `HX-Push-Url`, `HX-Replace-Url`, `HX-Redirect` (full reload),
`HX-Location` (ajax navigation), `HX-Refresh`, `HX-Retarget`, `HX-Reswap`, `HX-Reselect`.
Removed: `HX-Trigger-After-Swap`, `HX-Trigger-After-Settle`.

## Events

Pattern `htmx:phase:action`. Common: `htmx:config:request`, `htmx:before:request`, `htmx:after:request`,
`htmx:response:error`, `htmx:error`, `htmx:before:swap`, `htmx:after:swap`, `htmx:after:settle`. Inline handlers with
`hx-on:htmx:after:swap="..."`. Abort with `htmx.trigger(el, "htmx:abort")`. `detail.ctx` carries source element, target,
swap, request method/url/headers/body.

## Other attributes

`hx-select`, `hx-select-oob`, `hx-include`, `hx-vals`, `hx-headers`, `hx-indicator`, `hx-confirm`, `hx-sync`, `hx-boost`,
`hx-config`, `hx-preserve`, `hx-ignore` (was `hx-disable`), `hx-disable` (was `hx-disabled-elt`), `hx-preload`, `hx-pending`,
`hx-push-url`, `hx-replace-url`, `hx-encoding`, `hx-validate`, `hx-history-elt`.

## Config

`<meta name="htmx-config" content="defaultSwap:outerHTML, logAll:true">` (HCON). Useful keys: `defaultSwap`,
`defaultTimeout` (60000), `noSwap` ([204,304]), `transitions`, `history` (`true`, `false`, `"reload"`), `extensions`
(whitelist), `inlineScriptNonce`, `morphSkip`, `morphSkipChildren`.

## Extensions

Loaded by including the script; no `hx-ext` attribute. Files live under `dist/ext/`. Registration names differ from file
names. Relevant ones: `hx-alpine-compat.js` (`alpine-compat`, load before Alpine), `hx-live.js` (`hx-live`), `hx-sse.js`
(`sse`), `hx-ws.js` (`ws`), `hx-preload.js` (`preload`), `hx-history-cache.js`, `hx-download.js`, `hx-upsert.js`,
`hx-head.js`, `hx-csp.js`, `htmx-2-compat.js`.

## Patterns, one line each

- Active search: `<input name="q" hx-get="/search" hx-trigger="input changed delay:300ms, keyup[key=='Enter']" hx-target="#results" hx-indicator="#spin">`
- Lazy load: `<div hx-get="/slow" hx-trigger="load" hx-swap="outerHTML">`
- Infinite scroll: last row with `hx-get="/page/3" hx-trigger="revealed" hx-swap="afterend"`
- Edit in place: display view with `hx-get="/x/1/edit" hx-swap="outerHTML"`; edit form posts and returns the display view
- Tabs: `<div role="tablist" hx-target:inherited="#tab-content">` with `hx-get` buttons
- Indicator: element with class `htmx-indicator` referenced by `hx-indicator`; `.htmx-request .htmx-indicator { transition: opacity 200ms ease-in 200ms }` avoids flashing on fast requests
- Disable during request: `hx-disable="find button, find input"`
