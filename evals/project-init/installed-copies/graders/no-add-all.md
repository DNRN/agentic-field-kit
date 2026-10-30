---
type: tool_used
tool: Bash
input_match: 'git\b[^"]*\badd\b\s+(-A|--all|\.(\s|"|$)|\.claude/?(\s|"|$))'
min: 0
max: 0
---

Never git add -A, git add ., or all of .claude/.
