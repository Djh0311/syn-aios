# Task Queue Bridge

This file exists so the installed harness can find a `docs/task-queue.md` entrypoint.

Current task authority remains:

- `tasks/README.md`
- `tasks/*.md`
- `handoffs/*-result.md`
- `handoffs/*-review.md`
- `CURRENT.md`

Do not maintain a second task queue here. When a task changes, update the existing `tasks/**`, `handoffs/**`, `evidence/**`, and `CURRENT.md` files according to the project-line rules.

Harness task-package tooling may be used as a helper, but it does not replace the product-line task queue.
