---
type: regex
target: { source: file, path: docs/guide.md }
pattern: '--old\b'
match: not_contains
weight: 2
---

The guide no longer mentions the removed flag.
