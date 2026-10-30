# 0015. Commits that change an Accepted ADR outside its Status are denied

Date: 2026-10-30

## Status

Accepted

Plan: [roadmap phase 1](../plans/phase1-enforce-themes.md). Related: [ADR 0001](0001-record-architecture-decisions.md), [ADR 0013](0013-commit-gate-as-git-hook.md) (gate as git hook).

## Context

The `adr-planning` skill says Accepted ADRs are immutable: a change of direction is a new ADR, and the old one gets `Superseded by NNNN`. Nothing checks it, and an agent asked to "update the decision" edits the file in place.

This repository's own Accepted ADRs show what legitimately changes after acceptance: the Status section (`Superseded by …`, partial supersession notes, links to the plan). Nothing else changes.

## Decision

We will add an ADR check to the `--pre-commit` mode of the commit gate ([ADR 0013](0013-commit-gate-as-git-hook.md)). For each staged file matching `docs/adr/[0-9][0-9][0-9][0-9]-*.md` whose `HEAD` version has a Status section starting with `Accepted`, the check compares the two versions with the `## Status` section removed. Any difference denies the commit with the file name and the instruction to write a superseding ADR. Deleting or renaming an Accepted ADR is denied too. Proposed and Rejected ADRs, new files, and other paths pass. `--no-verify` bypasses it; unexpected state fails open.

The plugin may add the same check as a `PreToolUse` hook on Edit/Write for earlier feedback in Claude Code; the git check is the one that holds in every harness.

## Consequences

- The skill's immutability rule is enforced where it matters, in every harness.
- Typos in an Accepted ADR need `--no-verify`. We accept it: the waiver is explicit and rare.
- The ADR location is fixed to `docs/adr/` for now. Projects with another home (`doc/architecture/decisions/`) get no check until it reads a configured path, as [ADR 0004](0004-configurable-docs-paths.md) does for docs.

## Rejected alternatives

- **Deny any change to an Accepted ADR**: blocks the supersede step itself.
- **Allow only a `Superseded by NNNN` line**: this repository's own records add partial-supersession notes and plan links to the Status section.
- **Edit/Write hook only**: Claude Code only, and edits made outside the agent pass.
