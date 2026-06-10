# Unified Product Command Routing PCR0 Entry Matrix And Supervisor Decision Freeze v1

日期：2026-06-09

状态：已完成。

全局主管任务。本文是统一 Product Command Routing 开发的 PCR0 任务包，用于冻结真实执行入口矩阵、sidecar / store 决策、分线边界和后续 PCR1-PCR10 的开发前置条件。本文不授权真实 `codex exec` / `codex exec resume`，不授权发送 prompt，不授权读写 `/Users/yoyi/.codex`，不改产品代码，不同步权威入口。

## 0. 全局主管理解

已知事实：

- H2 Phase B、H5-Level-B1、H5-Level-B2 已证明 `mario test` 指定范围内真实 `codex exec resume` 可以受控执行，并留下 runtime log / audit / readback 证据。
- H3-B 已执行一次真实 new-session fixture probe，但失败并完成分类；产品路径已补 `--skip-git-repo-check`，未二次授权 retry。
- H4 Level A 已完成 readback / failure / timeout / duplicate guard 非真实产品边界。
- H5 checkpoint 已完成 preview / readiness / permission envelope / B1-B2 probe / acceptance matrix 的产品边界收束。
- 修补计划 v2 已完成 Level A / D 收束：旧 Tauri / CLI / MCP canvas 普通入口已 guard，Phase B / H3-B runner 前已有统一 gate，UI 信息层级已初步收敛。
- 当前仍不能声明通用真实 send / resume 产品化完成，也不能声明所有真实执行入口已统一。

PCR0 收口后已冻结：

- 新增 `real-execution-product-commands.v1.json` sidecar 的方向，用于 PCR1 类型 / store skeleton / read model。
- 旧 `execute_workflow_node_dispatch`、`run_workflow_machine`、`read_workflow_node_dispatch_result`、`__run_workflow_machine_real` 先保持 legacy / sealed / blocked，不作为普通产品入口。
- `mcp/codex_runner.rs` 与 H3-B internal runner path 先作为内部 runner / ignored probe path 纳入后续 PCR 盘点，不在 PCR1 暴露为普通入口。

后续仍待冻结：

- `run_workflow_machine` 是否在 PCR5 拆成 plan / preview / execute 多阶段 product command。
- UI 是否在 PCR6 前显示真实执行按钮，还是仅显示 readiness / permission / blocked 状态。

本任务假设：

- PCR0 只做任务包 / 入口矩阵 / 主管决策冻结 / 只读复核，不做代码实现。
- 默认推荐新增 product command read model；是否新增 sidecar 在本任务内定稿。
- 默认不做 H3-B retry；任何 retry 必须作为 PCR9 或独立 Level B 任务包重新授权。

## 1. 权威依据

必须读取并服从：

- `docs/plans/2026-06-09-unified-product-command-routing-development-plan-v1.md`
- `docs/plans/2026-06-08-middle-version-full-review-remediation-plan-v2.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `evidence/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md`
- `handoffs/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1-result.md`
- `evidence/2026-06-08-middle-version-full-review-remediation-plan-v2.md` 如后续存在；若不存在，以修补计划 v2 第 12 节为准。

必须只读参考的代码入口：

- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/codex_runner.rs`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`

## 2. 目标

PCR0 目标：

1. 冻结真实执行入口矩阵，覆盖 Tauri command、CLI、MCP、front-end invoke wrapper、UI action、test helper、ignored real probe。
2. 冻结每个入口的目标处理：统一迁移、legacy wrapper、developer-only、test-only、永久封存。
3. 冻结 product command store 决策：新增 sidecar、派生读模型，或混合方案。
4. 冻结 PCR1-PCR10 的分线工作边界，避免多线同时改同一核心文件。
5. 输出 PCR1 后端契约任务包所需的最低信息：类型边界、store 决策、旧入口处理策略、测试矩阵。
6. 由独立复核线只读确认矩阵完整性和 P0/P1 风险。

## 3. 非目标

PCR0 不做：

- 不改 Rust / TS / React 产品代码。
- 不新增 sidecar 文件。
- 不迁移 workflow state schema。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth、token、secret、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 不启动 Browser / Chrome / Tauri / Vite preview / screenshot。
- 不同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`；除非全局主管在 PCR0 收口时明确作为 checkpoint 同步。
- 不写 PCR1-PCR10 的产品代码。

## 4. 分线职责

### 主管线

- 创建并维护本任务包。
- 冻结入口矩阵和决策建议。
- 派发复核线做只读审查。
- 根据复核线回交决定是否进入 PCR1。
- 控制 checkpoint 文档同步时机。

### 复核线

- 只读检查入口矩阵。
- 查普通产品入口是否仍可达真实 runner。
- 查每个入口是否已标明写 `.codex`、发送 prompt、写项目文件、写 workflow state、写 runtime log、写 audit、readback、duplicate guard。
- 回交 P0/P1/P2 和是否可进入 PCR1。

### 后端线

- PCR0 期间不写代码。
- 可只读准备 PCR1 的类型 / store 方案建议。
- 等 PCR0 收口后再进入 PCR1。

### UI 线

- PCR0 期间不写代码。
- 可只读准备 PCR6 的 UI 链路草图。
- 不改 `App.tsx` / `ProjectsView.tsx` / `AgentView.tsx`。

### 真实探针线

- PCR0 期间不执行。
- 不继承 H2/H5 的旧授权。
- 等 PCR9 单独授权。

## 5. 入口矩阵模板

复核线和主管线必须补齐下表。若不确定，标 `unknown`，不得猜测。

| 入口 | 类型 | 当前状态 | 是否真实执行 | 是否发送 prompt | 是否写 `.codex` | 是否写项目文件 | 是否写 workbench state | permission envelope | runtime log | audit | readback | duplicate guard | 目标处理 | 风险 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `preview_h5_project_workflow_dispatch` | Tauri / H5 preview | 已存在 | false | false | false | false | false | preview | preview | preview | boundary | yes | 保留为 prepare / preview 来源 | 低 |
| `confirm_controlled_session_continuation` | Tauri / continuation | 已存在 | false | false | false | writes continuation | true | decision | no | yes | no | n/a | 纳入 user decision | 中 |
| `inspect_controlled_session_continuation_real_resume_authorization` | Tauri / H2 preflight inspect | 已注册 | false | false | false | false | true | preflight inspect | no | yes | no raw transcript | n/a | 保留为 authorization / guard inspect，不得归类为 execute | 中 |
| `run_controlled_session_continuation_real_resume_phase_a` | Tauri / H2 Phase A no-real runner path | 已注册 | false | false | false | false | true | phase_a authorization | no real runner log | yes | boundary only | n/a | 保留为 Phase A 结构化 runner path 记录，不得升级为真实执行 | 中 |
| `run_controlled_session_continuation_real_resume_phase_b` | Tauri / runner | 已存在 gated | true when authorized | true when authorized | true when authorized | per request | true | yes | yes | yes | yes | yes | 只能由 unified command execute 调度 | 高 |
| H3-B new-session runner | session store / internal runner / ignored probe path | 已存在 gated；未确认普通 Tauri command 注册 | true when explicitly authorized | true when explicitly authorized | true when explicitly authorized | per request | true | yes | yes | yes | yes | yes | 只能由 unified command execute 调度；PCR1 不能把它当普通 UI/Tauri 入口 | 高 |
| `execute_workflow_node_dispatch` | Tauri legacy | wrapper blocked | false from wrapper | false from wrapper | false from wrapper | false from wrapper | false from wrapper | no | no | no | no | no | legacy wrapper / hide / migrate | 高 |
| `run_workflow_machine` | Tauri legacy | wrapper blocked | false from wrapper | false from wrapper | false from wrapper | false from wrapper | false from wrapper | no | no | no | no | no | split / legacy wrapper / developer-only | 高 |
| `read_workflow_node_dispatch_result` | Tauri legacy readback | wrapper blocked | false | false | false | false | false | no | no | no | legacy stats | no | legacy-only; not H/H5 readback | 中 |
| `__run_workflow_machine_real` | CLI legacy | blocked | false from CLI | false from CLI | false from CLI | false from CLI | false from CLI | no | no | no | no | no | developer-only danger or permanent sealed | 高 |
| `canvas_start_run` | MCP canvas legacy | command blocked | false from command | false from command | false from command | false from command | false from command | no | no | no | no | unknown | sealed until migrated | 高 |
| `canvas_tick_run` | MCP canvas legacy | command blocked | false from command | false from command | false from command | false from command | false from command | no | no | no | no | unknown | sealed until migrated | 高 |
| `mcp/codex_runner.rs` | internal runner | exists | true if called | true if called | true if called | per cwd | no | no | no | no | no | unknown | internal runner only; must not be ordinary entry | 高 |
| ignored real probes | tests | ignored / env-gated | true if explicitly run | true | true | per fixture | true | evidence/task | yes | yes | yes | per test | test-only | 高 |

## 6. 主管决策建议

PCR0 默认建议冻结为：

1. 新增 product command read model 和服务契约。
2. 新增 `real-execution-product-commands.v1.json` sidecar 作为产品层 command / decision / attempt refs 存储；继续使用 `session-continuations.v1.json` 保存会话层 continuation / attempts。
3. `execute_workflow_node_dispatch` 保留为 legacy wrapper，普通 UI 不再直接使用；后续 PCR5 迁移为显式 `executeLegacy...` 或隐藏。
4. `run_workflow_machine` 保留为 legacy / developer-only，不作为普通产品入口；是否拆成 plan / preview / execute 在 PCR5 再冻结。
5. `__run_workflow_machine_real` 默认永久 sealed；如后续保留，必须 developer mode + danger flag + product command id + audit ref。
6. `canvas_start_run` / `canvas_tick_run` 保持 sealed；MCP canvas 真实 run 不进入本轮 product command 首批迁移。
7. `run_controlled_session_continuation_real_resume_phase_b` 和 H3-B runner 是真实执行 adapter 路径，不是产品入口；PCR4/PCR9 才能调度。
8. `inspect_controlled_session_continuation_real_resume_authorization` 和 `run_controlled_session_continuation_real_resume_phase_a` 是 inspect / no-real Phase A 路径，PCR1 可以引用它们的 guard / preview 语义，但不得把它们归类为 `execute`。
9. PCR1-PCR8 只做 Level A / fake / no-op / tests，不执行真实 Codex。
10. PCR9 才能做 Level B，且必须单独授权。

如全局主管最终选择不同方案，必须在 PCR0 收口记录中写明理由。

## 7. PCR1 输入要求

PCR0 通过后，PCR1 后端线必须拿到：

- 冻结后的入口矩阵。
- product command store 决策。
- `RealExecutionProductCommandRequest` 最小字段。
- `RealExecutionProductCommandPreview` 最小字段。
- `RealExecutionProductCommandDecision` 最小字段。
- `RealExecutionProductCommandAttempt` 最小字段。
- 旧入口处理策略。
- 不能修改的文件 / schema 边界。
- 测试矩阵。
- 明确区分 `prepare / inspect / phase_a_no_real` 与 `execute`；PCR1 后端契约不得让任一 inspect / Phase A command 调真实 runner。
- 明确 H3-B 当前先按 internal / ignored probe runner path 处理，除非 PCR1 只读核实到正式 Tauri command 注册，否则不得暴露为普通产品入口。

## 8. 只读复核要求

复核线必须只用 `rg` / `sed` / `awk` / `find` 等只读命令核对：

- 是否漏掉真实执行入口。
- `Command::new("codex")` 是否只在 runner adapter / legacy runner 定义中出现。
- 旧 Tauri / CLI / MCP wrapper 是否仍 blocked。
- inspect / preflight / Phase A 非真实入口是否已纳入矩阵，且未被误称为 execute。
- `App.tsx` 是否仍通过旧 alias 调用 legacy wrapper。
- Phase B / H3-B gate 是否仍在 runner 调用前。
- H5 preview 是否仍三项 false。
- transcript viewer 是否仍不是 execution readback。
- ignored real probes 是否仍 ignored / env gated。
- 是否存在普通 UI 文案把旧入口称为 H5 unified command。

复核线不得改文件、不得跑 npm / cargo，除非主管线明确要求。

## 9. 验收命令

PCR0 默认不需要运行产品测试，因为不改产品代码。主管线可做轻量文档检查：

```text
rg -n "状态：待执行|PCR0|不授权真实|/Users/yoyi/.codex|入口矩阵" tasks/2026-06-09-unified-product-command-routing-pcr0-entry-matrix-and-supervisor-decision-freeze-v1.md
```

如果 PCR0 后续补了入口文档，再按补改范围做对应扫描。

## 10. 完成标准

PCR0 完成必须满足：

- 入口矩阵已冻结。
- sidecar / store 决策已冻结。
- 旧入口目标处理已冻结。
- 分线职责已冻结。
- 复核线只读回交无 P0/P1，或 P0/P1 已转为阻断项。
- PCR1 是否可以开始有明确结论。

## 11. 不得声明

PCR0 完成后仍不得声明：

- 统一 product command routing 已实现。
- 通用真实 send / resume 产品化完成。
- H5 通用真实派发已开放。
- H3-B retry 已授权或成功。
- 所有真实 runner 已删除。
- planned adapters 已真实执行。
- provider credential / model verification 完成。
- 任意项目自由执行。

## 12. 回交格式

复核线回交应包含：

1. P0 / P1 / P2。
2. 入口矩阵是否完整。
3. sidecar / store 决策是否有风险。
4. 旧入口目标处理是否足以进入 PCR1。
5. PCR1 开发线应重点注意的文件和测试。
6. 不得声称完成的事项。

主管线收口应包含：

1. 最终入口矩阵。
2. 最终主管决策。
3. 复核线结论。
4. 是否进入 PCR1。
5. 是否同步 checkpoint 入口文档。

## 13. PCR0 收口记录

收口时间：2026-06-09。

主管线最终决策：

1. 冻结入口矩阵，以第 5 节为准。
2. 采用新增 `real-execution-product-commands.v1.json` sidecar 的方向进入 PCR1；PCR1 先做类型 / store skeleton / read model，不执行真实 Codex。
3. `execute_workflow_node_dispatch`、`run_workflow_machine`、`read_workflow_node_dispatch_result` 保持 legacy wrapper / blocked；PCR5 再迁移普通 UI alias 或隐藏。
4. `__run_workflow_machine_real` 默认 sealed；如保留必须 developer mode + danger flag + product command id + audit ref。
5. `canvas_start_run` / `canvas_tick_run` 保持 sealed；MCP canvas 真实 run 不进入首批 product command 迁移。
6. `inspect_controlled_session_continuation_real_resume_authorization` 与 `run_controlled_session_continuation_real_resume_phase_a` 只归类为 inspect / no-real Phase A，不得被 PCR1 当成 execute。
7. `run_controlled_session_continuation_real_resume_phase_b` 与 H3-B internal runner path 只能作为后续 execute adapter path，由统一 product command 调度；PCR1-PCR8 不执行真实 Codex。
8. PCR9 Level B 必须单独授权，不继承 H2/H5 旧授权。

复核线结论：

- 独立复核线 `019ea33a-23c4-7c10-8db3-95b8cf910fe7` 最终回交：无 P0 / 无 P1，可以开始 PCR1。
- 复核线 P2 建议：补充 preflight / Phase A 非真实入口、修正 H3-B 当前不是普通 Tauri command、拆明内部 runner 定义、App 旧 alias 后续 PCR5 迁移。
- 主管线已把 preflight / Phase A 和 H3-B 分类补入第 5 / 6 / 7 / 8 节；内部 runner 与 App alias 作为 PCR1/PCR5 后续重点保留。

PCR1 结论：

- 允许进入 PCR1 后端契约和只读模型开发。
- PCR1 任务包：`tasks/2026-06-09-unified-product-command-routing-pcr1-backend-contract-and-read-model-v1.md`。
- PCR1 状态为“待 PCR0 收口后执行”，本收口后可由后端线开始执行。

文档同步决策：

- 本轮不同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。
- 统一 Product Command Routing 属于开发过程内 checkpoint；权威入口等 PCR8 或 PCR10 checkpoint 再集中同步，避免每个小段都消耗入口文档维护成本。

本轮边界：

- 未改产品代码。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 未启动 Browser / Chrome / Tauri / Vite preview / screenshot。

过程备注：

- 主管线文档检查中有一条 `rg` 命令因 shell 双引号内 Markdown 反引号触发了字面量 `execute` 命令替换，结果为 `command not found: execute`。该偏差未触发 Codex、未访问 `.codex`、未改变产品文件；后续扫描改用单引号或避免反引号。
