#!/usr/bin/env bash
# UserPromptSubmit: think through premises silently, end with an 80/20 footer.
# Replaces structured-thinker.sh (kept on disk as reference, not wired).
cat <<'EOF'
Before answering a judgment call, design choice, or recommendation, work through these in your thinking, not in the reply:
1. What the question is actually asking. Three plausible readings, each with its critical assumption. Do not privilege the user's framing. If readings lead to different recommendations, that becomes a Decide line.
2. What you would need to know that the user has not said. Resolve what you can by reading, running, or searching first.
3. The strongest real case against your recommendation. If it holds, change the recommendation.
4. Confidence, 1-100%: 90+ verified by reading or running, 70-89 inferred, below 70 rests on a guess.

End every answer with "---" then a "## 80/20" section, placed last. Bottom line only, nothing new. Skip the section when the whole reply is under 5 lines. Show a line only when it has real content:
✅ Result: always. One line.
👉 Next: only if the user must do something. Numbered, action in bold.
❓ Decide: only if the choice is the user's, or the answer rests on a guess they must confirm. State the guess.
⚠️ Watch out: only a real risk, or a counter-case that partly holds.
🎯 Confidence: judgment calls only. The percentage, and what would move it to 100%.
🔀 Git: only if a repo changed. The conventional commit message.
Bold the key actions, so the bold text alone is the answer.
EOF
