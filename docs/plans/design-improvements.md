# Plan: design improvements

Follow-ups from the 2026-10-22 repo review. Bugs from that review are fixed (see [Done](#done)); this plan covers the design issues that remain. Items marked **ADR** change a recorded decision and go through the [adr-planning skill](../../skills/adr-planning/SKILL.md) before implementation.

## Order

| # | Item | Depends on | ADR |
| --- | --- | --- | --- |
| D1 | Accept ADRs 0001–0003 | — | status flip only |
| D2 | Commit outstanding work | D1 | — |
| D3 | CI pipeline | — | — |
| D4 | Configurable docs paths for the docs check | D3 | amends 0003 |
| D5 | Hook prefilter for non-commit Bash calls | D3 | — |
| D6 | Single-source review and audit rubrics | — | — |
| D7 | Convert evals to `claude plugin eval` format | D6 | — |
| D8 | Document and prevent duplicate skill loading | — | — |
| D9 | Release hygiene: `.gitignore`, CHANGELOG, version tags | D3 | — |

## D1. Accept ADRs 0001–0003

**Problem.** All three ADRs are `Proposed` but already implemented; the repo's own adr-planning skill says implementation follows acceptance.

**Change.** User confirms each decision; flip `Status` to `Accepted`. Anything the user rejects gets a superseding ADR instead.

**Acceptance.** No implemented ADR is `Proposed`.

## D2. Commit outstanding work

**Problem.** Code review, doc-sync, hooks, ADRs, and docs are untracked.

**Change.** Commit in logical units (ADR mechanism, code review, doc sync, gate fixes) through the commit gate itself — the first real dogfood run.

**Acceptance.** `git status` clean; each commit passed or consciously waived the gate.

## D3. CI pipeline

**Problem.** Nothing runs automatically; a broken link and a gate bypass shipped unnoticed.

**Change.** `.github/workflows/ci.yml` on push and PR:

- `shellcheck install.sh hooks/scripts/*.sh tests/*.sh`
- `tests/run.sh` (gate and installer regression tests)
- `./install.sh --check`
- `claude plugin validate .` (skip with a notice if the CLI is unavailable in CI)
- markdown link check over `*.md`
- mermaid syntax check (`mmdc`) over embedded diagrams

**Acceptance.** A PR that breaks any of the above fails CI.

## D4. Configurable docs paths for the docs check

**Problem.** The docs check treats every `.md` as documentation. In repos where markdown *is* the product (this one: skills, agents, commands), the docs check never fires, and doc paths outside `.md` (`.mdx`, `.rst`) are ignored.

**Change.** Read doc pathspecs from `git config commitgate.docs` (space-separated, e.g. `docs/ README.md`); default stays `*.md`. Use the same pathspecs for "docs touched", "repo tracks docs", and the code-only hash exclusion. Set it for this repo.

**ADR.** Amends the trigger rule in ADR 0003 (edit while Proposed, else supersede).

**Acceptance.** In this repo, a skill-only change without a `docs/` edit raises the docs check; default behavior unchanged elsewhere (regression tests for both).

## D5. Hook prefilter

**Problem.** The PreToolUse hook matches every Bash call and starts `python3` for each, including `ls`.

**Change.** Before invoking the parser, `case "$input" in *commit*) ;; *) exit 0 ;; esac`.

**Acceptance.** Non-commit Bash calls exit without spawning `python3`; gate tests still pass.

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

## Done

Fixed in the review follow-up, covered by `tests/`:

- Gate: doc-only reviews no longer approve every later doc-only diff (empty code-only hash).
- Gate: `git commit -am`, `git -c … commit`, `git --no-pager commit`, `git -C dir commit`, `cd dir && git commit`, `sh -c "git commit …"` are recognized.
- Gate: `-n` / `--no-verify` inside a commit message no longer bypasses.
- Gate: `git commit -a` stamps the same diff it checks (`--stamp-review --all`).
- `install.sh --mode copy` no longer deletes directories it did not create.
- Evals README no longer claims `claude plugin eval` runs the current files (see D7).
- `docs/why-docs-matters.md` doc-sync link fixed.
