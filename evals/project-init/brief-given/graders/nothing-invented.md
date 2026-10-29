---
type: regex
target: { source: file, path: PROJECT.md }
pattern: 'PostgreSQL|Postgres|React|Docker|Kubernetes|Redis'
match: not_contains
weight: 2
---

No technology the user didn't name.
