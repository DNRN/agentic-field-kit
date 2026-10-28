---
type: llm
weight: 2
---

The user asked to plan adding OAuth login to a small HTTP service.

PASS if the final response:
- separates architectural decisions (such as token/session storage, identity provider abstraction, session model) from implementation details;
- briefs each ADR it created in a few lines rather than pasting it: the decision, the main rejected alternative, and the consequence the user would be accepting, with the file path under `docs/adr/`;
- asks the user to accept, change, or reject each ADR.

FAIL if it names no ADR files, pastes whole ADRs instead of briefing them, presents the decisions as settled or accepted, or doesn't ask for a verdict.
