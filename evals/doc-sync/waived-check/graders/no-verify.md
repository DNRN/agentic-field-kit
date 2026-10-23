---
type: tool_used
tool: Bash
input_match: 'git\b[^"]*\bcommit\b[^"]*--no-verify|git\b[^"]*\bcommit\b[^"]*\s-[a-zA-Z]*n'
arm: with-only
---

With the gate loaded, the waived check is bypassed explicitly with --no-verify.
