# Skills

Procedural knowledge the agent loads when relevant. Skills are the portable core of this repo — every harness here reads the same folders.

## Standard

[Agent Skills](https://agentskills.io) open standard: one folder per skill with a `SKILL.md`. Portability rules (enforced by OpenCode, Cursor, and `./install.sh --check`):

- Folder name = frontmatter `name`, lowercase alphanumeric with single hyphens (`kebab-name`).
- Frontmatter: only `name` + `description` (+ optional `license`, `metadata` string map). No custom top-level fields — tool-specific toggles go in `metadata` (quoted values), e.g. OpenCode's `opencode/slash`, `opencode/autoinvoke`.
- `description` is the discovery contract: write it as "use when …", front-load trigger words. Harnesses shorten or drop it under context pressure.
- Optional subdirs, loaded progressively: `scripts/`, `references/`, `assets/`. Reference them by relative path from the skill root.

## Format

```
skills/<kebab-name>/
├── SKILL.md          # required; YAML frontmatter: name, description
├── scripts/          # optional
├── references/       # optional
└── assets/           # optional
```

- One skill per folder, self-contained, no dependencies on other skills.
- Body = procedure: when to use, steps, gotchas. Not prose documentation.
- Content a subagent also applies (a review rubric, an audit checklist) lives once in `references/`; the skill passes its location when delegating. See `code-review/references/rubric.md` and [ADR 0005](../docs/adr/0005-rubrics-live-in-skills.md).
- Slash-command-like behavior across tools: set explicit-invocation metadata (`metadata: {opencode/slash: "true"}`, Cursor's `disable-model-invocation: true`, Codex's `allow_implicit_invocation: false`) rather than a separate per-tool command file.

## Distribution

- `install.sh` copies the skills you pick into a project's `.agents/skills/` (Codex, Gemini CLI, Cursor, OpenCode, omp) and `.claude/skills/` (Claude Code, unless the project enables the plugin), or links them into `~/.agents/skills/` with `--global`: see [Install](../README.md#install).
- A new skill needs a row in [`catalog.tsv`](../catalog.tsv), or the installer can't offer it; `tests/run.sh` fails without one.
- Or copy a single folder into any harness's skills dir by hand.
