---
description: "Feature planning in a repo without ADRs: bootstrap docs/adr/, record each architectural decision as a Proposed ADR, reference them from the plan."
tags: [adr-planning]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Agent, TodoWrite, Bash, Edit, Write]
---

We need to add OAuth login to the service: people sign in with their Google account in the browser, and we keep their sessions server-side in a SQLite table. Plan it out. There is no issue or spec for it; this is the whole description.
