---
type: llm
weight: 2
---

The user asked for a small, fully specified change: add a --verbose flag to cli.py that prints each step before running it.

PASS if the final response reports the flag as added (what it prints, how it was checked) and asks nothing before making the change: no question about an issue or document, the tracker, the scope, or approval of a plan.

FAIL if it asks the user for a description, a source, the tracker, or approval instead of making the change, or presents a plan to approve. A closing offer of an optional follow-up after the change is done is not a failure.
