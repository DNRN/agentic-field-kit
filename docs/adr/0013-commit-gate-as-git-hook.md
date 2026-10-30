# 0013. The commit gate also runs as a git `pre-commit` hook

Date: 2026-10-30

## Status

Accepted

Plan: [roadmap phase 1](../plans/phase1-enforce-themes.md). Supersedes the "Claude Code gets a deterministic gate; other harnesses get the model-driven rule" consequence of [ADR 0002](0002-code-review-mechanism.md). Related: [ADR 0003](0003-doc-sync-at-commit-gate.md), [ADR 0004](0004-configurable-docs-paths.md), [ADR 0014](0014-git-hook-layout.md).

## Context

`commit-gate.sh` is a Claude Code `PreToolUse` hook. Codex, Cursor, OpenCode, and Gemini CLI users get only the skill's "offer review before committing" rule, which the model remembers some of the time. Every harness commits by running `git commit`, so a gate that git runs fires everywhere.

- A `pre-commit` hook sees exactly what is being committed: under `git commit -a` git stages into a temporary index first, and `git diff --cached` inside the hook equals `git diff HEAD` before it (checked 2026-10-30). The existing `--stamp-review --all` stamps therefore match without change.
- Git runs the hook only for commits, so the command parsing in `parse-commit.py` (chains, `sh -c`, `git -C`) is not needed: bash and git only. Git for Windows ships bash.
- A hook cannot return structured JSON. It writes to stderr and exits non-zero; harnesses show a failed command's output to the agent, and a person at a terminal reads it too.
- `--no-verify` already skips `pre-commit`, so it keeps its meaning as the waiver.

## Decision

We will add a `--pre-commit` mode to `hooks/scripts/commit-gate.sh` and keep one script for both triggers. The mode always checks the staged diff, reuses `load_docs`, `diff_hashes`, and the stamps in `.git/`, prints the pending checks and the instruction to ask the user on stderr, and exits 1. Any unexpected state exits 0 (fail open), as in the plugin mode. The plugin's `PreToolUse` mode is unchanged.

## Consequences

- The gate fires in every harness and in a plain terminal commit. A person committing by hand is also asked; `--no-verify` is the answer when they don't want the checks.
- One script, two entry points: a fix to the checks applies to both. The tests in `tests/commit-gate.test.sh` gain a `--pre-commit` section.
- The gate needs a per-clone step to run at all, because git never enables hooks from the repository ([ADR 0014](0014-git-hook-layout.md)).
- Still a nudge, not enforcement: anyone can skip it, and CI does not check stamps.

## Rejected alternatives

- **Per-harness hook configs (Cursor `hooks.json`, OpenCode plugins, Codex lifecycle)**: four formats to write and test, and harnesses without hooks get nothing.
- **A separate `pre-commit` script that copies the checks**: two copies of the hash rules drift, and a drift breaks stamps silently.
- **Server-side or CI check of stamps**: stamps live in `.git/` and are never pushed; too late to fix cheaply (as in ADR 0002).
- **The `pre-commit` framework (Python)**: adds a dependency and a config file most projects using this toolkit don't have.
