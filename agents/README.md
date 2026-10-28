# Agents

Subagent definitions. Keep files flat: subdirectories become part of the agent name.

## Format

```
agents/<kebab-name>.md
```

- Frontmatter: `name`, `description` ("use when …" trigger), optional `tools`, `model`.
- One concern per agent (reviewer, scout, migrator). Give it its own context, tools, and model in frontmatter — that's the whole point of a subagent.
- Heavier or rarer work → cheaper/faster model; judgment work → stronger model.
- Register each new file in `.claude-plugin/plugin.json` under `agents`. The explicit list keeps this README from loading as an agent; an unlisted file never loads (`tests/plugin-manifest.test.sh` fails on it).
- Add a row to [`catalog.tsv`](../catalog.tsv) too, with the skill whose rubric it reads under `requires`, so `install.sh` can copy it into a project; `tests/catalog.test.sh` fails on a missing row.
- Agents are harness-specific; skills are not. Anything a skill also needs (a rubric, a checklist, a report format) lives in the skill's `references/`, and the agent is a thin wrapper that reads the file it is given ([ADR 0005](../docs/adr/0005-rubrics-live-in-skills.md)). Never copy that content into the agent.
