---
name: code-review
description: Use when code changes are ready to commit, when the user asks for a review or second opinion on a diff or branch, or when a git commit was denied by a review gate - fresh-eyes review of the diff producing severity-ranked findings before committing.
---

# Code review

Review is a separate act from writing. The author's context already believes the code is correct; a review run from the author's memory reviews nothing.

## When this applies

- Work is complete and about to be committed: **offer review first**. Never commit a non-trivial change unreviewed unless the user explicitly declines.
- The user asks for a review, sanity check, or second opinion on a diff, branch, or PR.
- A PreToolUse review gate denied a `git commit` as unreviewed.

Skip for whitespace/formatting-only diffs, lockfiles, and generated/vendored files.

## Procedure

1. **Fix the scope.** Default: staged diff (`git diff --cached`); for `git commit -a`, the worktree diff (`git diff HEAD`). Otherwise branch vs its base (`git merge-base <base> HEAD`), or the ref the user named. State the scope in the report; exclude vendored and generated paths.
2. **Get fresh eyes.** If the harness supports subagents, delegate to a reviewer subagent (`agents/code-reviewer.md` where bundled), passing only the diff command and repo path — never your reasoning about the change. Without subagents: re-read every changed hunk from the diff itself before evaluating it, deliberately checking it against (not through) your intent.
3. **Evaluate each changed hunk:**
   - Correctness: edge cases, error paths, off-by-one/boundaries, empty/nil states, concurrency.
   - Security: injection, secrets, authorization on new endpoints, unsafe deserialization, path traversal.
   - Tests: does a test exercise the changed behavior and fail on its regression? Changed behavior without a covering test is a finding.
   - Contracts: public API/schema/config changes — are all callers updated?
   - Dead weight: unreachable code, scope creep beyond the stated task, code made obsolete by the change.
   - Consistency: project conventions and Accepted ADRs (`docs/adr/`) — contradicting an Accepted ADR is a finding.
4. **Verify every finding against the actual source** before reporting: cite `file:line`, quote the offending line, state the concrete failure mode. No failure mode, no finding — style preference is not a finding unless it masks a bug.
5. **Report:**

   | Severity | Location | Finding | Fix |
   | --- | --- | --- | --- |

   Severity: `blocker` = must fix before commit; `major` = fix or explicit user waiver; `minor` = follow-up. End with a verdict: **approve** or **fix then commit**.
6. **Resolve.** Fix blockers and majors, then re-review the hunks the fixes touched. Findings are reported, not silently rewritten — the author applies fixes; the reviewer stays read-only.
7. **Unblock the commit.** If the commit-gate hook denied the commit, stamp the reviewed diff by running the stamp command named in the hook's denial message (`commit-gate.sh --stamp-review`). Only the user can waive review — if they declined, commit with `--no-verify` instead. Never stamp a diff that was not reviewed.

## Gotchas

- Never review from memory of writing the code — only from the diff.
- Diff over ~2000 lines: review in file-group passes, one report per pass.
- The stamp records a diff hash, not permanent approval — any new staged change re-triggers the gate. That is by design.
- The gate is a nudge, not enforcement (`--no-verify` exists). Do not bypass it silently.
