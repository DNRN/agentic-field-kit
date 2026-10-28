#!/usr/bin/env bash
set -euo pipefail
# Small service with no ADRs yet.
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
git add -A
git commit -qm "Initial service"
