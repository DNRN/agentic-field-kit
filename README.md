# agentic-toolkit

Harness-agnostic collection of agent components: skills, subagents, commands, hooks, output styles, plus copy-paste rules, prompt templates, and permission policies. Works across projects and tools.

Why this repo starts with an [ADR skill](docs/why-adrs.md): tests verify behavior; ADRs verify design — the missing half of most agent workflows. Why the [code review mechanism](docs/why-code-review.md) is three layers: fresh eyes, right moment, ask instead of nag. Why [docs must be kept true](docs/why-docs-matters.md): agents believe stale docs.

## Compatibility

| Component | Claude Code | Codex | Gemini CLI | OpenCode | Cursor | omp |
| --- | --- | --- | --- | --- | --- | --- |
| [`skills/`](skills/) — SKILL.md folders | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| [`agents/`](agents/) · [`commands/`](commands/) · [`hooks/`](hooks/) · [`output-styles/`](output-styles/) | ✓ (plugin; agents also via the installer) | adapt | adapt | adapt | adapt | adapt |
| [`rules/`](rules/) · [`prompts/`](prompts/) · [`policies/`](policies/) | copy | copy | copy | copy | copy | copy |

Skills follow the [Agent Skills](https://agentskills.io) open standard — identical folders load natively in every harness above. The other component categories currently auto-load only via the Claude Code plugin; adapters for other harnesses are welcome.

## Install

Install only the components you need, into the project that needs them. The `agentic-toolkit` installer copies them into the repository; commit them, and every teammate and every harness gets the same version. It is one binary with the components built in: no clone, git, or runtime needed.

```bash
# macOS, Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/DNRN/agentic-toolkit/releases/latest/download/agentic-toolkit-installer.sh | sh
```

```powershell
# Windows
powershell -ExecutionPolicy Bypass -c "irm https://github.com/DNRN/agentic-toolkit/releases/latest/download/agentic-toolkit-installer.ps1 | iex"
```

Both put `agentic-toolkit` in `~/.cargo/bin`; the [release page](https://github.com/DNRN/agentic-toolkit/releases/latest) has the archives and checksums. With Rust: `cargo install --locked --git https://github.com/DNRN/agentic-toolkit agentic-toolkit`.

```bash
cd ~/code/your-project
agentic-toolkit                  # the menu: what can be installed, what is installed
agentic-toolkit code-reviewer    # or name components; what they require comes along
git add .agents .claude && git commit -m "Add agentic-toolkit components"
```

| Command | Effect |
| --- | --- |
| `agentic-toolkit` | The menu (on a terminal): space or a number marks a component to install or remove, `u` marks updates, `g` switches to the global scope, Enter shows the plan, `y` applies it |
| `agentic-toolkit --list` | Status of every component, in this project and globally |
| `agentic-toolkit NAME…` · `--all` | Install components and what they require |
| `agentic-toolkit --update [NAME…]` | After upgrading the installer: refresh what changed, restore deleted copies, add new requirements |
| `agentic-toolkit --remove [NAME…]` | Remove what the installer installed, and requirements nothing else needs |
| `--project DIR` · `--global` | Scope: the git root of the working directory by default |
| `--force` · `--dry-run` · `--yes` | Also overwrite or remove locally edited copies · print the plan only · don't ask |
| `agentic-toolkit --check` | Validate skills against the Agent Skills portability rules |
| `--source DIR` | Use a toolkit working tree instead of the built-in components (contributors) |

Where a project install puts things ([ADR 0006](docs/adr/0006-project-scope-install-default.md)):

| Path | Read by |
| --- | --- |
| `.agents/skills/<name>/` | Codex, Gemini CLI, OpenCode, Cursor, omp |
| `.claude/skills/<name>/` | Claude Code. Left out when the project's `.claude/settings.json` enables the plugin, which already provides the skills |
| `.claude/agents/<name>.md` | Claude Code subagents; same condition |
| `.agents/hooks/<name>.sh` | Git, through the clone's `pre-commit` hook: the commit gate, in every harness and in a terminal ([ADR 0014](docs/adr/0014-git-hook-layout.md)) |
| `.agents/agentic-toolkit.tsv` | The installer: what it installed, why, and a content hash of each ([ADR 0008](docs/adr/0008-project-install-manifest.md)) |

**Updating.** Install the new release the same way, then run `agentic-toolkit --update` in each project ([ADR 0010](docs/adr/0010-components-embedded-in-binary.md)). Components have no version numbers of their own. `--list` compares each copy with the hash recorded at install time and with the installer's: `outdated` (changed in the new release), `modified` (edited in the project; `--update` and `--remove` leave it alone without `--force`), `missing` (deleted by hand), `retired` (gone from the toolkit; `--update` asks before removing it), `inactive` (a hook whose copy is current but that git doesn't run in this clone yet). The installer never touches a file it didn't install (`foreign`).

Harness notes, checked per harness ([evidence](docs/plans/selective-install.md#evidence)):

- Gemini CLI loads project skills only in a [trusted folder](https://geminicli.com/docs/cli/trusted-folders/). Gemini CLI and Cursor find them only when started at the repository root.
- A skill installed both in the project and globally: Codex lists it twice, Claude Code and Cursor use the global copy. `--list` flags it, and a project install offers to remove the global copy.
- With the plugin enabled at user scope, Claude Code lists a project `code-review` next to the plugin's `agentic-toolkit:code-review`. Enable the plugin for the project instead, or install only the plugin.
- `commit-gate` (`agentic-toolkit commit-gate`) copies the gate to `.agents/hooks/commit-gate.sh` and writes a small `pre-commit` hook in this clone that runs it, where git looks for hooks (`core.hooksPath` counts). `.git/` isn't committed, so each teammate runs `agentic-toolkit --update` once after cloning; until then `--list` says `inactive`. A `pre-commit` hook the installer didn't write (husky, the pre-commit framework, your own) is never changed: the installer prints a line to add to it. The gate asks before unreviewed commits from any harness and from a terminal; `git commit --no-verify` skips it. With the plugin enabled too, you are asked once.

### Start a new project

Run `agentic-toolkit` in the folder where the project should be. Outside a git repository it opens a new project screen ([ADR 0016](docs/adr/0016-new-project-screen.md)): it shows the folder you are in and what is there, and asks whether to start the project here, in a new folder inside it, or to browse components instead. Your home folder is never made a project; you name a new folder inside it.

Starting a project is the `project-init` skill's job, and it runs in your agent: it sets up git (and a remote, if you have one), writes `AGENTS.md`, a `CLAUDE.md` that imports it, and `PROJECT.md` from a few questions you can skip. So the installer installs `project-init` into the folder you picked, then starts your agent there with "Start a new project in this folder with the project-init skill." ([ADR 0017](docs/adr/0017-hand-off-to-the-agent.md)). It starts Claude Code (`claude`) and omp (`omp`) when they are on your `PATH`, and asks which when both are; for other agents it prints the step to take yourself.

Without the menu, the same in two commands:

```bash
mkdir my-project && cd my-project
agentic-toolkit --project . project-init     # or once for every folder: --global project-init
```

then open your agent in the folder and ask it to start the project. With the Claude Code plugin, the skill is already there as `/agentic-toolkit:project-init`. Afterwards, install the rest from the new repository as usual, such as `agentic-toolkit commit-gate`.

### Global install

For a skill you want in every repository, such as `adr-planning`, or one you use before a repository exists, such as `project-init`: `agentic-toolkit --global NAME`. Global installs are copies. With `--source DIR` they are symlinks into that toolkit folder instead, except on Windows, where copies stay the default (`--mode symlink` needs Developer Mode); `--mode` picks either, and `--update` keeps each install's form.

| `--harness` | Target | Read by |
| --- | --- | --- |
| `agents` (default) | `~/.agents/skills/` | Codex, Gemini CLI, OpenCode, Cursor, omp |
| `claude` | `~/.claude/skills/` | Claude Code; also OpenCode, and Cursor with third-party extensibility on. Only without the plugin: a personal symlink to the plugin's skill folder hides the plugin's `agentic-toolkit:*` name, which the commit gate tells the agent to run, and a personal copy lists every skill twice |
| `omp` | `~/.omp/agent/skills/` | omp; needed only when its `~/.agents` provider is turned off |

- Codex names a *symlinked* skill `agentic-toolkit:<name>`, because its real path sits inside this plugin repo. Copies keep the bare name.
- In Claude Code, a personal `code-review` replaces the built-in `/code-review`.
- `agentic-toolkit --global --remove` also removes the links and copies made by the 0.1.0 installer (`install.sh`).

### Windows

The installer is a native Windows program; it needs no Git Bash or WSL. Project installs contain no symlinks, so a Windows checkout works without `core.symlinks`. From WSL, `--global` reaches only the Linux-side harnesses.

## Claude Code full bundle (also loads agents, commands, hooks, styles)

```
/plugin marketplace add DNRN/agentic-toolkit
/plugin install agentic-toolkit@agentic-toolkit
```

`.claude-plugin/` is an optional adapter, not the repo's identity — everything in `skills/` is plain standard folders usable without it.

## What's inside

| Directory | Contents | How it loads |
| --- | --- | --- |
| [`skills/`](skills/) | SKILL.md folders, auto-discovered when relevant | Auto, every harness; [the installer](#install) or the plugin |
| [`agents/`](agents/) | Subagent definitions (`.md`) | Plugin, or the installer into a project (Claude Code) |
| [`commands/`](commands/) | Slash commands (`.md`) | Plugin (Claude Code) |
| [`hooks/`](hooks/) | `hooks/hooks.json` + scripts, event-triggered | Plugin (Claude Code) |
| [`output-styles/`](output-styles/) | Output style / persona definitions | Plugin (Claude Code) |
| `.mcp.json` | MCP server configs (add with first server) | Plugin (Claude Code) |
| [`rules/`](rules/) | `AGENTS.md` / `CLAUDE.md` templates | Copy into a project |
| [`prompts/`](prompts/) | Plan / spec / review templates | Referenced by skills & commands |
| [`policies/`](policies/) | Permission allow/deny presets | Copy into project settings |
| [`evals/`](evals/) | Golden tasks per component in `claude plugin eval` format, scored against a no-plugin baseline | [`claude plugin eval .`](evals/README.md#run) |
| [`tests/`](tests/) | Deterministic regression tests for hooks, the installer, `catalog.tsv`, and release metadata | `tests/run.sh` |
| [`installer/`](installer/) | The Rust installer `agentic-toolkit`: command line and terminal UI, with the components built in ([plan](docs/plans/rust-installer.md)) | `cargo build --release` |
| `catalog.tsv` | Every installable component: kind, path, harnesses, requirements, one-line summary ([ADR 0007](docs/adr/0007-component-catalog.md)) | Read by the installer |
| [`assets/brand/`](assets/brand/) | The Entropy Fox name and logo the installer shows; not MIT ([ADR 0011](docs/adr/0011-brand-assets-not-mit.md)) | Built into the installer |
| [`docs/`](docs/) | Rationale essays, [ADRs](docs/adr/), [plans](docs/plans/) | Read |

## Conventions

- One component = one self-contained folder or file, kebab-case.
- Skills: Agent Skills standard — frontmatter `name` must equal the folder name; keep frontmatter to `name` + `description` (+ optional `license`, `metadata`); the description is the discovery contract — write it as "use when …".
- Every behavioral component ships with eval cases under `evals/<component>/`, including a negative case.
- No secrets, no machine-specific paths; scripts read relative to their component folder.
- Agents, commands, and output styles are registered explicitly in `.claude-plugin/plugin.json` (the folder scan would also load each folder's README); `tests/run.sh` fails on an unregistered file.
- Every skill, agent, and hook script has a row in `catalog.tsv`, with `requires` naming what it can't work without; `tests/run.sh` fails on a component without a row.
- Validate before pushing: `tests/run.sh` (it builds the installer with `cargo`), and `claude plugin validate .` if you touched `.claude-plugin/`. If you touched `installer/`, also `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`. CI ([`.github/workflows/ci.yml`](.github/workflows/ci.yml)) runs these plus shellcheck, the installer tests on Linux, macOS, and Windows, an internal link and anchor check, and mermaid rendering on every push to `main` and every PR. CI does not run the [eval suite](evals/README.md) (it needs model credentials and costs money): run it when a skill, agent, or hook changes.

## Releasing

Users of the plugin get a new release only when `version` in `.claude-plugin/plugin.json` changes; with an unchanged version, `claude plugin update` reports the plugin as current.

1. Move the `[Unreleased]` entries in [CHANGELOG.md](CHANGELOG.md) under a new `## [X.Y.Z] - YYYY-MM-DD` heading and add its compare link at the bottom.
2. Set the same `X.Y.Z` in `.claude-plugin/plugin.json` and `installer/Cargo.toml`, then run `cargo build` to update `Cargo.lock` (semver: minor for new components or behavior, patch for fixes; a breaking rename or removal is a major once past 1.0).
3. `tests/run.sh` (fails if the versions differ or have no dated changelog entry), then commit and merge to `main`.
4. `claude plugin tag --push` creates and pushes `agentic-toolkit--vX.Y.Z` (the plugin) after checking that `plugin.json` and the marketplace entry agree.
5. `git tag installer/vX.Y.Z && git push origin installer/vX.Y.Z` starts [`installer-release.yml`](.github/workflows/installer-release.yml), which builds the installer for every platform and publishes the GitHub release with its install scripts. dist can't read the plugin's tag, so the installer has its own.

## Contributing

PR per component. Self-contained, follows the conventions above, includes an eval. MIT.

## License

[MIT](LICENSE), except the Entropy Fox name and logo in [`assets/brand/`](assets/brand/), which have [their own licence](assets/brand/LICENSE).
