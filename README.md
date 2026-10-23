# agentic-toolkit

Harness-agnostic collection of agent components: skills, subagents, commands, hooks, output styles, plus copy-paste rules, prompt templates, and permission policies. Works across projects and tools.

Why this repo starts with an [ADR skill](docs/why-adrs.md): tests verify behavior; ADRs verify design — the missing half of most agent workflows. Why the [code review mechanism](docs/why-code-review.md) is three layers: fresh eyes, right moment, ask instead of nag. Why [docs must be kept true](docs/why-docs-matters.md): agents believe stale docs.

## Compatibility

| Component | Claude Code | Codex | Gemini CLI | OpenCode | Cursor | omp |
| --- | --- | --- | --- | --- | --- | --- |
| [`skills/`](skills/) — SKILL.md folders | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| [`agents/`](agents/) · [`commands/`](commands/) · [`hooks/`](hooks/) · [`output-styles/`](output-styles/) | ✓ (plugin) | adapt | adapt | adapt | adapt | adapt |
| [`rules/`](rules/) · [`prompts/`](prompts/) · [`policies/`](policies/) | copy | copy | copy | copy | copy | copy |

Skills follow the [Agent Skills](https://agentskills.io) open standard — identical folders load natively in every harness above. The other component categories currently auto-load only via the Claude Code plugin; adapters for other harnesses are welcome.

## Install skills

Pick the row for your setup. Each harness was checked for what it reads and how it handles a skill found twice ([D8](docs/plans/design-improvements.md#d8-duplicate-skill-loading--done)).

| You use | Install |
| --- | --- |
| Codex, Gemini CLI, OpenCode, Cursor, omp | `./install.sh` |
| Claude Code with the [plugin](#claude-code-full-bundle-also-loads-agents-commands-hooks-styles) | The plugin; add `./install.sh` only for other harnesses. Never `--harness claude`: a personal copy of a plugin skill hides the plugin's `agentic-toolkit:*` name, which the commit gate tells the agent to run |
| Claude Code without the plugin | `./install.sh --harness claude`, plus `--harness agents` for other harnesses |

```bash
./install.sh                              # symlink into ~/.agents/skills (the default)
./install.sh --harness claude --mode copy # another target, or copies instead of links
./install.sh --check                      # validate skills against portability rules
./install.sh --remove                     # undo, in every target
```

| Target | Read by |
| --- | --- |
| `~/.agents/skills/` | Codex, Gemini CLI, OpenCode, Cursor, omp |
| `~/.claude/skills/` | Claude Code; also OpenCode, and Cursor with third-party extensibility on |
| `~/.omp/agent/skills/` | omp; needed only when its `~/.agents` provider is turned off |

- A harness that reads one skill from two targets lists it once; OpenCode logs a warning. Codex is the exception: it lists a same-named skill twice unless both paths resolve to the same folder.
- Codex names a *symlinked* skill `agentic-toolkit:<name>`, because its real path sits inside this plugin repo. `--mode copy` keeps the bare name.
- In Claude Code, a personal `code-review` replaces the built-in `/code-review`.

Symlinks stay live: `git pull` in this repo updates every harness at once. Per-project pinning: git submodule or sparse checkout, then `--mode copy` into the repo's `.agents/skills/`.

## Claude Code full bundle (also loads agents, commands, hooks, styles)

```
/plugin marketplace add DNRN/agentic-toolkit
/plugin install agentic-toolkit@agentic-toolkit
```

`.claude-plugin/` is an optional adapter, not the repo's identity — everything in `skills/` is plain standard folders usable without it.

## What's inside

| Directory | Contents | How it loads |
| --- | --- | --- |
| [`skills/`](skills/) | SKILL.md folders, auto-discovered when relevant | Auto, every harness |
| [`agents/`](agents/) | Subagent definitions (`.md`) | Plugin (Claude Code) |
| [`commands/`](commands/) | Slash commands (`.md`) | Plugin (Claude Code) |
| [`hooks/`](hooks/) | `hooks/hooks.json` + scripts, event-triggered | Plugin (Claude Code) |
| [`output-styles/`](output-styles/) | Output style / persona definitions | Plugin (Claude Code) |
| `.mcp.json` | MCP server configs (add with first server) | Plugin (Claude Code) |
| [`rules/`](rules/) | `AGENTS.md` / `CLAUDE.md` templates | Copy into a project |
| [`prompts/`](prompts/) | Plan / spec / review templates | Referenced by skills & commands |
| [`policies/`](policies/) | Permission allow/deny presets | Copy into project settings |
| [`evals/`](evals/) | Golden tasks per component in `claude plugin eval` format, scored against a no-plugin baseline | [`claude plugin eval .`](evals/README.md#run) |
| [`tests/`](tests/) | Deterministic regression tests for hooks, `install.sh`, and release metadata | `tests/run.sh` |
| [`docs/`](docs/) | Rationale essays, [ADRs](docs/adr/), [plans](docs/plans/) | Read |

## Conventions

- One component = one self-contained folder or file, kebab-case.
- Skills: Agent Skills standard — frontmatter `name` must equal the folder name; keep frontmatter to `name` + `description` (+ optional `license`, `metadata`); the description is the discovery contract — write it as "use when …".
- Every behavioral component ships with eval cases under `evals/<component>/`, including a negative case.
- No secrets, no machine-specific paths; scripts read relative to their component folder.
- Agents, commands, and output styles are registered explicitly in `.claude-plugin/plugin.json` (the folder scan would also load each folder's README); `tests/run.sh` fails on an unregistered file.
- Validate before pushing: `./install.sh --check`, `tests/run.sh`, and `claude plugin validate .` if you touched `.claude-plugin/`. CI ([`.github/workflows/ci.yml`](.github/workflows/ci.yml)) runs these plus shellcheck, an internal link and anchor check, and mermaid rendering on every push to `main` and every PR. CI does not run the [eval suite](evals/README.md) (it needs model credentials and costs money): run it when a skill, agent, or hook changes.

## Contributing

PR per component. Self-contained, follows the conventions above, includes an eval. MIT.

## License

[MIT](LICENSE)
