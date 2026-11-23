---
name: erlang
description: Erlang/OTP development assistance. Use when working with .erl, .hrl, .app.src, .yrl, .xrl files, rebar3 projects, or asking about Erlang patterns. A rebar3 project will have rebar3.config in the directory.
allowed-tools: Read, Grep, Glob, Bash(rebar3:*, erl:*, dialyzer:*, cat:*, ls:*)
---

# Erlang/OTP Development

Assume the user is an experienced Erlang developer who knows OTP and the BEAM. Do not explain how Erlang works.

## General Guidelines

### Keep records module-private

Never export records or share them between modules. Instead, wrap records with accessor functions and keep the record definition internal to the module.

### Never use `if`

The `if` statement is almost always the wrong choice. Use `case` instead.

### Avoid `=:=` and `=/=`

These operators are only meaningful when comparing floats. For all other comparisons, use `==` and `/=`.

### Do not use the process dictionary

The process dictionary is global mutable state. Do not use it unless explicitly instructed.

### Let it crash

Do not use `try/catch` without good reason. It is usually better to let processes crash and rely on supervision trees for recovery.

## Working with rebar3

### Always use rebar3

When interacting with an Erlang project, always use rebar3 for compilation, testing, and other tasks.

### Finding dependency code

Dependency source code is located at `_build/default/lib/<dep>/src/`, not in a top-level `deps/` directory.

### Editing dependencies

It is acceptable to edit source code in a dependency under `_build/default/lib/<dep>/src/`, but you MUST remove the corresponding `.beam` file under `_build/default/lib/<dep>/ebin/` to force a recompile. Always inform the user when doing this.

For significant edits to a dependency, consider using rebar3 checkouts instead.

### Test output

EUnit and Common Test hide output for successful tests. To force logging to console during tests, use:

```erlang
io:format(user, "Debug: ~p~n", [Value])
```

### After making changes

Keep an eye on these tools after doing work (when available):
- `rebar3 xref`
- `rebar3 dialyzer`
- `rebar3 fmt` (if available)

Always run the test suites (`rebar3 eunit`, `rebar3 ct`) to verify changes.

## Type specs

Add specs to all exported functions, and to complex internal functions. Write specs that are accurate and useful, not just `term() -> term()`.

## Code style

Prioritize clarity. Code should be self-documenting through good naming and structure. Only add comments where they provide necessary context that cannot be expressed in the code itself.

### Module structure

- Group exports logically (API, internal, test-only under `-ifdef(TEST)`)
- Record definitions should have type annotations and defaults where appropriate
- Define types for complex structures; export types when needed (`-export_type`)

### Pattern matching

- Prefer pattern matching in function heads over `case` inside a single function
- Use multiple function clauses for different cases
- Use `#{key := Value}` for required keys in map matches, `#{key => Value}` for construction
- Use binary pattern matching whenever possible

### JSON

Prefer the built-in `json` module over `jsx` for new code. Existing `jsx` usage is technical debt to remove over time.

### Error handling

Use tagged tuples (`{ok, Result}`, `{error, Reason}`) for functions that can fail.

## Testing

Use EUnit for simple code with little state or setup. Use Common Test for complex scenarios involving multiple processes or nodes. It is fine to use both in the same project.

When writing Common Test suites, include the EUnit header to get access to its assertion macros:

```erlang
-include_lib("eunit/include/eunit.hrl").
```

When running tests, capture the full output so you can refer back to it. Avoid re-running tests multiple times to diagnose failures.

Be mindful of context window limits. For large outputs from eunit, ct, dialyzer, or xref, redirect to a file and grep for relevant information. If the output is very large, ask the user for help.

### Coverage Analysis

Generate coverage with:
```bash
rebar3 eunit --cover
rebar3 cover -v
```

Or for Common Test:
```bash
rebar3 ct --cover
rebar3 cover -v
```

Note: The `--cover` flag is redundant if `{cover_enabled, true}` is set in `rebar.config`.

`rebar3 cover -v` prints an ASCII summary to stdout - use this for quick analysis. HTML reports are in `_build/test/cover/aggregate/` for detailed line-by-line analysis.

To find uncovered code in a module, grep the HTML for uncovered lines:

```bash
grep -A3 'class="miss"' _build/test/cover/aggregate/<module>.html | \
  awk '
    /<a href=.*>[0-9]+<\/a>/ {
      match($0, />([0-9]+)<\/a>/)
      line = substr($0, RSTART+1, RLENGTH-5)
    }
    /<code>.*<\/code>/ {
      match($0, /<code>.*<\/code>/)
      code = substr($0, RSTART+6, RLENGTH-13)
      print line ": " code
    }
  '
```

Common uncovered patterns that need explicit tests:
- Error handling clauses (`{error, _}` branches)
- Edge cases (empty lists, zero values, boundary conditions)
- Defensive code (guards, pattern match failures)
- Optional feature paths

## Logging

Use lager for logging, specifically the branch with OTP logger integration.

## Documentation

Add edoc comments to all exported functions. Run `rebar3 edoc` to verify documentation compiles.
