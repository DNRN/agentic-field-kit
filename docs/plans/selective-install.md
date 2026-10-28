# Plan: selective, project-scoped installer

Most users need one or two components from this repository, not all of them in every session. This plan turns `install.sh` into an installer that:

- shows what can be installed and what is already installed,
- installs only the selected components, plus anything they depend on,
- installs into the current project by default; global install is opt-in,
- updates by pulling the toolkit and rerunning the installer, and removes what it installed.

Decisions, all `Accepted`:

- [ADR 0006](../adr/0006-project-scope-install-default.md): project scope is the default, and project installs are copies.
- [ADR 0007](../adr/0007-component-catalog.md): installable components are declared in `catalog.tsv`.
- [ADR 0008](../adr/0008-project-install-manifest.md): installs are recorded in a manifest per scope; the content hash is the version.

## Project or global?

Make project the default, steer users there, and keep global as an explicit `--global`. Don't drop global install:

- **Mixing scopes is the real risk, not global installs as such.** With the same skill in a project and in the personal directory, Codex lists it twice, and Claude Code and Cursor use the personal copy instead of the project's. The installer flags every component installed in both scopes and offers to remove the global copy.
- **Project copies are committed.** Teammates get the same version, and so does every harness they use. The skill loads only in the repositories that chose it.
- **Global still has uses.** It suits a personal skill wanted in every repository (`adr-planning`), the Claude Code plugin at user scope, and removing existing global installs. Without `--global`, current users would have no way to undo `~/.agents/skills` links.

## Evidence

Observed 2026-10-23 with an isolated `HOME`, a scratch git repository, and the listing commands from [D8](design-improvements.md#d8-duplicate-skill-loading--done). Cursor was checked from its docs and bundled code, because listing its skills requires a login.

| Harness | Project dirs read | Found from a subdirectory | Same skill in project and personal dirs | Project symlink to outside the repo |
| --- | --- | --- | --- | --- |
| Claude Code 2.1.288 | `.claude/skills`, `.claude/agents` | yes (`.claude/settings.json`: no) | listed once, personal copy used | loads |
| Codex 0.164.0 | `.agents`, `.codex` | yes | **listed twice** | loads, renamed `agentic-toolkit:<name>` |
| Gemini CLI 0.65.0 | `.agents`, `.gemini`; trusted folders only | no | listed once, project copy used | loads |
| OpenCode 1.19.4 | `.agents`, `.claude`, `.opencode` | yes | listed once, project copy used | loads |
| omp 18.12.5 | `.agents`, `.claude`, `.codex`, `.omp`, `.opencode` | `.agents`, `.claude`, `.omp` only | project copy keeps the name; the personal one becomes `agents/<name>` | loads |
| Cursor Agent 2026.10.22 (docs, code) | `.agents`, `.cursor`; `.claude`, `.codex` with third-party extensibility on | no | personal copy used [inference] | probably skipped [inference] |

Identical copies in a project's `.agents/skills` and `.claude/skills`, observed 2026-10-28 in a scratch repository: OpenCode 1.19.4 (`opencode debug skill`) lists the skill once, from `.agents/skills`; omp 18.12.5 lists it once. Codex and Gemini CLI don't read `.claude`. Cursor reads both only with third-party extensibility on, and is expected to list it once, as it does for identical personal copies [inference].

Claude Code plugin findings from the 2026-10-23 run:

- A project `.claude/skills/<name>` does **not** hide the plugin's `agentic-toolkit:<name>`. Both are listed.
- A personal `~/.claude/skills/<name>` hides the plugin skill only when it is a symlink to the plugin's own folder. A personal copy with identical content does not hide it. D8 and the README said a copy does too (corrected in S0).
- `claude plugin install … --scope project` writes `enabledPlugins` into `.claude/settings.json`. That file is read only when Claude Code starts at the repository root. `marketplace add <dir> --scope project` writes an absolute path, which breaks on other machines. The GitHub source `DNRN/agentic-toolkit` doesn't have that problem.

## Target layout

| Scope | Every harness except Claude Code | Claude Code without the plugin | Claude Code with the plugin enabled for the project | Manifest |
| --- | --- | --- | --- | --- |
| Project (default) | copy in `.agents/skills/<name>` | second copy in `.claude/skills/<name>`; agents copied to `.claude/agents/<name>.md` | nothing extra: the plugin provides skills and agents | `.agents/agentic-toolkit.tsv`, committed |
| Global (`--global`) | symlink in `~/.agents/skills/<name>` (copy on Windows) | `~/.claude/skills/<name>` (`--harness claude`) | nothing extra | `~/.agents/agentic-toolkit.tsv` |

"Enabled for the project" means the project's committed `.claude/settings.json` enables `agentic-toolkit@…`, so the decision is the same for every teammate. A plugin enabled only at user scope on one machine doesn't change what gets committed; Claude Code then lists both `code-review` and `agentic-toolkit:code-review`. `--list` shows where the plugin is enabled in its header, and the README says what that means.

## User experience

Run from inside the project, with no arguments, on a terminal:

```
$ cd ~/code/shop && ~/src/agentic-toolkit/install.sh
agentic-toolkit 0.2.0 → project /home/u/code/shop
Harnesses found: claude, codex, opencode
Claude Code plugin: not enabled

   #  component      kind   project   global    summary
   1  adr-planning   skill  installed -         Record design decisions as ADRs
   2  code-review    skill  outdated  linked    Fresh-eyes review before commit
      ! in both scopes
   3  doc-sync       skill  -         -         Keep docs true to the code
   4  code-reviewer  agent  -         n/a       Read-only code review subagent
      Claude Code only; needs code-review
   5  doc-auditor    agent  -         n/a       Read-only doc audit subagent
      Claude Code only; needs doc-sync
   6  commit-gate    hook   n/a       n/a       Ask before unreviewed commits
      plugin only until roadmap phase 1; needs code-review, doc-sync

! in both scopes: Codex lists it twice; Claude Code and Cursor use the global copy.
Scope: project (/home/u/code/shop)
Number: mark + install / - remove · u: mark updates (~) · g: other scope
Enter: review and apply · q: quit
> 3 4
(table again, rows 3 and 4 marked +)
> 
  copy    .agents/skills/doc-sync
  copy    .claude/skills/doc-sync
  ok      code-review: already installed
  copy    .claude/agents/code-reviewer.md
Apply? [y/N]
```

Command-line surface. Every interactive action has a scriptable form:

| Command | Effect |
| --- | --- |
| `install.sh` on a terminal | interactive menu, project scope |
| `install.sh --list` | status table for the project and global scopes; changes nothing |
| `install.sh NAME…` | install the named components and their dependencies |
| `install.sh --all` | every component the scope supports |
| `install.sh --update [NAME…]` | bring installed components up to date (see [updates](#updates)); all of them if no names are given |
| `install.sh --remove [NAME…]` | remove the components we installed (all of ours if no names are given) |
| `--project DIR` · `--global` | choose the scope (default: git root of the working directory) |
| `--force` | let `--update` and `--remove` overwrite or delete a locally modified copy |
| `--dry-run` · `--yes` · `--interactive` | print the plan only · skip confirmations · force the menu (used by tests) |
| `--harness`, `--mode` | global scope only, unchanged; `--mode symlink` with project scope is an error (ADR 0006) |
| `--check` | unchanged |

Without a terminal, and with neither names nor `--all`, the installer exits with an error that lists the components. It doesn't silently install everything.

### Status values

| Value | Meaning | Detected by |
| --- | --- | --- |
| `-` | not installed | not in the manifest, path absent |
| `installed` | ours, up to date | disk hash = manifest hash = source hash |
| `outdated` | ours, the toolkit has changed since | disk hash = manifest hash ≠ source hash |
| `modified` | ours, edited in place | disk hash ≠ manifest hash; `--update` and `--remove` skip it without `--force` |
| `missing` | in the manifest, but a recorded path is gone | path absent |
| `retired` | ours, no longer in the catalog | manifest row without a catalog row |
| `linked` | our global symlink | link target inside this clone's `skills/` |
| `dangling` | our global symlink whose target is gone | `-L` and not `-e` |
| `foreign` | someone else's component with the same name | exists, not ours; never touched |
| `plugin` | provided by the Claude Code plugin enabled for this scope | `enabledPlugins` in `.claude/settings.json` (project) or `~/.claude/settings.json` (global) |
| `n/a` | the component doesn't support this scope | catalog `harnesses` column |

When several apply, the first wins: `modified`, `retired`, `missing`, `dangling`. An edit anywhere in a component therefore always needs `--force`.

Global installs made before the manifest still count as ours: symlinks into this clone (`linked`), and copies with the old `.agentic-toolkit-install` marker (`installed` when identical to the source, `modified` otherwise).

### Updates

`git pull` in the toolkit clone, then `install.sh --update` (or `u` in the menu) in the project:

| Change | What `--update` does |
| --- | --- |
| A component's files changed in the toolkit (`outdated`) | copies it again and records the new hash |
| A copy was deleted by hand (`missing`) | restores it |
| A component gained a new dependency | installs the dependency, recorded as `dependency` |
| A component was removed or renamed in the toolkit (`retired`) | asks to remove it (`--yes` removes it); never deletes it without asking |
| The copy was edited in the project (`modified`) | skips it; `--force` overwrites |

Components have no version numbers of their own (ADR 0008).

## Order

| # | Item | Depends on | ADR | Status |
| --- | --- | --- | --- | --- |
| S0 | Correct D8: only a personal symlink hides the plugin skill | — | — | done |
| S1 | Catalog and `--list` (read-only) | — | [0007](../adr/0007-component-catalog.md) | done |
| S2 | Selective install, update, and remove, project scope by default | S1 | [0006](../adr/0006-project-scope-install-default.md), [0008](../adr/0008-project-install-manifest.md) | done |
| SW | Windows | S2 | [0006](../adr/0006-project-scope-install-default.md) | done; the Windows CI job has not run yet |
| S3 | Interactive menu | S2 | — | done |
| S4 | Claude Code agents in the project | S2 | — | done |
| S5 | `commit-gate` outside the plugin | S4, roadmap phase 1 | new, if the gate's skill names become configurable | deferred |
| S6 | Docs and release | S2–S4, SW | — | docs done, ADRs accepted; release 0.2.0 open |

## S0. Correct the D8 finding

**Problem.** The README install table and D8 say a personal skill with identical content hides the plugin's skill. Only a symlink to the same real path does; a personal copy is listed alongside.

**Change.** Fix the Claude Code row in D8, the README line about `--harness claude`, and the `install.sh` header. The global default stays: a symlink into this clone is exactly the case that hides.

**Acceptance.** All three texts match the 2026-10-23 observation.

## S1. Catalog and status listing

**Change.**

- Add `catalog.tsv` with the six current components (ADR 0007).
- Add `tests/catalog.test.sh` to check catalog against disk, `requires` targets, and frontmatter names. `tests/run.sh` picks it up.
- `install.sh --list [--project DIR]` prints the status table and changes nothing. Outside a project it shows the global column only.
- Harness detection uses `command -v` for each CLI and the presence of its home directory. It only affects the header line; it never changes targets.

**Acceptance.** `tests/install.test.sh` has a fixture for each status value, plus a both-scopes fixture that raises the duplicate warning. A component on disk without a catalog row fails `tests/run.sh`.

## S2. Selective install, update, and remove

**Change.**

- Positional names, `--all`, `--project`, `--global`, `--update`, `--remove`, `--force`, `--dry-run`, and `--yes`.
- `requires` is resolved before anything is written; removing a component that another installed component requires is refused unless both are named.
- Without `--project`, the installer refuses a directory outside git unless `--global` is given. It refuses the toolkit clone, and a project root that is `$HOME` (that is the global scope).
- Manifest paths count only if they are one of the component's own folders or files; anything else in a committed manifest is ignored, never deleted.
- Project scope writes the copies of the [target layout](#target-layout) and the manifest (ADR 0008).
- `--remove NAME` also offers to remove dependencies that were pulled in for it and that nothing else needs.
- After a project install, a both-scopes duplicate prompts "remove the global copy?" (with `--yes`, it only warns).
- The existing safety rules carry over unchanged: never replace or remove anything that isn't ours.

**Acceptance.**

- The existing `install.test.sh` cases pass with `--global --all` added.
- New cases:
  - Default scope is the git root, also from a subdirectory; `--project DIR` works outside git.
  - Refusal outside git, and inside the clone.
  - `requires` gets installed, and is offered for removal with its dependent.
  - Every row of the [updates](#updates) table.
  - `--remove NAME` removes only that component; a modified copy needs `--force`.
  - No `.claude` copies while the project enables the plugin.
- Smoke run in a scratch repository: Claude Code `system/init` lists the project skill. A teammate clone (fresh checkout) shows every component `installed`.

## SW. Windows

**Change.**

- On Git Bash, MSYS, and Cygwin (`uname -s`), `--global` defaults to `--mode copy`.
- Hashes ignore carriage returns (ADR 0008), so a copy checked out with CRLF line endings isn't `modified`.
- `.gitattributes` in this repository keeps text files LF in every checkout, so `install.sh` and its tests run in Git Bash.
- CI runs `tests/install.test.sh` and `tests/catalog.test.sh` on `windows-latest` under Git Bash.
- README: run the installer from Git Bash or WSL; from WSL, `--global` reaches only Linux-side harnesses.

**Acceptance.** The Windows CI job is green. Project layout needs no Windows-specific code: it contains no symlinks (ADR 0006).

## S3. Interactive menu

**Change.** No arguments on a terminal (`[ -t 0 ] && [ -t 1 ]`), or `--interactive`, opens the menu shown above. It is a thin layer: it builds the same argument lists S2 accepts, prints their dry-run plans, asks once, and runs them. It is plain bash 3.2 using `read` and numbered marks, so it needs no `gum`, `fzf`, or `dialog`. A number marks an uninstalled component for install and an installed one for removal; `u` marks every component that `--update` would change.

**Acceptance.** `tests/install.test.sh` pipes scripted input through `--interactive` (mark, review, confirm; quit without changes). Manual run in a real terminal: the table fits 80 columns.

## S4. Claude Code agents in the project

**Change.** The `agent` kind copies to `.claude/agents/<name>.md` and records it in the manifest. Its `requires` brings in the skill, because the agent reads the rubric through it ([ADR 0005](../adr/0005-rubrics-live-in-skills.md)). Agents show as `plugin` and can't be selected when the project enables the plugin; otherwise they would be listed twice.

**Acceptance.** Smoke run: the project agent appears in `system/init` `agents`.

## S5. `commit-gate` outside the plugin

**Problem.** The gate's denial message names `/agentic-toolkit:code-review` and `/agentic-toolkit:doc-sync`. Those are the plugin's names. A project install exposes them as bare `code-review` and `doc-sync`. Wiring the hook into a project also means merging JSON into `.claude/settings.json`.

**Change.** Defer this until roadmap phase 1 (portable commit gate as a git `pre-commit` hook). That item already plans an installer for every harness. When it lands, `catalog.tsv` gets a `hook` row whose install runs that installer, and the gate takes the skill names as a parameter. Until then, the catalog row's `harnesses` is `plugin` and `--list` says so.

**Acceptance.** Before roadmap phase 1, `commit-gate` can't be selected outside the plugin, and the menu says why.

## S6. Docs and release

**Change.**

- Rewrite the README's "Install skills" section around project scope:
  - the target layout table,
  - updating and removing,
  - committing the copies,
  - Gemini CLI's trusted-folder requirement,
  - starting Cursor and Gemini CLI at the repository root,
  - Windows.
- Update `skills/README.md` "Distribution".
- Add CHANGELOG entries under `[Unreleased]`:
  - Added: catalog, manifest, `--list`, selective, interactive, and project install; `--update`.
  - Changed (breaking): the default scope, and non-interactive runs now require names or `--all`.
- Release as minor `0.2.0`.
- After user confirmation, flip ADRs 0006, 0007, and 0008 to `Accepted` (done 2026-10-28).

**Acceptance.** `tests/run.sh`, `./install.sh --check`, and CI are green. Following the README from a fresh clone installs one skill into a scratch project.

## Rejected for the installer itself

- **TUI dependency (`gum`, `fzf`, `whiptail`)**: a nicer menu, but one more thing to install before the installer runs, and one more thing CI has to provide.
- **Rewrite in Python or Node**: easier to work with JSON, but it throws away the tested bash installer and its shellcheck coverage, and adds a runtime the skills don't need.
- **PowerShell installer for Windows**: a second implementation of every rule. Git for Windows ships bash, and Claude Code on Windows already relies on Git Bash.
- **Delegate to `npx skills add DNRN/agentic-toolkit@<skill>`**: the open Skills CLI already installs single skills, by default into the project. It doesn't cover agents, hooks, the plugin interaction, or the ours/foreign safety rules, and it needs Node. Whether it handles this repository is unverified. Check it in S6; if it works, list it in the README as the no-clone path for skills.
