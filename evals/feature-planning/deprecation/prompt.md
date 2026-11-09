---
description: "The feature replaces a seeded CSV export with two callers: the plan's Deprecations names it with its callers and the user is asked remove or deprecate."
tags: [feature-planning]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Agent, TodoWrite, Bash, Edit, Write]
---

Plan a new export module: one `export(notes, fmt, path)` entry point with CSV and JSON formats, so we can add more formats later. There's no issue or doc for it; that's the whole description.
