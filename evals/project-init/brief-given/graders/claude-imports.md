---
type: regex
target: { source: file, path: CLAUDE.md }
pattern: '(^|\n)@AGENTS\.md'
weight: 2
---

CLAUDE.md imports AGENTS.md (an @AGENTS.md line), so Claude Code loads it.
