# Evals

Golden tasks that regression-test components. `claude plugin eval` reads this directory by default.

## Format

- One eval (or eval set) per component, named after it: `evals/<component-name>.md`.
- Skills/commands: a task the component must handle + the observable success criteria.
- Hooks: trigger sequence + expected effect.
- Skills and hooks rot silently without evals — a component without an eval here is unfinished.
