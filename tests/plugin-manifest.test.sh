#!/usr/bin/env bash
# plugin.json lists agents, commands, and output styles explicitly: those keys
# replace Claude Code's folder scan, which would otherwise load each folder's
# README.md as a component. Explicit lists mean an unlisted file silently
# never ships; this test fails on that and on listed files that do not exist.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
python3 - "$ROOT" <<'EOF'
import json, pathlib, sys

root = pathlib.Path(sys.argv[1])
manifest = json.loads((root / ".claude-plugin/plugin.json").read_text())
fails = 0
for key, folder in (("agents", "agents"), ("commands", "commands"), ("outputStyles", "output-styles")):
    listed = manifest.get(key)
    if not isinstance(listed, list):
        print(f"FAIL {key}: must be an explicit list (folder scan loads {folder}/README.md)")
        fails += 1
        continue
    listed_paths = {(root / p).resolve() for p in listed}
    for path in sorted(listed_paths):
        if not path.is_file():
            print(f"FAIL {key}: listed file missing: {path.relative_to(root)}")
            fails += 1
    for path in sorted((root / folder).glob("*.md")):
        if path.name != "README.md" and path.resolve() not in listed_paths:
            print(f"FAIL {key}: {path.relative_to(root)} is not listed in plugin.json, so it never loads")
            fails += 1
    if (root / folder / "README.md").resolve() in listed_paths:
        print(f"FAIL {key}: {folder}/README.md must not be listed")
        fails += 1
print("plugin-manifest: all passed" if not fails else f"plugin-manifest: {fails} failed")
sys.exit(1 if fails else 0)
EOF
