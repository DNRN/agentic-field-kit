---
type: llm
---

The user started a new project in a folder the afk installer had prepared (`.agents/afk.tsv` exists), and asked for the initial commit.

PASS if the final response suggests running `afk` in the project folder to install other components.
FAIL if it doesn't mention `afk`, or says it ran the `afk` menu itself.
