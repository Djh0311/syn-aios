# Task Package: Stage G / G2 Diagnostics Health And Degraded State v1

状态：已完成。  
用途：建立中间版本阶段 G 的最小只读诊断 / 健康 / degraded state 读模型，让工作台能解释 store 损坏、readback 失败、adapter unavailable、provider / credential 边界、Tauri bridge / 测试环境未验收等问题。G2 只接受为只读诊断完成，不接受为 G3 真实 Tauri 验收、G4 回放、G5 最终冻结或阶段 G 完成。

## 0. 先说薄弱点

- 诊断层容易滑向“自动修复 / 自动重试”；本任务只读解释，不修复、不重试。
- store integrity 容易误判历史缺失 sidecar 为阻断；本任务把 missing 记录为 warning，把损坏 JSON 记录为 degraded。
- `readback_unavailable` 容易被显示成 0 条结果；本任务明确它是不可读回，不是空结果。
- UI 只允许进入既有 `管理` 入口；真实窗口 / 截图验收仍留给 G3。

## 1. 已知事实 / 未知 / 假设

已知事实：

- 阶段 C、D/M、E、F 已完成；阶段 F 最终结论为 `accepted_with_deferred_items`。
- G1 Runtime Log Boundary And Minimal Store 已完成并经全局主管接受。
- G2 是 G3/G4/G5 的前置，不能跳过。

未知：

- G3 真实 Tauri 截图工具链是否稳定。
- 后续是否需要把 diagnostic bundle 落盘为独立文件。

假设：

- G2 可以把 `WorkbenchSnapshot.diagnostic_summary` 作为只读 diagnostic bundle 引用，不在本轮新增导出文件。
- 缺失 sidecar 不等于损坏；损坏 JSON 才进入 degraded。

## 2. 接受范围

接受为：

- 新增 `DiagnosticSummary`、`ServiceDegradedState`、`StoreIntegrityFinding` 后端 / 前端类型。
- `WorkbenchSnapshot.diagnostic_summary` 输出健康计数、degraded 状态、最近 error 摘要、store integrity 和边界说明。
- 只读检查 workflow state、index、tasks、formal memory、memory candidate、blackboard candidate、observation、memory lint、entity relation、mature pattern、plan authorization、project proposal、session continuation、runtime log 等关键文件 / sidecar。
- 管理入口显示健康 / 诊断摘要、store integrity 和诊断边界说明。
- 解释 adapter unavailable、provider / credential / model boundary、runtime attention / readback boundary、runtime log error、Tauri bridge / session index 缺失、测试环境未验证。
- `diagnostic_summary` 可作为只读 diagnostic bundle 引用，不含 secret。

不接受为：

- 自动修复 store、自动初始化 missing sidecar、自动迁移数据库。
- 自动重试、真实 worker 执行、真实 `codex exec` / `codex exec resume`、真实 prompt 发送。
- provider / model / credential 真实探测。
- G3 真实 Tauri / 截图验收完成。
- G4 中间版本端到端回放完成。
- G5 最终权威验收、阶段 G 完成或中间版本最终完成。

## 3. UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取 / 继承：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- G1 task / evidence / handoff

本任务允许显示：

- `管理` 入口中的健康 / 诊断边界。
- store integrity 的 status、item count、warning count、schema、revision 摘要。
- degraded state 的标题、原因、source refs 和下一步建议。
- runtime log 与 audit event 边界。

本任务禁止显示：

- token、secret、完整 transcript、raw provider credential、auth、`.env`、keychain、OAuth。
- raw command、prompt body、runner output、provider material、conversation body。
- “已自动修复 / 已恢复 / 已重试成功”。
- “G3/G4/G5 已完成”或“阶段 G 已完成”。

显示位置：

- 一级入口：不新增。
- 右侧入口：复用既有 `管理`。
- 项目页：不改。
- 画布：不改。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：不改。

验收：

- 类型检查：必须运行。
- 离线交互测试：必须运行。
- 构建：必须运行。
- 真实窗口 / 截图验收：不属于 G2；留给 G3。

## 4. 实现摘要

- Rust `types.rs` 新增 G2 诊断类型，并接入 `WorkbenchSnapshot`。
- Rust `lib.rs` 新增 `derive_diagnostic_summary`、store integrity probe、只读 degraded state 派生和 G2 定向测试。
- TS `types.ts` 新增 `DiagnosticSummary`、`ServiceDegradedState`、`StoreIntegrityFinding`。
- `App.tsx` 复用右侧 `管理` 入口，新增健康 / 诊断边界卡片，不新增导航。
- 离线测试补 G2 管理入口文案和敏感内容边界。

## 5. 验证

需通过并记录：

- `cargo test --lib g2_diagnostic`
- `cargo test --lib runtime_log`
- `cargo test --lib`
- `rustfmt --check src/types.rs src/lib.rs src/runtime_log_store.rs`
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

## 6. 下一步

下一步只能进入：

```text
G3 Real Tauri Acceptance Harness And Screenshot Evidence 待开始 / 待拆
```

不得声明 G3-G5 或阶段 G 已完成。
