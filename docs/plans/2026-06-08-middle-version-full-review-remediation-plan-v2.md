# Middle Version Full Review Remediation Plan v2

日期：2026-06-08

状态：综合修补计划。本文综合两份复核反馈，不改变权威入口，不声明阶段完成，不创建执行授权；用于指导下一轮修补任务包和小修分流。

## 1. 来源和修正口径

本计划综合两份反馈：

- UI / 低风险修补复核：普通主导航已收敛，开发 / 内部入口进入设置页，首页围绕五个主对象，`result_count = null` 不再显示为空，旧 `H5 命令` 和旧 H6 口径已清理；结论为通过，带 P2 后续修补建议。
- 全量中间版本架构复核：当前代码架构可接受为中间版本 `accepted_with_deferred_items`，但不能称最终蓝图完成或通用真实多 agent 自动化工作台完成；主要架构债是真实执行入口未统一、会话正文 / rollout 读取口径需和 readback 区分、大文件职责混杂、UI 仍偏内部状态面板。

对全量复核报告的主管修正：

- H5 product command formalization / acceptance checkpoint 当前已经完成并通过主管复核，不能再按“checkpoint 未完成”理解。
- 当前代码里仍只有 `preview_h5_project_workflow_dispatch` 作为 H5 preview command，没有独立 `execute_h5...` / `run_h5...` Tauri command。
- 显式批准后的真实执行当前复用 continuation Phase B runner；旧 `execute_workflow_node_dispatch`、`run_workflow_machine` 和 CLI `__run_workflow_machine_real` 仍存在并注册。
- 因此 P1 应表述为：H5 checkpoint 已完成为产品边界收束，但统一真实执行入口 / product command routing 仍是必须修补的架构债。

## 2. 当前可接受结论

可以接受：

- 中间版本和 H / I 后续阶段可按 `accepted_with_deferred_items` 收口。
- `codex-local` 已具备受控真实 resume 和 H5 B1 / B2 单项目 probe 证据。
- H5 checkpoint 可接受为 preview / readiness / permission envelope、B1 / B2 probe 和 acceptance matrix 的产品边界收束。
- 记忆层 M1-M13 可接受为中间版治理产品化，具备正式记忆、候选、观察、lint、任务记忆包、生命周期、实体关系和成熟模式审计链。
- UI 信息架构最新修补方向可以接受，主导航和开发者入口分层比上一版清楚。

不能接受：

- 不能称最终蓝图完成。
- 不能称通用真实 send / resume 产品化全部完成。
- 不能称 H5 通用项目工作流真实派发已经开放。
- 不能称 planned adapters 已真实接入。
- 不能称 provider credential / model verification 完成。
- 不能称 H3-B new session 已成功。
- 不能称 H4-Level-B 真实失败 / 超时探针完成。
- 不能把 session transcript / rollout viewer 说成 H/H5 execution readback。

## 3. 修补原则

- 先小修误导，再动架构。
- 所有真实执行入口必须后端统一收束，前端不能直接拼或绕过产品命令。
- 旧真实执行路径不能静默删除；需要主管决策后封存、迁移或兼容代理。
- 没有执行点任务包和明确授权，不执行新的真实 `codex exec` / `codex exec resume`。
- 没有执行点任务包和明确授权，不读写 `/Users/yoyi/.codex`。
- 不读取 auth / token / secret / `.env` / keychain / OAuth / provider credential。
- 不把 UI 文案修补包装成真实 Tauri / 截图验收。
- 不频繁同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`；只在 checkpoint 收口时同步。

## 4. 修补轨道 A：P2 小修，直接执行

目标：消除仍会误导普通用户的低风险 UI / 文案问题，不改变真实执行行为。

### A1. PermissionDialog 普通动作按钮精确化

范围：

- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

修补：

- 保留真实 Codex 动作的强确认按钮，例如“确认启动多轮真实执行”“确认真实派发”。
- 移除 fallback 的宽泛“允许一次”。
- 按动作类型输出更精确按钮，例如“确认记录”“确认写入状态”“确认复制”“确认提交决定”“确认创建候选”。
- 未识别动作使用“确认继续”，不要使用“允许一次”。

验收：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- 扫描普通产品文案中不再出现误导性 `允许一次`，测试 fixture 允许保留必要断言说明。

### A2. Skill / Harness 普通页面对象化

范围：

- `prototypes/productized-desktop-shell/src/views/SkillsBoardView.tsx`
- `prototypes/productized-desktop-shell/src/views/HarnessBoardView.tsx`
- 必要时补 `styles.css`

修补：

- 首屏从“只读索引 / 字段 / 候选 / 适配器”改为用户对象视角。
- Skill 显示为“可复用能力、适用场景、最近使用、是否可用于当前项目”。
- Harness 显示为“运行器能力、可运行范围、最近运行、等待配置 / 不可用原因”。
- raw id、schema、缺字段、adapter/provider 细节放入折叠详情或 `设置 > 开发者` 指向，不在普通首屏铺开。
- `Skill`、`Harness` 英文可保留。

验收：

- 普通可见区域不再像内部状态面板。
- 不新增真实执行按钮。
- 不暗示 planned adapter / provider 已可用。

### A3. UI 全中文和开发者内容归位扫描

范围：

- `prototypes/productized-desktop-shell/src`

修补：

- 运行器相关英文可保留。
- `raw status`、`sidecar`、`schema`、`adapter descriptor`、`provider availability`、`credential`、`debug` 等内部信息默认归位到设置开发者区或详情层。
- 秘书文案保持“解释、整理、提醒、影响面”，不能像派任务、批准者或裁判。

验收：

- `rg` 扫描关键内部词在普通首屏大块文案中的残留。
- 离线交互测试覆盖右侧入口和秘书边界。

## 5. 修补轨道 B：P1 统一真实执行入口，必须另开任务包

目标：把当前并存的旧真实执行路径和 H5 / continuation 新路径收束成统一 product command routing。

这不是小修，必须单独任务包、单独复核。

### B0. 主管决策

任务包开始前必须确认：

- `__run_workflow_machine_real` CLI 是封存、保留开发者专用，还是改为统一 routing 的兼容入口。
- `execute_workflow_node_dispatch` 是保留为 deprecated wrapper，还是直接迁移为统一 command。
- `run_workflow_machine` 是保留为高级工作流机器，还是拆为 plan / preview / execute 多阶段 command。
- 是否要求所有真实执行都必须经过同一套 permission envelope、continuation、runtime log、audit、readback、duplicate guard。
- 旧 UI 按钮是否立即隐藏，还是保留但标为旧入口并强制走统一后端。

推荐决策：

- 不删除历史函数，先封存 UI 暴露面。
- 新增统一 `RealExecutionProductCommand` 或等价应用服务，旧 Tauri command 只能作为 wrapper 调用它。
- CLI `__run_workflow_machine_real` 改为开发者入口，并要求显式 danger / debug 标记；普通 UI 不直连。

### B1. 入口盘点和风险矩阵

只读列出所有真实执行入口：

- Tauri command
- CLI
- 前端 invoke wrapper
- UI 按钮
- 测试 fixture
- ignored real probe

输出：

- 每个入口是否真实执行。
- 是否读写 `/Users/yoyi/.codex`。
- 是否发送 prompt。
- 是否写 project files。
- 是否写 workbench state。
- 是否有 permission envelope。
- 是否写 runtime log / audit / readback。
- 是否有 duplicate guard。

### B2. 统一 product command 契约

定义统一阶段：

```text
prepare / preview
-> permission envelope
-> explicit user decision
-> execute
-> runtime log
-> readback
-> worker report candidate
-> project director process fact decision
-> final review / handoff
```

必须包含：

- request id / attempt id / continuation id。
- project / workflow / node / work item / task package / memory packet 绑定。
- adapter id 和 operation id。
- prompt summary / prompt ref / prompt hash，不把完整 prompt 拼进 argv 或 evidence。
- allowed write roots / denied paths。
- `.codex` 最小授权范围。
- readback plan 和 `result_count = null` 规则。
- runtime log ref。
- audit ref。
- failure classification。
- user rejected / blocked_by_guard / duplicate_blocked / timed_out / readback_failed 等状态。

### B3. 代码迁移

候选落点：

- `src-tauri/src/codex_local_runner.rs`
- `src-tauri/src/session_continuation_store.rs`
- 新增 `src-tauri/src/real_execution_command.rs` 或等价模块
- `src-tauri/src/commands.rs`
- `src-tauri/src/types.rs`

迁移方向：

- `preview_h5_project_workflow_dispatch` 继续保持非执行 preview。
- 显式执行统一进入 continuation Phase B / real execution product service。
- `execute_workflow_node_dispatch` 改为 wrapper，不能直接绕过统一 permission / runtime / audit / readback 契约。
- `run_workflow_machine` 改为 wrapper 或拆成多阶段 command。
- CLI 入口不得成为普通产品路径的隐形后门。

### B4. 测试矩阵

必须覆盖：

- preview 不执行真实 Codex。
- user rejected 不调用 runner，`result_count = null`。
- guard blocked 不调用 runner。
- diagnostics blocked 不调用 runner。
- duplicate blocked 不调用 runner。
- stale memory blocked。
- secret / `.env` / token / rollout / full transcript 默认禁读。
- readback unavailable / failed / timed out 均不显示为 0。
- successful fake runner 写 continuation / runtime log / audit / readback refs。
- old command wrapper 不再绕过统一服务。
- ignored real probe 仍需要显式授权，默认测试不跑。

验证：

```text
cargo test --lib h5_project_dispatch_bridge
cargo test --lib session_continuation
cargo test --lib codex_local_runner
cargo test --lib runtime_log
cargo test --lib diagnostics
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check ...
npm run typecheck
npm run test:offline-interaction
npm run build
```

## 6. 修补轨道 C：session transcript / readback 口径隔离

目标：避免“会话正文读取能力”和“H/H5 真实执行 readback”在产品口径中混同。

修补：

- 在 Agent 页把 session transcript viewer 明确标为“会话查看 / 历史内容查看”。
- H/H5 readback 显示为“执行读回 / 结果读回”，只展示 readback plan 允许的摘要。
- transcript / rollout 查看默认进入详情或开发者态，不在普通执行结果中冒充 worker report。
- 如果读取 transcript / rollout，需要清楚显示来源、范围、敏感内容 warning 和不等于正式事实。
- 复核所有文档和 UI 文案，避免写“完全不读 full transcript / rollout”这种绝对口径；正确口径是“真实执行链路默认不得读取，除非另有显式查看授权和边界”。

验收：

- UI 中 transcript viewer 和 execution readback 的标题、说明、状态字段不同。
- readback failed / unavailable 不显示为 0。
- 相关测试覆盖至少一个 transcript viewer 不等于 readback 的文案断言。

## 7. 修补轨道 D：UI Shell 产品化，任务包执行

目标：把中间版 UI 从阶段状态面板推进到日常可用的桌面工作台 Shell。

原则：

- 不改风格，保持当前 inkwash 方向。
- 只做桌面端，不做手机端。
- 普通主界面服务五个对象：项目、智能体、Skill、Harness、运行中工作流。
- 开发 / 内部边界信息进入 `设置 > 开发者`。
- 管理入口承载审计、日志、诊断、健康状态和数据位置。
- 秘书只解释、整理、提醒、说明影响面。

建议拆分：

- D1：首页和左侧导航稳定化。
- D2：项目页从“状态堆叠”改为“项目工作流 Shell”，节点详情只展示摘要，raw evidence / handoff / id 进详情。
- D3：智能体页把会话、adapter、执行边界、readback、transcript viewer 分层。
- D4：Skill / Harness 对象化。
- D5：设置 > 开发者汇总 raw status、sidecar、schema、adapter/provider 细节。
- D6：右侧秘书 / 通知 / 待办 / 运行中 / 管理职责扫描和文案收口。

验收：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- 普通浏览器 smoke 只能作为辅助。
- 真实 Tauri / 截图验收若要声明完成，必须另有任务包和截图证据。

## 8. 修补轨道 E：大文件拆分和可审计性

目标：降低后续统一真实执行入口和 UI Shell 继续开发的审计成本。

必须另开 refactor 任务包，不能夹在功能修补里偷偷做。

建议优先级：

- `lib.rs`：迁出真实 runner、CLI、read model builder、测试 fixture，保留 app bootstrap 和 handler 注册。
- `ProjectsView.tsx`：拆出项目工作流侧栏、权限区、任务包摘要、真实派发区、记忆区、节点详情区。
- `AgentView.tsx`：拆出 session list、conversation viewer、adapter boundary、continuation / real execution panel、readback panel、transcript viewer。
- `offline-permission-dialog.test.tsx`：按 UI 区域和后端边界分文件，避免一个测试文件承载所有阶段历史。

验收：

- 行为不变。
- 相关测试先跑 before，再跑 after。
- 不新增真实执行。
- 不修改 workflow state schema / sidecar schema，除非任务包明确包含 migration。

## 9. 修补顺序

推荐顺序：

1. A1-A3：低风险 UI / 文案小修，直接执行，不同步权威入口。
2. B0-B2：统一真实执行入口任务包设计和主管决策，不改产品代码。
3. B3-B4：统一 product command routing Level A 实现，默认 fake / no-op / unit test，不真实执行。
4. C：transcript / readback 口径隔离，跟 B 并行或紧随 B。
5. D：UI Shell 产品化第二轮，保持 inkwash 风格，只重排信息层级。
6. E：大文件拆分，在 B/D 稳定后做，避免同时改行为和结构。
7. 如需新的真实 probe，另开 Level B 执行任务包，只在测试项目或 `mario test` 明确授权范围内执行。

## 10. 主管决策清单

下一轮开始前需要确认：

- 是否按推荐方案把旧真实执行入口改成统一 product command wrapper。
- 是否保留 `__run_workflow_machine_real`，若保留，是否只允许开发者 / CLI 明确 danger 模式。
- H3-B 是否需要 retry；如果需要，是否允许再次真实 `codex exec` 和写 `/Users/yoyi/.codex`。
- H4-Level-B 是否需要真实失败 / 超时探针。
- UI Shell 是否先做 Skill / Harness 对象化，还是先做项目 / 智能体主页面拆分。
- 是否接受 checkpoint 节奏：小修不更新权威入口，只有 B/D/E 这类阶段性收口才同步入口文档。

## 11. 不得冒领

执行本计划期间不得声称：

- UI 文案修补等于真实执行产品化。
- H5 checkpoint 等于 H5 通用项目派发完成。
- B1/B2 `mario test` probe 等于任意项目可自由执行。
- planned adapters descriptor 等于真实接入。
- provider availability 等于 credential / model 已验证。
- session transcript viewer 等于 execution readback。
- readback unavailable / failed / timed out 等于 0 条结果。
- fake / no-op / unit test 等于真实 Codex 执行。
- 普通浏览器 smoke 等于真实 Tauri 截图验收。

## 12. 分工修补最终收口记录

日期：2026-06-09

结论：本轮按多会话分工完成修补计划 v2 的 Level A / D 范围，可接受为“通过，带 P2 / 后续统一 product command routing 债”。主管线复核和独立只读复核线均未发现 P0 / P1 阻断。

已完成范围：

- UI Shell 线完成 D1-D6 低风险信息层级收敛：普通主导航为 `项目 / 智能体 / Skill / Harness / 运行中工作流`，设置固定底部，开发 / 内部入口进入 `设置 > 开发者`；首页围绕五个主对象；右栏职责化为通知摘要 / 待处理事项 / 管理摘要 / 运行中摘要；秘书只解释、整理、提醒和说明影响面；Skill / Harness 首屏对象化；项目页 / 智能体页仅做低风险文案与折叠分层。
- 真实执行入口线完成 B / C 的 Level A 收束：新增 `src-tauri/src/real_execution_command.rs` 并在 `lib.rs` 注册；旧 `execute_workflow_node_dispatch`、`run_workflow_machine`、`read_workflow_node_dispatch_result` Tauri wrapper 和 CLI `__run_workflow_machine_real` 返回 `legacy_product_command_blocked`，不启动旧真实 runner。
- MCP canvas 普通入口继续封存：`canvas_start_run` / `canvas_tick_run` 入口返回 `mcp_canvas_real_execution_blocked...`，不进入 `OrchestratorState.start_run` / `tick`；`CanvasView` 不再导入或调用 `canvasStartRun` / `canvasTickRun`。
- Phase B / H3-B runner 前新增统一 `decide_real_execution_command` gate：默认 blocked / user rejected / duplicate / guard / diagnostics / stale memory / readback missing 均不调用 runner；显式授权路径仍保留 ignored real probe 能力，但本轮未执行。
- H5 preview 保持非真实：`prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`。
- session transcript viewer 与 execution readback 已隔离：后端 `viewer_boundary.is_execution_readback=false`，前端显示“会话历史查看；不是 H/H5 执行读回”；readback unavailable / failed / timed out 不显示为 0 或空。

主管线验证记录：

- `cargo test --lib`：272 passed / 5 ignored；保留既有 `mcp/protocol.rs::invalid_params` dead-code warning。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，13 passed。
- `npm run build`：通过，仅既有 Vite chunk size warning。

独立只读复核结论：

- P0：无。
- P1：无。
- 验收建议：可接受为“修补计划 v2 Level A / D 完成，通过，带 P2 后续债”。
- 复核确认旧 Tauri / CLI / MCP 普通产品入口不能绕过 boundary / gate 触发真实 runner；Phase B / H3-B gate 位于 runner 调用前；preflight / Phase A 非执行语义未被污染；H5 preview 未变成真实执行；transcript viewer 未冒充 execution readback；UI D1-D6 可接受为低风险信息层级修补。

仍保留为 P2 / 后续债：

- `App.tsx` 仍通过 deprecated alias 名 `executeWorkflowNodeDispatch` / `runWorkflowMachine` 调用旧 wrapper；后端已 guard，notice 会追加旧入口边界，因此不构成 P1，但后续应改为显式 `executeLegacy...` / `runLegacy...` 或隐藏旧入口。
- 内部真实 runner 定义仍保留，包括 `RealCodexResumeRunner`、`mcp/codex_runner.rs` 和 `RealCodexLocalPhaseBProcessRunner`；普通入口已封或需 gate，但后续统一 product command routing 任务包必须继续盘点并归口。
- 旧内部 helper `execute_workflow_node_dispatch_for_index_at` / `run_workflow_machine_for_index_at` 仍保留用于测试 / 历史路径；后续应封存、迁移或纳入统一产品命令服务。
- 仍不能声明所有真实执行 runner 已删除、所有真实执行入口已统一、H 阶段通用 product command routing 已完成、H5 通用真实派发已开放或通用真实 send / resume 产品化完成。

边界记录：

- 本轮主管线未执行真实 `codex exec` / `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 auth / token / secret / `.env` / keychain / OAuth / provider credential / full transcript / rollout。
- 未启动 Browser / Chrome / Tauri / Vite preview / 截图工具。
- UI 线早前 Browser smoke 读取 `.codex/plugins/cache/.../browser-client.mjs` 属于已记录过程偏差；本轮 D1-D6 最新修补、真实执行线修补、主管线验证和独立复核均未重复该偏差。
