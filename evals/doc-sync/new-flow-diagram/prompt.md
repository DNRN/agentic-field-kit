---
description: "Staged diff adds a submit -> poll -> fetch job flow with no doc; expect one new doc under docs/ with exactly one mermaid diagram."
tags: [doc-sync]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Agent, TodoWrite, Bash, Edit, Write]
---

Sync the docs for my staged changes.
