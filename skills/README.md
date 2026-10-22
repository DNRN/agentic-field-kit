# Skills

Procedural knowledge the agent loads when relevant. Discovery is driven entirely by the frontmatter `description`, so write it as a "use when …" trigger, not a summary.

## Format

```
skills/<kebab-name>/
├── SKILL.md          # required; YAML frontmatter: name, description
└── ...               # optional supporting files, references, scripts
```

- One skill per folder, self-contained, no dependencies on other skills.
- Body = procedure: when to use, steps, gotchas. Not prose documentation.
- Portable: the same folder works in `~/.claude/skills/`, `~/.omp/agent/skills/`, or any harness with skills support.
- Prefer a skill over a command when the *agent* should decide when to apply it.
