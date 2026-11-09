#!/usr/bin/env bash
set -euo pipefail
# Notes CLI whose notes.json holds an imported title in decomposed Unicode (NFD); issue #15 reports the search miss without the cause.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
mkdir -p notes tests
: > notes/__init__.py
: > tests/__init__.py
cat > notes/__main__.py <<'EOF'
from notes.cli import main

main()
EOF
cat > notes/store.py <<'EOF'
import json
from pathlib import Path

DB = Path("notes.json")


def load() -> list[dict]:
    if not DB.exists():
        return []
    return json.loads(DB.read_text(encoding="utf-8"))


def save(notes: list[dict]) -> None:
    DB.write_text(json.dumps(notes, indent=2, ensure_ascii=False), encoding="utf-8")


def add(title: str, body: str) -> dict:
    notes = load()
    note = {"id": len(notes) + 1, "title": title, "body": body}
    notes.append(note)
    save(notes)
    return note


def search(query: str) -> list[dict]:
    q = query.lower()
    return [n for n in load() if q in n["title"].lower() or q in n["body"].lower()]
EOF
cat > notes/cli.py <<'EOF'
import argparse

from notes import store


def main() -> None:
    parser = argparse.ArgumentParser(prog="notes")
    sub = parser.add_subparsers(dest="command", required=True)
    add = sub.add_parser("add")
    add.add_argument("title")
    add.add_argument("body", nargs="?", default="")
    sub.add_parser("list")
    search = sub.add_parser("search")
    search.add_argument("query")
    args = parser.parse_args()
    if args.command == "add":
        note = store.add(args.title, args.body)
        print(f"added {note['id']}")
    elif args.command == "list":
        for note in store.load():
            print(f"{note['id']}: {note['title']}")
    elif args.command == "search":
        for note in store.search(args.query):
            print(f"{note['id']}: {note['title']}")
EOF
cat > tests/test_store.py <<'EOF'
import os
import tempfile
import unittest
from pathlib import Path

from notes import store


class StoreTest(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        store.DB = Path(self.dir.name) / "notes.json"

    def tearDown(self):
        self.dir.cleanup()

    def test_add_and_search(self):
        store.add("Groceries", "milk, eggs")
        self.assertEqual([n["title"] for n in store.search("milk")], ["Groceries"])


if __name__ == "__main__":
    unittest.main()
EOF
cat > README.md <<'EOF'
# notes

Command-line notes app. `python -m notes add <title> [body]`, `python -m notes list`, `python -m notes search <query>`.
EOF
cat > notes.json <<'EOF'
[
  {"id": 1, "title": "Cafe\u0301 plans", "body": "Imported from the old app."},
  {"id": 2, "title": "Café menu", "body": "Written in notes."}
]
EOF
cat > AGENTS.md <<'EOF'
# notes

Command-line notes app. Python 3.11, standard library only.

- Run: `python -m notes <command>`
- Test: `python -m unittest`

## Feature planning

- Tracker: GitHub issues, via `./tools/gh` (a repo-local stand-in for `gh` with the same `issue view` commands)
- Scout model: sonnet
EOF
mkdir -p tools
cat > tools/gh <<'EOF'
#!/usr/bin/env bash
# Repo-local stand-in for the GitHub CLI: `gh auth status` and `gh issue view <N> [--comments]`.
set -euo pipefail
case "${1:-} ${2:-}" in
  "auth status")
    echo "github.com"
    echo "  Logged in to github.com account eval-user"
    exit 0
    ;;
  "issue view") shift 2 ;;
  *)
    echo "tools/gh: supports 'auth status' and 'issue view <N> [--comments]'" >&2
    exit 1
    ;;
esac
num="" comments=0
for arg in "$@"; do
  case "$arg" in
    -c | --comments) comments=1 ;;
    --json* | -w | --web)
      echo "tools/gh: $arg is not supported; use --comments" >&2
      exit 1
      ;;
    *[0-9]) num="${arg##*[!0-9]}" ;;
  esac
done
case "$num" in
  15)
    cat <<'EOT'
title:	Search misses some notes
state:	OPEN
author:	maria
labels:	bug
comments:	2
number:	15
--
`notes search café` finds "Café menu" but not my note "Café plans", which is right there in `notes list`. Other searches work. Repro data: the `notes.json` in the repo.
EOT
    if [ "$comments" = 1 ]; then
      echo "--"
      cat <<'EOT'
author:	priya
association:	none
edited:	false
status:	none
--
Same here with `Zoë`: one of my Zoë notes shows up, the other doesn't.
--
EOT
      cat <<'EOT'
author:	maria
association:	none
edited:	false
status:	none
--
Both of my missing notes came over from the old app's import, if that matters.
--
EOT
    else
      echo "--"
      echo "2 comments not shown. Use --comments to view the full conversation."
    fi
    ;;
  *)
    echo "GraphQL: Could not resolve to an issue or pull request with the number of ${num:-?}." >&2
    exit 1
    ;;
esac
EOF
chmod +x tools/gh
git add -A
git commit -qm "Initial notes CLI"
