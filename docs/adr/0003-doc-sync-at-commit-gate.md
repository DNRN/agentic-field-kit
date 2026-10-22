# 0003. Documentation kept in sync at the commit gate

Date: 2026-10-22

## Status

Accepted. Decision item 4's docs-check trigger rule (what counts as docs) superseded by [0004](0004-configurable-docs-paths.md).

## Context

Agents produce code faster than humans update prose, so documentation rots silently: stale commands, renamed flags still documented, diagrams that no longer match the code, architecture described per a superseded ADR. Requirement: every commit leaves documentation correct; earlier docs are updated when features change; docs are human-readable markdown with mermaid diagrams where structure warrants one; docs must correlate with ADRs.

Constraints inherited from ADR 0002: skills are the only portable layer; fresh-eyes judgment needs a subagent; model-remembered rules do not fire reliably; a second hook on the same event would produce two separate asks per commit.

## Decision

We will mirror the code-review mechanism and share its gate:

1. **`skills/doc-sync/`** — the portable procedure: map the diff to affected docs, update/create/delete them, embed one mermaid diagram per structural concept, keep docs correlated with ADRs, stamp the docs gate.
2. **`agents/doc-auditor.md`** — fresh-eyes read-only subagent: given the diff and candidate docs, reports factual drift, stale diagrams, broken doc↔ADR links, superseded decisions presented as current, and missing docs for user-visible changes. Proposes replacement passages; never edits.
3. **`commands/doc-sync.md`** — explicit user entry point.
4. **Generalize the gate**: `review-gate.sh` becomes `commit-gate.sh` with two independent checks and one combined denial. The docs check fires when the staged set changes code (non-`.md`) without touching any `.md` in a repo that tracks at least one `.md`, and no docs stamp matches. Editing docs in the same commit is accepted as evidence docs were considered. Stamps are independent (`--stamp-review` / `--stamp-docs`), so either check can be completed or waived separately; the review stamp also records the code-only diff hash so doc edits after a review do not force re-review.

## Consequences

- One ask per commit covering both checks; waiving either is a conscious `--no-verify` or an untamped re-run.
- A code change committed together with any doc edit always passes the docs check — the gate detects forgetting, not inaccuracy; accuracy is the auditor's job.
- Doc-only commits are still review-gated (review covers the whole diff, including prose correctness).
- Gate remains Claude Code-only; other harnesses get the model-driven rule from the skill.

## Rejected alternatives

- **Second hook script**: two denials and possibly two asks for one commit; also duplicated plumbing. One gate, two checks is simpler operationally.
- **Auto-regenerate docs wholesale on commit**: unreviewable churn, destroys hand-written prose, and no generator knows project voice; surgical updates win.
- **CI-only doc checks**: outside the agent loop; drift found after merge is expensive to fix.
- **Auditor edits docs directly**: the author owns intent; proposals keep the judgment loop visible and re-checkable (same reasoning as the read-only code reviewer).
- **Reference-only triggering** (fire only when a doc mentions a changed file): misses new features with no docs yet and README-level accuracy; fires too rarely for "every commit leaves docs correct".
