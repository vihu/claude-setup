# claude-setup

A Claude Code plugin marketplace that installs my whole setup: skills, hooks, working rules, a context meter, and the third-party plugins they build on.

## Install

1. In Claude Code, run `/plugin marketplace add vihu/claude-setup`
2. Run `/plugin install everything@claude-setup`. This pulls in every plugin listed below.
3. Turn on always-on ADHD mode: `touch ~/.claude/.i-have-adhd-always`
4. Optional: load the rules as your global `CLAUDE.md` instead of through the `rules` plugin. Run `ln -s ~/.claude/plugins/marketplaces/claude-setup/plugins/rules/rules.md ~/.claude/CLAUDE.md`, then `/plugin disable rules@claude-setup` so the rules do not load twice. No plugin can create this link for you.
5. Merge `settings.example.json` into `~/.claude/settings.json` by hand. Plugins cannot set any of these settings.
6. Gitignore the AI docs folder globally: `echo '.ai-docs/' >> ~/.config/git/ignore`

To pick and choose instead, skip step 2 and install single plugins, such as `/plugin install core@claude-setup`.

## Update

Run `/plugin marketplace update claude-setup`. No plugin here pins a `version`, so every push to `main` counts as an update.

To update automatically instead, run `/plugin`, open the Marketplaces tab, select `claude-setup`, and turn auto-update on. It is off by default for third-party marketplaces.

## Plugins

| Plugin                                                                                    | What it adds                                                                                                                                                                            |
| ----------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `everything`                                                                              | Nothing itself. Depends on all the plugins below.                                                                                                                                       |
| `core`                                                                                    | 14 skills (`rust`, `erlang`, `pr-review`, `pr-scoper`, `plan-draft`, `handoff`, `away`, ...), the 80/20 footer hook, the `/away` compaction hook, and the `obscura` browser MCP server. |
| `rules`                                                                                   | My `CLAUDE.md` working rules, injected at every session start. Skip it if you have your own `CLAUDE.md`.                                                                                |
| `context-meter`                                                                           | Context usage against the autocompact threshold, shown above the prompt. `/ctx` shows usage by category.                                                                                |
| `livemark`, `pagebin`, `roughdraft`                                                       | My tools' own skills, installed straight from each tool's repo.                                                                                                                         |
| `ponytail`, `i-have-adhd`, `diagram-design`, `impeccable`, `mattpocock-skills`, `archify` | Third-party plugins, listed here so dependencies resolve from one marketplace. Each keeps its own license and upstream repo.                                                            |

Skills from a plugin are namespaced: `/core:pr-review`, not `/pr-review`.

## Requirements

| Tool                                                                                       | Needed by                                           |
| ------------------------------------------------------------------------------------------ | --------------------------------------------------- |
| [`obscura`](https://github.com/h4ckf0r0day/obscura/releases) on `PATH`                     | `core` MCP server and the `rules` web-reading rules |
| [`gh`](https://cli.github.com), logged in                                                  | `pr-*` skills                                       |
| [`roughdraft`](https://github.com/vihu/roughdraft)                                         | `roughdraft` plugin                                 |
| [`livemark`](https://github.com/vihu/livemark)                                             | `livemark` plugin                                   |
| A [Pagebin](https://github.com/vihu/pagebin) server, `PAGEBIN_URL` and `PAGEBIN_API_TOKEN` | `pagebin` plugin                                    |
| [`uv`](https://docs.astral.sh/uv/) and a `TYPESAFE_API_KEY`                                | `recall` skill                                      |
| `node`                                                                                     | `i-have-adhd` hooks, `impeccable`, `archify`        |

## Developing

Add the marketplace from a local checkout to load the plugins in place, so edits take effect on `/reload-plugins`:

```
claude plugin marketplace add ~/personal/claude-setup
claude plugin validate ~/personal/claude-setup
```
