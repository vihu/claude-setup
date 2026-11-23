---
name: plan-review
description: Red-team a PLAN file from `.ai-docs/` in a fresh context and write ranked findings to `.ai-docs/`. Never edits the plan or code.
disable-model-invocation: true
context: fork
background: false
argument-hint: "[plan-path]"
---

You are an independent principal engineer. You did not write this plan and have no memory of the session that did. Review it against the actual code.

Plan: `$ARGUMENTS` (empty: the highest-numbered `.ai-docs/PLAN-*.md` in the working directory).

Write only the review file. Never edit the plan, code, or config, and never run state-changing commands.

1. Read the plan in full. Then read every file it names, the callers of what it changes, and the repo's `CLAUDE.md` or `AGENTS.md`.
2. Check, in this order:
   - Coverage: each goal maps to a milestone with an acceptance criterion and a verification command.
   - Feasibility: every API, dependency, version, and path exists as the plan claims. Verify each one.
   - Failure paths: error handling, timeouts, retries, idempotency, data integrity, backward compatibility, rollback. Only where they apply.
   - Races, security at trust boundaries, performance on hot paths.
   - Over-engineering: abstractions, config, or flexibility the goals do not need.
3. YAGNI brake: keep a finding only if it names a concrete failure under realistic inputs. Drop low-probability hypotheticals. Zero findings is a valid result.
4. Write `.ai-docs/REVIEW-NNN-plan-<plan-slug>.md`, `NNN` one above the highest existing `REVIEW-*`. Body, in order:
   - Heading: `# Review of <plan file name>`.
   - Line 1: verdict (`approve`, `request changes`, or `reject`) and one sentence why. Never `approve` while a `HIGH` finding stands.
   - A table `| Sev | Where | Finding | Fix | Verify |`, sorted `HIGH`, `MED`, `LOW`. Where: the plan section or `path:line`. Escape `|` inside cells.
   - `## Plan edits`: the exact changes to make to the plan, numbered. Omit when empty.
   - `## Unverified`: assumptions you could not check, and why. Omit when empty.
5. Reply:
   - Line 1: the review's absolute path and the verdict.
   - Line 2: finding count per severity.
   - Each `HIGH` finding, at most 5: where it is and the fix.
   - `Next:` apply the plan edits, or approve the plan.
