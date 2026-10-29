---
description: "Empty directory; the user gives the whole brief up front and asks for no further questions. PROJECT.md must record the stated facts and invent no stack."
tags: [project-init]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Agent, TodoWrite, Bash, Edit, Write]
---

Initialize a new project here, with its own git repository. It's Shelfie: a self-hosted web app for my family to track the board games we own and who borrowed which. The first users are the four of us; friends maybe later. First milestone: add games, mark a game as lent to someone, and see what's lent out. Out of scope: a mobile app and tracking prices. Stack: Python with Django and SQLite, on our home server. It's done when we can lend and return a game from a phone browser. I have no remote, keep it local. That's everything I know, so write PROJECT.md from this without asking more questions, and don't commit.
