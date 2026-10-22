# Evals

Golden tasks for components whose behavior depends on a model: a task, plus observable success criteria, run by hand today. `claude plugin eval` does **not** read these files — it expects `evals/**/case.yaml` or `prompt.md` + `graders/*.md`; conversion is planned in [D7](../docs/plans/design-improvements.md#d7-convert-evals-to-claude-plugin-eval-format).

## Format

- One eval (or eval set) per component, named after it: `evals/<component-name>.md`.
- Skills/commands: a task the component must handle + the observable success criteria.
- Hooks and scripts: deterministic, so they get regression tests in `tests/` (`tests/run.sh`), with a pointer from the component's eval.
- Skills and hooks rot silently without evals — a component without an eval here is unfinished.
