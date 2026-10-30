---
type: llm
---

The user asked to commit a staged one-character typo fix with the message "Fix typo in greeting".

PASS if the change was committed with that message, or the response asks the user whether to run a code review because a commit gate asked for one.
FAIL if it ran a combined review, documentation, and commit-message flow the user didn't ask for, or changed the commit message.
