# Changelog

Notable changes to Agentic Field Kit (afk; agentic-toolkit before 0.4.0). Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html) and match `version` in `.claude-plugin/plugin.json`. Release steps: [README](README.md#releasing).

## [Unreleased]

### Changed

- Renamed to Agentic Field Kit, command `afk` ([ADR 0018](docs/adr/0018-rename-to-agentic-field-kit.md)). The installer binary is `afk` (package `agentic-field-kit`; the install scripts are `agentic-field-kit-installer.sh` and `.ps1`), the plugin and marketplace are `afk`, so skills are `/afk:<name>`, the manifest is `.agents/afk.tsv`, and the hook shim is marked `# afk commit-gate`. To upgrade: install the new release, remove `~/.cargo/bin/agentic-toolkit`, and run `afk --update` in each project, which moves `.agents/agentic-toolkit.tsv` to `.agents/afk.tsv` and rewrites the shim; commit the manifest. Plugin users: `/plugin install afk@afk`, then uninstall `agentic-toolkit@agentic-toolkit`. The installer still reads the old manifest, shim marker, and plugin name.

## [0.3.0] - 2026-10-30

The commit gate works in every harness, Accepted ADRs are enforced, and the installer starts new projects. Plans: [roadmap phase 1](docs/plans/phase1-enforce-themes.md), [new project from the installer](docs/plans/new-project.md).

### Added

- The commit gate as a git `pre-commit` hook, so review and docs checks ask before commits from every harness and from a terminal ([ADR 0013](docs/adr/0013-commit-gate-as-git-hook.md)). `agentic-toolkit commit-gate` installs it, the installer's new `hook` kind ([ADR 0014](docs/adr/0014-git-hook-layout.md)): a committed copy at `.agents/hooks/commit-gate.sh`, recorded in the manifest, and a marked `pre-commit` shim in the clone that runs it (honouring `core.hooksPath`). Same stamps, docs paths, and `--no-verify` waiver as the Claude Code hook; the denial goes to stderr. `--list` shows `inactive` in a clone without the shim, and `--update` writes it. An existing `pre-commit` hook is never changed: the installer prints a line to add, which does nothing once the gate is removed. `--remove` deletes the copy and its own shim only. Global installs are refused: git hooks are per repository.
- Accepted ADRs are guarded at commit: the git hook denies a commit that changes an Accepted ADR (`docs/adr/NNNN-*.md`) outside its `## Status` section, or deletes or renames one ([ADR 0015](docs/adr/0015-accepted-adrs-guarded-at-commit.md)). Status edits such as `Superseded by NNNN` pass; the waiver is `--no-verify`.
- `ship` skill: on an explicit "ship it", one flow from finished change to commit: code review, documentation check, commit message, commit, leaving the stamps the commit gate looks for. It stops to ask when the review blocks or a doc change needs a decision.
- `adr-index` SessionStart hook (plugin): lists the project's Accepted ADRs, one line each, at the start of every Claude Code session.
- `adr-planning` records a decision as an ADR on request, and supersedes an Accepted ADR: a new ADR, and only the old one's Status changes, as the ADR check requires. A verdict given in the request ("record it as Accepted") counts.
- `project-init` skill: starts a project. It runs `git init`, asks whether to connect a remote (connect one now, keep going while you create one, or stay local), writes `AGENTS.md` and a `CLAUDE.md` that imports it with `@AGENTS.md`, and writes `PROJECT.md` (purpose, users, scope, constraints, open questions) from a dialogue you can skip at any point. Run it again to answer the open questions later. Named `project-init`, not `init`, so it doesn't collide with the built-in `/init` in Claude Code, Codex, and Gemini CLI.
- A new project screen: `agentic-toolkit` run outside a git repository shows the folder you are in and offers to start a project there or in a new folder inside it ([ADR 0016](docs/adr/0016-new-project-screen.md)). It installs `project-init` into that folder and starts your agent there with an opening prompt to use it: Claude Code or omp, picked when both are installed; other agents get the step to take yourself ([ADR 0017](docs/adr/0017-hand-off-to-the-agent.md)). `b` keeps the old behaviour, the component menu in the global scope.
- README: a components table, and sections on the commit gate and on starting a new project.

### Changed

- With the plugin and the git hook both installed, the plugin's gate steps aside and the git hook asks, so an unreviewed commit asks once ([ADR 0014](docs/adr/0014-git-hook-layout.md)).
- The commit gate's message names the skills and stamp command as installed: the project's `code-review` and `doc-sync` copies and `.agents/hooks/commit-gate.sh` when present, the plugin's names otherwise. Under `git commit -a`, the git hook names the `--all` stamp.
- `adr-planning` adds a line to `AGENTS.md` (or `CLAUDE.md` when there is none) when it creates `docs/adr/`, telling every session where decisions live and that Accepted ones are superseded, not edited. `project-init` writes the same line when the project already has ADRs. Harnesses without the `adr-index` hook learn about the ADRs this way.
- `adr-planning` briefs each Proposed ADR in a few lines (decision, strongest rejected alternative, costliest consequence, assumptions, path) instead of pasting it, and asks you to accept, change, or reject each one. It records the verdict in the ADR's Status: `Accepted`, or `Rejected` with the reason, keeping the file. It no longer implements a decision while it is Proposed, and it now also triggers when you give a verdict on a Proposed ADR.
- `project-init`'s initial commit also stages what the installer put in the folder: `.agents/agentic-toolkit.tsv` and the paths it records. It still never stages everything (`git add -A`) or all of `.claude/`.
- Outside a git repository the installer explains how to start a project: the error points at `agentic-toolkit --project . project-init`, `project-init`'s details say it runs in your agent, and installing it ends with "Next: open your agent … and ask it to start the project". It no longer suggests a `git add` in a folder without git.
- The installer draws the Entropy Fox as coloured ASCII, so every terminal shows it, including terminals without truecolor or image support, tmux, and `NO_COLOR` (uncoloured) ([ADR 0012](docs/adr/0012-logo-drawn-as-text.md)). It replaces the image and half-blocks, which showed only in truecolor terminals; the name's orange is now a 256-colour index too. At 80 columns, header lines longer than the 37 columns beside the logo wrap.
- The terminal UI has an orange colour scheme from the Entropy Fox palette: rust borders (fox orange around popups), amber titles and keys, a brown bar on the selected row, tan labels, and status colours (green, yellow, red, blue) in matching mid tones. All are 256-colour indices of middle lightness, so they read on dark and on light backgrounds; `NO_COLOR` keeps bold, dim, and reverse only.
- At 80×24 the terminal UI keeps the Entropy Fox and shows 7 component rows; the rest scroll into view with the cursor (the catalog now has 9).

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

[Unreleased]: https://github.com/DNRN/agentic-field-kit/compare/agentic-toolkit--v0.3.0...HEAD
[0.3.0]: https://github.com/DNRN/agentic-field-kit/compare/agentic-toolkit--v0.2.0...agentic-toolkit--v0.3.0
[0.2.0]: https://github.com/DNRN/agentic-field-kit/compare/agentic-toolkit--v0.1.0...agentic-toolkit--v0.2.0
[0.1.0]: https://github.com/DNRN/agentic-field-kit/releases/tag/agentic-toolkit--v0.1.0
