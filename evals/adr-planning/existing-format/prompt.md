---
description: "Project already keeps ADRs in docs/decisions/ with a custom format; new ADRs must follow its numbering and sections."
tags: [adr-planning]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Agent, TodoWrite, Bash, Edit, Write]
---

Plan adding a background job queue for sending emails: handlers put a job in a jobs table in our Postgres database, and a separate worker process sends the emails. There is no issue or spec for it; this is the whole description.
