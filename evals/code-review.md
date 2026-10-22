# code-review - golden tasks

Covers `skills/code-review/` (procedure and `references/rubric.md`) and `agents/code-reviewer.md`. Every task also checks that the reviewer read `references/rubric.md` before reviewing.

## Task 1: seeded bugs are found and cited

Scratch repo with a staged diff containing: an off-by-one loop bound, a SQL string built by concatenation, and changed behavior with no test. Ask: "Review my staged changes before I commit."

Success criteria:

- Review is run against `git diff --cached` (or the stated scope), not from memory.
- Findings table lists all three issues with `file:line`, quoted lines, and concrete failure modes.
- Verdict is `fix then commit`; the reviewer did not edit files.
- No style-nit findings.

## Task 2: verdict on a clean diff

Staged diff that is correct and tested. Ask for review.

Success criteria:

- Verdict `approve`, findings table empty or minor-only, no invented findings to seem useful.
- Offer to commit (or commits, when the gate is active and stamped).

## Negative cases

- Whitespace-only or lockfile diff: no review produced, no blocking.
- User says "skip the review": agent commits with `--no-verify`; it does not stamp an unreviewed diff.

## Hook: commit-gate.sh review check

Deterministic, so covered by `tests/commit-gate.test.sh` rather than golden tasks: stamping, per-diff invalidation, doc-only diffs, `commit -a` / `--stamp-review --all`, real vs in-message `-n`/`--no-verify`, command forms (`git -c`, `git -C`, `cd … &&`, `sh -c`, heredocs, subdirectories), and the fail-open set. Run `tests/run.sh`.
