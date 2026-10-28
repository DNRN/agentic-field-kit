# Changelog

Notable changes to agentic-toolkit. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html) and match `version` in `.claude-plugin/plugin.json`. Release steps: [README](README.md#releasing).

## [Unreleased]

## [0.2.0] - 2026-10-28

### Added

- `agentic-toolkit`, a single installer binary for Linux, macOS, and Windows with the components built in ([ADR 0009](docs/adr/0009-installer-in-rust.md), [ADR 0010](docs/adr/0010-components-embedded-in-binary.md)). Install it with the shell or PowerShell script from the GitHub release, or `cargo install --git`. No clone, git, Bash, or runtime needed.
- Selective install into the project ([ADR 0006](docs/adr/0006-project-scope-install-default.md)): name components (`agentic-toolkit code-reviewer`) and what they require comes along. Copies go to `.agents/skills`, plus `.claude/skills` and `.claude/agents` for Claude Code unless the project enables the plugin; commit them. `code-reviewer` and `doc-auditor` can now be installed without the plugin.
- A terminal UI when `agentic-toolkit` runs on a terminal without arguments: status per component in the project and globally, details, marks to install, remove, or update, a plan to review before anything changes, and the Entropy Fox logo.
- `--list`, `--update`, `--remove`, `--all`, `--project DIR`, `--global`, `--force`, `--dry-run`, `--yes`, `--check`, and `--source DIR` for a toolkit working tree.
- A manifest per scope, `.agents/agentic-toolkit.tsv`, records what was installed, why, and a content hash ([ADR 0008](docs/adr/0008-project-install-manifest.md)). `--update` refreshes what changed in a new release, restores deleted copies, installs new requirements, and asks before removing retired components; it and `--remove` leave locally edited copies alone without `--force`.
- `catalog.tsv` lists every installable component with its kind, harnesses, requirements, and a summary ([ADR 0007](docs/adr/0007-component-catalog.md)). `tests/catalog.test.sh` fails on a component without a row.
- CI builds and tests the installer on Linux, macOS, and Windows; releases are built with dist on `installer/vX.Y.Z` tags. `.gitattributes` keeps every checkout LF.
- The Entropy Fox name and logo in `assets/brand/`, outside the MIT licence ([ADR 0011](docs/adr/0011-brand-assets-not-mit.md)).

### Changed

- **Breaking:** `install.sh` is gone; use `agentic-toolkit`. It installs into the project (the git root of the working directory) by default; the old behavior is `agentic-toolkit --global --all`. Without a terminal it needs component names or `--all`. `--harness` and `--mode` require `--global`. Global installs are copies; symlinks into a toolkit folder need `--source DIR`.
- Corrected the 0.1.0 install advice: in Claude Code, only a personal *symlink* to the plugin's skill folder hides the plugin's `agentic-toolkit:*` skill; a personal copy is listed next to it.

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

[Unreleased]: https://github.com/DNRN/agentic-toolkit/compare/agentic-toolkit--v0.2.0...HEAD
[0.2.0]: https://github.com/DNRN/agentic-toolkit/compare/agentic-toolkit--v0.1.0...agentic-toolkit--v0.2.0
[0.1.0]: https://github.com/DNRN/agentic-toolkit/releases/tag/agentic-toolkit--v0.1.0
