#!/usr/bin/env bash
set -euo pipefail
# A repository whose rules live only in CLAUDE.md.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
cat > CLAUDE.md <<'EOF'
# Rules

- Use pnpm, never npm.
- Run `pnpm test` before every commit.
EOF
cat > package.json <<'EOF'
{"name": "web", "private": true, "scripts": {"test": "vitest run"}}
EOF
git add -A
git commit -qm "Add app"
