# Commands

Slash commands, explicitly invoked by the user as `/afk:<name>`. None ship today: code review and doc sync are skills, which Claude Code already exposes as `/afk:code-review` and `/afk:doc-sync` ([ADR 0005](../docs/adr/0005-rubrics-live-in-skills.md)).

## Format

```
commands/<kebab-name>.md
```

- Frontmatter: `description`, optional `argument-hint`, `allowed-tools`.
- Use a command when the *user* triggers the action; use a skill when the agent should decide.
- Commands that just inject instructions age better as skills — only keep user-invoked entry points here.
- Portability: commands are the least standard category. Codex has deprecated custom prompts in favor of skills, and OpenCode/Cursor expose skills as slash commands via metadata. For multi-harness behavior, model the command as a skill with explicit-invocation metadata; keep this directory for the Claude Code plugin bundle.
- Register each new file in `.claude-plugin/plugin.json` under `commands`. The explicit list keeps this README from loading as a command; an unlisted file never loads (`tests/plugin-manifest.test.sh` fails on it).
- Never give a command the same name as a skill: in Claude Code the command shadows the skill, and loading the skill by name re-invokes the command.
