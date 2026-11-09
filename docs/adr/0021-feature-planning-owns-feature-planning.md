# 0021. feature-planning plans features; adr-planning records their decisions

Date: 2026-11-09

## Status

Accepted

Decided in the planning dialogue recorded in [the feature-planning plan](../plans/feature-planning.md#refinement-log). Related: [ADR 0005](0005-rubrics-live-in-skills.md) (what a skill and its subagent share lives in the skill).

## Context

A feature built from a one-line request or half an issue gets the gaps filled with guesses: the agent doesn't read the issue's comments, doesn't check what the change breaks or makes obsolete, and asks nothing until the code is written. The toolkit has `adr-planning`, which triggers on "planning or defining a new feature", but it records architectural decisions. It does not gather the description, map the impact, or question the user in rounds.

A new `feature-planning` skill does that work. Three questions have to be settled before it can be built: how it relates to `adr-planning`, which triggers on the same requests; where the plan lives; and where a project keeps what the skill has to remember between sessions (which issue tracker, if any, and which model the scout subagent uses).

## Decision

We will make `feature-planning` the skill for planning, defining, implementing, or adding a feature, and for bug reports that come from an issue or have an unknown cause. It hands each architectural decision to `adr-planning`. `adr-planning`'s description narrows to those decisions: it no longer claims the planning itself.

- **Size gate.** Small, fully specified changes (one place in the code, the behavior stated, nothing to read first) and plain failing tests with a clear cause skip `feature-planning` and go straight to the change.
- **The plan** is a committed file, `docs/plans/<slug>.md`, refined in place each round, with its status going Draft → Ready → Done. It stays in the repository after the feature ships.
- **Project settings** live in a `## Feature planning` section of `AGENTS.md`, or of `CLAUDE.md` when there is no `AGENTS.md`: the issue tracker and how to reach it (or `none`), and the scout model. Each is asked once and recorded.
- **The impact checklist** lives in the skill's `references/`; the `feature-scout` agent is a thin read-only wrapper that reads it (ADR 0005).

## Consequences

- One skill owns "plan this feature"; the two skills no longer give the agent competing instructions for the same request.
- `adr-planning` still fires on its own when a decision comes up outside a planning flow, or when asked to record or supersede one.
- Every feature leaves a plan in `docs/plans/`. Reviewable in the PR, and `doc-sync` keeps it true; the folder grows with each feature.
- The `## Feature planning` section of `AGENTS.md` loads into every session in every harness: three lines of context, which buys settings that teammates can read and edit.
- The size gate is a judgment call by the model. Two negative evals pin it: a fully specified flag (`small-change`) and a failing test (`adr-planning`'s `bug-fix`).

## Rejected alternatives

- **Extend `adr-planning` into the full flow**: one skill mixing "what to build" with "how to decide", too big to stay reliable.
- **Two independent skills with overlapping triggers**: both load on the same request and contradict each other.
- **Plans outside the repository (chat or the issue only)**: lost after the session and invisible in review.
- **Settings in their own file (like `.commit-gate-docs`)**: no context cost, but another file to know about. In `PROJECT.md`: that file's format belongs to `project-init`.
- **Always running, compressed for small tasks**: an extra turn for every small change.
