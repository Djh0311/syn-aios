# Review: Root Treatment / R4-H3-5 Agent Developer Boundary Panel Split v1

日期：2026-06-13

复核线：Singer

状态：`STATUS: CLEAR`

## 1. 结论

H3-5 可放行。P0 / P1 / P2 均无。

## 2. 复核发现

P0：无。

P1：无。

P2：无本包阻断项。

边界备注：工作树里除主管线声明的外部脏文件外，另有未声明未跟踪文件 `docs/workbench-architecture-principles-v1.md`。复核线未将其计入本包问题；主管线提交前需排除。

## 3. 验证命令

复核线运行：

- `git diff --check`：通过，无输出。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，`Status: pass`，`Errors: 0`，`Warnings: 0`，`AgentView.tsx: 285/285`。
- `npm run typecheck`：在 `/Users/yoyi/workspace/product-line` 根目录因无 `package.json` 失败；随后在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell` 重跑通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`。

复核线未运行 `npm run build`；主管线已运行并通过，只有既有 Vite chunk size warning。

## 4. 关键证据

- `AgentView.tsx` 当前 285 行，低于 1000 行目标。
- 新增目标文件分别为 102 / 518 / 185 / 523 / 559 行，均低于 2000 行，未见新巨型文件。
- `AgentDeveloperPanels.tsx` 的 11 个面板顺序与 `HEAD:AgentView.tsx` 原开发者详情顺序一致。
- `CodexControlEntryPanel` 未持久化 prompt body：只保留本地 state，构造输入只写 `prompt_ref` / `prompt_hash`，未出现 `prompt_body`。
- Phase B / real Codex guard 未放宽：新增执行面板只 import preview / prepare / confirm / Phase A，Phase A 使用 `execution_decision: "phase_a_noop"`；changed files 中未命中 Phase B API 或 `prompt_body`。
- adapter / provider / credential / model 仍是边界展示：provider 文案明确“不读取密钥、不验证模型、不发起供应方调用”，session operation 文案明确“不执行”。
- readback unknown 未显示成真实 0：结果数展示走 `readbackCountLabel`，该 helper 对 `null` / `undefined` 返回 `未知/不可用`。
- 未修改 `ProjectsView.tsx`、`styles.css`、Rust / Tauri / DB / sidecar / workflow schema。
- 只有 `scripts/harness/workbench-shape-gate.js` 更新 `AgentView.tsx` waterline 为 285。

## 5. 边界确认

复核线未发现 H3-5 越界：

- 未启动 Tauri / Browser / Chrome / Vite dev。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未修改 UI / CSS / 水墨风格。
- 未修改 ProjectsView。
- 未修改 Rust / Tauri / DB / sidecar / workflow state schema。

## 6. 主管线提交提醒

提交时只应纳入 H3-5 文件，排除当前外部脏文件：

- `AGENTS.md`
- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`
- `docs/workbench-architecture-principles-v1.md`
- `handoffs/2026-06-13-r4-hard-targets-execution-plan-supervisor-v1.md`
