---
name: pr-comments
description: Investigate review comments on a GitHub PR and report back. Read-only, never posts to GitHub.
disable-model-invocation: true
argument-hint: "[pr-number-or-url] [repo-dir]"
---

Investigate the comments on a PR. Arguments: `$ARGUMENTS`.

Resolve the target first. The working directory may be a different repo, so every command names the target:

- `<D>`: the argument that is a directory, as an absolute path. No directory argument: the working directory.
- `<R>` (`owner/repo`): from a PR URL argument. Else from `git -C <D> remote get-url upstream`, falling back to `origin`.
- `<n>`: the PR number from the URL or the number argument. Else the PR for `<D>`'s branch: `gh pr view <branch> -R <R> --json number -q .number`, with `<branch>` from `git -C <D> branch --show-current`.
- Local checkout: `<D>`, when its remote names `<R>`. Without one, read every file with `gh api -H 'Accept: application/vnd.github.raw' 'repos/<R>/contents/<path>?ref=<sha>'`.

Read-only. Never post, reply, react, resolve, approve, request changes, or edit the PR. Never push. Do not change code unless I ask.

1. Fetch conversation comments and reviews: `gh pr view <n> -R <R> --comments`
2. Fetch inline review comments: `gh api repos/<R>/pulls/<n>/comments`
3. For each comment, read the code it points at. Verdict: valid, partly valid, not valid, or already fixed.
4. Reply as an action plan, not a findings report. No table.
   - Line 1: the action that unblocks the merge, verb first, with a time estimate. Stretch it to hold the merge status and the already-fixed and no-op comments.
   - Numbered steps for the most urgent open comment, at most 5, verb first, one action each. Stretch a step to hold its claim, verdict, and evidence.
   - Last step: push and post the reply. Stretch it to hold the draft reply text.
   - Other open comments: one `Then:` line each. Follow-ups outside this PR: one `Later:` line.
5. End with `Next:` and one action that takes under 2 minutes.
