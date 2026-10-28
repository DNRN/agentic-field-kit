# 0006. Install into the project by default; global install is opt-in

Date: 2026-10-23

## Status

Accepted

Plan: [selective install](../plans/selective-install.md). Related: [ADR 0007](0007-component-catalog.md) (catalog), [ADR 0008](0008-project-install-manifest.md) (manifest).

## Context

`install.sh` installs every skill into `~/.agents/skills`, so each one loads in every session of every repository. Most users want one or two components in the repository where they need them. Four forces decide where installs should go:

- **Mixing scopes breaks things in three harnesses.** With the same skill in a project and in the personal directory (observed 2026-10-23 in a scratch repo, see the plan's [evidence](../plans/selective-install.md#evidence)), Codex lists it twice, while Claude Code and Cursor use the personal copy and ignore the project's version. Gemini CLI, OpenCode, and omp use the project copy.
- **One project directory covers almost every harness.** Codex, Gemini CLI, OpenCode, Cursor, and omp read `<repo>/.agents/skills`. Claude Code reads only `<repo>/.claude/skills`, and a project skill there does not hide the plugin's `agentic-toolkit:<name>` skill.
- **Symlinks don't work in a project.** A link to `~/src/agentic-toolkit/skills/x` is machine-specific, so a teammate gets a dangling link. Cursor skips project symlinks that point outside the root (from its code). Codex renames a linked skill to `agentic-toolkit:<name>`. Even a relative link inside the repository breaks on Windows: git checks symlinks out as small text files unless `core.symlinks` is on, which needs Developer Mode.
- **Identical copies in two project directories are harmless.** OpenCode and omp read both `.agents/skills` and `.claude/skills`; with an identical copy in each, both list the skill once (observed 2026-10-28, OpenCode 1.19.4 and omp 18.12.5).

During planning, someone proposed going further and supporting project installs only.

## Decision

We will make the current project the default install scope:

- The project is the git root of the working directory, or `--project DIR`, which may be any directory. Without `--project`, the installer refuses to run outside a git repository unless `--global` is given. It refuses to treat the toolkit's own clone, or `$HOME` (the global scope), as the project.
- Project installs are **copies**, meant to be committed: `<repo>/.agents/skills/<name>` for every harness, and a second copy in `<repo>/.claude/skills/<name>` for Claude Code. The installer leaves the `.claude` copy out when the project's `.claude/settings.json` enables the agentic-toolkit plugin, so the skill isn't listed twice.
- The project's manifest ([ADR 0008](0008-project-install-manifest.md)) records each copy and its content hash, so the installer can tell an outdated copy from a locally edited one and detect drift between the two copies.
- Global install (`--global`) stays available. It keeps today's targets and its symlink default, except on Windows (Git Bash, MSYS, Cygwin), where `ln -s` makes a copy without telling you, so copy is the default there.
- The installer warns when a component is installed in both scopes and offers to remove the global copy.

## Consequences

- A skill loads only in repositories that chose it. Teammates and every harness get the same pinned version through git, on every OS.
- Copies don't update when the toolkit is pulled. `--list` shows them as outdated, and `--update` refreshes them without overwriting local edits.
- Each skill is committed twice when Claude Code is a target. An edit to one copy shows as `modified`; `--update --force` puts both back in line.
- `./install.sh` run from inside the toolkit clone, the usual way today, now stops and points to `--global`. This is a deliberate breaking change for 0.x, recorded in the changelog.
- Project skills load in Gemini CLI only in a trusted folder. Cursor and Gemini CLI find them only when started at the repository root. The README has to say both.
- The installer is bash. On Windows it runs in Git Bash or WSL; there is no PowerShell installer.

## Rejected alternatives

- **Projects only (no global install)**: there would be no way to clean up existing global installs, and no way to keep a personal skill like `adr-planning` in every repository. The real risk is mixing scopes, and a warning handles that.
- **Global by default (today)**: every skill loads in every session, there is no per-project version pinning, and teammates don't get the same set.
- **Project symlinks into the clone**: machine-specific paths, skipped by Cursor, and renamed by Codex.
- **Relative symlink `.claude/skills/<name>` → `../../.agents/skills/<name>`**: one copy instead of two, but a Windows checkout without `core.symlinks` turns it into a text file and Claude Code loses the skill. The layout is committed, so it can't depend on the OS of whoever ran the installer.
