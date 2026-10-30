---
type: regex
target: { source: file, path: .git/logs/HEAD }
pattern: '\scommit: '
match: not_contains
---

Nothing was committed on top of the initial commit.
