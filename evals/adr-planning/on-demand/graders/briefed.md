---
type: llm
---

The user asked for an ADR recording that the project uses Postgres; the project has ADRs 0001 and 0002.

PASS if the final response briefs the new ADR 0003 in a few lines (the decision; an alternative and why it lost, or that the reason isn't recorded; a consequence) and asks the user to accept, change, or reject it.
FAIL if it pastes the whole ADR, marks it Accepted without asking, or invents a specific reason for choosing Postgres that the user and the code don't give.
