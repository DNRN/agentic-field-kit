#!/usr/bin/env bash
set -euo pipefail
# Staged diff changes whitespace only (trailing spaces, blank lines).
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
cat > app.py <<'EOF'
def greet(name):   
    return "hello " + name   



def shout(name):
    return greet(name).upper()
EOF
git add -A
git commit -qm "Add greet"
cat > app.py <<'EOF'
def greet(name):
    return "hello " + name


def shout(name):
    return greet(name).upper()
EOF
git add -A
