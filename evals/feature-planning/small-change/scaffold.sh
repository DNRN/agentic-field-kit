#!/usr/bin/env bash
set -euo pipefail
# A project with feature-planning settings recorded; the task is a small, fully specified change.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
cat > cli.py <<'EOF'
import argparse


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("name")
    args = parser.parse_args()
    print(f"hello {args.name}")


if __name__ == "__main__":
    main()
EOF
cat > AGENTS.md <<'EOF'
# hello

Greeting CLI. Python 3.11, standard library only.

## Feature planning

- Tracker: none
- Scout model: sonnet
EOF
git add -A
git commit -qm "Add cli"
