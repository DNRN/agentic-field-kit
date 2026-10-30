---
type: llm
---

The user asked to ship a staged change adding a `--verbose` flag to `greet.py`, with a test; the README did not mention the flag.

PASS if the final response reports a commit (its subject or hash), says the review approved the change (or found nothing blocking), and says the README was updated for `--verbose`.
FAIL if it stopped before committing without a blocking reason, asked the user to confirm the commit, or doesn't mention the docs.
