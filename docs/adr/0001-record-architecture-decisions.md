# 0001. Record architecture decisions

Date: 2026-10-22

## Status

Accepted

## Context

This repo ships behavioral components for coding agents across harnesses. Component-level design choices (skill vs agent vs command vs hook, gating mechanics, portability tradeoffs) constrain every later component and get re-debated unless written down.

## Decision

We will record each significant component design decision as an Architecture Decision Record in `docs/adr/`, numbered `NNNN-kebab-title.md`, with Status / Context / Decision / Consequences, and rejected alternatives recorded with reasons.

## Consequences

- Decisions are auditable and not re-litigated; contributors can point at a record instead of arguing from memory.
- Each significant new component now carries the cost of one short ADR.
