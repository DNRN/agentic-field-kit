# Plan: new project from the installer

Running `agentic-toolkit` in a folder without git should lead to a started project: the installer shows where the user stands, lets them pick that folder or a new subfolder, installs `project-init` there, and starts their agent with the skill.

Decisions, all `Accepted` (2026-10-30):

- [ADR 0016](../adr/0016-new-project-screen.md): outside a repository, the menu opens a new project screen: this folder, a new subfolder, or browse components. The installer never runs `git init` or writes the project files; `project-init` does.
- [ADR 0017](../adr/0017-hand-off-to-the-agent.md): the installer ends the flow by starting the picked agent in the project folder with an opening prompt, from a table of agents whose argument form has been checked. Others get the printed next step.

Unchanged: [ADR 0006](../adr/0006-project-scope-install-default.md) for every run inside a repository and for the command line; the `project-init` skill.

## Order

| # | Item | Depends on | ADR |
| --- | --- | --- | --- |
| N0 | This plan; ADRs 0016–0017 accepted (done 2026-10-30) | — | 0016, 0017 |
| N1 | Hints: CLI error, menu line, `project-init` details, "Next:" after install, README section (done 2026-10-30) | — | — |
| N2 | New project screen (done 2026-10-30) | N0, N1 | 0016 |
| N3 | Hand-off to the agent (done 2026-10-30; macOS and Windows manual runs pending) | N2 | 0017 |
| N4 | Check and add Codex, Gemini CLI, OpenCode, Cursor to the agent table | N3 | 0017 |

## N2. New project screen

**Change.**

- `main`: on a terminal, no arguments, no `--project`/`--global`, and `scope::project` fails with `NO_REPO` → the TUI starts on the new project screen instead of the list.
- Screen, top to bottom: the folder's full path (home shown as `~`), what's in it (empty / `N` files / project files found), the choices with keys (`h` here, `s` subfolder, `b` browse, `q` quit) and the highlighted default from ADR 0016. "Here" is absent in `$HOME` and `/`, with a line saying why.
- Subfolder: a one-line name input; refused when empty, containing `/` or `\`, `.` or `..`, or already existing; shows `Will create <full path>` before Enter.
- Then install `project-init` with the project scope rooted at the chosen folder (the engine's existing plan and apply), and go to N3.
- Inside the browse menu, `project-init`'s details keep N1's text.

**Tests** (`TestBackend`): the path shows at 80×24; empty folder → default here; `$HOME` → no "here"; a folder with a git subfolder → default subfolder; `package.json` → listed and default here; subfolder name rules; `b` opens today's menu in the global scope; choosing here installs `project-init` into that folder and nothing else; nothing is written before a choice.

## N3. Hand-off to the agent

**Change.**

- `agents.rs`: the table `(name, command, how the prompt is passed, checked on)`, starting with `claude` (`claude <prompt>`) and `omp` (`omp <prompt>`), both checked 2026-10-30.
- After the install: detected agents from the table → pick one (skip the question when there is one); none → the printed next step from N1 and exit 0.
- Launch with `std::process::Command`: the agent's command, the prompt as one argument, `current_dir` the project folder. Restore the terminal first. Unix: `exec`. Windows: spawn, wait, exit with its status. A launch error prints the next step and exits 1.

**Tests.** Unit: argument vector per table entry; the prompt is one argument even with spaces and quotes in the folder name. Integration (`tests/install.test.sh` can't drive the TUI): a test-only entry pointing at a script on `PATH` that records its working directory and arguments, launched through the same function. Manual: Claude Code and omp, from an empty folder and from a subfolder choice, on Linux and macOS; Windows Terminal with Claude Code.

## Done (N2, N3), 2026-10-30

- `installer/src/newproject.rs`: inspect the folder, the choices and the default, subfolder names, `~` display. `installer/src/agents.rs`: the table (`claude`, `omp`, each with its check), the prompt, the command (no shell, one argument, the folder as working directory), `exec` on Unix and spawn-and-wait on Windows. The TUI gains the new project and agent screens; `tui::run` returns an `Outcome` with the agent to start, which `main` launches after the terminal is restored.
- Keys on the new project screen: `↑↓` and Enter, or `h` here, `s` new folder (then type the name; Enter creates, Esc goes back), `b` browse, `q` quit. One agent on PATH starts without a question; several get a numbered list; none, or `q` on the list, ends with the printed next step.
- Project files that set the "here" default: `package.json`, `Cargo.toml`, `pyproject.toml`, `setup.py`, `requirements.txt`, `go.mod`, `pom.xml`, `build.gradle(.kts)`, `Gemfile`, `composer.json`, `mix.exs`, `CMakeLists.txt`, `Makefile`.
- Tests: 6 `newproject` and 3 `agents` unit tests (including a started script that sees the folder and the prompt as one argument), 8 TUI tests (path shown at 80×24, home without "here", browse, here without an agent, a created subfolder with one agent, a refused name, picking among two, dry run).
- Checked by hand on Linux in a pseudo-terminal: the release binary in an empty `~/code/shop` drew the screen at 80×24; `h` installed `project-init`, printed "Starting Claude Code in ~/code/shop…", and replaced itself with a `claude` stand-in on `PATH`, which ran in that folder with the prompt as its only argument. Not yet run: a real Claude Code or omp session, macOS, Windows.

## N4. More agents

For each of Codex, Gemini CLI, OpenCode, Cursor: confirm from its documentation and `--help` how to start an interactive session with an opening prompt, add the table entry with the date, and run the manual check once.

## Risks

- **An agent changes its arguments**: the hand-off starts the agent wrongly or not at all. The fallback text names the folder, and each entry carries its check date.
- **The skill isn't where that agent reads skills**: the install writes `.agents/skills` and `.claude/skills` (no plugin in a new folder), which covers the table's agents; N4 checks each new one.
- **Default choice heuristics**: wrong in some folders; the screen always shows the path and the user picks.
