# agentic-toolkit

A shared, installable collection of agent components — subagents, skills, commands, hooks, output styles — plus copy-paste rules, prompt templates, and permission policies. Usable across projects and harnesses.

## What's inside

| Directory | Contents | How it loads |
| --- | --- | --- |
| [`skills/`](skills/) | `SKILL.md` folders, auto-discovered when relevant | Auto, via plugin or any skills dir |
| [`agents/`](agents/) | Subagent definitions (`.md`) | Auto, via plugin |
| [`commands/`](commands/) | Slash commands (`.md`) | User-invoked, via plugin |
| [`hooks/`](hooks/) | `hooks/hooks.json` + scripts, event-triggered | Auto, via plugin |
| [`output-styles/`](output-styles/) | Output style / persona definitions | Auto, via plugin |
| `.mcp.json` | MCP server configs (add with first server) | Auto, via plugin |
| [`rules/`](rules/) | `AGENTS.md` / `CLAUDE.md` templates | Copy into a project |
| [`prompts/`](prompts/) | Plan / spec / review templates | Referenced by skills & commands |
| [`policies/`](policies/) | Permission allow/deny presets | Copy into project settings |
| [`evals/`](evals/) | Golden tasks per component | `claude plugin eval` |

## Install

**Claude Code** (bundles skills, agents, commands, hooks, styles):

```
/plugin marketplace add dnrn/agentic-toolkit
/plugin install agentic-toolkit@agentic-toolkit
```

**Other harnesses:** skills are plain `SKILL.md` folders — copy or symlink `skills/<name>/` into your harness's skills directory (e.g. `~/.claude/skills/`, `~/.omp/agent/skills/`).

**Pin per project:** git submodule or sparse checkout of this repo.

## Conventions

- One component = one self-contained folder or file, kebab-case.
- Skills, agents, and commands carry frontmatter `name` + `description`; the description is the discovery contract — write it as "use when …".
- Every behavioral component ships with a matching eval or checklist under `evals/`.
- No secrets, no machine-specific paths; scripts read `${CLAUDE_PLUGIN_ROOT}`.
- Validate before pushing: `claude plugin validate .`

## Contributing

PR per component. Self-contained, follows the conventions above, includes an eval. MIT.

## License

[MIT](LICENSE)
