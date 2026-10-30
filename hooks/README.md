# Hooks

Event-triggered scripts. Config lives in `hooks/hooks.json` (shape: `{"hooks": {<Event>: [...]}}`), scripts in `hooks/scripts/`.

Least portable category: hook event models and config formats differ per harness (Claude Code `hooks.json`, Cursor `hooks.json`, OpenCode JS plugins, Codex lifecycle config). Keep the scripts generic and self-contained; the wiring is per-harness.

## Format

- Declare matchers per event (`PreToolUse`, `PostToolUse`, `Stop`, …); keep hooks fast and silent on success.
- Reference bundled scripts via `"${CLAUDE_PLUGIN_ROOT}"/hooks/scripts/<script>.sh` — never absolute local paths.
- Hooks must be idempotent and safe when the tool state is unexpected; they run on every matching event.
- Every hook ships with regression tests in `tests/<script>.test.sh` (JSON on stdin, decision on stdout); run `tests/run.sh`.
- Every hook script has a row in [`catalog.tsv`](../catalog.tsv) (`harnesses` is `plugin` while it ships only with the Claude Code plugin); `tests/run.sh` fails without one.

## commit-gate.sh

Gate on `git commit`: asks the user before an unreviewed commit, or one that changes code without touching docs. Why and how: [why-code-review.md](../docs/why-code-review.md), ADRs [0002](../docs/adr/0002-code-review-mechanism.md), [0003](../docs/adr/0003-doc-sync-at-commit-gate.md), [0004](../docs/adr/0004-configurable-docs-paths.md), [0013](../docs/adr/0013-commit-gate-as-git-hook.md).

**Two modes, one script.**

- *Plugin mode* (JSON on stdin): the Claude Code `PreToolUse` hook wired in `hooks.json`. Parses the Bash command with `parse-commit.py`; denies with JSON.
- *Git hook mode* (`commit-gate.sh --pre-commit`): run by a `pre-commit` hook, so it gates commits from every harness and from a terminal. Bash and git only. Checks the staged diff (git has already staged `git commit -a`), prints the reason on stderr prefixed `commit-gate:`, and exits 1. When the commit uses `-a`, the message names the `--all` stamp. The installer's `hook` kind will write the hook ([ADR 0014](../docs/adr/0014-git-hook-layout.md)); until then, enable it by hand with a `.git/hooks/pre-commit` that runs `exec bash <path>/commit-gate.sh --pre-commit`.

Both modes check the same stamps in `.git/` and the same docs paths, and `--no-verify` skips both. The message names the skills and stamp command as installed: `the code-review skill` and `.agents/hooks/commit-gate.sh` when the project has those copies, `/agentic-toolkit:code-review` and the script's own path otherwise.

**What counts as docs.** By default every `*.md`. Override per repository with a committed `.commit-gate-docs` at the root: one git pathspec per line, relative to the root; blank lines and `#` comments ignored; long-form magic such as `:(glob)docs/**` allowed.

```
# .commit-gate-docs
docs/
README.md
*.rst
```

Everything outside these paths is code. The list decides whether a commit touched docs, whether the repo has docs at all (no match means the docs check is off), and which edits leave a review stamp valid. Changing the list invalidates existing docs stamps.
