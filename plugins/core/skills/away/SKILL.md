---
name: away
description: "Hand the current task to Claude for an unattended stretch: questions first, milestone checkpoints, a running handoff file, and a commit script for when I return."
disable-model-invocation: true
argument-hint: "[hours] [focus]"
---

I am stepping away. Hours and focus: `$ARGUMENTS` (empty: about 6 hours, on the task in this conversation). You work alone until I return.

## Steps

1. **Brief.** Ask every question whose answer changes what you build, in one `AskUserQuestion` call. Then post the plan as numbered milestones sized to the hours: each one a commit-sized unit with the command that proves it works. If the session is not in auto mode, say so first: a permission prompt stalls the run until I return. Wait for my "go".
2. **Work.** After "go", run milestone after milestone inside one turn. A milestone is a checkpoint, not a pause. End the turn only when every milestone is done or every remaining one is blocked on me.
3. **Checkpoint** at each finished milestone:
   - Run the project formatter, linter, and the milestone's proving command.
   - Rewrite the handoff file to the real state, in the format of steps 1 and 2 of `${CLAUDE_PLUGIN_ROOT}/skills/handoff/SKILL.md`.
   - Append the milestone to the commit script, then run `bash -n` on it.
4. **Finish.** Final checkpoint, then reply with: line 1 the command to run the commit script, then milestones done, milestones blocked, and the handoff file's open questions.

## Files

One pair per stretch in `.ai-docs/`, sharing one stem picked at the first checkpoint (`NNN` one above the highest `HANDOFF-*`):

- `HANDOFF-NNN-<slug>.md`: the handoff file. The source of truth for the stretch.
- `HANDOFF-NNN-<slug>.commit.sh`: the commit script, for me to run. Shape:

  ```sh
  #!/usr/bin/env bash
  set -euo pipefail
  cd "$(git rev-parse --show-toplevel)"
  # M1: <milestone>
  git add -- <every path this milestone touched>
  git commit -m "<conventional message>"
  ```

  One commit per milestone. `git add` stages whole files, so milestones that touch a shared file become one merged commit.

## Judgement calls while I am away

- **Reversible choice:** pick the conventional default, log it with the reason under `## Decisions`, keep going.
- **Needs me** (destructive command, network write, publishing, anything expensive to undo): log it under `## Open questions`, move to the next unblocked milestone.
- **After a context compaction:** re-read the handoff file before the next action. It outranks the compaction summary.
