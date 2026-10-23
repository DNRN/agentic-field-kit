#!/usr/bin/env bash
set -euo pipefail
# Accepted ADR 0002 mandates REST for every service API.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
cat > app.py <<'EOF'
from http.server import BaseHTTPRequestHandler, HTTPServer


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"ok")


if __name__ == "__main__":
    HTTPServer(("", 8000), Handler).serve_forever()
EOF
cat > README.md <<'EOF'
# notes-service

Small HTTP service for shared notes. Run `python app.py`.
EOF
mkdir -p docs/adr
cat > docs/adr/0001-record-architecture-decisions.md <<'EOF'
# 0001. Record architecture decisions

Date: 2026-01-05

## Status
Accepted

## Context
We need to record the architectural decisions made on this project.

## Decision
We will use Architecture Decision Records.

## Consequences
Decisions are numbered, immutable, and superseded rather than edited.
EOF
cat > docs/adr/0002-rest-apis.md <<'EOF'
# 0002. REST for service APIs

Date: 2026-02-14

## Status
Accepted

## Context
Services are called by browsers and by each other; one style keeps tooling simple.

## Decision
We will expose every service API as REST over HTTP/JSON.

## Consequences
Uniform tooling and docs; no streaming or generated clients.
EOF
git add -A
git commit -qm "Initial service with ADRs"
