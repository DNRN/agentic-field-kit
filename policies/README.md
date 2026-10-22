# Policies

Declarative security and permission presets — `permissions.allow` / `permissions.deny`, sandbox settings — merged into a project's `.claude/settings.json` or the harness equivalent.

## Format

```
policies/<kebab-name>.json
```

- Fragments in settings shape, documented with a header comment field or companion `.md`.
- Declarative: prefer a deny rule over a hook script that checks the same thing.
- Default posture: read-only tools allowed broadly, write/execute enumerated.
