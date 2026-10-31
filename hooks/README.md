# Hooks

Event-triggered scripts. Config lives in `hooks/hooks.json` (shape: `{"hooks": {<Event>: [...]}}`), scripts in `hooks/scripts/`.

Least portable category: hook event models and config formats differ per harness (Claude Code `hooks.json`, Cursor `hooks.json`, OpenCode JS plugins, Codex lifecycle config). Keep the scripts generic and self-contained; the wiring is per-harness.

## Format

- Declare matchers per event (`PreToolUse`, `PostToolUse`, `Stop`, …); keep hooks fast and silent on success.
- Reference bundled scripts via `"${CLAUDE_PLUGIN_ROOT}"/hooks/scripts/<script>.sh` — never absolute local paths.
- Hooks must be idempotent and safe when the tool state is unexpected; they run on every matching event.
- Every hook ships with regression tests in `tests/<script>.test.sh` (JSON on stdin, decision on stdout); run `tests/run.sh`.
- Every hook script has a row in [`catalog.tsv`](../catalog.tsv) (`harnesses` is `plugin` when it ships only with the Claude Code plugin, `all` when the installer's `hook` kind installs it as a git hook); `tests/run.sh` fails without one.

## commit-gate.sh

Gate on `git commit`: asks the user before an unreviewed commit, or one that changes code without touching docs. Why and how: [why-code-review.md](../docs/why-code-review.md), ADRs [0002](../docs/adr/0002-code-review-mechanism.md), [0003](../docs/adr/0003-doc-sync-at-commit-gate.md), [0004](../docs/adr/0004-configurable-docs-paths.md), [0013](../docs/adr/0013-commit-gate-as-git-hook.md).

**Two modes, one script.**

- *Plugin mode* (JSON on stdin): the Claude Code `PreToolUse` hook wired in `hooks.json`. Parses the Bash command with `parse-commit.py`; denies with JSON.
- *Git hook mode* (`commit-gate.sh --pre-commit`): run by a `pre-commit` hook, so it gates commits from every harness and from a terminal. Bash and git only. Checks the staged diff (git has already staged `git commit -a`), prints the reason on stderr prefixed `commit-gate:`, and exits 1. When the commit uses `-a`, the message names the `--all` stamp. `afk commit-gate` installs it ([ADR 0014](../docs/adr/0014-git-hook-layout.md)): a committed copy at `.agents/hooks/commit-gate.sh`, and a `pre-commit` shim in the clone, marked `# afk commit-gate`, that runs the copy and allows the commit when the copy is missing. An existing `pre-commit` hook is never changed; add `[ ! -f .agents/hooks/commit-gate.sh ] || bash .agents/hooks/commit-gate.sh --pre-commit || exit 1` to it instead, which also fails open without the copy.

Both modes check the same stamps in `.git/` and the same docs paths, and `--no-verify` skips both. **One gate at a time:** the plugin mode allows the commit when git's `pre-commit` hook (`git rev-parse --git-path hooks/pre-commit`, so `core.hooksPath` counts) is executable, names `agents/hooks/commit-gate.sh`, and that copy exists. The git hook then asks, so a user with both is asked once. The message names the skills and stamp command as installed: `the code-review skill` and `.agents/hooks/commit-gate.sh` when the project has those copies, `/afk:code-review` and the script's own path otherwise.

**What counts as docs.** By default every `*.md`. Override per repository with a committed `.commit-gate-docs` at the root: one git pathspec per line, relative to the root; blank lines and `#` comments ignored; long-form magic such as `:(glob)docs/**` allowed.

```
# .commit-gate-docs
docs/
README.md
*.rst
```

Everything outside these paths is code. The list decides whether a commit touched docs, whether the repo has docs at all (no match means the docs check is off), and which edits leave a review stamp valid. Changing the list invalidates existing docs stamps.

**Accepted ADRs are immutable** ([ADR 0015](../docs/adr/0015-accepted-adrs-guarded-at-commit.md)). The git hook mode also denies a commit that changes `docs/adr/NNNN-*.md` outside its `## Status` section when the `HEAD` version's Status starts with `Accepted`, and one that deletes or renames such an ADR. Status edits (`Superseded by NNNN`, notes, links), new ADRs, and Proposed or Rejected ADRs pass; carriage returns don't count. There is no stamp: the answer is a superseding ADR, or `--no-verify`. One denial lists every failing check. An ADR whose Status isn't a `## Status` heading followed by a line starting `Accepted` isn't recognised, and passes. Only the git hook checks ADRs; the plugin mode doesn't.

## adr-index.sh

`SessionStart` hook: prints the project's Accepted ADRs, one line each (`0014 The gate script is committed; … - docs/adr/0014-git-hook-layout.md`), which Claude Code adds to the session's context. Every session then knows which decisions hold without reading `docs/adr/`. Accepted is read as the commit gate's ADR check reads it ([ADR 0015](../docs/adr/0015-accepted-adrs-guarded-at-commit.md)), so partly superseded ADRs are listed and fully superseded ones aren't. The project is `$CLAUDE_PROJECT_DIR`, else the git root. Silent when there is no `docs/adr/`, no Accepted ADR, or an error. Plugin only (`harnesses` `plugin`); other harnesses rely on `AGENTS.md` pointing at `docs/adr/`, a line [`adr-planning`](../skills/adr-planning/SKILL.md) adds when it creates the folder and [`project-init`](../skills/project-init/SKILL.md) writes when the folder exists.

