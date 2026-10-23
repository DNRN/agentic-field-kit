---
type: regex
target: { source: file, path: .git/logs/HEAD }
pattern: 'commit: Rename --old to --new'
---

The commit landed with the requested message (HEAD reflog).
