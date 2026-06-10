# Codex Workflow Runtime Prototype

## Purpose

This prototype models the smallest Codex workflow orchestration loop:

1. Director creates a structured task.
2. Worker receives the task.
3. Orchestrator waits for worker completion.
4. Orchestrator reads back the worker transcript.
5. Director reviews the result.
6. Runtime records the final summary.

The user-facing direction is Codex session management and workflow orchestration, not a task-package manager.

## Modes

Dry run:

```bash
python3 product-line/prototypes/codex-workflow-runtime/workflow_runtime.py \
  --dry-run \
  --output-dir /tmp/codex-workflow-runtime-v1
```

Dry run does not start Codex CLI and does not write `/Users/yoyi/.codex`.

Real no-business probe:

```bash
python3 product-line/prototypes/codex-workflow-runtime/workflow_runtime.py \
  --real-codex-probe \
  --approval-text "批准执行 Codex 工作流运行模型 v1 的真实无业务探针" \
  --output-dir /tmp/codex-workflow-runtime-v1
```

Do not run the real probe unless the user has provided the exact approval text in the current task.

## Output

The runtime writes these files under the output directory:

- `run.json`
- `director-task.json`
- `worker-events.jsonl`
- `worker-last-message.txt`
- `index.json`
- `worker-transcript.json`
- `director-review.json`

`run.json` records nodes, edges, events, sessions, artifacts, warnings, and status.

## Safety

The v1 scenario uses a no-business prompt and never resumes or forks an existing session. The dry-run path is the default safe validation path.
