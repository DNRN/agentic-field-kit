---
type: llm
---

The staged diff only renamed local variables inside `mean()`; behavior is identical.

PASS if the final response says the docs need no change because nothing observable/user-visible changed.
FAIL if it proposes or reports doc edits, or does not say why no change is needed.
