---
name: rust
description: Use whenever writing, editing, reviewing, or generating **any** Rust code (`.rs` files), including small snippets. Applies these conventions to doc comments, error types, builders, newtypes, boolean args, self-consuming APIs, attributes, and idioms.
---

Use whenever writing, editing, reviewing, or generating **any** Rust code (`.rs` files), including small snippets. Applies these conventions to doc comments, error types, builders, newtypes, boolean args, self-consuming APIs, attributes, and idioms.

## Doc comments

- Start with a **single-sentence summary**.
- If more is needed, add a **blank comment line**, then the rest of the docs.
- Never put a second sentence in the summary line.
- Only add details when the function is more than an obvious one-liner.
- All public fields must be documented.
- Name every new private field so obviously that it needs no doc comment. If a name can't carry that on its own, document it.
- Add a `# Panics` section when the caller must uphold a contract. Document what the _caller_ has to guarantee; don't trace through underlying calls.
- Add an `# Errors` section on fallible functions, same shape: say what makes it fail; link the spec when relevant.
- Every file starts with a module doc comment (`//!`).
- Wrap doc comments at **80 columns**. Long URLs are the only exception.
- Public items must be documented: `#![deny(missing_docs)]` at the crate root.
- Use reference-style and intra-doc links: `[Cell]`, `[HexTreeMap][crate::HexTreeMap]`, `[u64]`, and `[label]: https://...` at the bottom.
- Public examples are **runnable doctests**, with setup hidden behind `# ` lines.
- A crate's `README.md` is its crate-level docs. Pull it in at the crate root so the two never drift. Prefer letting the README examples run as doctests (keeps them honest):

```rust
#![doc = include_str!("../README.md")]
```

Only add the `not(doctest)` guard when the README's code fences can't be valid, compiling doctests:

```rust
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]
```

```rust
/// Returns the number of active sessions.
pub fn active_count(&self) -> usize {
    self.sessions.iter().filter(|s| s.is_active()).count()
}

/// Splits the buffer into two at the given index.
///
/// The first slice covers `[0, mid)`, the second covers `[mid, len)`.
///
/// # Panics
///
/// Panics if `mid > len`.
pub fn split_at(&self, mid: usize) -> (&[u8], &[u8]) {
    (&self.buf[..mid], &self.buf[mid..])
}

/// Constructs a new Cell from a raw H3 index.
///
/// # Errors
///
/// Returns an error if `raw` is not a valid [bit-representation] of
/// an H3 cell (mode 1 H3 index).
///
/// [bit-representation]: https://h3geo.org/docs/core-library/h3Indexing/
pub const fn from_raw(raw: u64) -> Result<Self> {
    // ...
}
```

## Builders

- If it is possible to construct an object in an **illegal or invalid state** without a builder, then a builder is **mandatory**.
- Simple, always-valid objects don't need one.

```rust
// Invalid states are constructable directly, so a builder is required.
pub struct Server {
    host: String,
    port: u16,
    max_conns: usize,
}

pub struct ServerBuilder {
    host: Option<String>,
    port: Option<u16>,
    max_conns: usize,
}

impl ServerBuilder {
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Builds the server.
    ///
    /// # Panics
    ///
    /// Panics if `host` or `port` was not set.
    pub fn build(self) -> Server {
        Server {
            host: self.host.expect("host is required"),
            port: self.port.expect("port is required"),
            max_conns: self.max_conns,
        }
    }
}
```

## Boolean arguments

- Avoid boolean function arguments.
- Exception: allowed only when it is the **sole argument** and completely unambiguous.

```rust
// Bad: unclear at the call site.
fn render(&self, cache: bool, inline: bool) { /* ... */ }
render(true, false); // what do these mean?

// Prefer an enum.
enum Caching { Enabled, Disabled }
fn render(&self, caching: Caching) { /* ... */ }

// Acceptable: sole, unambiguous argument.
fn set_visible(&mut self, visible: bool) { /* ... */ }
```

## Consuming self

- If an object shouldn't be used after a certain point, take `self` **by value** so the type system prevents reuse.

```rust
impl Transaction {
    /// Commits the transaction, consuming it.
    pub fn commit(self) -> Result<(), Error> {
        // ...
        Ok(())
    }
}

let tx = Transaction::begin();
tx.commit()?;
// tx.commit()?; // compile error: value used after move
```

## Newtypes & invariants

- Wrap primitives in newtypes instead of passing bare `u64`/`usize` around.
- When invariants matter, split a **low-level unchecked** type (can build invalid states) from a **high-level enforced** type that validates on construction.
- `debug_assert!` the invariants a caller must uphold. This pairs with the `# Panics` doc.
- **Custom datastructures live in their own module** (`mod cell;`), even a bare newtype. Keep the type, its impls, and its `#[cfg(test)]` tests together.
- Test both directions: **positive** (valid input constructs / round-trips) and **negative** (invalid input is rejected with the right error).

```rust
/// Low-level type: allows constructing invalid values.
pub struct Index(u64);

/// High-level type: enforces invariants on construction.
pub struct Cell(pub(crate) u64);

impl Cell {
    /// Constructs a new Cell from a raw H3 index.
    ///
    /// # Errors
    ///
    /// Returns an error if `raw` is not a valid H3 cell index.
    pub const fn from_raw(raw: u64) -> Result<Self> {
        let idx = Index(raw);
        if !idx.reserved() && idx.mode() == 1 && idx.base() < 122 {
            Ok(Cell(idx.0))
        } else {
            Err(Error::Index(raw))
        }
    }

    /// Returns this cell's resolution.
    pub const fn res(&self) -> u8 {
        let res = Index(self.0).res();
        debug_assert!(res < 16);
        res
    }
}
```

## Errors

- **`anyhow`: never in libraries.** Hard rule — it erases the typed errors
  callers need. Acceptable only in tiny throwaway applications.
- **`thiserror` / `snafu`: prefer hand-rolled.** Not banned, but hand-writing
  avoids an extra dependency and a proc-macro. Lean hand-rolled by default.
- The two patterns below (enum / `kind` + `#[track_caller]`) are those
  hand-rolled impls.
- Two patterns, chosen by how tightly the code is coupled:
  - **General-purpose library → enum.** Distinct callers need to match on the failure and recover, so give them a typed variant per case.
  - **Internal project where app + library are tightly coupled → `kind` + `#[track_caller]`.** One team owns both sides, nobody matches on fine-grained variants, but you want the source location for free. Carry a coarse `kind` plus file/line.
- Provide a crate `Result` alias with a default type parameter.

### Enum pattern (general-purpose libraries)

- `#[non_exhaustive]`; each variant carries the offending value.
- Hand-write `Display` and `Error::source`. Messages are lowercase, `"description, got {value}"` shape.
- `From` impls for infallible conversion, `TryFrom` for fallible.

```rust
/// Result type for this crate.
pub type Result<T = ()> = std::result::Result<T, Error>;

/// Error type for this crate.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An invalid raw source value was used for an H3 cell.
    Index(u64),
    /// Unsupported version.
    Version(u8),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Index(_) => None,
            Error::Version(_) => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Index(bits) => write!(f, "raw u64 is not a valid H3 index: {bits}"),
            Error::Version(version) => write!(f, "unsupported version, got {version}"),
        }
    }
}
```

### `kind` + `#[track_caller]` pattern (tightly-coupled app + library)

- Struct carrying a `kind` enum plus captured file/line, still hand-written.
- Constructors are `#[track_caller]` so the call site is recorded automatically.
- Callers rarely match variants; they log/propagate. The location is the payload that matters.

```rust
/// Result type for this crate.
pub type Result<T = ()> = std::result::Result<T, Error>;

/// Coarse classification of what went wrong.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Request failed validation.
    Validation(String),
    /// Referenced entity was not found.
    NotFound(String),
}

/// Error type for this crate.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    file: &'static str,
    line: u32,
}

impl Error {
    #[track_caller]
    pub fn validation(msg: impl Into<String>) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            kind: ErrorKind::Validation(msg.into()),
            file: loc.file(),
            line: loc.line(),
        }
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?} at {}:{}", self.kind, self.file, self.line)
    }
}
```

### Unwrapping

- Prefer `.expect("why this holds")` with a reason over a bare `.unwrap()`. Exception: unwrap is fine in tests.

```rust
let promoted = self
    .to_parent(common_res)
    .expect("we already checked to the min common resolution");
```

## Attributes

- `const fn` wherever possible.
- `#[inline]` on small hot functions; `#[must_use]` on `self`-consuming setters.
- Optional derives via `#[cfg_attr(feature = "serde", derive(...))]`; gate items with `#[cfg(feature = "...")]`.
- `#[repr(align(64))]` on hot enums/structs to control cache-line layout.
- `pub(crate)` for internal API; keep the public surface minimal.
- Crate root: `#![deny(unsafe_code, missing_docs, rustdoc::broken_intra_doc_links)]`.

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct Index(u64);

impl Index {
    /// Consumes `self` and returns a new Index with its resolution set to `res`.
    #[must_use]
    #[inline]
    pub const fn set_res(self, res: u8) -> Self {
        debug_assert!(res < 16);
        let mask = 0b1111 << 0x34;
        let masked_index = self.0 & !mask;
        let shifted_res = ((res & 0b1111) as u64) << 0x34;
        Self(masked_index | shifted_res)
    }
}
```

## Constants

- **No magic numbers.** Name every non-trivial literal as a `const`.
- **Scope to use.** A single-use const lives in the module that uses it. Only hoist to a shared ancestor module when 2+ modules use it.
- **Bounded min/max → one range const**, never two scalars. Use `Range` (exclusive upper) or `RangeInclusive` (inclusive), check with `.contains()`, and format the bounds in messages with `{:?}` so the const is the single source of truth.

```rust
// Bad: two scalars, bounds duplicated in the message.
const NAME_MIN_LEN: usize = 1;
const NAME_MAX_LEN: usize = 50;
if !(NAME_MIN_LEN..=NAME_MAX_LEN).contains(&len) {
    fail!("name must be between {NAME_MIN_LEN} and {NAME_MAX_LEN} chars");
}

// Good: one range const, Debug-formatted bounds.
const NAME_LEN: RangeInclusive<usize> = 1..=50;
if !NAME_LEN.contains(&len) {
    fail!("name length must be in {NAME_LEN:?} chars, got {len}");
}
```

## Module layout

- Order items **most to least important**: the object definition, then its
  associated types, then error types.
- **Split `impl` blocks by visibility**, public first, then private. Do not
  interleave.

```rust
/// The object this module exists for.
pub struct Session { /* ... */ }

/// A type the object hands back.
pub enum Event { /* ... */ }

/// What can go wrong here.
pub enum Error { /* ... */ }

// Public API
impl Session {
    pub fn new() -> Session { /* ... */ }
}

// Private API
impl Session {
    fn advance(&mut self) { /* ... */ }
}
```

## Idioms

- Tuple matching for state machines: `match (digits.next(), self)`.
- Match guards over nested `if`: `v if v < res => ...`.
- Name intermediates for bit ops (`mask`, `masked_index`, `shifted_res`). Hex for bit positions, binary for masks.
- **Never glob-import** (`use foo::*;`). Name each item so its origin is explicit. Only exception: re-exporting via `pub use` in a prelude module.
- New `use` statements: no blank line separating them from existing ones.
- Insert each new `use` in its sorted position within the group; don't reorder the rest (`cargo fmt` handles the final ordering).
- Only blank line allowed in the import block: one line between the private `use` group and the `pub use` group.

```rust
// Bad: hides where names come from.
use crate::error::*;
use std::collections::*;

// Good: explicit.
use crate::error::{Error, Result};
use std::collections::{HashMap, HashSet};
```

```rust
match (digits.next(), self) {
    (None, _) => Some((cell, self)),
    (Some(_), Self::Leaf(_)) => Some((cell.to_parent(res).expect("invalid condition"), self)),
    (Some(digit), Self::Parent(children)) => match &children[digit as usize] {
        Some(node) => node.get(res + 1, cell, digits),
        None => None,
    },
}
```

## Paths

- **Applications: prefer `camino`'s `Utf8PathBuf` / `Utf8Path`** over std's `PathBuf` / `Path`.
- UTF-8 guaranteed, so paths print, serialize, and compare as `str` with no lossy conversions.
- Libraries: only pull in `camino` if UTF-8 paths are part of the public contract; otherwise stay on std.

## Allocator

- **Binary applications: default to jemalloc.** Set it as the global allocator unless there's a specific reason not to (musl/cross-compilation trouble, WASM targets, embedded).
- Libraries: never set a global allocator. That's an application-level decision, not the library's to make.
- Use the `tikv-jemallocator` crate (the maintained fork).
- jemalloc doesn't support MSVC, so gate it out on that target.

```rust
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
```

- Gate the dependency itself the same way, so it's not even compiled on MSVC:

```toml
[target.'cfg(not(target_env = "msvc"))'.dependencies]
tikv-jemallocator = "0.6"
```

## Cargo.toml

- keep all items sorted
- with each top level section, align on first `=`
