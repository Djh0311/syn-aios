# Adaptive Harness v0.5 task packages（历史）

> 2026-08-08 起本目录仅作旧任务包审计材料。包内 lifecycle、scope、verification 和 next 字段不再激活工作，也不授予权限；当前工作只认 Harness Lite 当前链。

This directory was the canonical carrier for v0.5 task packages before the Harness Lite cutover.

- A package named `<TASK_ID>.md` is created only by the v0.5 `task start` flow.
- The package must match one active task node with the same full task ID.
- Plans and draft text do not activate work by themselves.
- Do not manually overwrite, reuse, or promote a historical package.

The directory is committed ahead of the first task because the fail-closed package writer requires the canonical parent directory to exist before it creates a package.
