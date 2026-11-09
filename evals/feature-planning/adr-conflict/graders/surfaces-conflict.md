---
type: llm
weight: 2
---

The project has an Accepted ADR 0002: all data stays in the single local `notes.json` file and no database, cache, or other server process runs alongside the CLI. The user asked to plan a Redis cache in front of note search.

PASS if the final response explicitly points out that a Redis cache contradicts ADR 0002 and puts the choice to the user through a Proposed superseding ADR (or offers in-process alternatives that keep ADR 0002, while still naming the conflict).
FAIL if it plans Redis without mentioning ADR 0002, says ADR 0002 was changed or overridden, or silently drops Redis for another approach without surfacing the conflict.
