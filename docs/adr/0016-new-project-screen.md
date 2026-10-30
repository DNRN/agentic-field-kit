# 0016. Outside a repository, the installer offers to start a project

Date: 2026-10-30

## Status

Accepted

Plan: [new project from the installer](../plans/new-project.md). Related: [ADR 0006](0006-project-scope-install-default.md) (project scope is the git root), [ADR 0017](0017-hand-off-to-the-agent.md) (handing off to the agent).

## Context

`agentic-toolkit` finds the project as the git root of the working directory ([ADR 0006](0006-project-scope-install-default.md)). Run in a folder without git, the menu shows the global scope and a one-line hint; the CLI fails and prints the `--project . project-init` command. Starting a project is the `project-init` skill's job, and it runs in the agent, so a new user has to learn a flag, install a skill, and know to ask their agent, before anything happens.

Where a user runs `atk` outside a repository varies:

- an empty folder made for the new project;
- a folder with files but no git (code never put under version control);
- a folder of projects (`~/code`), or `$HOME`, where the new project belongs in a subfolder.

The installer must not decide on its own where a project goes, and must never make `$HOME` or `/` a project.

## Decision

We will open a "new project" screen instead of the component list when `agentic-toolkit` runs without arguments on a terminal, outside any git repository, and without `--project` or `--global`. The command line is unchanged.

- The screen shows the current folder's full path first, and what is in it: empty, `N` files, or the project files it found (`package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`, …).
- Choices:
  1. **Start a project here.** Existing files stay. Not offered in `$HOME` or `/`.
  2. **Start a project in a new subfolder.** Asks for a name (a single path component that doesn't exist yet), shows the full path it will create, creates it.
  3. **Browse components.** Today's menu in the global scope.
  4. **Quit.**
- The highlighted default: "here" when the folder is empty or has project files; "subfolder" when it is `$HOME`, has subfolders that are git repositories, or has files but none that mark a project.
- Choosing 1 or 2 installs `project-init` into that folder as `--project <folder>` does, then hands off to the agent ([ADR 0017](0017-hand-off-to-the-agent.md)). The installer itself never runs `git init` or writes `AGENTS.md` or `PROJECT.md`: the skill stays their only author.

## Consequences

- Running `atk` in a fresh folder leads straight to a started project, with no flag to learn.
- The installer acts in a folder that isn't a git repository without `--project`, but only after the user picks it on a screen that shows its path. It creates at most one folder, the subfolder the user named.
- A second screen to maintain and test (`TestBackend`), and a heuristic for the default choice that will be wrong sometimes; the user always confirms.
- Users who wanted the global menu outside a repository press one more key, or run `atk --global`.

## Rejected alternatives

- **Offer it only when the folder is empty or has no project files**: hides the choice exactly when a user has code without git, and the screen already makes "here" an explicit choice.
- **Pick the folder automatically**: guesses wrong in `$HOME` and in folders of projects, and the user can't see where files will land.
- **The installer runs `git init` and writes the files itself**: a second copy of `project-init`'s rules in Rust that drifts from the skill (see [ADR 0005](0005-rubrics-live-in-skills.md) for the same reasoning about rubrics), and no dialogue.
- **Keep the hints only** (the error text and the menu line): works, but leaves every step to the user.
