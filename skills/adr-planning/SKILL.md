---
name: adr-planning
description: Use when planning or defining a new feature and architectural decisions arise - record each significant decision as an Architecture Decision Record (ADR) before implementation starts. Skip for bug fixes, chores, and choices with no lasting structural impact.
---

# ADR planning

Architecture decisions made during feature planning get captured as [ADRs](https://adr.github.io/) - short, numbered, immutable records. A decision that is not written down gets re-litigated; one buried in a PR description gets lost.

## When this applies

- The user asks to plan, design, or scope a new feature, service, or migration.
- During planning, a choice appears that is hard to reverse, crosses module or service boundaries, constrains future work, or carries real tradeoffs: frameworks, data models, protocols, storage engines, sync vs async, monolith vs services.

Trivial choices - naming, formatting, single-file changes, library upgrades without alternatives - do not get ADRs.

## Procedure

1. **Surface the decisions** while planning. Test each candidate: would changing it later require coordinated edits in multiple places? Does it constrain what can be built next? No to both means it is not architectural - leave it as a plan note, not an ADR.
2. **Find the project's ADR home** before creating anything: look for `docs/adr/`, `doc/architecture/decisions/`, `adr/`, or grep for existing `NNNN-*.md` decision records. If the project already has ADRs, follow its numbering and section format - never impose a second convention.
3. **If none exists**, create `docs/adr/` and start with `0001-record-architecture-decisions.md` (the bootstrap record), using the template below.
4. **One ADR per decision.** Number sequentially with zero-padded 4 digits (`0002-kebab-title.md`); never renumber or reuse numbers.
5. **Write it** with these sections:

   ```markdown
   # NNNN. <Short noun-phrase title>

   Date: YYYY-MM-DD

   ## Status
   Proposed

   ## Context
   The forces at play: requirements, constraints, and the options on the table.

   ## Decision
   We will <decision in active voice>.

   ## Consequences
   What becomes easier, what becomes harder, which risks we now accept.
   ```

6. **Record rejected alternatives** with the reason each was rejected. This is the core value: it prevents re-debating settled questions six months later.
7. **Link both ways**: the feature plan references the ADR file; ADRs covering related decisions reference each other.
8. **Leave status `Proposed`** until the user confirms the decision; flip to `Accepted` only on confirmation. Accepted ADRs are immutable - a change of direction means a new ADR marked `Superseded by NNNN`, and the old one updated to `Superseded by <new NNNN>`.

## Gotchas

- One page max per ADR. If it grows into a design document, the design doc is a separate file the ADR links to.
- Never leave decisions only in chat, commit messages, or PR descriptions.
- Never rewrite an Accepted ADR - supersede it. The history is the point.
- If the user's request contradicts an existing Accepted ADR, surface the conflict and propose a superseding ADR instead of silently deviating.
