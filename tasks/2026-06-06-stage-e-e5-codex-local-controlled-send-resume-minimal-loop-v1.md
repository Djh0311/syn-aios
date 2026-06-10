# Task Package：Stage E / E5 Codex-local Controlled Send Resume Minimal Loop v1

状态：已完成 Level A（代码路径、guard、stub、工作台自有 continuation sidecar、只读 UI 和离线验收）；Level B 真实执行未授权、未执行。  
完成记录：见 `evidence/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md` 与 `handoffs/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1-result.md`。  
用途：在 E4 会话继续协议 / 权限预览之上，实现 `codex-local` 受控 send / resume 最小闭环。E5 是中间版本“发消息继续会话”的最小实现任务，但不是通用会话控制器，不支持 planned adapters，不支持 stop / restart / delete / export / favorite。  
执行方式：分级执行。默认只允许完成代码路径、guard、stub / dry-run、工作台自有 continuation 记录和离线验收；如要真实执行 `codex exec resume` 或写 `/Users/yoyi/.codex`，必须在执行前另行取得用户对具体读写范围、目标 session、cwd、prompt、回滚和证据的明确批准。

## 0. 先说薄弱点

- E5 是阶段 E 第一项接近真实执行的任务，风险显著高于 E1-E4；如果边界不硬，容易把“会话继续最小闭环”滑成无限制会话控制器。
- E4 已有 preview / guard，但没有真实发送；E5 可以复用 E4 协议，不能绕过 E4 直接调用历史 workflow dispatch 或 `codex exec resume`。
- 历史上曾发生过搜索命令反引号误触发 `codex exec resume` 的过程偏差；E5 必须把 shell 安全、搜索写法和真实执行审批写进任务包硬规则。
- 真实 `codex exec resume` 会触碰 Codex 原生状态，通常涉及 `/Users/yoyi/.codex`；这不是普通前端按钮或本地 read model，可以默认执行。
- E5 必须绑定 project / workflow / node / session；不允许自由聊天输入框绕过项目、任务包、权限、记忆包和审计。
- E5 如无真实执行授权，只能回收为“代码路径 / guard / stub 验收完成，真实执行待授权或 G 阶段补证据”，不能声称真实会话继续已验收。
- planned adapters 仍不可执行；E5 只允许 `codex-local`。
- GEPA / Paseo / Odysseus 研究资料仍只作为蓝图参考；E5 不吸收优化器、daemon、workspace 复刻或外部项目融合点。

## 1. 已知事实 / 未知 / 假设

已知事实：

- 阶段 C / C1-C6 已完成，接受为受控自动化工作流闭环。
- 阶段 D / M1-M13 已完成，M13 结论为 `accepted_with_deferred_items`。
- 阶段 E / E1 已完成，`WorkbenchSnapshot.agent_adapters[]` 可区分 `codex-local` 和 planned adapters。
- 阶段 E / E2 已完成，`WorkbenchSnapshot.session_operations[]` 覆盖会话操作边界，但不执行真实操作。
- 阶段 E / E3 已完成，`WorkbenchSnapshot.provider_availability[]` 提供 provider / model / credential 只读边界。
- 阶段 E / E4 已完成，`WorkbenchSnapshot.session_continuation_previews[]` 提供 `SessionContinuationRequest` / `SessionContinuationPreview` / `SessionContinuationGuardResult` 等价模型和 preview-only UI。
- E4 明确不接受为真实 send / resume、prompt 已发送、attempt / dispatch / readback 已写入。
- 当前默认禁止执行真实 `codex exec` / `codex exec resume`，也禁止默认读写 `/Users/yoyi/.codex`。
- E4 收尾有一次 shell 反引号命令替换误触发 `codex exec resume` 的过程偏差；后续搜索含反引号文本必须用单引号或 `rg -F`。
- UI 任务包必须落实 `docs/plans/task-package-ui-display-boundary-rule-v1.md` 的“UI 显示边界确认”章节。

未知：

- 真实执行授权是否会在本任务执行前获得。
- E5 continuation 记录最终应复用现有 workflow state 的 execution_attempts / audit，还是新增独立 sidecar。
- readback 的第一版是否在 E5 内读取，还是只记录 readback unavailable 并留给 E6 / G1 深化。
- 真实 `codex exec resume` 的 sandbox、cwd、timeout、stdin 关闭、失败恢复和 readback 策略是否完全复用既有 workflow dispatch runner。
- 真实 Tauri 截图验收工具是否可用。

本任务采用的假设：

- E5 默认执行级别是 Level A：代码路径 / guard / stub / dry-run / 工作台自有记录，不真实执行 Codex。
- Level B：真实 `codex exec resume` 验收必须单独取得用户明确批准；未获批准时不能执行，也不能把 E5 回收为真实会话继续完成。
- E5 可以新增工作台自有最小 continuation sidecar 或等价记录，用来记录 permission、attempt、readback summary、audit ref 和状态；如果实现者选择写入既有 workflow state，必须证明不改顶层结构、不破坏 C1-C6 语义。
- E5 不能读取真实完整 transcript 作为开发证据；如 readback 需要真实 transcript，必须在 Level B 授权里列明。
- 如果实现者发现必须支持 planned adapter、自由聊天、stop / restart / delete / export / favorite、读取 secret 或无审计执行，必须停下回传。

## 2. 执行级别和用户授权

E5 分两级：

### Level A：默认允许范围

可以完成：

- E5 后端 command / control core / store / read model / UI 路径。
- stub runner 或 dry-run runner。
- E4 preview -> E5 confirmed continuation 的状态转换。
- 工作台自有 continuation attempt / permission / audit / readback placeholder 记录。
- UI 显示 waiting / running-stub / succeeded-stub / failed-stub / readback unavailable 等非真实执行状态。
- 离线测试、Rust 单测、前端测试、禁止文案扫描。

不能声称：

- 真实 `codex exec resume` 已执行。
- Codex 已收到 prompt。
- 真实 readback 已完成。
- 真实 worker / agent 已执行。

### Level B：需要用户明确批准

只有用户明确批准后，才能做：

- 执行真实 `codex exec resume`。
- 读写 `/Users/yoyi/.codex` 或触碰 Codex 原生状态。
- 对真实 session 发送 prompt。
- 读取真实 transcript / rollout 做 readback。
- 把真实执行结果写入 continuation attempt / readback / audit。

执行前必须单独列出并获得批准：

- 目标 project / workflow / node / session。
- 目标 cwd 和 allowed write roots。
- sandbox、timeout、runner 命令预览。
- prompt summary 和完整 prompt 存放 / 展示方式。
- 会触碰的 `/Users/yoyi/.codex` 范围。
- 会写入的工作台状态文件 / sidecar。
- 备份和回滚方式。
- readback 策略和 readback unavailable 处理。
- 验收证据和真实 Tauri / 截图是否纳入。

没有 Level B 批准时，E5 最多回收为：

```text
accepted_stub_or_code_path_only
```

不能回收为：

```text
real_session_continuation_accepted
```

## 3. 任务目标

完成阶段 E 第五刀：

```text
E4 preview / guard
-> user confirmation boundary
-> codex-local only continuation request
-> controlled runner abstraction
-> workbench-owned continuation attempt / permission / readback / audit record
-> UI status for queued / waiting_permission / running / succeeded / failed / timed_out / readback_unavailable
-> evidence + handoff
```

E5 完成后，在 Level A 下可以说：

- 工作台已有 `codex-local` 受控 send / resume 最小代码路径。
- E4 preview 可进入 E5 受控确认和 continuation attempt 记录。
- guard、权限、prompt preview、runner abstraction、readback summary placeholder 和 audit ref 已形成闭环。
- UI 能解释状态：待确认、准备中、stub 运行中、stub 成功 / 失败、readback unavailable。
- planned adapters、越界 cwd、缺 binding、缺 readback strategy 等仍被阻断。

E5 完成后，在 Level B 且真实授权执行通过时，才可以额外说：

- 指定 `codex-local` 会话完成了一次受控真实 send / resume 验收。
- 指定 prompt 已经通过授权路径发送。
- 工作台自有 continuation attempt / audit / readback 记录可追溯。

E5 完成后仍不能说：

- 通用会话中心自由发消息完成。
- Claude Code / OpenClaw / OpenCode / OpenCode-like send / resume 完成。
- stop / restart / delete / export / favorite 完成。
- 自动重试、取消恢复、完整 runtime log 完成。
- 阶段 E 完成；E6/E7 仍未完成。
- 阶段 G 真实 Tauri 全面验收完成，除非后续 G 阶段单独验收。

## 4. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`

UI 边界：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`

阶段 E 前置：

- `tasks/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`
- `tasks/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`
- `tasks/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`
- `tasks/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`
- `evidence/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`
- `handoffs/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1-result.md`

会话 / 工作流 / runner 前置：

- `tasks/2026-06-03-session-center-foundation-hardening-v1.md`
- `evidence/2026-06-03-session-center-foundation-hardening-v1.md`
- `handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`
- `tasks/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`
- `evidence/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`
- `handoffs/2026-06-03-workflow-dispatch-readback-native-parser-v1-result.md`
- `tasks/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `evidence/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `handoffs/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1-result.md`

过程偏差前置：

- `evidence/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md` 第 2 节偏差记录。
- `handoffs/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1-result.md` 第 2 节偏差记录。

主要代码入口：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/sessionContinuation.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

搜索固定文本必须用 `rg -F '...'` 或单引号，禁止用 shell 双引号包住未转义反引号。

## 5. 范围

允许：

- 新增或扩展类型：
  - `ControlledSessionContinuation`
  - `SessionContinuationAttempt`
  - `SessionContinuationRunStatus`
  - `SessionContinuationRunnerKind`
  - `SessionContinuationReadbackSummary`
  - `SessionContinuationAuditRef`
  - `SessionContinuationDecision`
- 新增工作台自有最小 continuation sidecar，例如 `session-continuations.v1.json`，前提是必须有 revision、lock、backup、atomic write、corrupt JSON refuse overwrite 和 sidecar 路径说明。
- 或复用既有 workflow state 内已有 execution / audit 结构，前提是不新增顶层结构、不改状态枚举、不破坏 C1-C6。
- 新增 Tauri command，例如：
  - `prepare_controlled_session_continuation`
  - `confirm_controlled_session_continuation`
  - `run_controlled_session_continuation_stub`
  - `load_session_continuation_store`
- Level A 下实现 stub / dry-run runner。
- Level B 授权后再启用真实 `codex-local` runner。
- 只允许 `adapter_id = codex-local`。
- 必须复用 E4 preview / guard；guard 不通过不能进入 continuation attempt。
- 必须经过用户确认弹层或等价确认对象。
- 必须记录 command preview、prompt summary、target session、cwd、sandbox、timeout、readback strategy、audit impact。
- 必须显示运行中、成功、失败、超时、readback unavailable 的用户可理解状态。
- 秘书可以提醒和解释 continuation 状态；不能代替用户确认、不能发送、不能重试。
- 更新 TypeScript 类型、Tauri wrapper、前端 UI、Rust 单测、离线 UI 测试、evidence 和 handoff。

禁止：

- 未获 Level B 授权时，不执行 `codex exec`。
- 未获 Level B 授权时，不执行 `codex exec resume`。
- 未获 Level B 授权时，不发送真实 prompt。
- 未获 Level B 授权时，不读写 `/Users/yoyi/.codex`。
- 不读取 token、secret、`.env`、keychain、OAuth session、provider credential、auth 文件内容或环境变量值。
- 不调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 不调用外部模型 provider。
- 不支持 planned adapters 的 send / resume。
- 不支持 stop / restart / delete / export / favorite。
- 不提供自由聊天输入框绕过项目 / workflow / node / session binding。
- 不跳过 E4 preview / guard。
- 不在 guard blocked 时创建 runnable attempt。
- 不把 `preview confirmed` 显示成 `execution started`。
- 不把 readback unavailable 显示成真实 0 条读回。
- 不新增完整 runtime log 系统；G1 负责运行日志最终边界。
- 不迁移数据库。
- 不改 `workflow-state.v0.json` 顶层结构或状态枚举，除非任务包执行前被单独修订并批准。
- 不新增一级入口、右侧顶级入口或项目页 tab。
- 不把 GEPA / Paseo / Odysseus 研究项并入本任务实现。

## 6. 建议数据模型

如果选择 sidecar，建议：

```text
SessionContinuationStoreV1 {
  schema_version,
  revision,
  updated_at,
  continuations[],
  attempts[],
  audit_events[]
}
```

```text
ControlledSessionContinuation {
  continuation_id,
  preview_id,
  adapter_id,
  project_id,
  workflow_id,
  node_id,
  session_id,
  target_cwd,
  allowed_write_roots,
  sandbox,
  prompt_summary,
  command_preview,
  readback_strategy,
  status,
  created_at,
  updated_at,
  requested_by,
  confirmed_by,
  audit_refs
}
```

```text
SessionContinuationAttempt {
  attempt_id,
  continuation_id,
  runner_kind,
  execution_level,
  status,
  started_at,
  finished_at,
  timeout_ms,
  command_preview,
  prompt_sent,
  real_codex_executed,
  writes_codex_home,
  readback_summary,
  failure_reason,
  audit_refs
}
```

状态建议：

- `status`: `preview_confirmed` / `queued` / `waiting_permission` / `running_stub` / `succeeded_stub` / `failed_stub` / `running_real` / `succeeded_real` / `failed_real` / `timed_out` / `readback_unavailable` / `blocked`
- `runner_kind`: `stub` / `dry_run` / `codex_local_real`
- `execution_level`: `level_a_stub_only` / `level_b_real_user_approved`
- `prompt_sent`: Level A 永远 false。
- `real_codex_executed`: Level A 永远 false。
- `writes_codex_home`: Level A 永远 false。

审计事件建议：

- `session_continuation_preview_confirmed`
- `session_continuation_attempt_created`
- `session_continuation_stub_started`
- `session_continuation_stub_completed`
- `session_continuation_real_execution_requested`
- `session_continuation_real_execution_approved`
- `session_continuation_real_execution_started`
- `session_continuation_real_execution_completed`
- `session_continuation_readback_unavailable`
- `session_continuation_blocked_by_guard`

warning 建议：

- `controlled_session_continuation_only`
- `codex_local_only`
- `requires_project_workflow_node_session_binding`
- `requires_user_confirmation`
- `level_b_real_execution_requires_user_approval`
- `no_planned_adapter_execution`
- `readback_unavailable_is_not_zero_results`

## 7. UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增入口、面板、tab、按钮或确认动作。

说明：允许新增的是智能体页或项目相关会话区域内部的受控 continuation 面板、确认弹层摘要、stub / real 状态展示；不允许新增一级入口、右侧顶级入口、项目页 tab 或自由聊天输入框。真实发送按钮只有在 Level B 授权路径中、且必须经过确认弹层后才能出现；默认 Level A 不显示真实发送按钮。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- E4 preview 通过 / 阻断结果。
- 受控 continuation 状态：等待确认、queued、running stub、succeeded stub、failed stub、readback unavailable。
- Level B 未授权时的边界说明。
- command preview、prompt summary、target session、cwd、sandbox、timeout、readback strategy、audit impact。
- real execution disabled / requires user approval。
- planned adapters blocked。

本任务禁止显示：

- Level A 下可点击真实发送 / resume 按钮。
- 自由聊天输入框或无限制 prompt 编辑器。
- `已发送`、`已 resume`、`Codex 已收到任务`、`真实 Codex 已执行`、`worker 执行中`、`readback 已完成`，除非 Level B 真实执行已授权且有对应事实记录。
- readback unavailable 被显示成“0 条结果”。
- planned adapter 的 send / resume 按钮或可执行态。
- raw transcript、raw adapter JSON、raw workflow state、raw audit、完整日志、token、secret、keychain、OAuth、provider key、环境变量值或路径大表。
- 新的 `模型与 Agent` 一级入口。

显示位置：

- 一级入口：不新增；继续使用既有 `智能体`。
- 右侧入口：不新增；秘书只读摘要可解释状态，不新增顶级图标。
- 项目页：不新增 tab，不占用工作流画布主区域；如涉及项目上下文，只能在项目相关会话 / 节点详情附近显示。
- 画布：不改画布主区域；如未来需要节点入口，必须进入 F 阶段。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：可显示受控 continuation 面板和状态。
- 管理入口：可选显示只读健康摘要，不显示 raw schema / secret。

中间版本范围：

- 本轮必须落地：`codex-local` 受控 send / resume 最小闭环代码路径和 Level A stub / guard / record 验收。
- 本轮只做读模型 / 摘要：非真实执行状态、readback unavailable、guard reasons、audit refs。
- 本轮后置：planned adapters、stop / restart、完整 runtime log、自动重试、取消恢复、阶段 G 真实 Tauri 全面验收。

后端和数据依赖：

- 需要后端正式读模型：continuation store 或等价状态记录必须从后端读模型输出。
- 需要审计 / 日志 / 权限 / 状态机：本轮必须有工作台自有 permission / attempt / audit ref；完整 runtime log 进入 G1。
- 不能用假数据伪装：stub 成功不能显示成真实 Codex 成功；readback unavailable 不能显示成 0 条结果。

UI 文案边界：

- 禁止说：`已发送`、`已 resume`、`Codex 已收到任务`、`真实 Codex 已执行`、`worker 执行中`、`readback 已完成`、`Claude Code 可继续会话`、`OpenClaw 可 resume`、`OpenCode 已支持发送`，除非有真实授权和事实记录。
- 允许说：`stub 验收`、`真实执行未授权`、`需要用户确认`、`受控 continuation`、`readback unavailable`、`不会把 unavailable 当 0 条结果`、`planned adapter 不可执行`。

验收：

- 类型检查：如改前端必须 `npm run typecheck`。
- 离线交互测试：如改前端必须 `npm run test:offline-interaction`。
- 构建：如改前端必须 `npm run build`。
- 真实窗口 / 截图验收：尽量做浏览器 / Tauri smoke；未完成必须写入 evidence / handoff，且不能接受为阶段 G 验收。
- 未验收项必须写入 evidence / handoff。

## 8. 建议执行段

### 执行段 A：安全复核和 Level 选择

目标：

- 在写代码前确认本轮是 Level A 还是 Level B。

必须完成：

1. 复读 E4 偏差记录。
2. 明确本轮是否获得 Level B 用户授权。
3. 如果没有 Level B 授权，代码只能走 stub / dry-run runner。
4. 搜索含反引号、`codex exec resume`、`.codex` 等文本时必须使用单引号或 `rg -F`。

验收：

- evidence 写明本轮执行级别。
- 如无 Level B 授权，evidence 明确没有执行真实 Codex。

### 执行段 B：Continuation store / record

目标：

- 工作台拥有自己的 continuation 状态，而不是靠 UI 临时状态伪装执行。

建议实现：

1. 新增最小 sidecar 或复用已有工作台状态记录。
2. 记录 preview confirmed、attempt created、stub started/completed、readback unavailable。
3. sidecar 如存在必须有 lock、revision、backup、atomic write、corrupt JSON refuse overwrite。
4. 记录必须能追溯 E4 preview id、project / workflow / node / session 和 user confirmation。

验收：

- Rust 单测覆盖 atomic write / corrupt JSON / revision conflict，如果新增 store。
- 不改 workflow state 顶层结构。

### 执行段 C：Controlled runner abstraction

目标：

- 将 stub runner 和真实 runner 分开，避免测试路径误触发真实 Codex。

建议实现：

1. 新增 `SessionContinuationRunner` trait / enum 或等价封装。
2. Level A 只注册 stub / dry-run runner。
3. Level B 才允许真实 `codex-local` runner。
4. 真实 runner 必须要求 explicit user approval token / flag，不允许默认启用。
5. 真实 runner command preview 和实际 command 必须一致。

验收：

- 单测证明 Level A 不会调用真实 runner。
- 搜索证明 E5 新增真实 `Command::new("codex")` 路径如果存在，默认不可达且需 Level B flag。

### 执行段 D：PermissionDialog / UI 状态

目标：

- 用户能理解这是受控 continuation，不是自由聊天。

建议实现：

1. E4 preview 通过后，显示确认摘要。
2. 确认后创建 continuation attempt。
3. Level A 状态显示为 stub / dry-run，不显示真实发送。
4. readback unavailable 单独显示，不能写成 0 条结果。
5. planned adapters 保持 blocked。

验收：

- UI 离线测试覆盖 no free chat input、no planned adapter execution、readback unavailable 文案。
- 禁止文案扫描无误导。

### 执行段 E：Readback summary

目标：

- 建立 readback 结果的最小占位和不可用边界。

建议实现：

1. Level A 可写 `readback_status = unavailable_stub` 或 `not_attempted_stub`。
2. Level B 才能读真实 transcript / rollout，且必须在用户授权里列明。
3. readback 失败 / 不可用必须给用户可理解原因。
4. readback unavailable 不进入正式事实、正式记忆或 observation。

验收：

- 测试覆盖 unavailable 不等于 0 results。
- 不写 observation / memory candidate / formal memory。

### 执行段 F：文档回收

目标：

- 明确本轮到底完成到 Level A 还是 Level B。

evidence 必须给出：

- 执行级别。
- 是否获得真实执行批准。
- 是否执行 `codex exec resume`。
- 是否读写 `/Users/yoyi/.codex`。
- 是否发送 prompt。
- 是否写 attempt / readback / audit。
- 是否完成真实 Tauri / readback / audit 证据。

## 9. 验收命令

必须运行或明确说明无法运行原因：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib session_continuation
cargo test --lib controlled_session_continuation
cargo test --lib session_operation
cargo test --lib provider_availability
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check src/types.rs src/lib.rs src/control_core.rs src/commands.rs
```

如果新增 store 文件，必须补对应 store filter，例如：

```text
cargo test --lib session_continuation_store
rustfmt --check src/session_continuation_store.rs
```

如果新增的 Rust 单测无法用 `controlled_session_continuation` filter 覆盖，必须在 evidence 中写清实际 filter。

必须做禁止文案扫描：

```text
rg -n '已发送|已 resume|Codex 已收到任务|真实 Codex 已执行|worker 执行中|readback 已完成|Claude Code 可继续会话|OpenClaw 可 resume|OpenCode 已支持发送|自动派发已开始' prototypes/productized-desktop-shell/src
```

预期：

- Level A：无误导命中。
- Level B：如果有真实执行文案，必须有真实授权和事实记录支撑。

必须做真实执行 / 敏感路径扫描，注意只能用单引号或 `rg -F`：

```text
rg -n 'Command::new\("codex"\)|codex exec resume|\.codex|read_to_string\(.*auth|read_to_string\(.*token|read_to_string\(.*secret|read_to_string\(.*\.env|keychain|oauth|provider credential' prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
```

预期：

- Level A：E5 不新增可达真实 Codex runner、secret 读取、provider credential 读取或 `.codex` 写入。
- Level B：所有真实执行命中必须在 evidence 中逐条对应用户授权。

必须做 shell 安全复核：

```text
rg -n '``|`codex exec resume`|`codex exec`' tasks evidence handoffs CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans
```

执行该类搜索时必须使用单引号或 `rg -F`；禁止双引号包住反引号文本。

## 10. evidence / handoff 要求

E5 完成后必须新增：

- `evidence/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`
- `handoffs/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1-result.md`

evidence 必须记录：

- E5 接受为什么。
- 本轮执行级别：Level A 或 Level B。
- 是否获得真实 `codex exec resume` 授权。
- 是否读写 `/Users/yoyi/.codex`。
- 是否发送真实 prompt。
- continuation store / record 最终字段或等价结构。
- E4 preview 如何进入 E5 attempt。
- user confirmation 如何记录。
- runner abstraction 如何保证 Level A 不触发真实 Codex。
- planned adapters 如何保持 blocked。
- readback unavailable 如何表达且不等于 0 results。
- UI 显示位置和不显示内容。
- 秘书是否生成 action proposal；如没有，写清测试或代码证据。
- 禁止文案扫描结果。
- 真实执行 / 敏感路径扫描结果和合理命中解释。
- shell 安全复核和是否发生命令替换偏差。
- 验证命令和结果。
- 是否完成真实窗口 / 截图验收；如未完成，写清不接受为阶段 G 验收。

handoff 必须写清：

- E5 接受为什么。
- E5 不接受为什么。
- 本轮是 Level A 还是 Level B。
- 若 Level A：真实执行 deferred 到何处。
- 若 Level B：真实执行目标、授权、结果和回滚状态。
- 后续建议：E6 runtime session attention and readback failure boundary。
- 当前权威入口文件。

## 11. Stop 条件

遇到以下情况必须停下回传：

- 未获 Level B 授权却需要执行 `codex exec` 或 `codex exec resume`。
- 未获 Level B 授权却需要读写 `/Users/yoyi/.codex`。
- 未获 Level B 授权却需要发送真实 prompt。
- 需要调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 需要调用外部模型 provider。
- 需要读取 token、secret、`.env`、keychain、OAuth session、provider credential、auth 文件内容或环境变量值。
- 需要读取真实完整 transcript 作为 Level A 开发证据。
- 需要新增完整 runtime log 系统。
- 需要迁移数据库。
- 需要改 `workflow-state.v0.json` 顶层结构或状态枚举。
- 需要支持 planned adapter send / resume。
- 需要支持 stop / restart / delete / export / favorite。
- 需要自由聊天输入框绕过项目 / workflow / node / session binding。
- 需要把 readback unavailable 显示成 0 条结果。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 需要把 GEPA / Paseo / Odysseus 研究点合入当前实现。
- 搜索命令将包含反引号的文本放进 shell 双引号且无法确认不会触发命令替换。

## 12. 回收口径

完成后 Level A 可接受为：

- 阶段 E / E5 `codex-local` controlled send / resume minimal loop 的代码路径、guard、stub / dry-run、工作台自有 continuation 记录完成。
- E4 preview 可以受控进入 continuation attempt。
- 用户确认、command preview、prompt summary、readback unavailable 和 audit ref 已形成闭环。
- planned adapters 和越界请求仍被阻断。

完成后 Level A 不接受为：

- 真实会话继续已验收。
- `codex exec resume` 已成功执行。
- prompt 已发送给 Codex。
- readback 已真实完成。
- worker / agent 已真实执行。

完成后 Level B 在用户授权和真实证据齐备时可额外接受为：

- 指定 `codex-local` 会话的一次受控真实 send / resume 最小闭环完成。

无论 Level A / B，E5 仍不接受为：

- 通用自由聊天控制器。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 真实接入。
- stop / restart / delete / export / favorite 完成。
- 完整自动重试、取消恢复、runtime log 或运维诊断完成。
- 阶段 E 总验收完成。
- 阶段 G 真实 Tauri 全面验收完成。

建议下一步：

- E6：Runtime Session Attention And Readback Failure Boundary。E6 应把 waiting_permission、running、timed_out、readback_failed、readback_unavailable、blocked_by_guard、needs_user 收成用户可理解 read model；不做完整自动重试系统。
