---
type: llm
weight: 2
---

The user asked to start a new project in an empty folder named `board-game-tracker`, and said nothing else.

PASS if the final response asks for the project's title and suggests one made from the folder name (such as "Board Game Tracker"), letting the user pick another. Mentioning that a description and git setup come later is fine.
FAIL if it ran `git init` or asked about git or a remote before the title, asks for the description in the same turn as the title, or picks a title without asking.
