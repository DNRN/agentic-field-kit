---
type: regex
target: { source: file, path: docs/plans/note-tags.md }
pattern: '## Open questions\n(?:(?!\n## )[\s\S])*(case-sensitive|maximum number of tags|renaming a tag)'
flags: i
match: not_contains
weight: 2
---

The three answered questions leave the Open questions section.
