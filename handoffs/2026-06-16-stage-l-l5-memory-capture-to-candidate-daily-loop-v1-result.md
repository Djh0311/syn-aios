# Stage L / L5 Memory Capture To Candidate Daily Loop Result Handoff v1

日期：2026-06-16

状态：实现、本地验证与独立复核完成；Aquinas 复审 `STATUS: CLEAR_WITH_NOTE`，P0 / P1 / P2 / P3 均无；提交前停止，不 `git add` / `git commit`。

## 当前状态

L5 当前完成的是“日常可见闭环”的第一条产品路径：

- 运行中工作流页新增日常记忆候选收件箱，显示待确认候选数量、来源、风险和候选边界。
- 候选可单条采纳、批量采纳、暂不处理或拒绝候选；采纳进入 PermissionDialog，确认后由 App 调用既有 M2 `adoptMemoryCandidateToFormalMemory`；暂不 / 拒绝复用既有 `recordMemoryCandidateDecision`，只写候选状态。
- L3 operation-control 决策确认后，pending action 携带 `memoryCaptureEvent`；App 先记录 L3 决策，再调用 `captureMemoryEvent` 生成 observation / candidate。
- K3 Level-A 过程事实 capture 从 `audit_only` 调为 `observation_only`，仍不生成 candidate / FormalMemory。

## Coverage

已接：

- `operation_control_decision` -> `candidate_allowed`。
- `process_fact_decision` -> `observation_only`。
- 日常候选收件箱 -> 单条 / 批量采纳 -> M2 用户确认门。
- 日常候选收件箱 -> 暂不处理 / 拒绝候选 -> 既有候选决策路径，不写 FormalMemory。

Deferred：

- 计划/方案采纳 capture。
- 全局主管最终复核 capture。
- worker report 更广覆盖 capture。

这些 deferred 不阻断本包，因为本包先修“前端 captureMemoryEvent 零调用”和“日常候选不可见”两个硬缺口；其余落账点应另包接线，避免触碰业务语义。

## 边界

- 未改 R3 记忆 schema / 17 表，未加表，未改 SQLite schema/migration。
- 未自动写 FormalMemory；只有用户确认 PermissionDialog 后才调用 M2 采纳。
- 暂不处理 / 拒绝候选只更新 `memory-candidates.v1.json` 候选状态，不写正式记忆。
- 未新增真实执行、runner、`Command::new("codex")`、`codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未自动捕获普通聊天。
- 未把候选、observation、knowledge hit 或 LLM summary 文案写成正式记忆。
- UI 只改桌面 Tauri 工作台的运行中页局部面板，不做手机端 UI。

## 验证

已通过：

- `node scripts/harness/capability-scan.js --target .`：PASS 7 / FAIL 0 / WARN 10。
- `node scripts/harness/guard-state-files.js --target .`：PASS 19 / `envFiles: []`。
- `npm run typecheck`。
- `npm run test:offline-interaction`：offline interaction tests passed 15，R4 4 个页面读模型测试通过。
- `npm run build`：通过，仅既有 Vite chunk-size warning。
- P2 修复后重跑 `npm run typecheck` / `npm run test:offline-interaction` / `npm run build`：均通过；build 仍只有既有 chunk-size warning。
- `cargo test --lib memory_daily_loop`：2 passed。
- `cargo test --lib memory_capture_bus`：8 passed。
- `cargo test --lib observation_store`：1 passed。
- `cargo test --lib memory_candidate_store`：1 passed。
- `cargo test --lib formal_memory_lifecycle`：7 passed。
- `cargo test --lib page_read_model`：7 passed。
- `cargo test --lib project_workflow_automation`：15 passed / 4 ignored。
- `cargo test --lib`：518 passed / 21 ignored。
- `cargo fmt -- --check`：无输出，exit 0。
- `node scripts/harness/workbench-shape-gate.js --mode check`：Status pass，Errors 0，Warnings 1；warning 为 `tauri_command_total_increased`（98 vs 97），继承 L3 既有窄命令形状，本包未新增 Tauri command。
- `git diff --check`：无输出，exit 0。
- `node scripts/harness/checkpoint-audit.js --package stage-l-l5-memory-capture-to-candidate-daily-loop --review evidence/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-review-aquinas-v1.md --allow-dirty --skip-gates`：VERDICT PASS；因本包按指令停在提交前，implementation commit / CURRENT.md checkpoint / commit file-boundary 为 N/A，tree clean 为 declared-dirty WARN，review_status PASS。
- 独立复核线 Aquinas（`019ece6b-4b39-7830-9553-86b979ec322c`）：初审 `CLEAR_WITH_P2`，P2 为缺少“暂不 / 拒绝”动作；修复后复审 `CLEAR_WITH_NOTE`，P0 / P1 / P2 / P3 均无。

## 扫描

危险串扫描已按 `git status --short` 显式列 changed/untracked 文件：

- L5 新增命中均为反向断言、禁止边界或任务包文字。
- 既有 `PermissionDialog.tsx` / `lib.rs` / 历史测试命中 `codex exec`、`.codex`，是既有权限/guard 文案，不是 L5 新增执行路径。
- 未发现 L5 新增 `Command::new`、runner 调用、自动正式化正向文案或“已执行真实操作”正向文案。

## 残余风险

- 真实浏览器/Tauri 可视化验收未在本包完成；当前以 offline React render、typecheck 和 build 覆盖。该残余风险与 L1/L3 的 L4 深层 Tauri 验收残余同类。
- L5 §4 仍有三类 capture 来源 deferred，下一包需要逐落账点设计，不应把本包解释为 capture 覆盖全量完成。
- “暂不处理”当前映射到既有 `candidate_discarded`，这是为了遵守既有状态机且不加新状态；后续若产品需要“稍后再看但仍留在收件箱”的持久状态，应另包设计。

## 下一步

1. 独立复核线只读核验 evidence / handoff / diff / scan / verification。
2. 主线根据复核结论补复核文件和 evidence 状态。
3. 停在提交前交主管线核实物；未经用户授权不 `git add` / `git commit`，不更新 `CURRENT.md`。
4. 主管线放行并提交后，用实际 L5 commit 再跑 checkpoint-audit，并由主管线落 `CURRENT.md` checkpoint。
