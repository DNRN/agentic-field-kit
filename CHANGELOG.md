# Changelog

Notable changes to agentic-toolkit. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html) and match `version` in `.claude-plugin/plugin.json`. Release steps: [README](README.md#releasing).

## [Unreleased]

### Added

- `commit-gate.sh --pre-commit`: the commit gate as a git `pre-commit` hook, so review and docs checks ask before commits from every harness and from a terminal ([ADR 0013](docs/adr/0013-commit-gate-as-git-hook.md)). Same stamps, same docs paths, same `--no-verify` waiver as the Claude Code hook; the denial goes to stderr. Plan: [roadmap phase 1](docs/plans/phase1-enforce-themes.md), with ADRs [0014](docs/adr/0014-git-hook-layout.md) (how the installer will enable it) and [0015](docs/adr/0015-accepted-adrs-guarded-at-commit.md) (Accepted ADRs guarded at commit).
- `project-init` skill: starts a project. It runs `git init`, asks whether to connect a remote (connect one now, keep going while you create one, or stay local), writes `AGENTS.md` and a `CLAUDE.md` that imports it with `@AGENTS.md`, and writes `PROJECT.md` (purpose, users, scope, constraints, open questions) from a dialogue you can skip at any point. Run it again to answer the open questions later. Named `project-init`, not `init`, so it doesn't collide with the built-in `/init` in Claude Code, Codex, and Gemini CLI.

### Changed

- With the plugin and the git hook both installed, the plugin's gate steps aside and the git hook asks, so an unreviewed commit asks once ([ADR 0014](docs/adr/0014-git-hook-layout.md)).
- The commit gate's message names the skills and stamp command as installed: the project's `code-review` and `doc-sync` copies and `.agents/hooks/commit-gate.sh` when present, the plugin's names otherwise.
- The installer draws the Entropy Fox as coloured ASCII, so every terminal shows it, including terminals without truecolor or image support, tmux, and `NO_COLOR` (uncoloured) ([ADR 0012](docs/adr/0012-logo-drawn-as-text.md)). It replaces the image and half-blocks, which showed only in truecolor terminals; the name's orange is now a 256-colour index too. At 80 columns, header lines longer than the 37 columns beside the logo wrap.
- The terminal UI has an orange colour scheme from the Entropy Fox palette: rust borders (fox orange around popups), amber titles and keys, a brown bar on the selected row, tan labels, and status colours (green, yellow, red, blue) in matching mid tones. All are 256-colour indices of middle lightness, so they read on dark and on light backgrounds; `NO_COLOR` keeps bold, dim, and reverse only.
- `adr-planning` briefs each Proposed ADR in a few lines (decision, strongest rejected alternative, costliest consequence, assumptions, path) instead of pasting it, and asks you to accept, change, or reject each one. It records the verdict in the ADR's Status: `Accepted`, or `Rejected` with the reason, keeping the file. It no longer implements a decision while it is Proposed, and it now also triggers when you give a verdict on a Proposed ADR.

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
