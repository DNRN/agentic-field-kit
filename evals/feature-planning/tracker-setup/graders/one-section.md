---
type: regex
target: { source: file, path: AGENTS.md }
pattern: '^## Feature planning\s*$'
flags: m
match: "count:1"
---

The settings live in one `## Feature planning` section.
