---
name: pr-review
description: Review a GitHub PR thoroughly and write the findings, each pinned to a file and line, to the SilverBullet `Hermes` space. Read-only, never posts to GitHub.
disable-model-invocation: true
argument-hint: "[pr-number-or-url] [repo-dir]"
---

Review a PR and write the review to the SilverBullet `Hermes` space. Arguments: `$ARGUMENTS`.

Resolve the target first. The working directory may be a different repo, so every command names the target:

- `<D>`: the argument that is a directory, as an absolute path. No directory argument: the working directory.
- `<R>` (`owner/repo`): from a PR URL argument. Else from `git -C <D> remote get-url upstream`, falling back to `origin`.
- `<n>`: the PR number from the URL or the number argument. Else the PR for `<D>`'s branch: `gh pr view <branch> -R <R> --json number -q .number`, with `<branch>` from `git -C <D> branch --show-current`.
- Local checkout: `<D>`, when its remote names `<R>`. Without one, read every file with `gh api -H 'Accept: application/vnd.github.raw' 'repos/<R>/contents/<path>?ref=<sha>'`.

Read-only. Never post, comment, review, approve, request changes, react, resolve, or edit anything on GitHub. `gh api` is GET only: never `-X` other than `GET`, `-f`, `-F`, or `--input` (each one turns it into a write). Never fetch, checkout, commit, or push. Do not change code.

1. Load the PR: `gh pr view <n> -R <R> --json number,title,body,url,baseRefName,headRefName,headRefOid,closingIssuesReferences`. Read linked issues with `gh issue view <issue> -R <R>`. CI: `gh pr checks <n> -R <R>`.
2. Read the diff: `gh pr diff <n> -R <R>`. It is what GitHub shows, so it wins over local refs.
3. Read each changed file in full at the head SHA (`headRefOid`):
   - Local checkout, `git -C <D> rev-parse HEAD` equals it, and `git -C <D> status --short` is empty: the working tree under `<D>`.
   - Else, if there is a local checkout and `git -C <D> cat-file -e <sha>` succeeds: `git -C <D> show <sha>:<path>`.
   - Else: the `gh api` read above.
4. Read what the change touches: callers of changed functions, tests for the changed code, and the repo's `AGENTS.md`, `CLAUDE.md`, or `CONTRIBUTING.md`.
5. Hunt, in this order: correctness and edge cases, error handling and data loss, security at trust boundaries, what the PR body or linked issue asks for that the code does not do, then missing tests for new logic. Skip what a formatter would fix.
6. Verify each finding against the code before keeping it: trace the path end to end. Drop anything you cannot point at. Zero findings is a valid result.
7. Write the review with the `silverbullet` MCP, `space: "Hermes"`, as page `<repo>/REVIEW-<NNN>-pr-<n>` with `status: open`, where `<repo>` is the name half of `<R>` in lowercase. Read `Library/Writing Hermes Docs` in that space first and follow it for `NNN` and frontmatter (`repo: <R>`). Body, in order:
   - Heading: `# Review <repo> PR <n> <short PR subject>`.
   - Line 1: verdict (`approve`, `comment`, or `request changes`) and one sentence why.
   - Line 2: PR URL, `<head>` into `<base>`, short head SHA, CI status. If there is no local checkout, or its `HEAD` differs from the head SHA, say the review covers the pushed head only.
   - A table `| Sev | Location | Comment |`, sorted `HIGH`, `MED`, `LOW`. Location: `` `path:line` `` or `` `path:start-end` ``, new-side lines at the head SHA. Comment: one or two sentences, the problem then the fix, ready to paste as a review comment. Escape `|` inside cells.
   - `## Not reviewed`: skipped files (generated, lockfiles, vendored) and why. Omit when empty.
   Then run `space.lint("<page>")` with `lua`. Fix the page until it returns an empty list.
8. Reply:
   - Line 1: the review's page name and the verdict.
   - Line 2: finding count per severity.
   - Each `HIGH` finding, at most 5: `path:line` and its comment.
   - `Next:` open the review.
