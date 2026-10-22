# Agents

Subagent definitions. Keep files flat: subdirectories become part of the agent name.

## Format

```
agents/<kebab-name>.md
```

- Frontmatter: `name`, `description` ("use when …" trigger), optional `tools`, `model`.
- One concern per agent (reviewer, scout, migrator). Give it its own context, tools, and model in frontmatter — that's the whole point of a subagent.
- Heavier or rarer work → cheaper/faster model; judgment work → stronger model.
