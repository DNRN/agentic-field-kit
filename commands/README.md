# Commands

Slash commands, explicitly invoked by the user as `/agentic-toolkit:<name>`.

## Format

```
commands/<kebab-name>.md
```

- Frontmatter: `description`, optional `argument-hint`, `allowed-tools`.
- Use a command when the *user* triggers the action; use a skill when the agent should decide.
- Commands that just inject instructions age better as skills — only keep user-invoked entry points here.
- Portability: commands are the least standard category. Codex has deprecated custom prompts in favor of skills, and OpenCode/Cursor expose skills as slash commands via metadata. For multi-harness behavior, model the command as a skill with explicit-invocation metadata; keep this directory for the Claude Code plugin bundle.
