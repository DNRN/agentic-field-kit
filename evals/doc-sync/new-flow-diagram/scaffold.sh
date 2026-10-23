#!/usr/bin/env bash
set -euo pipefail
# Project keeps feature docs in docs/ (one file per feature, indexed in docs/index.md).
# The staged diff adds a three-step job flow with no doc.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
mkdir -p docs client
: > client/__init__.py
cat > client/auth.py <<'EOF'
def login(session, token: str) -> None:
    session.headers["Authorization"] = f"Bearer {token}"
EOF
cat > docs/index.md <<'EOF'
# Docs

- [Authentication](auth.md): log in with an API token.
EOF
cat > docs/auth.md <<'EOF'
# Authentication

Call `client.auth.login(session, token)` once per session; every later request carries the token.
EOF
cat > README.md <<'EOF'
# report-client

Python client for the report service. Docs: [docs/index.md](docs/index.md).
EOF
git add -A
git commit -qm "Add auth client"
cat > client/jobs.py <<'EOF'
import time


# Start a report job; returns its job id.
def submit_report(session, query: str) -> str:
    return session.post("/jobs", json={"query": query}).json()["id"]


# Block until the job is done; raise on failure or timeout.
def poll_status(session, job_id: str, interval: float = 2.0, timeout: float = 300.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        state = session.get(f"/jobs/{job_id}").json()["state"]
        if state == "done":
            return
        if state == "failed":
            raise RuntimeError(f"job {job_id} failed")
        time.sleep(interval)
    raise TimeoutError(f"job {job_id} still running after {timeout}s")


# Download the finished report.
def fetch_result(session, job_id: str) -> bytes:
    return session.get(f"/jobs/{job_id}/result").content


def run_report(session, query: str) -> bytes:
    job_id = submit_report(session, query)
    poll_status(session, job_id)
    return fetch_result(session, job_id)
EOF
git add -A
