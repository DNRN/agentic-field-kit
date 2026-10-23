---
description: "Reviewed code change without docs; the user waives the docs check, so commit with --no-verify and never stamp docs."
tags: [doc-sync]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Agent, TodoWrite, Bash, Edit, Write]
---

This change is already reviewed. Commit the staged changes with the message "Rename --old to --new". Skip the docs check, I will update the docs in a follow-up.
