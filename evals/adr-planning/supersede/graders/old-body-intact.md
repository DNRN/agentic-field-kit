---
type: regex
target: { source: file, path: docs/adr/0002-rest-apis.md }
pattern: '## Context\n+Services are called by browsers and by each other; one style keeps tooling simple\.\n+## Decision\n+We will expose every service API as REST over HTTP/JSON\.\n+## Consequences\n+Uniform tooling and docs; no streaming or generated clients\.'
---

ADR 0002 is unchanged outside its Status section, so the commit gate's ADR check lets the change through.
