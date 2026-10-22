---
description: Check and update documentation against a diff so every commit leaves docs accurate and ADR-correlated
argument-hint: "[target: staged | worktree | <branch> | <ref..ref> — defaults to staged diff]"
allowed-tools: Task, Bash(git diff:*), Bash(git log:*), Bash(git merge-base:*), Read, Grep, Glob, Write, Edit
---

# Doc sync

Bring documentation in line with the requested changes, then report.

1. **Determine scope.** `$ARGUMENTS` is `staged`/empty → `git diff --cached`; `worktree` (the gate's scope for `git commit -a`) → `git diff HEAD`; a branch → `git diff $(git merge-base <branch> <default>)..HEAD`; two refs → `git diff <ref>..<ref>`.
2. **Collect candidate docs**: grep the project's docs (README, `docs/`, root `*.md`) for references to changed files, commands, flags, or symbols, plus any doc describing the changed feature.
3. **Delegate to the `doc-auditor` subagent** via Task, passing the diff command, repo path, and candidate list — not your reasoning. Expect a findings table with proposed fixes and a verdict.
4. **Apply** the accepted fixes with Edit/Write (surgical edits; one mermaid diagram per structural concept; ADR links per the doc-sync skill). New architectural decision surfaced → stop and run the adr-planning skill first.
5. **Report** per doc: updated / created / deleted / unchanged (why), then offer to commit. If the commit gate denied with "documentation" pending, stamp with the command from its denial message. If the user waived the check, commit with `--no-verify`.
6. If no subagent capability is available, run the audit yourself following `skills/doc-sync/SKILL.md` — claim by claim against the diff, never from memory.
