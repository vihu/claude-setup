---
name: handoff
description: Write a self-contained resume prompt for this session to the SilverBullet Hermes space, so a fresh session can continue after /clear. Writes only the handoff page, never continues the task.
disable-model-invocation: true
argument-hint: "[next-focus]"
---

Write a handoff prompt for this session. Focus for the next session: `$ARGUMENTS` (empty: infer it from where the work stopped).

Stop after the handoff. Never continue the task, edit any file or other page, or run state-changing git commands.

1. Check the real state instead of recalling it: `git status --short`, `git diff --stat`, `git log --oneline -10`, the branch and its upstream. Re-read any file whose current content the handoff depends on.
2. Write the prompt to the next agent, in second person and self-contained: no "as discussed", no "above". Body, in order:
   - Line 1: the goal in one sentence, then repo path and branch.
   - `## State`: done, in progress, not started. Each item with the `path:line` where it lives. List uncommitted changes.
   - `## Decisions`: what was chosen and why, what was rejected and why. Include preferences I stated this session.
   - `## Gotchas`: failed attempts, misleading errors, traps. Omit when empty.
   - `## Verify`: the exact commands that prove the current state works.
   - `## Open questions`: for me to answer, not for the agent to guess. Omit when empty.
   - `## Next`: numbered steps, at most 5, the first one doable without asking anything.
   Prefer pointers over copies: cite `path:line` instead of pasting code the next session can read. Spend words on what it cannot re-derive: decisions, reasons, dead ends.
3. Write it with the `silverbullet` MCP, `space: "Hermes"`, as page `<project>/HANDOFF-<NNN>-<slug>` with `status: open`. Read `Library/Writing Hermes Docs` in that space first and follow it for the project name, `NNN` and frontmatter. If an earlier handoff covers the same work, wiki-link it on the last line. Then run `space.lint("<page>")` with `lua`. Fix the page until it returns an empty list.
4. Reply:
   - Line 1: the handoff's page name.
   - Line 2: the goal and the first `## Next` step.
   - `Next:` run `/clear`, then paste: `Read the SilverBullet page <page> in space Hermes and continue from its Next section.`
