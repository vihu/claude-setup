#!/usr/bin/env bash
# SessionStart(compact): point Claude back at an active /away stretch after compaction.
# Active = a HANDOFF-*.commit.sh in .ai-docs/ changed in the last 12 hours.
script=$(find "${CLAUDE_PROJECT_DIR:-.}/.ai-docs" -maxdepth 1 -name 'HANDOFF-*.commit.sh' -mmin -720 2>/dev/null | sort | tail -1)
[ -n "$script" ] || exit 0
skill="$(cd "$(dirname "$0")/.." && pwd)/skills/away/SKILL.md"
echo "An /away stretch is active. Before the next action, re-read $skill and ${script%.commit.sh}.md, then continue from the handoff file."
