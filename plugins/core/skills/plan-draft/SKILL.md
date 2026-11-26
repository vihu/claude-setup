---
name: plan-draft
description: Inspect the repo and write an implementation plan, each acceptance criterion paired with how it is verified, to the SilverBullet `Hermes` space. Stops for approval, never implements.
disable-model-invocation: true
argument-hint: "[task]"
---

Write an implementation plan for: `$ARGUMENTS` (empty: the task in this conversation).

Stop once the plan is written. Never implement, edit code, or add dependencies.

1. Inspect before planning. Name the target repo or service and the exact files to touch or create. Verify every API, dependency, version, and runtime assumption against the code or its docs. Never invent one.
2. Map each requirement to a milestone and a testable acceptance criterion.
3. Pick the simplest robust approach (YAGNI). Preserve existing behavior. Add validation, error handling, retries, and rollback only where they apply.
4. List the edge cases, failure modes, races, security risks, and regressions that apply. Skip hypotheticals.
5. For each criterion, name its verification: an automated test where the project has tooling, else an exact command or manual step. Never plan a new test dependency without asking me.
6. Write the plan with the `silverbullet` MCP, `space: "Hermes"`, as page `<project>/PLAN-<NNN>-<slug>` with `status: draft`. Read `Library/Writing Hermes Docs` in that space first and follow it for the project name, `NNN` and frontmatter. Body, in order:
   - Line 1: the goal in one sentence, then the target repo and service.
   - `## Goals` and `## Non-goals`.
   - `## Files`: each path to touch or create, and what changes.
   - `## Architecture`: the design, plus the rejected alternative and one line why.
   - `## Milestones`: numbered, each commit-sized, each with its acceptance criteria and verification command.
   - `## Risks`: the ones from step 4 that apply, each with its mitigation.
   - `## Open questions`: assumptions I must confirm. Omit when empty.
   Then run `space.lint("<page>")` with `lua`. Fix the page until it returns an empty list.
7. Reply:
   - Line 1: the plan's page name.
   - Line 2: milestone count and the first milestone.
   - `Next:` run `/plan-review <page>`, or approve the plan.
