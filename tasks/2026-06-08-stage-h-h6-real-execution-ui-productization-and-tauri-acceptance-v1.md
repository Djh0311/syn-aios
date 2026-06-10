# Stage H / H6 Real Execution UI Productization And Tauri Acceptance v1

日期：2026-06-08

状态：已完成，结论为 `accepted_with_deferred_items`。

用途：把 H2/H5 已证明的 `codex-local` 真实执行能力、H4/G1/G2 的失败 / 运行日志 / 诊断边界，以及项目页 / 智能体页 / 右侧入口 / 管理入口的 UI 表达收成一个合并型 H6 checkpoint。H6 不再拆过细小 probe；入口文档只在本任务完成、阻断或范围变化时同步。

回收记录：

- `evidence/2026-06-08-stage-h-h6-real-execution-ui-productization-and-tauri-acceptance-v1.md`
- `handoffs/2026-06-08-stage-h-h6-real-execution-ui-productization-and-tauri-acceptance-v1-result.md`

最终边界：

- 接受为真实执行状态 UI 产品化、权限弹层边界、unknown readback 文案修补、前端 / Rust 边界验证和真实 Tauri 窗口探针完成。
- 不接受为真实 Tauri H6 关键截图清单完整完成、阶段 H 完成、通用自由 send / resume、H3-B retry 成功、自动重试、planned adapters 真实接入或 provider/model verification。

## 0. 全局主管理解

已知事实：

- H2 Phase B 已完成一次 `mario test` 总指导 session 的真实 `codex exec resume` 探针。
- H3.1 已完成 `new_session` 非执行产品路径；H3-B 已执行一次真实 new-session fixture run 但失败分类完成，等待新的 retry 授权。
- H4 Level A 已完成 readback / failure / timeout / duplicate guard 非真实产品化。
- H5 Level A、H5-Level-B1、H5-Level-B2 和 H5 product command formalization / acceptance checkpoint 已完成并通过全局主管复核。
- G1/G2 已提供 runtime log 和 diagnostics 最小底座。
- G3-B 真实 Tauri 验收已采集 10 / 13 张截图，但 `05-agent-session-center.png`、`06-send-resume-boundary.png`、`09-task-memory-packet-preview.png` 仍是 deferred。
- 用户已明确要求后续不要把任务拆得太细，入口文档只在 checkpoint 同步。

未知项：

- 当前 UI 是否已经能安全覆盖 H6 所需的智能体真实执行状态、send / resume 边界和任务记忆包预览截图。
- 当前真实 Tauri 启动、窗口定位和截图权限是否稳定。
- 是否需要补少量 UI 文案 / 读模型摘要，还是只需补截图证据和验收矩阵。

本任务假设：

- H6 默认不触发新的真实 `codex exec` / `codex exec resume`，不读写 `/Users/yoyi/.codex`。
- H6 可使用既有 H2/H5 真实执行证据和 current sidecar / read model 展示真实执行状态。
- 如必须新增真实执行，只能作为执行点授权清单提交给全局主管；未批准前不得执行。
- H6 允许启动真实 Tauri 和截图，但需要遵守本任务的窗口区域、路径、敏感信息和停止条件。

## 1. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `tasks/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md`
- `evidence/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-supervisor-review-v1.md`
- `handoffs/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-supervisor-review-v1-result.md`
- `tasks/2026-06-07-stage-g-g3-b-real-tauri-manual-screenshot-acceptance-v1.md`
- `tasks/2026-06-07-stage-g-g3-c-screenshot-evidence-recovery-and-gap-matrix-v1.md`

## 2. 目标

H6 的目标是让用户在工作台里看得懂真实执行状态，而不是靠 Markdown evidence 或原始日志猜。

本任务必须完成：

1. 智能体页真实执行状态产品化：展示 `codex-local`、target session、operation、permission/readiness、attempt、readback、runtime log / audit 引用和失败 / unknown-result 边界。
2. 项目页工作流真实执行状态产品化：展示节点执行状态、任务包摘要、任务记忆包摘要、权限、readback、worker report candidate / process fact handoff 和 H5 product command 边界。
3. 右侧入口保持分离：运行中显示正在做什么 / 是否卡住 / 是否需要权限；通知显示发生了什么；待办显示用户需要处理什么。
4. 管理入口脱敏展示 runtime log、diagnostics、audit、权限、健康状态和数据位置；raw log / internal id / path 大表不得铺在普通主界面。
5. 权限弹层说明做什么、为什么、影响哪里、谁提出、批准后发生什么、失败如何处理，并明确是否会触发真实 Codex / 写 `/Users/yoyi/.codex`。
6. 真实 Tauri 验收覆盖 H6 关键 UI，优先补齐 G3-B 剩余 3 个 deferred 截图项。
7. 新增 H6 evidence / handoff，给出 accepted / deferred / blocked 矩阵。

## 3. 非目标

H6 默认不做：

- 不开放任意项目 / 任意 session / 任意写入范围自由执行。
- 不新增裸 `codex exec` / `codex exec resume` UI。
- 不默认执行新的真实 Codex。
- 不创建新的真实 Codex session；H3-B retry 仍需单独授权。
- 不执行 H4-Level-B 真实失败 / 超时探针。
- 不做自动重试、自动恢复、stop / kill / restart 产品化。
- 不接 planned adapters 真实执行。
- 不做 provider credential store / model verification。
- 不读取 auth、token、secret、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 不把 worker report、readback、tool output、observation 或 candidate 自动写正式事实 / 正式记忆。
- 不把 H6 完成说成阶段 H 完成；阶段 H 是否完成由 H7 总复核冻结。

## 4. 工作线职责

尽量复用长期对话线程；如当前没有可复用线程，只建立少量长期职责线程，不为每个小点开新线程。

开发线职责：

- 复核现有 UI 和读模型是否已覆盖 H6 目标。
- 只在必要时补 UI 文案、读模型摘要、前端类型或 Tauri wrapper。
- 不新增一级入口，不把智能体页变成自由 Codex 控制台。
- 不执行真实 Codex，不读写 `/Users/yoyi/.codex`。
- 提交测试结果和变更说明给主管线。

验证线职责：

- 复核 UI 显示边界、误导文案、敏感路径、readback unknown-result、runtime/audit/diagnostics 边界。
- 执行前端 / Rust 相关验证。
- 准备并执行真实 Tauri 截图验收；如需要启动 GUI、截图或清理端口，必须按工具权限机制请求授权。
- 不把普通浏览器 smoke 冒充真实 Tauri。

全局主管职责：

- 冻结 H6 范围，避免继续拆小 probe。
- 审核开发线和验证线回交。
- 决定是否存在执行点授权需求；默认不批准新的真实 Codex 执行。
- 任务完成、阻断或范围变化时，同步入口文档。

## 5. UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- `codex-local` 的可用边界和 planned adapters 不可执行状态。
- 已授权真实执行产生的 attempt / runtime log / audit / readback 摘要。
- 未执行、等待权限、被 guard 阻断、readback unavailable / failed / timed out 等边界状态。
- H2/H5 真实 probe 只作为指定项目 / session 的证据。
- 任务记忆包 included / excluded / review materials、lint / blocking 和召回理由。
- 权限弹层的人话摘要、影响范围、allowed write roots、prompt summary / ref / hash、readback plan、runtime / audit preview。

本任务禁止显示：

- `Codex 已收到任务`、`worker 执行中`、`自动派发已开始`，除非有真实 attempt/runtime 证据。
- `readback unavailable` / `readback failed` 被显示为真实 0 条结果。
- planned adapters 可执行、provider credential 已配置、模型已验证。
- candidate / observation / knowledge hit 被写成正式记忆或正式事实。
- raw transcript、raw stdout / stderr、完整 prompt、secret、token、`.env`、rollout 正文。
- 任务包中心、日志中心或 schema 管理器占据项目页主界面。

显示位置：

- 一级入口：不新增；仍使用项目、智能体、画布、记忆、知识库、设置。
- 右侧入口：运行中 / 通知 / 待办 / 管理保持分离。
- 项目页：工作流 tab 和节点详情侧栏展示执行状态与摘要。
- 画布：节点只显示摘要 badge，不铺内部日志。
- 记忆入口：不新增正式记忆写入动作。
- 知识库入口：不改。
- 智能体入口：显示 session、adapter、operation boundary、send/resume 状态和 readback。
- 管理入口：显示 runtime log、diagnostics、audit 和数据位置脱敏摘要。

中间版本范围：

- 本轮必须落地：H6 关键状态 UI、边界文案、真实 Tauri 截图证据或明确降级。
- 本轮只做读模型 / 摘要：自动重试、stop/kill/restart、provider credential/model verification、planned adapters。
- 本轮后置：多模型真实接入、自动修复、复杂运维控制台、完整 Tauri 自动化。

后端和数据依赖：

- 需要后端正式读模型：WorkbenchSnapshot、session continuation、runtime log、diagnostics、H5 preview/bridge、workflow read model。
- 需要审计 / 日志 / 权限 / 状态机：真实执行状态必须来自 attempt/runtime/audit/readback 或 H5 preview，不从 UI 自造事实。
- 不能用假数据伪装：真实执行、readback、task memory packet、formal memory、provider credential。

UI 文案边界：

- 禁止说：`已自动执行`、`Codex 已收到任务`、`worker 正在执行`、`readback 0 条`、`模型已验证`、`planned adapter 可执行`、`已写正式记忆`。
- 允许说：`等待权限`、`执行前预览`、`真实执行证据来自 H2/H5 probe`、`readback unavailable`、`result_count=null`、`planned adapter 不可执行`、`需要执行点授权`。

验收：

- 类型检查：`npm run typecheck`
- 离线交互测试：`npm run test:offline-interaction`
- 构建：`npm run build`
- 真实窗口 / 截图验收：优先真实 Tauri；如果工具/权限失败，必须写入 evidence / handoff，不能声称完成。
- 未验收项必须写入 evidence / handoff。

## 6. 真实 Tauri 验收范围

截图目录：

```text
evidence/tauri-verification/2026-06-08-stage-h-h6/
```

H6 最小截图清单：

- `01-permission-dialog-real-execution-boundary.png`
- `02-agent-session-center-runtime-state.png`
- `03-send-resume-boundary.png`
- `04-project-workflow-real-execution-state.png`
- `05-workflow-node-execution-detail.png`
- `06-task-memory-packet-preview.png`
- `07-running-panel.png`
- `08-notifications-panel.png`
- `09-todos-panel.png`
- `10-admin-runtime-diagnostics-audit.png`

截图规则：

- 必须确认目标窗口为真实 Tauri `Codex 治理工作台`。
- 优先只截目标窗口区域，避免截到无关敏感窗口。
- 每张截图在 evidence 中记录操作路径、预期、实际、是否真实 Tauri、是否存在未覆盖项。
- 如果某项需要触发真实 Codex 或读取敏感数据才能截图，必须降级为 deferred，不得越界补图。
- 普通浏览器 smoke 只能作为辅助证据。

## 7. 执行点授权规则

H6 任务包创建本身不授权新的真实执行。

如果开发线或验证线认为必须追加真实执行，必须先提交执行点授权清单：

- project root、workflow、node、work item、dispatch id。
- operation：`resume` 或 `new_session`。
- target session id 或 new session strategy。
- sandbox、cwd、allowed write roots、denied paths。
- prompt summary / ref / sha256。
- expected readback marker。
- `/Users/yoyi/.codex` 最小副作用说明。
- rollback / cleanup / hash-diff plan。
- runtime log / audit / evidence / handoff refs。
- stop condition。

全局主管未确认前，不能执行真实 `codex exec` / `codex exec resume`。

## 8. 验收标准

H6 可接受条件：

- 智能体页能看懂真实执行状态、target session、permission/readiness、attempt、runtime/audit/readback 和 failure boundary。
- 项目工作流节点和详情能看懂 H5 项目派发状态、任务包摘要、记忆包摘要、权限、readback、worker report candidate / process fact handoff。
- 运行中 / 通知 / 待办 / 管理保持分离，且管理入口脱敏展示 runtime log / diagnostics / audit。
- 权限弹层能说明真实执行边界、影响范围、失败处理和是否写 `/Users/yoyi/.codex`。
- Unknown readback 不被写成 0 条。
- 禁止文案和敏感路径扫描无新增误导命中。
- 前端验证通过；若改 Rust，Rust 定向和全量验证通过。
- 真实 Tauri 截图完成，或未完成项清楚写入 deferred / blocked 矩阵。
- 新增 H6 evidence / handoff，明确接受范围和不接受范围。

H6 不可接受条件：

- 新增裸执行按钮或绕过权限的 send/resume。
- 为了截图触发真实 Codex 或读写 `/Users/yoyi/.codex`。
- 把普通浏览器 smoke 说成真实 Tauri 验收。
- 把 H6 说成阶段 H 完成。
- 把 H2/H5 单项目 probe 写成任意项目自由执行。

## 9. 推荐验证命令

按实际改动裁剪：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib h5_project_dispatch_bridge
cargo test --lib session_continuation
cargo test --lib codex_local_runner
cargo test --lib runtime_log
cargo test --lib diagnostics
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check src/h5_project_dispatch_bridge.rs src/session_continuation_store.rs src/codex_local_runner.rs src/runtime_log_store.rs src/types.rs src/commands.rs
```

如果只改前端，不需要强行跑无关 Rust 定向测试；但 evidence 必须说明裁剪原因。

## 10. 收口要求

任务完成时必须新增：

- `evidence/2026-06-08-stage-h-h6-real-execution-ui-productization-and-tauri-acceptance-v1.md`
- `handoffs/2026-06-08-stage-h-h6-real-execution-ui-productization-and-tauri-acceptance-v1-result.md`

checkpoint 同步范围：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`

不要在中间小修时反复同步入口文档；只在完成、阻断或范围改变时同步。

## 11. 不接受口径

即使 H6 完成，也不接受为：

- H3-B retry 成功。
- `new_session` 产品化完成。
- H4-Level-B 真实失败 / 超时探针完成。
- 任意项目自由执行开放。
- 自动重试 / 自动恢复 / stop / kill / restart 产品化完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 完整多 agent / 多模型协作抽象完成。
- 阶段 H 整体完成。

H6 完成后下一步应进入 H7：H 阶段最终验收和冻结。
