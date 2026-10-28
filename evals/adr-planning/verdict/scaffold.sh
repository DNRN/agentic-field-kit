#!/usr/bin/env bash
set -euo pipefail
# An OAuth plan with two Proposed ADRs waiting for the user's verdict.
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
mkdir -p docs/adr docs/plans
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
cat > docs/adr/0002-redis-sessions.md <<'EOF'
# 0002. Store login sessions in Redis

Date: 2026-03-02

## Status
Proposed

## Context
OAuth login needs server-side sessions that every instance can read.

## Decision
We will store login sessions in Redis with a 24-hour TTL.

## Consequences
One more service to run and back up; fast lookups and built-in expiry.

## Rejected alternatives
- Postgres table: an extra query per request.
EOF
cat > docs/adr/0003-oidc-provider-interface.md <<'EOF'
# 0003. One interface for identity providers

Date: 2026-03-02

## Status
Proposed

## Context
We start with Google login and expect GitHub next.

## Decision
We will put every identity provider behind one OIDC client interface.

## Consequences
A second provider is configuration, not code; provider-specific claims need mapping.

## Rejected alternatives
- Provider-specific handlers: duplicated callback and token logic.
EOF
cat > docs/plans/oauth-login.md <<'EOF'
# Plan: OAuth login

1. Sessions per [ADR 0002](../adr/0002-redis-sessions.md).
2. Google login through the interface of [ADR 0003](../adr/0003-oidc-provider-interface.md).
3. Logout clears the session.
EOF
git add -A
git commit -qm "Plan OAuth login; propose ADRs 0002 and 0003"
