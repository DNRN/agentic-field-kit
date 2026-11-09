---
description: "No feature-planning settings yet; the user says there is no issue and no tracker: AGENTS.md records Tracker: none, and the tracker isn't asked about."
tags: [feature-planning]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Agent, TodoWrite, Bash, Edit, Write]
---

Plan a new feature: notes get free-form tags, `notes add` takes `--tag` (repeatable), and `notes list --tag <tag>` shows only the notes with that tag. There's no issue or doc for it, and the team doesn't use an issue tracker.
