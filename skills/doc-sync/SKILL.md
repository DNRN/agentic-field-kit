---
name: doc-sync
description: Use when code changes are about to be committed and documentation may need updating, when the user asks to write or update docs, or when a git commit was denied by the documentation check - keeps human-readable markdown docs and their mermaid diagrams accurate, current, and correlated with ADRs.
---

# Doc sync

Documentation rots silently: the flag gets renamed, the doc keeps the old name, tests stay green. Every commit must leave documentation correct — updated, not appended. Stale docs are worse than no docs; readers trust them.

## When this applies

- Code changes are complete and about to be committed: **offer a doc check first** when the commit changes code without touching any doc.
- The user asks to write, update, or check documentation.
- A PreToolUse commit gate denied the commit with "documentation" pending.

Skip for changes with no observable surface: internal refactors with identical behavior, test-only changes, formatting.

## Procedure

1. **Summarize the change for a human.** From the diff, state what now behaves differently: renamed flags/commands/endpoints, new or removed features, changed defaults, new flows. No observable change → no doc change; say so.
2. **Find affected docs.** Docs live where the project keeps them (`docs/`, `doc/`, `README*.md`, root `*.md` — follow the existing convention, never create a second one). Candidates:
   - Docs referencing changed files, paths, commands, flags, or symbols (grep the docs tree).
   - Docs describing the changed feature, even without naming it.
   - `README.md` claims about setup, usage, structure.
3. **Get fresh eyes.** If the harness supports subagents, delegate to the doc-auditor (`agents/doc-auditor.md`) with the diff and candidate docs — it reports drift and proposes replacements; the author applies them. Without subagents: audit each candidate yourself against the diff, claim by claim.
4. **Update surgically.** Edit the stale passages; never regenerate whole files. Delete docs for removed features. Add docs for new user-visible features in the project's existing voice and structure; if the project has no docs yet, start `docs/` with a short index that links the README.
5. **Diagrams.** A feature with flow, sequence, state, or dependency structure gets exactly one mermaid diagram embedded in its doc, drawn from the actual code paths. Update the diagram when the structure changes; delete it when the structure collapses. No decorative diagrams, no second diagram of the same concept. Validate syntax if `mmdc` is available; keep diagrams simple enough to verify by reading.
6. **Correlate with ADRs.** Find the project's ADR home (`docs/adr/` or the convention in use).
   - A doc making an architectural claim links the governing ADR.
   - Never describe a superseded design as current — update the prose and link the superseding ADR.
   - The change implements or contradicts an Accepted ADR → surface it; a genuine direction change goes through the adr-planning skill first, then the docs.
   - A new architectural decision appeared during doc writing → stop, adr-planning skill, then continue.
7. **Report** per doc: updated / created / deleted / unchanged (and why). Apply the edits — doc-sync's output is applied documentation, not just findings.
8. **Unblock the commit.** If the commit gate denied with "documentation" pending, stamp with the command from its denial message (`commit-gate.sh --stamp-docs`). Only the user can waive the check — if they declined, commit with `--no-verify`. Never stamp a code diff whose docs were not actually checked.

## Gotchas

- Docs are for humans: what it does, how to use it, how it fits together — not a restatement of the code.
- A commit that already edits docs passes the docs gate by design; the gate detects forgetting, the auditor detects inaccuracy.
- Never touch docs unrelated to the change; never reorganize the docs tree while syncing.
- One pass, one report — don't leave half-updated docs because the diff was large; split by doc instead.
