#!/usr/bin/env bash
set -euo pipefail
# An existing project with no AGENTS.md or PROJECT.md; the task is ordinary feature work.
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
git add -A
git commit -qm "Add cli"
