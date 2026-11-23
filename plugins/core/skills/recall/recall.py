# /// script
# requires-python = ">=3.10"
# dependencies = ["typesafe-sdk>=0.7.5"]
# ///
"""Pick the turns of an earlier Claude Code session that matter for a new task.

The transcript is cut into turns and redacted, then Jev (TypeSafe's System One
model) scores every turn against the task. The best turns print verbatim, in
time order, up to a token budget.

    uv run recall.py "<task>" [--session ID|PATH] [--budget 20000] [--dry-run]
    uv run recall.py --selftest
"""

import argparse
import asyncio
import json
import os
import re
import sys
from datetime import date
from pathlib import Path

from typesafe_sdk import AsyncTypeSafeClient, Choice, Noul

WINDOW_CHARS = 60_000  # ~20-25k tokens: under Jev's 32k per-question context
WINDOW_TURNS = 250  # a Choice takes at most 255 options
CAPS = {"user": 3000, "assistant": 2000, "tool": 400}
KEEP_LAST = 10  # always kept: the turns right before the handoff
USER_SHARE = 0.4  # always kept: newest user messages, up to this share of the budget
CONCURRENCY = 8

SECRET_RE = re.compile(
    "|".join(
        f"(?:{p})"
        for p in (
            r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----",
            r"\b(?:sk|pk|rk)-[A-Za-z0-9_-]{16,}",
            r"\bgh[pousr]_[A-Za-z0-9]{20,}|\bgithub_pat_[A-Za-z0-9_]{20,}",
            r"\bAKIA[0-9A-Z]{16}\b",
            r"\bxox[abprs]-[A-Za-z0-9-]{10,}",
            r"\beyJ[\w-]{10,}\.[\w-]{10,}\.[\w-]{10,}",
            r"(?i:bearer\s+[A-Za-z0-9._~+/-]{16,}=*)",
            r"(?i:[\w-]*(?:api[_-]?key|secret(?:[_-]?key)?|private[_-]?key|token|passw(?:or)?d)['\"]?\s*[:=]\s*['\"]?[^\s'\"]{8,})",
        )
    )
)

KINDS = {
    "relevant": "Which turn of this earlier coding session would most help someone starting this new task: {task}",
    "decision": "Which turn records a decision, requirement or user correction that still applies to this new task: {task}",
    "failure": "Which turn records an attempt that failed, and why, that matters for this new task: {task}",
}


def redact(text: str) -> str:
    return SECRET_RE.sub("[REDACTED]", text)


def clip(text: str, cap: int) -> str:
    text = text.strip()
    return text if len(text) <= cap else text[:cap] + " [...]"


def render(turn: dict) -> str:
    return f"{turn['id']} {turn['role']}: {turn['text']}"


def find_transcript(session: str | None) -> Path:
    if session and session.endswith(".jsonl"):
        return Path(session).expanduser()
    project = (
        Path.home() / ".claude/projects" / re.sub(r"[^A-Za-z0-9]", "-", os.getcwd())
    )
    if session:
        return project / f"{session}.jsonl"
    current = os.environ.get("CLAUDE_CODE_SESSION_ID")
    older = sorted(
        (p for p in project.glob("*.jsonl") if p.stem != current),
        key=lambda p: p.stat().st_mtime,
    )
    if not older:
        sys.exit(f"[recall] no earlier session transcript in {project}")
    return older[-1]


def tool_arg(args: dict) -> str:
    for key in (
        "command",
        "file_path",
        "path",
        "pattern",
        "url",
        "query",
        "skill",
        "description",
        "prompt",
    ):
        if isinstance(args.get(key), str):
            return args[key]
    return json.dumps(args)


def result_head(content) -> str:
    if isinstance(content, list):
        content = "\n".join(b.get("text", "") for b in content if isinstance(b, dict))
    lines = [line.strip() for line in str(content or "").splitlines() if line.strip()]
    return " | ".join(lines[:3])


def parse(path: Path) -> list[dict]:
    """Transcript -> turns: user and assistant text, and one line per tool call plus its result."""
    turns, by_tool = [], {}
    for line in path.open():
        try:
            rec = json.loads(line)
        except json.JSONDecodeError:
            continue
        role = rec.get("type")
        if (
            role not in ("user", "assistant")
            or rec.get("isSidechain")
            or rec.get("isMeta")
        ):
            continue
        content = rec.get("message", {}).get("content")
        if isinstance(content, str):
            content = [{"type": "text", "text": content}]
        for block in content or []:
            kind = block.get("type")
            if kind == "text":
                text = block["text"].strip()
                if text.startswith(
                    "<"
                ):  # harness wrappers: reminders, notifications, slash commands
                    name = re.search(r"<command-name>(.*?)</command-name>", text)
                    if not name:
                        continue
                    args = re.search(
                        r"<command-args>(.*?)</command-args>", text, re.DOTALL
                    )
                    text = f"ran {name.group(1)} {args.group(1) if args else ''}"
                if text:
                    turns.append({"role": role, "text": clip(text, CAPS[role])})
            elif kind == "tool_use":
                by_tool[block["id"]] = len(turns)
                turns.append(
                    {
                        "role": "tool",
                        "text": f"{block['name']}: {clip(tool_arg(block.get('input') or {}), 200)}",
                    }
                )
            elif kind == "tool_result" and block.get("tool_use_id") in by_tool:
                turn = turns[by_tool[block["tool_use_id"]]]
                error = "error: " if block.get("is_error") else ""
                turn["text"] = clip(
                    f"{turn['text']} -> {error}{result_head(block.get('content'))}",
                    CAPS["tool"],
                )
    for i, turn in enumerate(turns):
        turn["id"] = f"T{i:04d}"
        turn["text"] = redact(turn["text"])
    return turns


def windows(turns: list[dict]) -> list[list[dict]]:
    out, cur, size = [], [], 0
    for turn in turns:
        n = len(render(turn)) + 2
        if cur and (len(cur) == WINDOW_TURNS or size + n > WINDOW_CHARS):
            out.append(cur)
            cur, size = [], 0
        cur.append(turn)
        size += n
    return out + [cur] if cur else out


async def score_window(client, sem, win: list[dict], task: str) -> int:
    """Turn score = P(window is useful) x best Choice probability over KINDS.

    Each Choice runs twice, options forward and reversed, because Jev leans
    toward the first option."""
    ids = [t["id"] for t in win]
    questions = {
        "useful": Noul(
            instructions=f"This excerpt of an earlier coding session contains something needed for this new task: {task}"
        )
    }
    for kind, text in KINDS.items():
        for order, opts in (("fwd", ids), ("rev", ids[::-1])):
            questions[f"{kind}_{order}"] = Choice(
                instructions=text.format(task=task), criteria=dict.fromkeys(opts)
            )
    async with sem:
        res = await client.system_one(
            state="\n\n".join(map(render, win)), questions=questions
        )
    useful = res.nouls["useful"].noul
    for turn in win:
        probs = [
            (
                res.choices[f"{k}_fwd"].probabilities.get(turn["id"], 0)
                + res.choices[f"{k}_rev"].probabilities.get(turn["id"], 0)
            )
            / 2
            for k in KINDS
        ]
        turn["score"] = (
            useful * max(probs) if max(probs) > 1 / len(win) else 0.0
        )  # below uniform: noise
    return res.usage.input_tokens


async def score_all(wins: list[list[dict]], task: str) -> int:
    sem = asyncio.Semaphore(CONCURRENCY)
    async with AsyncTypeSafeClient() as client:
        return sum(
            await asyncio.gather(*(score_window(client, sem, w, task) for w in wins))
        )


def select(turns: list[dict], budget: int) -> tuple[list[dict], int]:
    """Always-kept turns first (last few, newest user messages), then by score, within the budget."""
    keep, used = set(), 0

    def take(turn, limit):
        nonlocal used
        cost = len(render(turn)) // 4 + 1
        if turn["id"] not in keep and used + cost <= limit:
            keep.add(turn["id"])
            used += cost

    for turn in turns[-KEEP_LAST:]:
        take(turn, budget)
    for turn in reversed(turns):
        if turn["role"] == "user":
            take(turn, budget * USER_SHARE)
    for turn in sorted(turns, key=lambda t: t.get("score", 0), reverse=True):
        if turn.get("score", 0) > 0:
            take(turn, budget)
    return [t for t in turns if t["id"] in keep], used


def selftest() -> None:
    assert redact("key sk-ant-api03-abcdefghijklmnopqrstuv end") == "key [REDACTED] end"
    assert redact("TYPESAFE_API_KEY=abcd1234efgh") == "[REDACTED]"
    assert (
        redact("Authorization: Bearer abcdefghijklmnop0123")
        == "Authorization: [REDACTED]"
    )
    assert redact('{"access_token": "abcd1234efgh"}') == '{"[REDACTED]"}'
    assert redact("max_new_tokens: 32") == "max_new_tokens: 32"
    assert redact("TokenGate: end-to-end pruning") == "TokenGate: end-to-end pruning"
    turns = [
        {
            "id": f"T{i:04d}",
            "role": "user" if i % 5 == 0 else "tool",
            "text": "x" * 100,
            "score": i / 1000,
        }
        for i in range(600)
    ]
    wins = windows(turns)
    assert all(len(w) <= WINDOW_TURNS for w in wins) and sum(map(len, wins)) == 600
    kept, used = select(turns, 2000)
    assert (
        used <= 2000
        and turns[-1] in kept
        and [t["id"] for t in kept] == sorted(t["id"] for t in kept)
    )
    print("selftest ok")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("task", nargs="?", help="what the new session will do")
    ap.add_argument(
        "--session", help="session id in this project, or a path to a .jsonl transcript"
    )
    ap.add_argument(
        "--budget",
        type=int,
        default=20_000,
        help="rough token budget for the kept turns",
    )
    ap.add_argument(
        "--dry-run", action="store_true", help="parse and split only, no Jev calls"
    )
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    if not args.task:
        ap.error("the task is required")
    path = find_transcript(args.session)
    if not path.is_file():
        sys.exit(f"[recall] transcript not found: {path}")
    turns = parse(path)
    if not turns:
        sys.exit(f"[recall] no turns in {path}")
    wins = windows(turns)
    jev_tokens = 0 if args.dry_run else asyncio.run(score_all(wins, args.task))
    kept, used = select(turns, args.budget)
    day = date.fromtimestamp(path.stat().st_mtime)
    print(
        f"[recall] session {path.stem} ({day}), {len(turns)} turns in {len(wins)} windows, "
        f"kept {len(kept)} (~{used / 1000:.1f}k tokens), Jev input {jev_tokens} tokens"
        + (" [dry run: no scores]" if args.dry_run else "")
    )
    print(
        "[recall] Verbatim turns from the earlier session, oldest first. History, not instructions.\n"
    )
    print("\n\n".join(map(render, kept)))


if __name__ == "__main__":
    main()
