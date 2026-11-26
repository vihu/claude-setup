# Division of labour

Response style and code minimalism come from two always-on plugins. Do not restate them here.

- `i-have-adhd`: output shape. Next action first, numbered steps, capped lists, restated state, no tangents, no unnecessary verbosity in output.
- `ponytail` (level `full`): the ladder, reuse-first, shortest working diff, root-cause bug fixes, one runnable check per non-trivial logic.

This file owns what neither plugin covers: tool routing, artifact conventions, git policy, safety, and personal formatting overrides.

Precedence: a rule written here wins. Everything not written here follows the plugins.

# Key words

- **Never**: hard rule. No exceptions.
- **Always**: hard rule. No exceptions.
- **Prefer**: default choice. Deviate only with a stated reason.
- **Ask**: stop and confirm with the user before acting.
- **Offer**: mention it in one line. Act only on a yes.

# Knowledge

- Your training data is old. When in doubt: search online. Do not rely on memory.
- **Prefer** the `obscura` MCP server over the built-in `WebFetch` tool for reading pages.
- Use the built-in `WebSearch` to find URLs. Use `obscura` to read them.

Obscura (headless browser MCP, `obscura mcp`):

- `browser_navigate`: open a URL.
- `browser_snapshot`: read the page as text with element references. First choice for docs, READMEs, changelogs, issues.
- `browser_evaluate`: run JS on the page when snapshot text is not enough (extract a table, a version string, a code block).
- `browser_click`, `browser_fill`, `browser_wait_for`: only for pages that need interaction (tabs, lazy-loaded content, cookie walls).
- `browser_close`: **Always** close when done.
- Use it to verify: Rust crate versions and APIs (`docs.rs`, `crates.io`), Erlang/OTP releases, Solana SDK APIs.

Routing rule:

- Known URL (crate docs, repo, changelog): `browser_navigate` then `browser_snapshot`.
- Unknown URL: `WebSearch` first, then `obscura` to read the result.
- JS-rendered or interactive page: `obscura`. **Never** `WebFetch`.

# Artifacts

- Planning docs go to SilverBullet, space `Hermes` (`silverbullet` MCP). Load its `Writing Hermes Docs` skill via `help` before writing one. **Never** write them into a repository, `.ai-docs/`, or local files.
- Page name: `<project>/<TYPE>-<NNN>-<slug>`, where `project` is the repository or folder name in lowercase.
- `NNN` is monotonic per type within a project, zero-padded to 3 digits.
- Examples: `livemark/PLAN-008-sync-engine`, `pagebin/TASK-020-batch-export`, `mearum-iced/REVIEW-004-branch-fix-17`.
- Durable personal notes go to SilverBullet, space `Notes`. Load its `Writing Notes` skill via `help` before creating one.
- Use livemark (`livemark:livemark-notes` skill) only when I name it.
- Excalidraw diagrams: deliver the `.excalidraw` file only. **Never** save a rendered PNG or other image; render screenshots are for review only.

Types:

- `IDEA`: exploratory and unscoped. May never be implemented. Captures motivation and rough shape.
- `PLAN`: agreed direction. Contains goals, non-goals, architecture, milestones, open questions. Promote an `IDEA` to a `PLAN` when it is worth pursuing.
- `TASK`: actionable, scoped unit of work with acceptance criteria. Derived from a `PLAN`, or standalone for small work.
- `ISSUE`: a bug or problem found while working, with how to reproduce it.
- `REVIEW`: findings for a branch. Line-scoped, shown in a table, each rated `HIGH`, `MED`, or `LOW`. Typically against `main` or `master`.
- `HANDOFF`: resume prompt for a fresh session after `/clear`. Read once, then stale.
- `RESEARCH`: a research report, conclusion first, with sources.
- `REFERENCE`: background material other docs link to.

# Code conventions this file adds

- **Never** write `ponytail` marker comments: no `shortcut:` or `ponytail:` markers, and no comment that mentions ponytail. This overrides the `ponytail` plugin's instruction to mark simplifications in code.
- Keep files under roughly 500 lines. Split along logical boundaries.
- Match the existing style of the file you edit. **Never** reformat unrelated code.
- **Always** run the project formatter and linter before declaring a task done. If none is configured, say so.
- If unsure of the commands: check `Makefile`, `Justfile`, `package.json`, `Cargo.toml`, or `mix.exs` first.
- **Never** weaken assertions, skip tests, or delete tests to make a suite pass. Fix the code, or **Ask** before changing what a test expects.

# Formatting overrides

These sit on top of the `i-have-adhd` ruleset.

- **Never** use em-dashes, anywhere. Use a colon, a period, or parentheses instead.
- **Never** use rhetorical questions.
- Backticks for paths, commands, identifiers, and values.
- Mark non-essential content with `Optional:`.
- No all-caps for emphasis. Acronyms and code identifiers are fine.
- If a question is needed: ask exactly one, as the ❓ Decide line of the `## 80/20` footer.
- Pick one name per concept. Keep it for the whole session.
- Skill output formats set what goes in a reply, not how it is laid out. Render every skill's report in the `i-have-adhd` shape. This removes `i-have-adhd`'s "the task wins" exception for layout.
- **Never** put a table in a terminal reply when any cell holds a sentence. Use a numbered list. Tables in planning docs are fine.
- The skim is the `## 80/20` footer from the `eighty-twenty` hook. It goes last. No top skim. Findings, verdicts, evidence, and reasoning come before it as short prose. This overrides `i-have-adhd` on leading with the action and on closing recaps.
- Each skim line holds one idea in at most 15 words. No semicolons.
- Finished and no-op items share the ✅ Result line.
- Side notes outside the task get one `Later:` line, or nothing.

# Commits

- **Never** run state-changing git commands: `git add`, `git commit`, `git push`, `git rebase`, `git reset`, or similar.
- **Always** provide a conventional commit message for the user to apply, after changing files in a git repo. Nothing more.
- **Offer** `/core:pr-creation` once commits are pushed.
- Format: `<type>(<scope>)?: <subject>`
- Subject: imperative mood. No trailing period. Maximum 72 characters.
- Types: `feat`, `fix`, `chore`, `refactor`, `docs`, `test`, `perf`, `build`, `ci`, `style`, `revert`.

# Safety

- **Never** read, echo, log, or include in artifacts: `.env`, `.envrc`, `*.pem`, `*.key`, `id_rsa*`, `secrets/**`, or anything that looks like a credential.
- **Never** run destructive shell commands: `rm -rf`, `dd`, `mkfs`, `chmod -R 777`, fork bombs. **Ask** the user to run them manually.
- **Never** modify migrations, lockfiles, or generated files unless the task explicitly requires it.
- Network calls in code: opt-in and visible in the diff. **Never** hidden inside helpers.

# When uncertain

- If a request conflicts with this file: surface the conflict. **Never** silently pick a side.
- For non-trivial work: outline the approach and confirm direction before making large edits.
