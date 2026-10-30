#!/usr/bin/env bash
set -euo pipefail
# ADRs 0001-0002 exist; the code already talks to Postgres.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
cat > db.py <<'PY'
import psycopg


def connect(dsn: str) -> psycopg.Connection:
    return psycopg.connect(dsn)
PY
mkdir -p docs/adr
cat > docs/adr/0001-record-architecture-decisions.md <<'MD'
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
MD
cat > docs/adr/0002-rest-apis.md <<'MD'
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
MD
git add -A
git commit -qm "Initial service with ADRs"
