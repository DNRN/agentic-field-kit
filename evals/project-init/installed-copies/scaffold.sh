#!/usr/bin/env bash
set -euo pipefail
# What `agentic-toolkit` leaves in a new project folder before the agent
# starts, as for a project with the plugin enabled: project-init in
# .agents/skills only (Claude Code takes the plugin's copy, so this one can't
# shadow it) and the manifest. Plus a personal Claude Code settings file that
# must not be committed. No git yet.
mkdir -p .agents/skills/project-init .claude
printf -- '---\nname: project-init\ndescription: Use when the user asks to start a project.\n---\n\n# Project init (installed copy)\n' > .agents/skills/project-init/SKILL.md
printf 'kind\tname\tversion\thash\treason\tpaths\nskill\tproject-init\t0.3.0\t0000\texplicit\t.agents/skills/project-init\n' > .agents/agentic-toolkit.tsv
printf '{"permissions": {"allow": ["Bash(ls:*)"]}}\n' > .claude/settings.local.json
