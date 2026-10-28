---
type: llm
weight: 3
---

The task was a review of a staged diff adding a correct `clamp()` function with tests covering inside, below, above, inclusive bounds, and inverted bounds.

PASS if the final response's verdict lets the commit proceed (approve, or "fine to commit") and it reports no blocker or major finding. Minor findings and optional suggestions (for example NaN handling) are allowed and do not change the verdict; asking whether to commit is fine.
FAIL if the verdict withholds the commit, or the response reports a blocker or major finding, or it claims a defect the diff does not have (such as a missing test for clamp's bounds).
