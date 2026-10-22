---
description: Fresh-eyes code review of staged changes, a branch, or a ref before committing
argument-hint: "[target: staged | worktree | <branch> | <ref..ref> — defaults to staged diff]"
allowed-tools: Task, Bash(git diff:*), Bash(git log:*), Bash(git merge-base:*), Read, Grep, Glob
---

# Code review

Review the requested changes with a fresh-eyes reviewer, then report findings and a verdict.

1. **Determine scope.** `$ARGUMENTS` is `staged`/empty → `git diff --cached`; `worktree` (the gate's scope for `git commit -a`) → `git diff HEAD`; a branch → `git diff $(git merge-base <branch> <default>)..HEAD` (find the default branch from `git remote show origin` or the repo's layout); two refs → `git diff <ref>..<ref>`.
2. **Delegate to the `code-reviewer` subagent** via Task, passing only the diff command and the repo path — not any reasoning about the change. The reviewer is read-only; expect a findings table and a verdict.
3. **Report** the findings table and verdict to the user verbatim, then:
   - `approve` → offer to commit; if the review-gate hook denied the earlier commit, stamp with the command from its denial message.
   - `fix then commit` → fix the blockers/majors, re-review just the hunks the fixes touched (re-delegate), then commit.
4. If no subagent capability is available, run the review yourself following `skills/code-review/SKILL.md` — re-reading every hunk from the diff, not from memory.
