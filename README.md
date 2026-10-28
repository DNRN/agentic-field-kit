# agentic-toolkit

Harness-agnostic collection of agent components: skills, subagents, commands, hooks, output styles, plus copy-paste rules, prompt templates, and permission policies. Works across projects and tools.

Why this repo starts with an [ADR skill](docs/why-adrs.md): tests verify behavior; ADRs verify design — the missing half of most agent workflows. Why the [code review mechanism](docs/why-code-review.md) is three layers: fresh eyes, right moment, ask instead of nag. Why [docs must be kept true](docs/why-docs-matters.md): agents believe stale docs.

## Compatibility

| Component | Claude Code | Codex | Gemini CLI | OpenCode | Cursor | omp |
| --- | --- | --- | --- | --- | --- | --- |
| [`skills/`](skills/) — SKILL.md folders | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| [`agents/`](agents/) · [`commands/`](commands/) · [`hooks/`](hooks/) · [`output-styles/`](output-styles/) | ✓ (plugin; agents also via `install.sh`) | adapt | adapt | adapt | adapt | adapt |
| [`rules/`](rules/) · [`prompts/`](prompts/) · [`policies/`](policies/) | copy | copy | copy | copy | copy | copy |

Skills follow the [Agent Skills](https://agentskills.io) open standard — identical folders load natively in every harness above. The other component categories currently auto-load only via the Claude Code plugin; adapters for other harnesses are welcome.

## Install

Install only the components you need, into the project that needs them. The installer copies them into the repository; commit them, and every teammate and every harness gets the same version.

```bash
git clone https://github.com/DNRN/agentic-toolkit ~/src/agentic-toolkit
cd ~/code/your-project
~/src/agentic-toolkit/install.sh                # menu: what can be installed, what is installed
~/src/agentic-toolkit/install.sh code-reviewer  # or name components; what they require comes along
git add .agents .claude && git commit -m "Add agentic-toolkit components"
```

| Command | Effect |
| --- | --- |
| `install.sh` | Menu (on a terminal): numbers mark components to install or remove, `u` marks updates, `g` switches to the global scope; shows the plan and asks before changing anything |
| `install.sh --list` | Status of every component, in this project and globally |
| `install.sh NAME…` · `--all` | Install components and what they require |
| `install.sh --update [NAME…]` | After `git pull` in the clone: refresh what changed upstream, restore deleted copies, add new requirements |
| `install.sh --remove [NAME…]` | Remove what the installer installed, and requirements nothing else needs |
| `--project DIR` · `--global` | Scope: the git root of the working directory by default |
| `--force` · `--dry-run` · `--yes` | Also overwrite or remove locally edited copies · print the plan only · don't ask |
| `install.sh --check` | Validate skills against the Agent Skills portability rules |

Where a project install puts things ([ADR 0006](docs/adr/0006-project-scope-install-default.md)):

| Path | Read by |
| --- | --- |
| `.agents/skills/<name>/` | Codex, Gemini CLI, OpenCode, Cursor, omp |
| `.claude/skills/<name>/` | Claude Code. Left out when the project's `.claude/settings.json` enables the plugin, which already provides the skills |
| `.claude/agents/<name>.md` | Claude Code subagents; same condition |
| `.agents/agentic-toolkit.tsv` | The installer: what it installed, why, and a content hash of each ([ADR 0008](docs/adr/0008-project-install-manifest.md)) |

**Updating.** Components have no version numbers of their own. `--list` compares each copy with the hash recorded at install time and with the clone: `outdated` (changed upstream), `modified` (edited in the project; `--update` and `--remove` leave it alone without `--force`), `missing` (deleted by hand), `retired` (gone from the toolkit; `--update` asks before removing it). The installer never touches a file it didn't install (`foreign`).

Harness notes, checked per harness ([evidence](docs/plans/selective-install.md#evidence)):

- Gemini CLI loads project skills only in a [trusted folder](https://geminicli.com/docs/cli/trusted-folders/). Gemini CLI and Cursor find them only when started at the repository root.
- A skill installed both in the project and globally: Codex lists it twice, Claude Code and Cursor use the global copy. `--list` flags it, and a project install offers to remove the global copy.
- With the plugin enabled at user scope, Claude Code lists a project `code-review` next to the plugin's `agentic-toolkit:code-review`. Enable the plugin for the project instead, or install only the plugin.
- `commit-gate` ships only with the plugin until the portable gate ([roadmap phase 1](docs/plans/roadmap.md#phase-1-enforce-existing-themes)) lands.

### Global install

For a skill you want in every repository, such as `adr-planning`: `install.sh --global NAME`. Global installs are symlinks into the clone, so `git pull` updates them; `--mode copy` makes copies, and `--update` keeps each install's form.

| `--harness` | Target | Read by |
| --- | --- | --- |
| `agents` (default) | `~/.agents/skills/` | Codex, Gemini CLI, OpenCode, Cursor, omp |
| `claude` | `~/.claude/skills/` | Claude Code; also OpenCode, and Cursor with third-party extensibility on. Only without the plugin: a personal symlink to the plugin's skill folder hides the plugin's `agentic-toolkit:*` name, which the commit gate tells the agent to run, and a personal copy lists every skill twice |
| `omp` | `~/.omp/agent/skills/` | omp; needed only when its `~/.agents` provider is turned off |

- Codex names a *symlinked* skill `agentic-toolkit:<name>`, because its real path sits inside this plugin repo. `--mode copy` keeps the bare name.
- In Claude Code, a personal `code-review` replaces the built-in `/code-review`.
- `install.sh --global --remove` also removes the links and copies made by the 0.1.0 installer.

### Windows

Run the installer from Git Bash or WSL. Project installs contain no symlinks, so a Windows checkout works without `core.symlinks`. Global installs are copies by default on Windows (`ln -s` in Git Bash copies without saying so). From WSL, `--global` reaches only the Linux-side harnesses.

## Claude Code full bundle (also loads agents, commands, hooks, styles)

```
/plugin marketplace add DNRN/agentic-toolkit
/plugin install agentic-toolkit@agentic-toolkit
```

`.claude-plugin/` is an optional adapter, not the repo's identity — everything in `skills/` is plain standard folders usable without it.

## What's inside

| Directory | Contents | How it loads |
| --- | --- | --- |
| [`skills/`](skills/) | SKILL.md folders, auto-discovered when relevant | Auto, every harness; [`install.sh`](#install) or the plugin |
| [`agents/`](agents/) | Subagent definitions (`.md`) | Plugin, or `install.sh` into a project (Claude Code) |
| [`commands/`](commands/) | Slash commands (`.md`) | Plugin (Claude Code) |
| [`hooks/`](hooks/) | `hooks/hooks.json` + scripts, event-triggered | Plugin (Claude Code) |
| [`output-styles/`](output-styles/) | Output style / persona definitions | Plugin (Claude Code) |
| `.mcp.json` | MCP server configs (add with first server) | Plugin (Claude Code) |
| [`rules/`](rules/) | `AGENTS.md` / `CLAUDE.md` templates | Copy into a project |
| [`prompts/`](prompts/) | Plan / spec / review templates | Referenced by skills & commands |
| [`policies/`](policies/) | Permission allow/deny presets | Copy into project settings |
| [`evals/`](evals/) | Golden tasks per component in `claude plugin eval` format, scored against a no-plugin baseline | [`claude plugin eval .`](evals/README.md#run) |
| [`tests/`](tests/) | Deterministic regression tests for hooks, `install.sh`, `catalog.tsv`, and release metadata | `tests/run.sh` |
| [`installer/`](installer/) | The Rust installer `agentic-toolkit`, which replaces `install.sh` in 0.3.0 ([plan](docs/plans/rust-installer.md)); so far `--list` and `--check` | `cargo build --release` |
| `catalog.tsv` | Every installable component: kind, path, harnesses, requirements, one-line summary ([ADR 0007](docs/adr/0007-component-catalog.md)) | Read by `install.sh` |
| [`docs/`](docs/) | Rationale essays, [ADRs](docs/adr/), [plans](docs/plans/) | Read |

## Conventions

- One component = one self-contained folder or file, kebab-case.
- Skills: Agent Skills standard — frontmatter `name` must equal the folder name; keep frontmatter to `name` + `description` (+ optional `license`, `metadata`); the description is the discovery contract — write it as "use when …".
- Every behavioral component ships with eval cases under `evals/<component>/`, including a negative case.
- No secrets, no machine-specific paths; scripts read relative to their component folder.
- Agents, commands, and output styles are registered explicitly in `.claude-plugin/plugin.json` (the folder scan would also load each folder's README); `tests/run.sh` fails on an unregistered file.
- Every skill, agent, and hook script has a row in `catalog.tsv`, with `requires` naming what it can't work without; `tests/run.sh` fails on a component without a row.
- Validate before pushing: `./install.sh --check`, `tests/run.sh`, and `claude plugin validate .` if you touched `.claude-plugin/`. If you touched `installer/`: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `AGENTIC_TOOLKIT_BIN=target/release/agentic-toolkit bash tests/install.test.sh` after `cargo build --release`. CI ([`.github/workflows/ci.yml`](.github/workflows/ci.yml)) runs these plus shellcheck, the installer tests on Linux, macOS, and Windows (Git Bash), an internal link and anchor check, and mermaid rendering on every push to `main` and every PR. CI does not run the [eval suite](evals/README.md) (it needs model credentials and costs money): run it when a skill, agent, or hook changes.

## Releasing

Users of the plugin get a new release only when `version` in `.claude-plugin/plugin.json` changes; with an unchanged version, `claude plugin update` reports the plugin as current.

1. Move the `[Unreleased]` entries in [CHANGELOG.md](CHANGELOG.md) under a new `## [X.Y.Z] - YYYY-MM-DD` heading and add its compare link at the bottom.
2. Set the same `X.Y.Z` in `.claude-plugin/plugin.json` and `installer/Cargo.toml`, then run `cargo build` to update `Cargo.lock` (semver: minor for new components or behavior, patch for fixes; a breaking rename or removal is a major once past 1.0).
3. `tests/run.sh` (fails if the version has no dated changelog entry), then commit.
4. `claude plugin tag --push` creates and pushes `agentic-toolkit--vX.Y.Z` after checking that `plugin.json` and the marketplace entry agree.

## Contributing

PR per component. Self-contained, follows the conventions above, includes an eval. MIT.

## License

[MIT](LICENSE)
