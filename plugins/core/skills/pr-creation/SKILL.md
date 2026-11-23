---
name: pr-creation
description: Draft a PR title, summary, and change bullets from the committed and pushed commits on the current branch. Read-only, never creates or edits the PR.
disable-model-invocation: true
argument-hint: "[repo-dir] [base-branch]"
---

Draft the PR text for the current branch of repo `<D>`. Arguments: `$ARGUMENTS`.

`<D>`: the argument that is a directory, as an absolute path. No directory argument: the working directory. The working directory may be a different repo, so every git command runs as `git -C <D>`.

Read-only. Never push, fetch, commit, or run `gh pr create` or `gh pr edit`. Only pushed commits count.

1. Find the pushed tip: `git -C <D> rev-parse --abbrev-ref @{u}`. No upstream: say the branch is not pushed and stop.
2. Pick the base ref `<base>`, first match wins:
   - The other argument: `origin/<argument>`.
   - The stacked parent: the closest of my open PRs that this branch is built on. List them with `gh pr list -R <R> --author @me --state open --json number,headRefName`, where `<R>` is `owner/repo` from `git -C <D> remote get-url upstream`, falling back to `origin`. Skip this branch's own PR. Keep each head where `git -C <D> merge-base --is-ancestor origin/<head> @{u}` succeeds. Closest: fewest commits in `git -C <D> rev-list --count origin/<head>..@{u}`.
   - `origin/HEAD`. If that ref is missing: `origin/main`.
3. List the range: `git -C <D> log --format='%h %s%n%b' <base>..@{u}` and `git -C <D> diff --stat <base>...@{u}`.
4. Read the diff: `git -C <D> diff <base>...@{u}`. Read changed files under `<D>` when the diff alone does not show intent. Describe what the code does, not what the commit messages claim.
5. Check what is left out: `git -C <D> log --oneline @{u}..HEAD` and `git -C <D> status --short`. If either is non-empty, say in one line that it is excluded.
6. Reply in this order:
   - Line 1: `<repo>`: `<branch>` into `<base>`, commit count, files changed. Stacked parent: add `stacked on #<number>`.
   - Title: one fenced block. Format `<type>(<scope>)?: <subject>`, imperative, no trailing period, at most 72 characters. Types: `feat`, `fix`, `chore`, `refactor`, `docs`, `test`, `perf`, `build`, `ci`, `style`, `revert`.
   - Body: one fenced `markdown` block holding `## Summary` (2 to 3 sentences: the problem, then the fix) and `## Changes` (at most 5 short bullets, verb first, related edits grouped, backticks for paths and identifiers).
   - Last line: `Next:` run `! cd <D> && gh pr create --web -B <base branch>` (`<base>` without `origin/`) and paste both blocks.
