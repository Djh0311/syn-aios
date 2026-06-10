# Evidence: project workflow bootstrap v1 Tauri smoke

## Verdict

通过。

薄弱点先说：

- 这次只验证真实 Tauri 窗口里创建一个项目默认工作流草稿，不是自动编排执行。
- 真实状态文件这次被创建并保留。依据：任务包明确允许用户确认后由工作台写入自己的 `workflow-state.v0.json`，并禁止删除真实状态文件，除非用户另行要求。
- 默认 workflow / node / edge 仍是草稿事实，不代表真实 Codex 会话、任务包文件或 review 已执行。

## Scope

- Task package: `product-line/tasks/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-tauri-smoke.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`
- Real state path:
  - `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

No real state file content was read.

## Files read

- `product-line/tasks/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-tauri-smoke.md`
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- `product-line/evidence/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-result.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-review.md`
- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src/`
- `product-line/prototypes/productized-desktop-shell/tests/`

## Pre-checks

Real state file before the smoke:

```bash
test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'
```

Result:

- absent.

Port `5173` had a stale listener before the final smoke attempt. It belonged to the same productized desktop shell dev app and was cleaned before the final run.

## Validation commands

Run from `product-line/prototypes/productized-desktop-shell/`:

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

Results:

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, `offline interaction tests passed: 3`.
- `npm run build`: passed.

Run from `product-line/prototypes/productized-desktop-shell/src-tauri/`:

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline
```

Result:

- passed.
- Rust unit tests: `10 passed; 0 failed`.

## Real Tauri window smoke

Started with:

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target npm run tauri:dev
```

Observed in real Tauri dev window:

- Window title: `Codex 治理工作台`.
- Window size at normal smoke start: `1280 x 820`.
- Home page showed: `已读取索引。所有本机动作仍需用户点击并确认。`

Selected project:

- `agent world`
- path shown by UI: `/Users/yoyi/gameai/agent world`

Before creation, project page showed:

- `项目工作流草稿`
- `当前项目还没有本地工作流草稿`
- `workflow 未创建`
- `state 未创建`
- `nodes 0`

The workflow-state panel also showed:

- `exists=false`
- `未初始化`
- `状态文件不存在；不会自动创建。`

## Confirmation dialog

Clicked `创建默认工作流草稿`.

Dialog appeared and showed:

- `创建项目默认工作流草稿`
- target path: `/Users/yoyi/gameai/agent world`
- source: `索引内项目路径`
- write boundary:
  - write workbench-owned `workflow-state.v0.json`
  - write project / workflow / default nodes / default edges / audit
  - not write `.codex`
  - not write Codex state DB
  - not write project business dir
- explanatory text:
  - write only after confirmation
  - append audit event
  - use temp file and atomic replace
  - back up old state when present

The visible project page text under the button also stated:

- `不会派发给真实 Codex 会话`
- `也不会生成任务包文件`

Clicked `确认执行`.

## After creation

UI refreshed and showed:

- `当前项目已有本地工作流草稿`
- workflow id: `workflow:users-yoyi-gameai-agent-world:default`
- `state draft`
- `nodes 7`
- `edges 6`

Real state file after smoke:

```bash
test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'
```

Result:

- present.

The real state file was kept, per task package.

## Cleanup

Stopped the Tauri dev session with `Ctrl-C`.

Post-cleanup checks:

- `lsof -nP -iTCP:5173 -sTCP:LISTEN`: no listener.
- `pgrep -x codex-governance-workbench`: no process.
- real state file: present and kept.

## Prohibited items status

- Directly wrote the real state file with a script: no.
- Bypassed UI to call backend command: no.
- Wrote `/Users/yoyi/.codex`: no evidence from performed commands.
- Modified Codex state DB: no evidence from performed commands.
- Wrote project business directories: no evidence from performed commands.
- Read/displayed `auth.json`, `.env`, secrets, tokens, auth contents: no.
- Read/displayed Codex session bodies, tool output, command output, input history, memory bodies: no.
- Ran harness: no.
- Upgraded index candidates into verified capability: no.
- Connected non-Codex agent: no.
- Did knowledge base/vector search/model scheduling/release packaging/network dependency pull/automatic orchestration execution: no.

## Risk and next step

- This proves the real Tauri UI create path for one indexed project, not duplicate-create behavior in the real window.
- It does not prove task package generation, node status transition, or real Codex session dispatch.
- Next useful step is task-package draft generation v1 or node state transition v1, still behind explicit confirmation and audit.
