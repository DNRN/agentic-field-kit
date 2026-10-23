# Changelog

Notable changes to agentic-toolkit. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html) and match `version` in `.claude-plugin/plugin.json`. Release steps: [README](README.md#releasing).

## [Unreleased]

## [0.1.0] - 2026-10-23

First release.

### Added

- Skills, portable across harnesses (Agent Skills standard): `adr-planning`, `code-review`, and `doc-sync`. The review and audit rubrics live once, in each skill's `references/rubric.md` ([ADR 0005](docs/adr/0005-rubrics-live-in-skills.md)).
- Claude Code plugin and marketplace: read-only `code-reviewer` and `doc-auditor` subagents, and the `commit-gate.sh` PreToolUse hook, which asks before an unreviewed commit or one that changes code without docs ([ADR 0002](docs/adr/0002-code-review-mechanism.md), [0003](docs/adr/0003-doc-sync-at-commit-gate.md)). Per-repository doc paths in `.commit-gate-docs` ([ADR 0004](docs/adr/0004-configurable-docs-paths.md)).
- `install.sh`: links or copies the skills into harness skill directories; `--check` validates them against the Agent Skills rules.
- Eval suite in `claude plugin eval` format: 13 cases across the three skills, each scored against a no-plugin baseline.
- Regression tests (`tests/run.sh`) for the commit gate, the installer, the plugin manifest, and release metadata. CI runs them with shellcheck, plugin validation, an internal link check, and mermaid rendering.
- ADRs 0001–0005 and rationale essays under `docs/`.

### Changed

- `./install.sh` installs into `~/.agents/skills` only (Codex, Gemini CLI, OpenCode, Cursor, omp). A copy in `~/.claude/skills` hid the plugin's `agentic-toolkit:*` skills in Claude Code, so that target is now opt-in (`--harness claude`) for Claude Code without the plugin. `--remove` still clears every target.

[Unreleased]: https://github.com/DNRN/agentic-toolkit/compare/agentic-toolkit--v0.1.0...HEAD
[0.1.0]: https://github.com/DNRN/agentic-toolkit/releases/tag/agentic-toolkit--v0.1.0
