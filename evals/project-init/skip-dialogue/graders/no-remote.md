---
type: regex
target: { source: file, path: .git/config }
pattern: '\[remote '
match: not_contains
---

The user stayed local: no remote was added.
