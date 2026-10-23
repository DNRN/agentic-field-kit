#!/usr/bin/env bash
set -euo pipefail
# ADRs already live in docs/decisions/ with custom sections (Problem / Decision / Outcome).
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
mkdir -p docs/decisions
cat > docs/decisions/0001-use-postgres.md <<'EOF'
# ADR-0001: Use Postgres for notes

**Status:** Accepted
**Date:** 2026-03-02

### Problem
Notes need durable, queryable storage.

### Decision
We use Postgres.

### Outcome
One database to operate; SQL for queries.
EOF
cat > docs/decisions/0002-sync-http-handlers.md <<'EOF'
# ADR-0002: Synchronous HTTP handlers

**Status:** Accepted
**Date:** 2026-04-11

### Problem
Request volume is low; async adds complexity.

### Decision
Handlers run synchronously.

### Outcome
Simple code; slow work must leave the request path.
EOF
git add -A
git commit -qm "Initial service with decisions"
