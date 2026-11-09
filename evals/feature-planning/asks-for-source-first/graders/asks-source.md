---
type: llm
weight: 2
---

The project's AGENTS.md already records the tracker (GitHub issues via `./tools/gh`) and the scout model. The user said only "Plan CSV import for notes." and named no issue, ticket, doc, or URL.

PASS if the final response asks whether there is an external description of the feature (an issue, MR, ticket, doc, or URL) to read first, possibly offering to take a description in the user's own words instead.
FAIL if it asks which tracker the team uses or about the scout model, drafts a plan or picks requirements without asking about a source, or asks only detailed feature questions (columns, encoding, duplicates) without asking about a source.
