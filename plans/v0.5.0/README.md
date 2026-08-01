# Adaptive Harness v0.5 task packages

This directory is the canonical carrier for v0.5 task packages.

- A package named `<TASK_ID>.md` is created only by the v0.5 `task start` flow.
- The package must match one active task node with the same full task ID.
- Plans and draft text do not activate work by themselves.
- Do not manually overwrite, reuse, or promote a historical package.

The directory is committed ahead of the first task because the fail-closed package writer requires the canonical parent directory to exist before it creates a package.
