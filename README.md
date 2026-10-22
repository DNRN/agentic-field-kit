# agentic-toolkit

Harness-agnostic collection of agent components: skills, subagents, commands, hooks, output styles, plus copy-paste rules, prompt templates, and permission policies. Works across projects and tools.

Why this repo starts with an [ADR skill](docs/why-adrs.md): tests verify behavior; ADRs verify design — the missing half of most agent workflows.

## Compatibility

| Component | Claude Code | Codex | Gemini CLI | OpenCode | Cursor | omp |
| --- | --- | --- | --- | --- | --- | --- |
| [`skills/`](skills/) — SKILL.md folders | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| [`agents/`](agents/) · [`commands/`](commands/) · [`hooks/`](hooks/) · [`output-styles/`](output-styles/) | ✓ (plugin) | adapt | adapt | adapt | adapt | adapt |
| [`rules/`](rules/) · [`prompts/`](prompts/) · [`policies/`](policies/) | copy | copy | copy | copy | copy | copy |

Skills follow the [Agent Skills](https://agentskills.io) open standard — identical folders load natively in every harness above. The other component categories currently auto-load only via the Claude Code plugin; adapters for other harnesses are welcome.

## Install skills (all harnesses)

```bash
./install.sh                              # symlink into all three targets below
./install.sh --harness agents --mode copy # subset, or copies instead of links
./install.sh --check                      # validate skills against portability rules
./install.sh --remove                     # undo
```

One symlink target covers most tools — `~/.agents/skills/` is read natively by Codex, Gemini CLI, Cursor, and OpenCode:

| Target | Serves |
| --- | --- |
| `~/.agents/skills/` | Codex, Gemini CLI, Cursor, OpenCode (interop path) |
| `~/.claude/skills/` | Claude Code (also read by Cursor and OpenCode) |
| `~/.omp/agent/skills/` | omp |

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
| [`evals/`](evals/) | Golden tasks per component | `claude plugin eval` |

## Conventions

- One component = one self-contained folder or file, kebab-case.
- Skills: Agent Skills standard — frontmatter `name` must equal the folder name; keep frontmatter to `name` + `description` (+ optional `license`, `metadata`); the description is the discovery contract — write it as "use when …".
- Every behavioral component ships with a matching eval or checklist under `evals/`.
- No secrets, no machine-specific paths; scripts read relative to their component folder.
- Validate before pushing: `./install.sh --check`, and `claude plugin validate .` if you touched `.claude-plugin/`.

## Contributing

PR per component. Self-contained, follows the conventions above, includes an eval. MIT.

## License

[MIT](LICENSE)
