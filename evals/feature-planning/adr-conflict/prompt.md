---
description: "The request contradicts an Accepted ADR: the conflict is raised and handed to adr-planning as a Proposed superseding ADR; the Accepted one is unchanged."
tags: [feature-planning]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Agent, TodoWrite, Bash, Edit, Write]
---

Plan a Redis cache in front of note search so search stays fast as notes grow. The team has decided on Redis: a Redis server runs next to the CLI and caches search results, and notes.json stays the source of truth. There's no issue or doc for it; that's the whole description.
