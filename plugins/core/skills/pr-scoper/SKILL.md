---
name: pr-scoper
description: Check a GitHub PR for scope creep and YAGNI, KISS, and DRY violations, each pinned to a file and line, and write the findings to `.ai-docs/`. Complements `/core:pr-review`. Read-only, never posts to GitHub.
disable-model-invocation: true
argument-hint: "[pr-number-or-url] [repo-dir]"
---

Scope-check a PR and write the review to `.ai-docs/`. Arguments: `$ARGUMENTS`.

Resolve the target first. The working directory may be a different repo, so every command names the target:

- `<D>`: the argument that is a directory, as an absolute path. No directory argument: the working directory.
- `<R>` (`owner/repo`): from a PR URL argument. Else from `git -C <D> remote get-url upstream`, falling back to `origin`.
- `<n>`: the PR number from the URL or the number argument. Else the PR for `<D>`'s branch: `gh pr view <branch> -R <R> --json number -q .number`, with `<branch>` from `git -C <D> branch --show-current`.
- Local checkout: `<D>`, when its remote names `<R>`. Without one, read every file with `gh api -H 'Accept: application/vnd.github.raw' 'repos/<R>/contents/<path>?ref=<sha>'`.

Read-only. Never post, comment, review, approve, request changes, react, resolve, or edit anything on GitHub. `gh api` is GET only: never `-X` other than `GET`, `-f`, `-F`, or `--input` (each one turns it into a write). Never fetch, checkout, commit, or push. Do not change code.

This review owns scope and simplicity: creep, YAGNI, KISS, DRY. Correctness, security, and performance belong to `/core:pr-review`; leave them there.

1. Load the PR: `gh pr view <n> -R <R> --json number,title,body,url,baseRefName,headRefName,headRefOid,closingIssuesReferences,commits,additions,deletions`. Read linked issues with `gh issue view <issue> -R <R>`. Read the repo's `AGENTS.md`, `CLAUDE.md`, or `CONTRIBUTING.md` for its scope and style rules.
2. Write the scope: one sentence stating what the PR sets out to do. Sources by weight: linked issues, then the body, then the title. Commit messages only fill gaps. If none states a goal, the title is the scope; say so in the review.
3. Read the diff: `gh pr diff <n> -R <R>`. It is what GitHub shows, so it wins over local refs. Read each changed file in full at the head SHA (`headRefOid`):
   - Local checkout, `git -C <D> rev-parse HEAD` equals it, and `git -C <D> status --short` is empty: the working tree under `<D>`.
   - Else, if there is a local checkout and `git -C <D> cat-file -e <sha>` succeeds: `git -C <D> show <sha>:<path>`.
   - Else: the `gh api` read above.
4. Label every hunk with the removal test: if deleting the hunk leaves the scope working, tested, and documented, it is `creep`; otherwise it is in scope. Typical creep: drive-by fixes, unrelated refactors or renames, formatting churn on lines the scope never needed, dependency bumps, and features, flags, or options nobody asked for. Group adjacent creep hunks with one cause into one finding. Done when every hunk has a label.
5. Hunt the in-scope hunks:
   - `yagni`: an abstraction with one implementation, a parameter, flag, or config no caller sets, generic code with one concrete use, an export nothing imports, code for a future case.
   - `kiss`: indirection a direct call would replace, clever code where boring code is as short, hand-rolled logic that the stdlib, the platform, or an installed dependency ships (name it).
   - `dry`: one source of truth per piece of knowledge. Flag logic that re-implements a helper already in the repo, and a constant, rule, or validation written twice. Look-alike blocks that change for different reasons stay separate until a third copy appears.
6. Verify each finding before keeping it. Point at the lines. For `creep`, name the scope it falls outside. For `yagni` and `dry`, cite the search that proves it: `rg` under the local checkout, else `gh search code -R <R> '<term>'`. Drop anything you cannot prove. Zero findings is a valid result.
7. Rate each finding:
   - `HIGH`: creep that changes behaviour outside the scope. A new public API, abstraction, or dependency the scope does not need.
   - `MED`: creep big enough to review on its own. A `yagni`, `kiss`, or `dry` finding that adds a layer, an option, or a second copy of a helper.
   - `LOW`: small drive-bys and local simplifications.
8. Pick the verdict:
   - `split`: any `HIGH` creep, or creep over a quarter of the changed lines.
   - `trim`: any other `HIGH` or `MED` finding.
   - `lean`: the rest.
9. Write `.ai-docs/REVIEW-NNN-<repo>-<n>-scope.md` under the local checkout, else under the working directory (create `.ai-docs/` if missing), `NNN` one above the highest existing `REVIEW-*` there. Body, in order:
   - Heading: `# Scope review <repo>#<n> <short PR subject>`.
   - Line 1: verdict and one sentence why.
   - Line 2: the scope sentence.
   - Line 3: PR URL, `<head>` into `<base>`, short head SHA, `+<additions> -<deletions>`, creep line count, and `net: -<N> lines possible` (lines the fixes would remove). If there is no local checkout, or its `HEAD` differs from the head SHA, say the review covers the pushed head only.
   - A table `| Sev | Kind | Location | Comment |`, sorted `HIGH`, `MED`, `LOW`. Kind: `creep`, `yagni`, `kiss`, or `dry`. Location: `` `path:line` `` or `` `path:start-end` ``, new-side lines at the head SHA. Comment: one or two sentences, the problem then the fix, ready to paste as a review comment. Escape `|` inside cells.
   - `## Split plan`, for verdict `split` only: one line per follow-up PR, a conventional-commit title and the hunks it takes.
   - `## Not reviewed`: skipped files (generated, lockfiles, vendored) and why. Omit when empty.
10. Reply:
    - Line 1: the review's absolute path and the verdict.
    - Line 2: finding count per kind, and `net: -<N> lines possible`.
    - Each `HIGH` finding, at most 5: `path:line` and its comment.
    - `Next:` open the review.
