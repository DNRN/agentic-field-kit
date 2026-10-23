---
description: "Whitespace-only staged diff: commit it without a review and without blocking."
tags: [code-review]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Agent, TodoWrite, Bash, Edit, Write]
---

I only fixed whitespace. Commit the staged changes with the message "Tidy whitespace".
