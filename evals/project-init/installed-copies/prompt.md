---
description: "A folder the installer prepared (project-init copy in .agents/skills, manifest) plus a personal .claude/settings.local.json. Start the project, local, no questions, and commit: the commit stages the skill's files and what the manifest records, never the personal file or git add -A."
tags: [project-init]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Agent, TodoWrite, Bash, Edit, Write]
---

Start a new project in this folder with the project-init skill. Its own git repository, keep it local, skip the project questions, and make the initial commit. My git name is "Eval User", email eval@example.com.
