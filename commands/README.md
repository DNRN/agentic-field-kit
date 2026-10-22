# Commands

Slash commands, explicitly invoked by the user as `/agentic-toolkit:<name>`.

## Format

```
commands/<kebab-name>.md
```

- Frontmatter: `description`, optional `argument-hint`, `allowed-tools`.
- Use a command when the *user* triggers the action; use a skill when the agent should decide.
- Commands that just inject instructions age better as skills — only keep user-invoked entry points here.
