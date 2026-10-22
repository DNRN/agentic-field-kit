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
| D6 | Single-source review and audit rubrics | — | — |
| D7 | Convert evals to `claude plugin eval` format | D6 | — |
| D8 | Document and prevent duplicate skill loading | — | — |
| D9 | Release hygiene: `.gitignore`, CHANGELOG, version tags | D3 | — |
| D10 | Resolve `code-review` / `doc-sync` skill-command name collisions | — | — |

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

## D6. Single-source rubrics

**Problem.** The review checklist exists twice (`skills/code-review/SKILL.md`, `agents/code-reviewer.md`), as does the doc audit rubric (`skills/doc-sync/SKILL.md`, `agents/doc-auditor.md`). They will drift.

**Change.** Move each rubric to `skills/<skill>/references/rubric.md`; the skill links it; the agent reads it. First verify whether Claude Code subagent frontmatter can preload a skill; otherwise the agent prompt names the plugin-relative path.

**Acceptance.** Each rubric has exactly one copy; code-review and doc-sync evals still pass.

## D7. Convert evals to `claude plugin eval` format

**Problem.** `claude plugin eval` reads `evals/**/case.yaml` or `prompt.md` + `graders/*.md`; the current flat `evals/<component>.md` files are manual checklists it does not run.

**Change.** Scaffold with `claude plugin eval init`, one case directory per golden task (`evals/code-review/seeded-bugs/…`), scaffold scripts building the scratch repos, graders encoding the existing success criteria. Hook checks stay in `tests/` (deterministic, no model needed).

**Acceptance.** `claude plugin eval . --trust-plugin` runs every golden task with a with/without-plugin comparison; README and `evals/README.md` updated to say so.

## D8. Duplicate skill loading

**Problem.** Default `./install.sh` links into `~/.agents/skills` and `~/.claude/skills`; Cursor and OpenCode read both, and Claude Code plugin users also get `skills/` from the plugin. Effect of duplicates per harness is unverified.

**Change.** Verify per harness; document the recommended install set (plugin users: `--harness agents --harness omp`); change the default only if duplicates cause real conflicts.

**Acceptance.** README states one recommended install per setup, backed by a checked observation per harness.

## D9. Release hygiene

**Change.** `.gitignore` (`evals/results/`), `CHANGELOG.md` (Keep a Changelog), bump `plugin.json` version per release, tag with `claude plugin tag`.

**Acceptance.** Each release has a changelog entry, a version bump, and a tag.

## D10. Skill-command name collisions

**Problem.** `skills/code-review` and `commands/code-review.md` both register as `agentic-toolkit:code-review`, and the same goes for doc-sync. A headless session lists one `agentic-toolkit:code-review` slash command, so one definition shadows the other. Which one wins, and whether the skill's auto-trigger still fires, is unverified.

**Change.** Check precedence. Then either rename the commands (e.g. `/agentic-toolkit:review`) or drop them in favor of the skills with explicit-invocation metadata, following `commands/README.md`'s own advice. Update ADRs 0002/0003 if the four-layer structure changes.

**Acceptance.** Every component name is unique in a session's init event; the review and doc-sync flows still start from both the user command and the skill trigger.

## Done

Fixed in the review follow-up, covered by `tests/`:

- Gate: doc-only reviews no longer approve every later doc-only diff (empty code-only hash).
- Gate: `git commit -am`, `git -c … commit`, `git --no-pager commit`, `git -C dir commit`, `cd dir && git commit`, `sh -c "git commit …"` are recognized.
- Gate: `-n` / `--no-verify` inside a commit message no longer bypasses.
- Gate: `git commit -a` stamps the same diff it checks (`--stamp-review --all`).
- `install.sh --mode copy` no longer deletes directories it did not create.
- Evals README no longer claims `claude plugin eval` runs the current files (see D7).
- `docs/why-docs-matters.md` doc-sync link fixed.
