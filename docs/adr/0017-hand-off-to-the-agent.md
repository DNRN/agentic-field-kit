# 0017. The installer hands off to the agent by starting it with an opening prompt

Date: 2026-10-30

## Status

Accepted

Plan: [new project from the installer](../plans/new-project.md). Related: [ADR 0009](0009-installer-in-rust.md) (the installer), [ADR 0016](0016-new-project-screen.md) (new project screen).

## Context

After the new project screen ([ADR 0016](0016-new-project-screen.md)) installs `project-init`, the next step happens in an agent: the skill asks about the remote and the project, and writes the files. The installer has no model and shouldn't get one. It already detects which agents are installed (command on `PATH`, or their home folder) for its header.

Most agent CLIs start an interactive session with an opening prompt given as an argument. Checked on 2026-10-30 from each tool's `--help`:

- Claude Code: `claude [prompt]`, "starts an interactive session by default".
- omp: `omp "<prompt>"`, "Interactive mode with initial prompt".
- Codex, Gemini CLI, OpenCode, Cursor: not installed on the machine used for this record; not checked.

## Decision

We will end the new project flow by starting the user's agent in the project folder, with an opening prompt that asks it to start the project with the `project-init` skill.

- Agents the installer can start are a table in the code: command and argument form, each entry checked against that tool's documentation or `--help`, with the date. It starts with `claude` and `omp`; others are added as they are checked.
- The screen lists the detected agents from that table and asks which one, unless there is one. A detected agent not in the table, or none at all, gets the printed next step instead ("open your agent in <folder> and ask it to start the project").
- The prompt names the folder and the skill: `Start a new project in this folder with the project-init skill.`
- The agent runs with the project folder as its working directory, given its arguments directly (no shell). The terminal is restored first. On Unix the installer replaces itself with the agent (`exec`); on Windows it starts the agent, waits, and exits with its status.

## Consequences

- `atk` in an empty folder ends in the agent, already asking the first question.
- The installer starts programs other than `git` for the first time. They are only ones it detected and the user picked, with fixed arguments.
- The table needs upkeep: a tool that changes its arguments breaks the hand-off until the entry is updated. Each entry records when it was checked; the fallback is the printed next step, so a missing entry costs one manual step, not a failure.
- What the agent does is up to the agent and its permission settings; the installer doesn't watch or check it.

## Rejected alternatives

- **Print the next step only** (the current hint): works everywhere, but every user does the hand-off by hand.
- **Run the agent non-interactively** (`claude -p`): `project-init` is a dialogue, so it needs an interactive session.
- **Start the agent through a shell command line**: quoting the folder path and prompt per shell, and an injection risk from folder names.
- **Assume a common `<cli> "<prompt>"` form for every agent**: unchecked tools may treat the argument differently (a file, a subcommand); a wrong guess is worse than the printed step.
