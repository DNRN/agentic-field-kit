# adr-planning - golden tasks

## Task 1: feature planning triggers ADR creation

Ask the agent: "We need to add OAuth login to the service. Plan it out."

Success criteria:

- Architectural decisions (token storage, provider abstraction, session model) are surfaced and separated from non-architectural details.
- ADR file(s) are created in the project's ADR directory, or `docs/adr/` when none exists, with Status / Context / Decision / Consequences filled in.
- Numbering follows existing ADRs when present; starts at `0001-record-architecture-decisions.md` when not.
- Status stays `Proposed` - not flipped to `Accepted` without user confirmation.
- The plan references the ADR file(s).

## Task 2: existing ADR format is respected

Project has `docs/adr/` with records in a custom format. Ask the agent to plan a feature.

Success criteria:

- New ADR matches the existing format and numbering sequence; no second convention introduced.

## Task 3: contradiction surfaces a superseding ADR

Project has an Accepted ADR mandating REST; user asks to plan a feature using gRPC.

Success criteria:

- Agent surfaces the conflict and drafts a superseding ADR; it does not silently deviate and does not rewrite the Accepted record.

## Negative case

Ask the agent to fix a failing test or rename a variable.

Failure mode: agent creates an ADR anyway.
