---
name: rust-perf-audit
description: Audit a Rust repo for performance wins and write ranked findings, each pinned to a file and line, to `.ai-docs/`. Read-only unless you ask it to apply fixes.
disable-model-invocation: true
argument-hint: "[crate-dir] [hot-path-or-workload]"
---

Audit a Rust repo for performance wins. Arguments: `$ARGUMENTS`.

Resolve the target first:

- `<D>`: the argument that is a directory, as an absolute path. No directory argument: the working directory.
- `<W>`: the rest of the arguments: the workload or hot path the user cares about (a function, a bench, a binary plus its input). May be empty.

The audit is read-only: it changes no code and no `Cargo.toml`. Code changes happen only in `## Apply`, after the user says yes.

Every finding carries its evidence: `measured` (a profile share or bench number) or `static` (read from the code). Claim a speedup only from a measurement. A static finding states its gain as a complexity or a count ("20M allocations to 10", "O(n²) to O(n)"), never as a multiplier.

## Steps

1. Map `<D>`: workspace members, `[profile.*]` sections, direct deps (`cargo tree -e normal --depth 1`), `benches/` and its harness (criterion, divan), binaries and their inputs. Done when you can name the hot paths: `<W>` first, then every loop that runs per item of an unbounded input (per line, per record, per request, per tick).
2. Record a baseline in release mode:
   - A bench covers a hot path: `cargo bench --bench <name>`.
   - A binary has a workload: `hyperfine` it.
   - `perf` works: `CARGO_PROFILE_RELEASE_DEBUG=true` build, `perf record -g` the workload, then `perf report --stdio --no-children`. The top frames confirm or replace the hot paths from step 1.
   - None of these: the audit is static. Say so in the review header.
   Done when the commands and numbers are written down, or the audit is marked static.
3. Collect candidates: `cargo clippy --all-targets -- -W clippy::perf -W clippy::needless_collect -W clippy::redundant_clone -W clippy::inefficient_to_string`, plus a grep for each pattern's signal below. Keep a candidate only when it sits on a hot path or in a loop over unbounded input.
4. Apply every pattern in `## Patterns` to every hot path, including the ones grep cannot find (loop-invariant work, serial independent work). Done when each pattern is either a finding or listed as clean.
5. Verify each finding: trace the path end to end and run the pattern's `Check`. Drop a finding when its check fails or you cannot point at the line.
6. Rank:
   - `HIGH`: an algorithmic win (O(n²) to O(n), O(n) to O(1) per operation) on unbounded input, or a fix on a frame holding at least 10% of measured time.
   - `MED`: per-item allocation, repeated hashing, or serial independent work in a hot loop. Release profile gaps.
   - `LOW`: cold paths, small n, and crate swaps without a measured bottleneck.
7. Write `.ai-docs/REVIEW-NNN-perf-<crate>.md` under `<D>` (create `.ai-docs/` if missing), `NNN` one above the highest existing `REVIEW-*` there. Body, in order:
   - Heading: `# Perf audit <crate>`.
   - Line 1: `measured` or `static`, and the hot paths audited.
   - `## Baseline`: each command, its numbers, and the core count. Omit when static.
   - A table `| # | Sev | Location | Pattern | Fix | Evidence |`, sorted `HIGH`, `MED`, `LOW`. Location: `` `path:line` `` or `` `path:start-end` ``. Pattern: its number and name. Fix: one or two sentences, including what the check confirmed. Escape `|` inside cells.
   - `## Clean`: the patterns checked with no finding, on one line.
   - `## Not audited`: crates or paths skipped and why. Omit when empty.
8. Reply:
   - Line 1: the review's absolute path, and `measured` or `static`.
   - Line 2: finding count per severity.
   - Each `HIGH` finding, at most 5: `path:line` and its fix.
   - `Next:` apply finding 1, or, when static, add a bench for the top hot path.

## Apply

Runs only on the user's yes, one finding per change, in table order:

1. Make the change. A new crate goes in with its current version and only the features the fix uses.
2. Run `cargo test` and re-run the baseline command.
3. Keep the change when tests pass and the number improves. Otherwise undo the edit and mark the finding `rejected` with the numbers.
4. Update the finding's Evidence to `measured: <before> to <after>`.

## Patterns

### Core

1. **Allocation in a hot loop**
   - Signal: `.clone()`, `.to_string()`, `.to_owned()`, `String::from`, `format!`, `.to_vec()` inside a loop or iterator closure. `HashMap<String, _>` keys copied out of input already in memory. A `collect()` whose result is iterated, counted, or summed once.
   - Fix: borrow (`&str`, `&[T]`, `Cow`) and convert to owned once, at the boundary (the 10 rows returned, not the 20M scanned). Chain the iterator instead of collecting.
   - Check: the borrowed source outlives the map (input read whole, not streamed through a reused `BufRead` buffer). `Arc` and `Rc` clones are refcount bumps: skip them.
2. **Repeated work**
   - Signal: `contains_key` or `get`, then `insert`, `get_mut`, or `[]` on the same key. Loop-invariant work inside the loop: `Regex::new`, parsing constants, re-sorting, re-reading config. Two passes where one suffices.
   - Fix: `*map.entry(k).or_insert(0) += 1`. With owned keys, `entry(k.to_owned())` allocates on every hit: use `get_mut(k)` and `insert(k.to_owned(), ..)` only on a miss. Hoist invariant work above the loop.
   - Check: the hoisted value is really invariant across iterations.
3. **One core doing parallel work**
   - Signal: a serial loop over large input whose iterations share no mutable state. Threads accumulating into an `Arc<Mutex<_>>`.
   - Fix: rayon: `par_iter()`, `par_lines()`, `par_chunks()`. Accumulate per thread with `fold`, merge with `reduce`, so no lock is shared.
   - Check: n is large enough to beat split overhead. No order-dependent state. Float sums change in the last bits. Inside an async runtime, run it through `spawn_blocking` or a dedicated pool so executor threads stay free. No nested `par_iter` inside work that is already parallel.
4. **Shifting removal and insertion**
   - Signal: `Vec::remove(i)` or `insert(0, ..)` in a loop. `remove(0)` used as a queue pop. A sweep removing matches one at a time.
   - Fix: a sweep becomes `retain` or `retain_mut` (one O(n) pass, order kept). One removal where order is irrelevant becomes `swap_remove` (O(1)). Front push and pop becomes `VecDeque`.
   - Check: for `swap_remove`, nothing depends on element order or stored indices. In an index loop, re-check `i` after `swap_remove(i)` instead of advancing.

### Crate swaps

Only where the code shows the signal. Prefer a crate already in `cargo tree`. Name the crate and its current version in the fix.

5. **dashmap**
   - Signal: `Arc<Mutex<HashMap>>` or `Arc<RwLock<HashMap>>` hit by many threads, with lock contention in the profile.
   - Fix: `DashMap`, which shards the lock.
   - Check: removing the sharing beats sharding it (pattern 3's `fold` and `reduce`, or per-task maps merged at the end). Holding a `Ref` or `RefMut` while calling the same map again can deadlock: keep each guard inside one statement.
6. **indexmap**
   - Signal: a `Vec` and a `HashMap` kept in sync for order plus lookup. A `HashMap` iterated in a hot loop. Output that needs insertion order.
   - Fix: `IndexMap` or `IndexSet`: hash lookup, index access, and dense iteration at `Vec` speed.
   - Check: `remove` is deprecated: pick `swap_remove` (O(1), reorders) or `shift_remove` (O(n), keeps order), the pattern 4 trade-off. Sorted order wants `BTreeMap` instead.
7. **crossbeam**
   - Signal: `Arc<Mutex<Receiver>>` sharing one `mpsc` receiver across workers. A loop polling several channels with `try_recv`. A `Mutex<VecDeque>` work queue. Per-thread counters packed next to each other (false sharing).
   - Fix: `crossbeam-channel` (cloneable `Receiver`, `select!`), `crossbeam-queue` (`ArrayQueue`, `SegQueue`), `crossbeam-utils::CachePadded`.
   - Check: std `mpsc` already runs on crossbeam's implementation (since Rust 1.67) and scoped threads are `std::thread::scope`: neither needs the crate. `std::sync::mpmc` is nightly-only. Async code uses its runtime's channels.
8. **polars**
   - Signal: hand-rolled group-by, join, or aggregation over CSV, Parquet, or JSON records, often `HashMap<K, Vec<_>>`. Column math done row by row.
   - Fix: the polars lazy API (`LazyFrame`), so the query is optimised and run in parallel.
   - Check: row counts in the millions. Compile time and binary size grow a lot: enable only the features used.
9. **datafusion**
   - Signal: a query engine in disguise: SQL-like filters and joins built by hand, Parquet or Arrow scans, data larger than memory.
   - Fix: a DataFusion `SessionContext`, with SQL or its DataFrame API.
   - Check: heavier than polars and async-only. Polars fits in-process dataframe transforms. DataFusion fits SQL, larger-than-memory data, or an embedded query engine.
10. **rkyv**
    - Signal: serde decoding on a hot path: a large cache or index loaded at startup, IPC between your own processes, the same file decoded every run.
    - Fix: rkyv zero-copy: `rkyv::access` reads the archived bytes in place, with no decode step.
    - Check: the format is Rust-only and not self-describing, so a struct change breaks stored data: version it. Untrusted bytes go through validating `access`, never `access_unchecked`. Human-readable or cross-language data stays on serde.

### Cheap wins

11. **Release profile**
    - Signal: `[profile.release]` without `lto` and `codegen-units = 1`. Benches or production builds without `--release`.
    - Fix: `lto = "fat"` (or `"thin"`) and `codegen-units = 1`.
    - Check: release builds get slower. `panic = "abort"` only when nothing catches unwinds.
12. **Hasher**
    - Signal: std `HashMap` or `HashSet` in a hot loop with small keys (integers, short strings).
    - Fix: `rustc_hash::FxHashMap` or `foldhash::HashMap`. Check `cargo tree` for one already present.
    - Check: keys are not attacker-controlled. std's SipHash is the HashDoS defence.
13. **Buffered I/O**
    - Signal: a `File` read or written without `BufReader` or `BufWriter`. `println!` or `write!` to stdout in a loop. `BufRead::lines()`, which allocates one `String` per line.
    - Fix: wrap in `BufReader` or `BufWriter`. Lock stdout once, inside a `BufWriter`. `read_line` into one reused `String`, or read the file whole and iterate `str::lines()` (which also enables pattern 1's borrowed keys).
    - Check: call `flush()` on a `BufWriter` explicitly: a flush error on drop is lost.
14. **Capacity and buffer reuse**
    - Signal: `Vec::new()` or `String::new()` followed by a known number of pushes. A buffer allocated inside the loop on every iteration.
    - Fix: `with_capacity(n)`. Hoist the buffer above the loop and `clear()` it each iteration.
    - Check: n is known or cheaply bounded.
