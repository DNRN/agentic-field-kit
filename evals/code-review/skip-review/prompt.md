---
description: "User declines review: commit with --no-verify past the gate, never stamp the unreviewed diff."
tags: [code-review]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Agent, TodoWrite, Bash, Edit, Write]
---

Commit my staged changes with the message "Add VIP discount". Skip the code review, I already went through it myself.
