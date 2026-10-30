---
type: regex
target: { source: file, path: .git/logs/HEAD }
pattern: '\scommit: '
---

A commit landed on top of the initial one (HEAD reflog).
