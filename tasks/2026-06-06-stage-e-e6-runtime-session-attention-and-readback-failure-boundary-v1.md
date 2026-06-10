# Task Package：Stage E / E6 Runtime Session Attention And Readback Failure Boundary v1

状态：已完成。  
完成记录：见 `evidence/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md` 与 `handoffs/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1-result.md`。  
用途：在 E5 Level A `codex-local` 受控 continuation 代码路径完成后，把会话运行关注、权限待处理、readback 失败 / 不可用、超时、guard 阻断和需要用户介入的状态收成最小读模型与用户可理解 UI。E6 是阶段 E 的运行关注和失败边界任务，不是真实执行任务、不是自动重试任务、不是运行日志最终形态、不是阶段 G 验收任务。  
执行方式：默认只允许读模型 / 摘要 / UI 可见化 / 测试 / 文档回收；不得进入 E5 Level B，不得执行真实 `codex exec` / `codex exec resume`，不得读写 `/Users/yoyi/.codex`。

## 0. 先说薄弱点

- E6 名字里有 runtime、attention、failure，容易被误解成要做完整运行日志、自动重试、stop / restart 或真实会话控制；这些都不是 E6 范围。
- E5 只完成 Level A stub / dry-run 路径，真实 prompt 没有发送，真实 readback 没有发生；E6 不能把 E5 Level A 的 unavailable / stub 状态显示成真实失败、真实 0 条读回或真实执行结果。
- readback failure 和 readback unavailable 是两个不同概念：failure 代表尝试读取但失败或读取不可信；unavailable 代表本轮没有真实读取来源。两者都不能写成“0 条结果”。
- 通知、待办、运行中、秘书摘要都可能展示同一件事，但不能混成一个列表，也不能把 raw logs、raw audit、raw sidecar、路径大表或内部 schema 铺给普通用户。
- G1 才负责运行日志边界；E6 只能给 G1 留接口意识，不能抢先新增完整 runtime log store。
- GEPA / Paseo / Odysseus 研究资料仍只作为蓝图参考；E6 不融合优化器、daemon、timeline、schedule、workspace 复刻或外部项目功能。

## 1. 已知事实 / 未知 / 假设

已知事实：

- E1 已完成 adapter descriptor 执行边界和模型 / 凭据只读状态底座。
- E2 已完成会话操作边界契约和智能体页只读 UI。
- E3 已完成模型、凭据和 provider availability 只读边界。
- E4 已完成会话继续协议和权限预览；E4 不执行真实 send / resume。
- E5 已完成 Level A：`session-continuations.v1.json`、受控 continuation store、Tauri commands、`WorkbenchSnapshot.session_continuation_store`、Level A stub attempt、readback unavailable placeholder 和智能体页只读 E5 面板。
- E5 明确不接受为真实 `codex exec resume`、真实 prompt 发送、真实 readback、真实会话继续验收或读写 `/Users/yoyi/.codex`。
- 当前没有 Level B 对具体 session、cwd、prompt、`.codex` 范围、回滚和证据的授权。
- UI 任务包必须落实 `docs/plans/task-package-ui-display-boundary-rule-v1.md` 的“UI 显示边界确认”章节。

未知：

- 现有右侧 `运行中` / `通知` / `待办` 入口当前是否已有足够承载 E6 状态的读模型和组件。
- E6 最小读模型是否完全可从 `WorkbenchSnapshot.session_continuation_store`、E2 operation boundary、E3 provider availability、E4 preview 和 workflow read model 派生，还是需要新增纯派生 helper。
- readback failure 的第一版是否只有 E5 unavailable / stub、历史 workflow dispatch readback failure 和 guard blocked 三类来源，还是还要纳入 C5 readback 最小可见化摘要。
- 真实 Tauri / 浏览器截图工具是否可用。

本任务采用的假设：

- E6 默认不新增持久 sidecar；优先新增纯读模型 / derived summary。如果实现者发现必须新增持久运行日志 store，应停下回传，把该能力留给 G1。
- E6 不读取真实完整 transcript / rollout；如果读回需要真实 transcript，应停下回传并进入 Level B 或 G 阶段授权讨论。
- E6 可以在 `WorkbenchSnapshot` 或前端纯读模型中新增 `runtime_session_attention` / `readback_failure_summary` / `session_run_status_summaries` 等等价字段。
- E6 可以在既有 `智能体` 页面、既有右侧 `运行中`、既有 `通知` / `待办` 摘要和秘书只读摘要里展示最小状态；不能新增一级入口、右侧顶级入口或项目页 tab。
- E6 完成后仍不能说阶段 E 完成；E7 负责阶段 E 总复核和 E-to-F handoff。

## 2. 任务目标

完成阶段 E 第六刀：

```text
E2 session operation boundary
+ E3 provider availability
+ E4 continuation preview / guard
+ E5 Level A continuation store / attempt
-> RuntimeSessionAttention read model
-> ReadbackFailureReason / ReadbackUnavailable boundary
-> SessionRunStatusSummary
-> intelligent page / running entry / notification / todo / secretary summary
-> evidence + handoff
```

E6 完成后可以说：

- 工作台能用读模型解释会话和 continuation 的运行关注状态。
- 用户能区分 `waiting_permission`、`running_stub`、`timed_out`、`readback_failed`、`readback_unavailable`、`blocked_by_guard`、`needs_user` 等状态。
- readback failure / unavailable 不会被显示成真实 0 条读回。
- 通知、待办、运行中和秘书能显示最小摘要和跳转建议，但不展示 raw logs，不替用户批准，不重试，不继续发送。
- E5 Level A 的 stub / unavailable 状态有统一人话解释。

E6 完成后仍不能说：

- 真实 send / resume 完成。
- 真实 `codex exec resume` 已执行。
- 真实 prompt 已发送。
- 真实 readback 已完成。
- 自动重试系统完成。
- stop / restart / delete / export / favorite 完成。
- 完整 runtime log / 运维诊断完成。
- 阶段 E 总验收完成。
- 阶段 G 真实 Tauri 全面验收完成。

## 3. 必须先读

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
- `tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`
- `evidence/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`
- `handoffs/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1-result.md`

会话 / readback / 工作流前置：

- `tasks/2026-06-03-session-center-foundation-hardening-v1.md`
- `evidence/2026-06-03-session-center-foundation-hardening-v1.md`
- `handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`
- `tasks/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`
- `evidence/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`
- `handoffs/2026-06-03-workflow-dispatch-readback-native-parser-v1-result.md`
- `tasks/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md`
- `evidence/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md`
- `handoffs/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1-result.md`

主要代码入口：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

搜索固定文本必须用 `rg -F '...'` 或单引号，禁止用 shell 双引号包住未转义反引号。

## 4. 范围

允许：

- 新增或扩展纯读类型：
  - `RuntimeSessionAttention`
  - `RuntimeSessionAttentionKind`
  - `ReadbackFailureReason`
  - `ReadbackBoundaryStatus`
  - `SessionRunStatusSummary`
  - `RuntimeAttentionSourceRef`
  - `RuntimeAttentionSeverity`
- 新增后端派生 helper 或前端纯读模型，用于从既有 snapshot / sidecar / workflow read model 派生 attention。
- 扩展 `WorkbenchSnapshot`，输出最小 runtime attention / readback failure summary。
- 在既有 `智能体` 页面显示 runtime attention 和 readback boundary 摘要。
- 如果已有右侧 `运行中`、`通知`、`待办` 入口，可接入摘要和跳转建议；不得新增顶级入口。
- 秘书只读模型可解释 attention、风险、用户下一步查看建议；不得生成批准、发送、resume、重试、stop、restart action proposal。
- 复用 E5 `session-continuations.v1.json` 的 continuation / attempt / audit / readback placeholder。
- 复用 C5 readback / permission / failure 最小可见化经验，但不改变 C5 语义。
- 更新 TS 类型、Tauri wrapper、前端 UI、Rust 单测、离线 UI 测试、evidence 和 handoff。

禁止：

- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 token、secret、`.env`、keychain、OAuth session、provider credential、auth 文件内容或环境变量值。
- 不读取真实完整 transcript / rollout 作为 E6 开发证据。
- 不调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 不调用外部模型 provider。
- 不支持 planned adapters 的 send / resume。
- 不支持 stop / restart / delete / export / favorite。
- 不新增完整自动重试系统。
- 不新增完整 runtime log store；G1 负责运行日志最终边界。
- 不新增运维诊断系统；G2 负责 diagnostics / health / degraded state。
- 不迁移数据库。
- 不改 `workflow-state.v0.json` 顶层结构或状态枚举。
- 不新增一级入口、右侧顶级入口或项目页 tab。
- 不把通知、待办、运行中混成一个列表。
- 不把 raw logs、raw audit、raw sidecar、raw workflow state、路径大表或内部 schema 展示给普通用户。
- 不把 readback unavailable 显示成“0 条结果”。
- 不把 readback failed 写成正式事实、observation、MemoryCandidate 或正式记忆。
- 不把 GEPA / Paseo / Odysseus 研究项并入本任务实现。

## 5. 建议读模型

建议形状：

```text
RuntimeSessionAttention {
  attention_id,
  project_id,
  workflow_id,
  node_id,
  session_id,
  adapter_id,
  source_refs[],
  kind,
  severity,
  status,
  title,
  user_message,
  technical_summary,
  recommended_next_step,
  requires_user_action,
  blocks_continuation,
  readback_boundary,
  created_at,
  updated_at
}
```

```text
ReadbackBoundaryStatus {
  status,
  reason,
  attempted,
  real_readback_performed,
  result_count,
  user_message,
  technical_summary,
  source_refs[]
}
```

```text
SessionRunStatusSummary {
  session_id,
  adapter_id,
  project_id,
  workflow_id,
  node_id,
  current_status,
  current_status_label,
  attention_count,
  blocking_count,
  needs_user_count,
  readback_status,
  latest_attention_ids[],
  source_refs[]
}
```

状态建议：

- `waiting_permission`
- `waiting_level_b_authorization`
- `running_stub`
- `succeeded_stub`
- `failed_stub`
- `timed_out`
- `readback_failed`
- `readback_unavailable`
- `blocked_by_guard`
- `needs_user`
- `degraded`
- `not_started`
- `unknown`

readback reason 建议：

- `not_attempted_stub`
- `level_b_not_authorized`
- `readback_source_missing`
- `readback_parser_failed`
- `readback_permission_blocked`
- `session_binding_missing`
- `rollout_unavailable`
- `guard_blocked`
- `timeout_before_readback`
- `unknown_failure`

severity 建议：

- `info`
- `warning`
- `needs_user`
- `blocking`

派生规则建议：

- E5 Level A attempt 且 `real_codex_executed=false`：输出 `readback_unavailable`，reason 为 `not_attempted_stub` 或 `level_b_not_authorized`。
- guard blocked：输出 `blocked_by_guard`，且 `blocks_continuation=true`。
- waiting permission / waiting Level B：输出 `waiting_permission` 或 `waiting_level_b_authorization`，且 `requires_user_action=true`。
- timeout：输出 `timed_out`，但不能暗示已经自动停止 agent。
- parser / rollout / source 失败：输出 `readback_failed`，但 `result_count=null`，不能写 0。
- no source / no attempt：输出 `readback_unavailable`，不能写成失败或 0。

## 6. UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增入口、面板、tab、按钮或确认动作。

说明：允许新增的是既有 `智能体` 页面内的 runtime attention / readback boundary 面板，和既有右侧 `运行中` / `通知` / `待办` / `秘书` 内部的摘要或跳转建议。不得新增一级入口、右侧顶级入口、项目页 tab、真实执行按钮、自动重试按钮、stop / restart 按钮或自由聊天输入框。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- runtime attention count / severity。
- waiting permission / waiting Level B authorization。
- running stub / succeeded stub / failed stub。
- timed out。
- readback failed。
- readback unavailable。
- blocked by guard。
- needs user。
- “unavailable 不是 0 条读回”的解释。
- 跳转到对应智能体会话、continuation、workflow node 或管理详情的建议。

本任务禁止显示：

- `已自动重试`
- `已停止 agent`
- `已重启 agent`
- `真实派发已完成`
- `真实 prompt 已发送`
- `Codex 已收到任务`
- `真实 readback 已完成`
- `readback 0 条`
- `失败已自动恢复`
- `Claude Code 已接管`
- `OpenClaw 已运行`
- `OpenCode 已 resume`
- raw transcript、raw rollout、raw sidecar、raw workflow state、raw audit、完整日志、token、secret、keychain、OAuth、provider key、环境变量值或路径大表。

显示位置：

- 一级入口：不新增；继续使用既有 `智能体`。
- 右侧入口：不新增；如已有 `运行中` / `通知` / `待办` / `秘书`，只在内部显示摘要。
- 项目页：不新增 tab，不占用工作流画布主区域；如需要项目上下文，只能在节点详情或相关会话区域附近显示。
- 画布：不改画布主区域；画布深化进入阶段 F。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：可显示 runtime attention 和 readback boundary 面板。
- 管理入口：可选显示只读健康摘要，不显示 raw schema / secret / full logs。

中间版本范围：

- 本轮必须落地：runtime attention / readback failure 最小读模型和可见化。
- 本轮只做读模型 / 摘要：状态、原因、人话解释、跳转建议、秘书解释。
- 本轮后置：真实 Level B send / resume、完整 runtime log、自动重试、stop / restart、诊断中心、阶段 G 真实 Tauri 全面验收。

后端和数据依赖：

- 需要后端正式读模型：E6 attention 必须来自 E5 store、E2/E3/E4 read model、workflow read model 或明确的纯派生 helper。
- 需要审计 / 日志 / 权限 / 状态机：E6 可以引用已有 audit refs / permission / guard status，但不新增完整运行日志。
- 不能用假数据伪装：stub、unavailable、failed、timeout 都必须按真实来源解释；无真实 readback 时不能显示 0 条或完成。

UI 文案边界：

- 禁止说：`已自动重试`、`已停止 agent`、`真实派发已完成`、`真实 prompt 已发送`、`Codex 已收到任务`、`真实 readback 已完成`、`readback 0 条`、`Claude Code 已接管`、`OpenClaw 已运行`、`OpenCode 已 resume`。
- 允许说：`需要用户确认`、`等待 Level B 授权`、`stub 运行状态`、`readback unavailable`、`readback failed`、`unavailable 不是 0 条结果`、`guard 已阻断`、`需要查看详情`。

验收：

- 类型检查：如改前端必须 `npm run typecheck`。
- 离线交互测试：如改前端必须 `npm run test:offline-interaction`。
- 构建：如改前端必须 `npm run build`。
- 真实窗口 / 截图验收：尽量做浏览器 / Tauri smoke；未完成必须写入 evidence / handoff，且不能接受为阶段 G 验收。
- 未验收项必须写入 evidence / handoff。

## 7. 建议执行段

### 执行段 A：E5 Level A 复核和来源盘点

目标：

- 确认 E6 只消费 E5 Level A 事实，不进入 Level B。

必须完成：

1. 读取 E5 evidence / handoff。
2. 列出现有 attention 来源：E5 continuation store、E4 preview / guard、E2 operation boundary、E3 provider availability、C5 readback / permission / failure 摘要、workflow read model。
3. 确认本轮不会读真实 transcript / rollout。
4. 确认本轮不会执行真实 Codex。

验收：

- evidence 写明 E6 的数据来源。
- evidence 写明没有进入 E5 Level B。

### 执行段 B：Runtime attention 读模型

目标：

- 建立最小 `RuntimeSessionAttention` / `SessionRunStatusSummary`。

建议实现：

1. 后端派生优先；如果前端已有充足 snapshot，也可新增前端纯读模型，但必须测试。
2. 聚合 E5 continuation / attempt / readback summary。
3. 聚合 guard blocked、waiting permission、timeout、readback failed / unavailable。
4. 输出用户可理解 title / user_message / recommended_next_step。
5. 保留 source_refs，便于 evidence / audit 追溯。

验收：

- Rust 或 TS 单测覆盖主要状态。
- readback unavailable 的 `result_count` 必须为 `null` 或缺省，不能为 `0`。

### 执行段 C：智能体页和右侧摘要

目标：

- 让用户能看到需要关注什么，而不是看到内部日志。

建议实现：

1. 智能体页在 E5 面板附近显示 runtime attention 摘要。
2. 如已有右侧 `运行中` 入口，显示运行关注计数和最高严重级别。
3. 如已有通知 / 待办读模型，增加只读摘要或跳转建议；不要混成单列表。
4. 不新增真实操作按钮。
5. 不显示 raw logs / raw sidecar。

验收：

- 离线 UI 测试覆盖 `waiting_permission`、`readback_unavailable`、`readback_failed`、`blocked_by_guard`。
- 禁止文案扫描无误导。

### 执行段 D：秘书只读解释

目标：

- 秘书能解释风险和下一步查看建议，但不代替用户行动。

建议实现：

1. 增加 runtime attention risk signal。
2. 增加 inspect / view suggestion。
3. 不生成 approve / retry / send / resume / stop / restart action proposal。
4. 明确 readback unavailable 不是 0 条结果。

验收：

- 离线测试覆盖秘书不生成执行类 action proposal。

### 执行段 E：文档回收

目标：

- 明确 E6 接受范围和 deferred 项。

必须更新：

- E6 任务包状态。
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- 新增 evidence。
- 新增 handoff。

## 8. 验收命令

前端：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

Rust：

```text
cargo test --lib runtime_session_attention
cargo test --lib session_continuation
cargo test --lib session_operation
cargo test --lib provider_availability
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check src/types.rs src/lib.rs src/control_core.rs src/commands.rs src/session_continuation_store.rs
```

如果实现者新增文件，必须把新增 Rust 文件纳入 `rustfmt --check`。

禁止误导文案扫描：

```text
rg -n '已自动重试|已停止 agent|已重启 agent|真实派发已完成|真实 prompt 已发送|Codex 已收到任务|真实 readback 已完成|readback 0 条|失败已自动恢复|Claude Code 已接管|OpenClaw 已运行|OpenCode 已 resume' prototypes/productized-desktop-shell/src
```

真实执行 / 敏感路径扫描：

```text
rg -n 'Command::new\("codex"\)|codex exec resume|\.codex|read_to_string\(.*auth|read_to_string\(.*token|read_to_string\(.*secret|read_to_string\(.*\.env|keychain|oauth|provider credential' prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
```

说明：

- 命中历史 runner、guard、fixture 或 Markdown 文案时必须分类解释。
- E6 不应新增真实 `Command::new("codex")` 可达路径。
- E6 不应新增真实 `.codex` 读写。

## 9. Evidence / Handoff 要求

evidence 必须写清：

- E6 接受为什么。
- E6 不接受为什么。
- 数据来源：哪些来自 E5 store，哪些来自 E4 / E3 / E2 / C5 / workflow read model。
- 是否新增持久 store；默认应为否。如果新增，必须解释为什么没有越界到 G1。
- readback failed 和 readback unavailable 如何区分。
- unavailable 为什么不是 0 results。
- runtime attention 如何进入智能体页 / 运行中 / 通知 / 待办 / 秘书。
- 秘书是否生成执行类 action proposal；如没有，写清测试或代码证据。
- 禁止文案扫描结果。
- 真实执行 / 敏感路径扫描结果和合理命中解释。
- 验证命令和结果。
- 是否完成真实窗口 / 截图验收；如未完成，写清不接受为阶段 G 验收。

handoff 必须写清：

- E6 接受范围。
- E6 不接受范围。
- E6 是否进入 Level B：预期必须为否。
- 后续建议：E7 Stage E Acceptance And E-to-F Handoff。
- 当前权威入口文件。

## 10. Stop 条件

遇到以下情况必须停下回传：

- 需要执行 `codex exec` 或 `codex exec resume`。
- 需要发送真实 prompt。
- 需要读写 `/Users/yoyi/.codex`。
- 需要读取真实完整 transcript / rollout。
- 需要读取 token、secret、`.env`、keychain、OAuth session、provider credential、auth 文件内容或环境变量值。
- 需要调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 需要调用外部模型 provider。
- 需要新增完整 runtime log store。
- 需要新增自动重试系统。
- 需要支持 stop / restart / delete / export / favorite。
- 需要改 `workflow-state.v0.json` 顶层结构或状态枚举。
- 需要迁移数据库。
- 需要把 readback unavailable 显示成 0 条结果。
- 需要把 readback failed / unavailable 写 observation、MemoryCandidate 或正式记忆。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 需要把通知、待办、运行中混成一个列表。
- 需要把 GEPA / Paseo / Odysseus 研究点合入当前实现。
- 搜索命令将包含反引号的文本放进 shell 双引号且无法确认不会触发命令替换。

## 11. 回收口径

完成后可接受为：

- 阶段 E / E6 runtime session attention 和 readback failure boundary 完成。
- `waiting_permission`、`waiting_level_b_authorization`、`running_stub`、`timed_out`、`readback_failed`、`readback_unavailable`、`blocked_by_guard`、`needs_user` 等状态能被读模型解释。
- readback failed 和 readback unavailable 被区分展示。
- readback unavailable 不显示为 0 条结果。
- 智能体页、运行中 / 通知 / 待办摘要和秘书只读解释形成最小可见化。

完成后不接受为：

- 真实 send / resume 完成。
- 真实 `codex exec resume` 已执行。
- 真实 prompt 已发送。
- 真实 readback 已完成。
- 自动重试系统完成。
- stop / restart / delete / export / favorite 完成。
- 完整 runtime log / 运维诊断完成。
- 阶段 E 总验收完成。
- 阶段 G 真实 Tauri 全面验收完成。

建议下一步：

- E7：Stage E Acceptance And E-to-F Handoff。E7 应对 E1-E6 做阶段 E 总复核，冻结 accepted / deferred 项，并判断是否允许进入 F1。
