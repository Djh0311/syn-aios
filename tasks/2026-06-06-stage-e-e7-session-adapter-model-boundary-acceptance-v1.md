# Task Package：Stage E / E7 Session Adapter Model Boundary Acceptance v1

状态：已完成。  
用途：对阶段 E / E1-E6 做总复核，冻结阶段 E 的 accepted / deferred / blocked 项，判断是否允许进入阶段 F，并把 E-to-F handoff 写清楚。E7 是阶段验收和权威收口任务，不是新功能开发任务，不是 E5 Level B 真实执行任务，不是 G1 runtime log，也不是阶段 G 真实 Tauri 全面验收。  
执行方式：文档 / 证据 / 边界复核为主；默认不改产品代码、不新增 UI、不跑真实 Codex、不读写 `/Users/yoyi/.codex`。

完成记录：

- Evidence：`evidence/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`
- Handoff：`handoffs/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1-result.md`
- 总结论：`accepted_with_deferred_items`
- 下一步：允许进入 F1 项目工作流画布读模型收敛任务包；E5 Level B、runtime log、diagnostics、真实 Tauri 和 planned adapters 真实接入仍 deferred。

## 0. 先说薄弱点

- E1-E6 都已完成，但阶段 E 仍有重要 deferred：E5 Level B 真实 send / resume、planned adapters 真实接入、完整 runtime log、诊断、真实 Tauri 截图和 G 阶段验收。
- E7 如果写得太松，会把“底座和只读边界完成”误包装成“多 agent / 模型 / 会话控制完整完成”。
- E7 如果写得太宽，会越界去补真实执行、重试、runtime log 或 F1 画布实现；这些不是本任务范围。
- E7 的关键不是加能力，而是做全局主管复核：证据是否齐、边界是否没破、入口是否一致、是否可以让 F 阶段在不继承误导口径的前提下启动。
- 如果 E7 发现 E1-E6 证据缺失或入口文档冲突，应先标 `needs_changes`，不能硬推 F1。

## 1. 已知事实 / 未知 / 假设

已知事实：

- E1 已完成 adapter descriptor 执行边界和模型 / 凭据只读状态底座。
- E2 已完成会话操作边界契约和只读 UI。
- E3 已完成模型、凭据和 provider availability 只读边界。
- E4 已完成会话继续协议和权限预览。
- E5 已完成 Level A：`codex-local` controlled send / resume minimal loop 的代码路径、guard、stub / dry-run、工作台自有 continuation 记录和离线验收。
- E6 已完成 runtime session attention 和 readback failure boundary。
- E5/E6 均未进入 Level B，未执行真实 `codex exec` / `codex exec resume`，未发送真实 prompt，未读写 `/Users/yoyi/.codex`，未完成真实 readback。
- planned adapters 仍不可执行，不能显示为已接入。
- 阶段 F 已细化为 F1-F5，目标是项目工作流画布产品化；F 阶段不能继承 E 阶段 deferred 的真实执行能力假象。

未知：

- E1-E6 的 evidence / handoff 是否存在陈旧文案、入口冲突或边界表述不一致。
- E7 执行时是否需要补小范围文档修正。
- 是否存在新增产品代码后未同步到 `AUTHORITY.md` / `tasks/README.md` 的遗漏。
- 是否有可用真实浏览器 / Tauri 截图工具；即使有，E7 也不接受为 G 阶段全面验收。

本任务采用的假设：

- E7 不改产品代码；如执行者发现必须改产品代码，应停下回传并说明原因。
- E7 可以更新任务包状态、CURRENT、tasks/README、AUTHORITY、STAGE_PLAN、README、阶段计划、evidence 和 handoff。
- E7 的推荐结论是 `accepted_with_deferred_items`，除非复核发现 E1-E6 有阻断缺口。
- 如果 E7 结论为 `accepted_with_deferred_items`，可以允许进入 F1，但必须把 Level B、G1、G2、G3、planned adapter 真实接入等 deferred 明确留在后续阶段或蓝图，不挤进 F。

## 2. 任务目标

完成阶段 E 第七刀：

```text
E1 adapter descriptor boundary
E2 session operation boundary
E3 provider availability boundary
E4 session continuation preview / guard
E5 controlled continuation Level A
E6 runtime attention / readback failure boundary
-> Stage E acceptance matrix
-> accepted / deferred / blocked freeze
-> E-to-F handoff
-> authority sync
```

E7 完成后可以说：

- 阶段 E 已完成总复核。
- E1-E6 的 evidence / handoff 可追溯。
- 阶段 E 结论被明确冻结为 `accepted`、`accepted_with_deferred_items` 或 `needs_changes`。
- 如果结论允许，F1 可以开始拆任务包 / 执行。
- deferred 项已明确进入 G、后置蓝图、研究层或独立授权任务，不会混入 F。

E7 完成后仍不能说：

- 真实 `codex exec resume` 已执行。
- 真实 prompt 已发送。
- 真实 readback 已完成。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 已真实接入。
- provider credential store 或模型验证完成。
- stop / restart / delete / export / favorite 完成。
- 自动重试、完整 runtime log 或诊断中心完成。
- 阶段 G 真实 Tauri 全面验收完成。
- 中间版本整体最终验收完成；G5 才负责最终权威验收和 deferred freeze。

## 3. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`

阶段 E 任务包、证据和交接：

- `tasks/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`
- `evidence/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`
- `handoffs/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1-result.md`
- `tasks/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`
- `evidence/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`
- `handoffs/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1-result.md`
- `tasks/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`
- `evidence/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`
- `handoffs/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1-result.md`
- `tasks/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`
- `evidence/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`
- `handoffs/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1-result.md`
- `tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`
- `evidence/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`
- `handoffs/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1-result.md`
- `tasks/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`
- `evidence/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`
- `handoffs/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1-result.md`

UI 和边界：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`

F / G 后续：

- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md` 的 F1-F5 / G1-G5。

搜索固定文本必须用 `rg -F '...'` 或单引号，禁止用 shell 双引号包住未转义反引号。

## 4. 范围

允许：

- 复核 E1-E6 任务包、evidence、handoff、入口文档和阶段计划。
- 新增 E7 evidence / handoff。
- 更新 E7 任务包状态。
- 更新 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`docs/plans/middleware-version-stage-plan-v1.md`、`docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`。
- 生成 Stage E acceptance matrix，至少包含：
  - accepted items
  - deferred items
  - blocked / needs_changes items
  - evidence references
  - handoff references
  - F-stage handoff notes
- 做文档一致性扫描和误导文案扫描。
- 如果发现少量文档旧口径，可做文档修正；不得改产品代码。

禁止：

- 不改产品代码。
- 不新增后端类型 / command / store / read model。
- 不改前端 UI、TS 类型、Tauri wrapper、样式、测试。
- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 token、secret、`.env`、keychain、OAuth session、provider credential、auth 文件内容或环境变量值。
- 不读取真实完整 transcript / rollout 作为 E7 证据。
- 不调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 不调用外部模型 provider。
- 不新增完整 runtime log store。
- 不新增自动重试系统。
- 不启动 G1 / G2 / G3。
- 不启动 F1 实现；E7 最多允许 F1 作为下一步。
- 不迁移数据库。
- 不改 `workflow-state.v0.json`。
- 不把 GEPA / Paseo / Odysseus 研究项并入当前实现。

## 5. Stage E 复核矩阵要求

E7 evidence 必须至少包含以下矩阵：

```text
StageEAcceptanceItem {
  item_id,
  stage_item,
  title,
  status,
  accepted_as,
  not_accepted_as,
  evidence_path,
  handoff_path,
  deferred_to,
  notes
}
```

建议条目：

- E1：adapter descriptor execution boundary and model credential readonly foundation。
- E2：session operation boundary contract and readonly UI。
- E3：model credential provider availability readonly boundary。
- E4：session continuation protocol and permission preview。
- E5：codex-local controlled send / resume minimal loop Level A。
- E6：runtime session attention and readback failure boundary。
- Planned adapters deferred。
- E5 Level B real send / resume deferred。
- Runtime log / diagnostics / real Tauri deferred to G。
- Project workflow canvas handoff to F。

建议状态：

- `accepted`
- `accepted_with_deferred_items`
- `needs_changes`
- `blocked`
- `deferred`

推荐 E7 总结论：

```text
accepted_with_deferred_items
```

除非复核发现证据缺失、入口冲突、误导文案或边界破坏。

## 6. UI 显示边界确认

UI 显示边界：本任务不改前端、不改读模型、不改 UI 文案；因此不需要 UI 验收。

E7 只复核 E1-E6 的 UI 边界是否被正确记录：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不把 planned adapters 显示为可执行。
- 不把 E5 Level A stub 显示为真实 prompt 已发送。
- 不把 E6 readback unavailable / failed 显示成 0 条真实读回。
- 不把通知、待办、运行中混成一个列表。
- 不把 raw logs、raw sidecar、raw workflow state、secret、token、provider credential 暴露给普通 UI。

如果执行 E7 时需要改前端，必须停下回传；该需求不属于 E7。

## 7. 建议执行段

### 执行段 A：证据完整性复核

目标：

- 确认 E1-E6 每个任务都有 task / evidence / handoff，且入口文档能追溯。

必须完成：

1. 核对 E1-E6 task 文件存在。
2. 核对 E1-E6 evidence 文件存在。
3. 核对 E1-E6 handoff 文件存在。
4. 读取每份 evidence 的结论、不接受范围和验证记录。
5. 记录缺失项；缺失则 E7 不能直接 accepted。

### 执行段 B：边界一致性复核

目标：

- 确认阶段 E 没有被误包装。

必须复核：

- `codex-local` 仍是唯一可用 adapter。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 仍是 planned / unavailable / not verified。
- provider credential 未读取、未验证、未配置为真实可用。
- E4 preview 不是执行。
- E5 Level A stub 不是真实 send / resume。
- E6 readback unavailable / failed 不是真实 0 条读回。
- Level B 真实执行仍未授权。
- G1 runtime log、G2 diagnostics、G3 real Tauri 仍未完成。

### 执行段 C：入口文档一致性复核

目标：

- 把入口统一到阶段 E 收口状态。

必须检查：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`

如果 E7 结论允许进入 F1，入口应写成：

```text
阶段 E 总复核完成，结论为 accepted_with_deferred_items，下一步 F1。
```

如果 E7 结论不允许进入 F1，入口应写成：

```text
E7 needs_changes / blocked，必须先补阶段 E 缺口。
```

### 执行段 D：扫描

必须做：

- 旧口径扫描：`E7 仍待写任务包`、`E6-E7 仍待写任务包`、`阶段 E 已完成无 deferred`。
- 误导能力扫描：真实 prompt、Codex 已收到任务、真实 readback、已自动重试、planned adapter 已接入等。
- 敏感路径 / 真实执行扫描：`codex exec resume`、`.codex`、secret / token / auth 等，只分类解释文档和历史命中，不触碰真实敏感数据。

### 执行段 E：E-to-F handoff

目标：

- 明确 F1 能做什么、不能继承什么假设。

F1 可继承：

- adapter / provider / operation / continuation / runtime attention 的只读边界。
- E5 Level A continuation store 和 E6 attention summary。
- planned adapters 的不可执行状态。
- readback failure / unavailable 的不可伪装规则。

F1 不能继承：

- Level B 真实 send / resume。
- planned adapters 真实接入。
- 自动重试。
- runtime log store。
- 真实 Tauri 全面验收。
- provider credential 验证。

## 8. 验收命令

E7 默认不改产品代码，因此不要求 `npm` / `cargo` 测试作为必要验收。

必须做文档 / 文案扫描：

```text
rg -n -F 'E7 仍待写任务包' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans
rg -n -F 'E6-E7 仍待写任务包' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans
rg -n -F '阶段 E 已完成无 deferred' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans
```

误导文案扫描：

```text
rg -n '真实 prompt 已发送|Codex 已收到任务|真实 readback 已完成|真实会话继续已验收|已自动重试|已停止 agent|Claude Code 已接入|OpenClaw 已接入|OpenCode 已接入|planned adapter 已可执行' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans tasks evidence handoffs
```

真实执行 / 敏感路径扫描：

```text
rg -n 'Command::new\("codex"\)|codex exec resume|\.codex|read_to_string\(.*auth|read_to_string\(.*token|read_to_string\(.*secret|read_to_string\(.*\.env|keychain|oauth|provider credential' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans tasks evidence handoffs prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
```

说明：

- 文档、历史 evidence、guard、fixture、禁止项命中必须分类解释。
- E7 不应新增产品代码命中。
- 如果 E7 执行者改了产品代码，必须停止并重新定义任务边界；不能偷偷跑产品测试后当作 E7。

可选新鲜验证：

- 若执行者认为需要复核 E1-E6 最新代码状态，可以运行 `npm run typecheck`、`npm run test:offline-interaction`、`npm run build`、`cargo test --lib`，但这不是 E7 必需项。
- 如果运行这些命令，结果必须写入 evidence；失败不能忽略。

## 9. Evidence / Handoff 要求

E7 evidence 必须写清：

- E1-E6 evidence / handoff 是否齐全。
- Stage E acceptance matrix。
- Stage E 总结论：`accepted` / `accepted_with_deferred_items` / `needs_changes` / `blocked`。
- 如果允许进入 F1，说明 F1 的准入条件和禁止继承项。
- 如果不允许进入 F1，列出必须修补的 blocking / needs_changes。
- planned adapters 是否仍不可执行。
- E5 Level B 是否仍 deferred。
- provider credential / model verification 是否仍 deferred。
- runtime log / diagnostics / real Tauri 是否仍 deferred 到 G。
- 扫描结果和命中分类。
- 本轮是否改产品代码；预期为否。
- 本轮是否执行真实 Codex / 读写 `.codex`；预期为否。

E7 handoff 必须写清：

- Stage E 接受范围。
- Stage E deferred 项。
- 是否允许进入 F1。
- F1 开始前必须读哪些文档。
- 不能进入 F 的事项清单。
- 当前权威入口文件。

## 10. Stop 条件

遇到以下情况必须停下回传：

- 需要改产品代码。
- 需要新增后端类型 / command / store / read model。
- 需要改前端 UI / 类型 / wrapper / 样式 / 测试。
- 需要执行 `codex exec` 或 `codex exec resume`。
- 需要发送真实 prompt。
- 需要读写 `/Users/yoyi/.codex`。
- 需要读取真实完整 transcript / rollout。
- 需要读取 token、secret、`.env`、keychain、OAuth session、provider credential、auth 文件内容或环境变量值。
- 需要调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 需要调用外部模型 provider。
- 需要新增完整 runtime log store。
- 需要新增自动重试系统。
- 需要启动 F1 实现。
- 需要启动 G1 / G2 / G3。
- 需要迁移数据库。
- 需要改 `workflow-state.v0.json`。
- 需要把 GEPA / Paseo / Odysseus 研究点合入当前实现。
- 搜索命令将包含反引号的文本放进 shell 双引号且无法确认不会触发命令替换。

## 11. 回收口径

完成后可接受为：

- 阶段 E / E1-E6 总复核完成。
- 阶段 E 结论冻结。
- E-to-F handoff 完成。
- F1 是否允许开始被明确记录。
- deferred 项已明确归入 G、后置蓝图、研究层或独立授权任务。

完成后不接受为：

- E5 Level B 真实执行完成。
- planned adapters 真实接入完成。
- provider credential store / model verification 完成。
- stop / restart / delete / export / favorite 完成。
- 自动重试完成。
- runtime log / diagnostics 完成。
- 真实 Tauri 全面验收完成。
- 中间版本整体最终验收完成。

建议下一步：

- 如果 E7 结论允许进入 F：开始写 F1 Project Workflow Canvas Read Model Consolidation 任务包。
- 如果 E7 结论为 needs_changes / blocked：先写 E7.1 或对应修补任务包。
