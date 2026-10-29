---
description: "Empty directory; the user stays local and skips the dialogue. Expect git, AGENTS.md, CLAUDE.md importing it, and PROJECT.md. The run's home directory is itself a git repository, so the prompt asks for a repository of its own."
tags: [project-init]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Agent, TodoWrite, Bash, Edit, Write]
---

Initialize a new project in this directory, with its own git repository. I don't have a remote repository, keep it local. Skip the project questions, I'll fill in PROJECT.md later. Don't commit.
