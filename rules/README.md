# Rules

Always-on instruction-file templates (`AGENTS.md`, `CLAUDE.md`) copied into a project root. Not auto-loaded by the plugin — a `CLAUDE.md` at the plugin root is ignored by design; always-on context belongs in the project.

## Format

- One template per file (or per folder for multi-file variants), named for its intent: `rules/testing.md`, `rules/monorepo.md`.
- Write as constraints and conventions, not procedures — procedures go in `skills/`.
- Keep sections short and declarative; every line is permanent context cost.
