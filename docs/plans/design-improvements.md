# Plan: design improvements

Follow-ups from the 2026-10-22 repo review. Bugs from that review are fixed (see [Done](#done)); this plan covers the design issues that remain. Items marked **ADR** change a recorded decision and go through the [adr-planning skill](../../skills/adr-planning/SKILL.md) before implementation.

## Order

| # | Item | Depends on | ADR |
| --- | --- | --- | --- |
| D1 | ~~Accept ADRs 0001–0003~~ — done | — | status flip only |
| D2 | ~~Commit outstanding work~~ — done | D1 | — |
| D3 | ~~CI pipeline~~ — done | — | — |
| D4 | ~~Configurable docs paths for the docs check~~ — done | D3 | [0004](../adr/0004-configurable-docs-paths.md) supersedes 0003's trigger rule |
| D5 | ~~Hook prefilter for non-commit Bash calls~~ — done | D3 | — |
| D6 | ~~Single-source review and audit rubrics~~ — done | — | [0005](../adr/0005-rubrics-live-in-skills.md) |
| D7 | ~~Convert evals to `claude plugin eval` format~~ — done | D6 | — |
| D8 | ~~Document and prevent duplicate skill loading~~ — done | — | — |
| D9 | ~~Release hygiene: `.gitignore`, CHANGELOG, version tags~~ — done | D3 | — |
| D10 | ~~Resolve `code-review` / `doc-sync` skill-command name collisions~~ — done (with D6) | — | [0005](../adr/0005-rubrics-live-in-skills.md) |
| D11 | Commit gate ignores formatting-only diffs | D7 | — |

## D1. Accept ADRs 0001–0003 — done

**Problem.** All three ADRs are `Proposed` but already implemented; the repo's own adr-planning skill says implementation follows acceptance.

**Change.** User confirms each decision; flip `Status` to `Accepted`. Anything the user rejects gets a superseding ADR instead.

**Acceptance.** No implemented ADR is `Proposed`. Done 2026-10-22: user accepted all three.

## D2. Commit outstanding work — done

**Problem.** Code review, doc-sync, hooks, ADRs, and docs are untracked.

**Change.** Commit in logical units (ADR mechanism, code review, doc sync, gate fixes) through the commit gate itself — the first real dogfood run.

**Acceptance.** `git status` clean; each commit passed or consciously waived the gate. Done 2026-10-22 (`0de25d1`…`ce917e1`); the gate did not run, since the committing session did not load the plugin.

## D3. CI pipeline — done

**Problem.** Nothing ran automatically: shell bugs, broken links, and invalid manifests could reach `main` unnoticed.

**Change.** `.github/workflows/ci.yml` on push to `main` and on every PR, three jobs:

- **scripts**: `shellcheck install.sh hooks/scripts/*.sh tests/*.sh`, `./install.sh --check`, `tests/run.sh`
- **plugin**: installs the Claude Code CLI; `claude plugin validate --strict .` (marketplace) and `claude plugin validate .claude-plugin/plugin.json` (not strict: the validator scans `agents/` and `commands/` and warns on their `README.md`, which the manifest's explicit lists keep from loading)
- **docs**: lychee over `**/*.md`, offline, internal links and `#anchors`; every file with a mermaid block rendered by `@mermaid-js/mermaid-cli`

A link check catches missing files and anchors, not a link to the wrong existing file (the original doc-sync link bug); that remains the doc-auditor's job.

**Acceptance.** A PR that breaks any of the above fails CI. Done 2026-10-22: every step runs green locally (actionlint-clean workflow); an unregistered agent, a missing link target, and a broken mermaid diagram each fail their step.

**Found while wiring CI.** Claude Code loaded `agents/README.md` as an agent and `commands/README.md` as a command (`agentic-toolkit:README`, seen in a headless session's init event). `plugin.json` now lists `agents`, `commands`, and `outputStyles` explicitly (these keys replace the folder scan); `tests/plugin-manifest.test.sh` fails when a component file is not listed. `claude plugin details` reports `Agents (0)` under explicit lists, while a real session loads both agents, so don't use it to check this.

## D4. Configurable docs paths for the docs check — done

**Problem.** The docs check treats every `.md` as documentation. In repos where markdown *is* the product (this one: skills, agents, commands), the docs check never fires, and doc paths outside `.md` (`.mdx`, `.rst`) are ignored.

**Change.** As decided in [ADR 0004](../adr/0004-configurable-docs-paths.md): the gate reads doc pathspecs from a committed `.commit-gate-docs` at the repo root (one per line; default `*.md`). The original plan used `git config commitgate.docs`, which was rejected because `.git/config` is never committed, so clones and CI would silently get the default. The same pathspecs drive "docs touched", "repo tracks docs", and the code-only hash, through one `load_docs` function shared by stamping and checking. This repo's file lists `docs/`, `README.md`, and `*/README.md`.

**Acceptance.** In this repo, a skill-only change without a `docs/` edit raises the docs check; default behavior unchanged elsewhere (regression tests for both). Done 2026-10-22: 12 new cases in `tests/commit-gate.test.sh` (markdown-as-code, configured paths and single files, review-stamp survival, docs stamps, `*.rst`, comment-only fallback, no tracked docs, long-form magic, CRLF/comments/padding); 5 of them fail against the pre-D4 gate, and the earlier default-behavior cases still pass.

## D5. Hook prefilter — done

**Problem.** The PreToolUse hook matches every Bash call and starts `python3` for each, including `ls`.

**Change.** Before invoking the parser, `case "$input" in *commit*) ;; *) exit 0 ;; esac`.

**Acceptance.** Non-commit Bash calls exit without spawning `python3`; gate tests still pass. Done 2026-10-22: a `python3` shim in `tests/commit-gate.test.sh` counts launches (zero for a non-commit command; the test fails against the pre-D5 gate, and a commit command still launches it). A non-commit call dropped from about 25 ms to about 4 ms.

## D6. Single-source rubrics — done

**Problem.** The review checklist exists twice (`skills/code-review/SKILL.md`, `agents/code-reviewer.md`), as does the doc audit rubric (`skills/doc-sync/SKILL.md`, `agents/doc-auditor.md`). They will drift.

**Constraint added by the user.** The single copy must stay harness-agnostic: Claude Code, Codex, omp, and the rest.

**Change.** As decided in [ADR 0005](../adr/0005-rubrics-live-in-skills.md): each rubric lives once in `skills/<skill>/references/rubric.md`, the only location every harness installs. Skills keep the procedure and pass the rubric's location to whichever read-only subagent reviews; `agents/*.md` are thin wrappers that read the rubric they are given. The planned Claude Code `skills:` preload was rejected: it is Claude-only and injects `SKILL.md`, not the rubric.

**Acceptance.** Each rubric has exactly one copy; code-review and doc-sync evals still pass. Done 2026-10-22, observed in real runs on scratch repos with seeded defects:

- **omp**, generic `reviewer` subagent given only the rubric path: followed the rubric format exactly; found the off-by-one, the SQL injection, and the missing test.
- **Claude Code**, `/agentic-toolkit:code-review`: skill loaded, main agent passed the rubric's absolute path to `agentic-toolkit:code-reviewer`, which read the rubric first; same findings, rubric format.
- **Claude Code**, `/agentic-toolkit:doc-sync`: stale `--old` flag in `docs/guide.md` fixed surgically. With the auditor requested, `agentic-toolkit:doc-auditor` read the rubric first. Without the request, the main agent twice audited by itself, citing a Claude Code instruction not to spawn agents unasked; recorded in ADR 0005's consequences.

## D10. Skill-command name collisions — done (with D6)

Confirmed in a real session: loading `agentic-toolkit:code-review` from inside the command re-invoked the command, and a bare `code-review` loaded Claude Code's built-in skill. The commands were removed; the skills are the entry points (`/agentic-toolkit:code-review` still works, now naming the skill). The init event now lists each `agentic-toolkit:` skill, slash command, and agent once. See [ADR 0005](../adr/0005-rubrics-live-in-skills.md).

## D7. Convert evals to `claude plugin eval` format — done

**Problem.** `claude plugin eval` reads `evals/**/case.yaml` or `prompt.md` + `graders/*.md`; the flat `evals/<component>.md` files were manual checklists it did not run.

**Change.** One case directory per golden task and negative case, grouped by component (`evals/code-review/seeded-bugs/…`): 13 cases, each with a `scaffold.sh` that builds the scratch repo and graders encoding the old success criteria, mostly free ones (`regex` over produced files and the HEAD reflog, `tool_used`, `file_exists`), `llm` only for short replies. The flat checklists are gone. Hook checks stay in `tests/` (deterministic, no model needed).

**Acceptance.** `claude plugin eval . --trust-plugin` runs every golden task with a with/without-plugin comparison; README and `evals/README.md` updated to say so. Done 2026-10-23: the full suite ran with `--scaffold --allow-tools Bash Edit Write --runs 1`, both arms, in 157 s for $3.86. 12 of 13 cases passed with the plugin; mean Δ +0.16. The plugin raised the score on 5 cases (largest: `adr-planning-plan-feature`, 0.2 → 1.0; `code-review-seeded-bugs`, 0.43 → 1.0, where the baseline missed the untested VIP branch) and tied on 7, where Claude without the plugin already passes. It lost on `code-review-formatting-only`: the gate blocks a whitespace-only commit (D11). Every skill-fired and rubric-read indicator passed. The single run per arm is noisy; the default 3 runs is the number to trust.

Found while running: inside eval runs `git` is masked (`permission denied`, exit 126) on this Arch machine, though a normal sandboxed session runs it; a user-owned copy of the binary first on `PATH` works. Workaround in [`evals/README.md`](../../evals/README.md#known-issues).

## D8. Duplicate skill loading — done

**Problem.** Default `./install.sh` linked into `~/.agents/skills`, `~/.claude/skills`, and `~/.omp/agent/skills`; several harnesses read more than one of these, and Claude Code plugin users also get `skills/` from the plugin. Effect of duplicates per harness was unverified.

**Change.** Verify per harness; document the recommended install set; change the default only if duplicates cause real conflicts.

**Observed 2026-10-23**, each harness under an isolated `HOME`, two same-named skills with different content placed in two directories it reads:

| Harness | Reads | Same skill in two places |
| --- | --- | --- |
| Claude Code 2.1.288 (`system/init` skill list) | `~/.claude/skills`, not `~/.agents/skills` | Personal + plugin, identical content (symlink or marketplace copy): the plugin's `agentic-toolkit:<skill>` disappears; `/agentic-toolkit:code-review` then answers "isn't installed in this session". Different content: both load. A personal `code-review` replaces the built-in `/code-review` |
| omp 18.12.5 (`skill://` listing) | `~/.omp/agent/skills` and `~/.agents/skills` | Same folder or identical copies: listed once. Different content: the `~/.omp` copy keeps the name, the other becomes `agents/<name>` |
| Codex 0.164.0 (app-server `skills/list`) | `~/.agents/skills`, `~/.codex/skills` | Listed twice unless both resolve to the same folder. A symlink into this repo is named `agentic-toolkit:<name>` (real path under `.claude-plugin/`); a copy keeps the bare name |
| Gemini CLI 0.65.0 (`gemini skills list`) | `~/.agents/skills`, `~/.gemini/skills` | Listed once, `~/.agents` wins, "Skill conflict detected" on stderr |
| OpenCode 1.19.4 (`opencode debug skill`) | `~/.agents/skills`, `~/.claude/skills`, `~/.config/opencode/skills` | Listed once, last directory scanned wins, "duplicate skill name" in the log only |
| Cursor Agent CLI 2026.10.22 | `~/.agents/skills`, `~/.cursor/skills`; `~/.claude/skills` only with third-party extensibility on | Not run: every listing command needs a login. From [its docs](https://cursor.com/docs/skills) and the CLI's bundled code, listed once |

The Claude Code row is a real conflict: the old default hid every plugin skill, including the `/agentic-toolkit:code-review` and `/agentic-toolkit:doc-sync` that `commit-gate.sh`'s denial tells the agent to run. It also contradicts Claude Code's docs, which say plugin skills always load alongside personal ones. Everything else is harmless.

**Result.** `./install.sh` now installs into `~/.agents/skills` only, which covers Codex, Gemini CLI, OpenCode, Cursor, and omp and cannot shadow the plugin. `--remove` still defaults to every target, so links made by the old default go too. README lists one install per setup; `tests/install.test.sh` fails if the default touches `~/.claude/skills`.

**Acceptance.** README states one recommended install per setup, backed by a checked observation per harness. Done 2026-10-23; Cursor is the one harness checked from docs and code instead of a run.

## D9. Release hygiene — done

**Change.** `.gitignore` (`evals/results/`), `CHANGELOG.md` (Keep a Changelog), bump `plugin.json` version per release, tag with `claude plugin tag`.

**Acceptance.** Each release has a changelog entry, a version bump, and a tag. Done 2026-10-23: the release steps are in the README's [Releasing](../../README.md#releasing) section; `tests/release.test.sh` fails when `plugin.json`'s version has no dated changelog entry or the `Unreleased` section is missing. First release `0.1.0`, tagged `agentic-toolkit--v0.1.0`.

## D11. Commit gate ignores formatting-only diffs

**Problem.** The code-review skill and rubric skip whitespace and formatting-only diffs, and doc-sync skips changes with no observable surface, but `commit-gate.sh` denies such a commit as unreviewed (and undocumented when the repo tracks docs). The agent then has to ask the user before a whitespace fix. Caught by `code-review-formatting-only` (D7) and reproduced by feeding the gate that case's staged diff.

**Change.** In gate mode, allow a diff that is empty under `git diff --ignore-all-space --ignore-blank-lines`. Lockfiles and generated files, which the rubric also skips, need a path list and are out of scope here.

**Acceptance.** A regression test in `tests/commit-gate.test.sh` (whitespace-only diff allowed, whitespace plus a code change still denied); `code-review-formatting-only` passes with the plugin.

## Done

Fixed in the review follow-up, covered by `tests/`:

- Gate: doc-only reviews no longer approve every later doc-only diff (empty code-only hash).
- Gate: `git commit -am`, `git -c … commit`, `git --no-pager commit`, `git -C dir commit`, `cd dir && git commit`, `sh -c "git commit …"` are recognized.
- Gate: `-n` / `--no-verify` inside a commit message no longer bypasses.
- Gate: `git commit -a` stamps the same diff it checks (`--stamp-review --all`).
- `install.sh --mode copy` no longer deletes directories it did not create.
- Evals README no longer claims `claude plugin eval` runs the current files (see D7).
- `docs/why-docs-matters.md` doc-sync link fixed.
