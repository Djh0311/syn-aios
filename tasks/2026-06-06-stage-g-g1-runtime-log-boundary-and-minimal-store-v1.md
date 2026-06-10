# Task Package: Stage G / G1 Runtime Log Boundary And Minimal Store v1

状态：已完成。  
用途：建立 runtime log 与 audit event 的边界，并为中间版本阶段 G 提供最小运行记录 store、脱敏展示摘要和管理入口过滤摘要。G1 只接受为运行日志边界和最小 store，不接受为 G2 diagnostics、G3 真实 Tauri 验收、G4 回放或 G5 最终验收完成。

## 0. 先说薄弱点

- runtime log 容易被误当成 audit event；本任务明确二者不能互相替代。
- 运行日志如果保存 raw command、完整 transcript、provider credential 或授权材料，会直接破坏后续 G2/G3/G5 的安全边界。
- 本轮改了管理入口的可见摘要，但普通浏览器检查失败，真实 Tauri / 截图验收仍必须留给 G3。
- 本轮发生一次过程偏差：误读 `/Users/yoyi/.codex/skills/playwright/SKILL.md`。不得声称本轮完全未读 `.codex`。

## 1. 已知事实 / 未知 / 假设

已知事实：

- 阶段 C、D/M、E、F 均已完成；阶段 F 最终结论为 `accepted_with_deferred_items`。
- G1 是阶段 G 第一刀；G2-G5 尚未开始。
- E6 已提供 `runtime_session_attention[]` 和 `session_run_status_summaries[]`，但不包含完整 runtime log store。

未知：

- 后续 G2 diagnostics 是否需要更完整的 store integrity checker。
- G3 真实 Tauri 截图工具链是否稳定。

假设：

- G1 可以从工作台自有 continuation / attention 状态派生最小运行日志摘要。
- 不存在 `runtime-logs.v1.json` sidecar 时，可以返回派生的脱敏 store，但不能伪装为真实持久日志已写入。

## 2. Runtime Log / Audit Event 边界

runtime log：

- 记录运行过程状态、类别、开始 / 结束时间、耗时、source refs、audit refs 和脱敏摘要。
- 用于用户可见运行摘要、管理入口过滤、后续诊断输入和 G 阶段验收证据。
- 不记录可追责决策本体，不替代审计。

audit event：

- 记录可追责的 actor、权限、状态前后变化、原因、确认和审计证据。
- 用于责任链、权限链、状态变更和结果复核。
- 不替代运行过程日志。

固定规则：

- runtime log 与 audit event 不能互相替代。
- runtime log 只引用 `audit_refs`，不内嵌 audit event 本体。
- audit event 不承担运行状态 timeline / duration / category 摘要职责。

## 3. 接受范围

接受为：

- 新增 `runtime_log_store.v1` 后端读模型和最小 store 结构。
- 最小支持 `app_session`、`workflow_run`、`dispatch_attempt`、`readback`、`permission_wait`、`diagnostic_event` 六类运行记录。
- `WorkbenchSnapshot.runtime_log_store` 可返回脱敏运行日志摘要。
- 管理入口显示 runtime log 摘要、分类过滤 chip、日志 / 审计边界说明。
- 日志展示不包含 token、secret、完整 transcript、raw provider credential、auth、`.env`、keychain、OAuth 内容。
- `runtime-logs.v1.json` 存在时按 schema 读取并脱敏；不存在时从工作台自有状态派生安全摘要。

不接受为：

- G2 diagnostics / health / degraded state 完成。
- G3 真实 Tauri / 截图验收完成。
- G4 中间版本端到端回放完成。
- G5 最终权威验收或阶段 G 完成。
- 自动重试、真实 worker、真实 `codex exec` / `codex exec resume`、真实 prompt 发送。
- GEPA eval export。
- 读取或写入用户 Codex 会话数据、auth、token、`.env`、secret、keychain、OAuth、provider credential。

## 4. UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- 管理入口里的运行日志摘要。
- runtime log 分类、状态、severity、entry count、audit refs 数量。
- “Runtime log 与 audit event 不能互相替代”的边界说明。

本任务禁止显示：

- token、secret、完整 transcript、raw provider credential、auth、`.env`、keychain、OAuth。
- raw audit event 本体。
- raw command preview、prompt body、runner output、provider material、conversation body。
- “G2/G3/G4/G5 已完成”或“阶段 G 已完成”。

显示位置：

- 一级入口：不新增。
- 右侧入口：复用既有 `管理`。
- 项目页：不改。
- 画布：不改。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：新增 runtime log 摘要、分类过滤和日志 / 审计边界说明。

中间版本范围：

- 本轮必须落地：runtime log 最小结构、脱敏摘要、管理入口摘要。
- 本轮只做读模型 / 摘要：诊断输入和管理过滤。
- 本轮后置：G2 完整诊断、G3 真实 Tauri 截图验收、G4 回放、G5 freeze。

后端和数据依赖：

- 需要后端正式读模型：`WorkbenchSnapshot.runtime_log_store`。
- 需要审计 / 日志 / 权限 / 状态机：只引用 `audit_refs`，不内嵌 audit event。
- 不能用假数据伪装：sidecar 不存在时明确为派生摘要。

UI 文案边界：

- 禁止说：`runtime log 已完成为最终形态`、`diagnostics 已完成`、`真实 Tauri 验收完成`、`阶段 G 完成`。
- 允许说：`G1 runtime log boundary and minimal store completed`、`下一步 G2 待开始 / 待拆`。

验收：

- 类型检查：已运行。
- 离线交互测试：已运行。
- 构建：已运行。
- 真实窗口 / 截图验收：未完成；普通浏览器检查失败，不等于 G3。
- 未验收项必须写入 evidence / handoff：已记录。

## 5. 实现摘要

- 新增 Rust 模块 `runtime_log_store.rs`。
- 新增 Rust / TS 类型：`RuntimeLogStoreV1`、`RuntimeLogEntry`、`RuntimeLogSummary`、`RuntimeLogBoundary` 等。
- `WorkbenchSnapshot` 新增 `runtime_log_store`。
- 右侧 `管理` 入口显示 runtime log 摘要和过滤 chip。
- 离线测试新增 G1 runtime log boundary 场景。

## 6. 验证

通过：

- `cargo test --lib`
- `rustfmt --check src/runtime_log_store.rs src/types.rs src/lib.rs`
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

普通浏览器检查：

- Vite server 启动成功后，bundled Playwright 缺 Chromium；本机 Chrome headless 启动失败并被系统关闭。
- 因此普通浏览器检查未完成。
- 该缺口不等于 G3；G3 真实 Tauri / 截图验收仍未开始。

## 7. 边界偏差记录

- 误读：`/Users/yoyi/.codex/skills/playwright/SKILL.md`。
- 性质：违反本任务“不读写 `/Users/yoyi/.codex`”的过程边界。
- 未发生：未读取用户 Codex 会话数据、完整 transcript、auth、token、`.env`、secret、keychain、OAuth、provider credential；未写 `/Users/yoyi/.codex`；未执行真实 `codex exec` / `codex exec resume`；未发送真实 prompt。

## 8. 证据

- `evidence/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md`
- `handoffs/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1-result.md`

## 9. 下一步

下一步只能进入：

```text
G2 Diagnostics / Health / Degraded State 待开始 / 待拆
```

不得声明 G2-G5 或阶段 G 已完成。
