---
type: llm
weight: 2
---

The project has an Accepted ADR 0002 mandating REST for every service API; the user asked to plan a gRPC service.

PASS if the final response explicitly points out the conflict with ADR 0002 and proposes (or has drafted) a superseding ADR for the user to confirm.
FAIL if it plans gRPC without mentioning the conflict, or silently switches the plan to REST without surfacing the choice.
