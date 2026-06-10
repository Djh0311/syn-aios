# Unified Product Command Routing PCR9 Level B Real Probe Authorization And Fixture Freeze v1

日期：2026-06-09

状态：已完成，并已通过全局主管复核；PCR9A 统一 Product Command Phase B 桥已通过主管复核，用户随后明确“全部授权”，B1/B2 真实探针已按统一 product command 路径完成。结论为带 P2 接受；权威入口同步保留给 PCR10 checkpoint。

PCR9 是统一 Product Command Routing 的 Level B 真实探针任务。它接在 PCR0-PCR8 Level A checkpoint 之后，用来验证“真实 `codex-local resume` 是否只能通过统一 product command 链路触发，并且结果能回到 runtime log / audit / readback / UI 状态”。

本任务包最初冻结授权、fixture、prompt、hash、写入范围和验收规则；在用户明确“全部授权”后，B1/B2 已执行并回收 evidence / handoff / 主管验收。PCR9 执行阶段确实触发真实 `codex exec resume`，并允许 Codex 原生运行时对 `/Users/yoyi/.codex` 做最小必要写入。

开发线预检结论已更新：PCR9A 已补齐统一 product command Phase B bridge；后续 PCR9 真实探针必须以 `run_real_execution_product_command_phase_b`、product command attempt、runtime / audit / readback refs 为完成证据。既有底层 `run_controlled_session_continuation_real_resume_phase_b` 仍不能单独作为 PCR9 完成证据。

## 0. 先说薄弱点

- PCR8 只证明 Level A 产品链路和安全扫描通过，不证明真实执行已经产品化跑通。
- 历史 H5-Level-B1/B2 证明过 `mario test` 上的真实 resume / workspace-write 探针，但那是 H5 bridge 阶段证据，不能直接冒充 PCR9 统一 product command 证据。
- PCR9 会触发真实 Codex；真实执行必然让 Codex 原生运行时写入 `/Users/yoyi/.codex` 的最小会话状态。没有用户二次授权时，任何线程都不得执行。
- PCR9 只允许验证 `codex-local` 指定测试项目和指定 session，不验证任意项目自由执行，不验证 planned adapters，不验证 provider credential / model verification。
- PCR9 不做自动重试、stop / kill / restart。失败也可以是有效结果，但必须分类，不能包装成成功。

## 1. 前置事实

- PCR0 已冻结统一 Product Command Routing 决策：真实执行必须归口统一 product command。
- PCR1 已建立 product command sidecar / store skeleton / read model。
- PCR2 已完成 preview / prepare 服务。
- PCR3 已完成用户 decision / confirmation 服务。
- PCR4 已完成 Phase A no-op / fake runner。
- PCR5 已封口 legacy entry。
- PCR6 已完成 UI product linkage。
- PCR7 已完成 failure / stop / retry 产品状态。
- PCR8 已完成 test matrix and safety scan checkpoint，结论为 `accepted_with_deferred_items`，并由复核线确认可进入 PCR9 单独授权准备。
- PCR9A 已完成统一 Product Command Phase B 桥并通过主管线 / 只读复核线复核，结论为带 P2 通过；P2 是既有底层 continuation Phase B API 仍存在，PCR9 evidence 不得用它冒充统一 product command 路径。

## 2. 本任务目标

PCR9 必须完成：

1. 冻结测试项目、target session、operation family、prompt summary、prompt ref、prompt sha256、allowed write roots、denied paths、`.codex` access scope。
2. 在真实执行前确认 PCR0-PCR8 仍为完成状态，且没有 P0/P1。
3. 通过统一 product command 链路执行 B1 read-only resume probe。
4. 只有 B1 通过且主管线确认后，才进入 B2 workspace-write resume probe。
5. B1/B2 均必须写入或回收 product command attempt、continuation attempt、runtime log ref、audit refs、readback result 和 worker report candidate。
6. B2 必须记录核心项目文件 hash before / after，并证明只写允许的 `.workbench/pcr9/` 探针文件。
7. 新增 PCR9 evidence / handoff。
8. 交给复核线只读复核，再由主管线决定是否接受 PCR9。

## 3. 本任务不做

- 不创建新 Codex session。
- 不执行任意项目自由派发。
- 不把历史 H5-Level-B 结果直接作为 PCR9 完成证据。
- 不通过 legacy workflow dispatch、legacy workflow machine、H5 preview、test-only ignored probe 或 direct CLI 冒充统一 product command。
- 不做自动重试。
- 不做 stop / kill / restart。
- 不修改 `workflow-state.v0.json` 顶层结构。
- 不修改 `mario test` 核心文件：`index.html`、`styles.css`、`game.js`、`README.md`。
- 不读取 auth/token/secret/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 不接 planned adapters。
- 不做 provider credential / model verification。
- 不做真实 Tauri / Browser / screenshot 验收。
- 不同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`；这些只允许 PCR10 checkpoint 收口时同步。

## 4. 执行点授权状态

当前状态：

- 任务包授权准备：已冻结。
- 用户真实执行授权：已获得；用户在主管线明确回复“全部授权”。
- B1 read-only probe：已完成。
- B2 workspace-write probe：已在 B1 通过后完成。
- 主管复核：已完成，结论为带 P2 接受。

执行前要求用户明确授权以下内容；本轮已由用户“全部授权”满足：

```text
批准 PCR9 Level B 对 /Users/yoyi/Documents/mario test 执行统一 product command 真实 resume 探针。
允许 B1 read-only resume。
允许 B1 通过后执行 B2 workspace-write resume。
允许 Codex 原生运行时对 /Users/yoyi/.codex 做最小必要会话状态写入。
允许 B2 只写 /Users/yoyi/Documents/mario test/.workbench/pcr9/real-product-command-write-probe.md。
不允许读取 secret/token/.env/keychain/OAuth/provider credential/full transcript/rollout。
不允许修改 mario test 核心文件。
```

如果后续新增真实执行任务，仍必须重新明确授权；PCR9 的一次性授权不能继承到其他项目、其他 session、planned adapter 或自动重试。

## 5. 冻结的测试对象

```text
project_label: mario test
project_root: /Users/yoyi/Documents/mario test
project_id: project:users-yoyi-documents-mario-test
workflow_id: workflow:users-yoyi-documents-mario-test:default
target_node_id: workflow:users-yoyi-documents-mario-test:default:node:codex-dev
target_session_id: 019e798a-ac37-7771-b982-e38084fcd22e
adapter_id: codex-local
operation_family: real_execution_product_command
operation: resume
```

如执行线发现 target session 不存在、不可 resume、或需要读取完整 transcript 才能确认，必须停止并回交阻断，不得扩大读取范围。

## 6. B1 Read-Only Probe

### 6.1 授权范围

```text
sandbox: read-only
readback_marker: PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_READ_ONLY_OK_2026_06_09
allowed_project_writes: none
allowed_workspace_writes: host-side workbench writes only, limited to product-line evidence / handoff and existing workbench stores
codex_home_scope: minimal native Codex runtime state written by real resume only
```

host-side workbench writes include only:

- `evidence/2026-06-09-unified-product-command-routing-pcr9-level-b-real-probe-v1.md`
- `handoffs/2026-06-09-unified-product-command-routing-pcr9-level-b-real-probe-v1-result.md`
- `real-execution-product-commands.v1.json`
- `session-continuations.v1.json`
- existing runtime log / audit / readback refs generated by the workbench store layer

These host-side writes are workbench bookkeeping, not worker project writes. The worker must not write project files in B1.

### 6.2 Prompt 合同

prompt summary：

```text
PCR9 Level B read-only unified product command resume probe for mario test codex-dev worker.
```

prompt ref：

```text
workbench-managed:pcr9:mario-test:codex-dev:read-only-unified-product-command-probe:v1
```

prompt sha256：

```text
99f65e9f986272da4b1dfda91261b0bed32621b963b515e08296384443d650cc
```

canonical prompt source：

```text
You are the codex-local worker for the workbench PCR9 Level B read-only unified product command probe.

Scope:
- Project: /Users/yoyi/Documents/mario test
- Operation: resume only
- Sandbox: read-only
- Marker: PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_READ_ONLY_OK_2026_06_09

Rules:
- Do not modify files.
- Do not run commands.
- Do not read secrets, auth tokens, .env files, keychain data, OAuth credentials, provider credentials, rollout data, or full transcripts.
- Reply only with the marker and a minimal structured worker report candidate.
```

sha256 口径：按上方 canonical prompt source 代码块内文本计算，不包含代码块后的额外换行。

执行线必须在真实执行前重新计算 sha256；如果 hash 不一致，停止。

### 6.3 成功验收

B1 成功必须同时满足：

- 真实 resume 从统一 product command 链路触发，不从 legacy/H5/direct CLI 触发。
- `prompt_sent=true`。
- `real_codex_executed=true`。
- `writes_codex_home=true`。
- `writes_project_files=false`。
- readback / last message 包含 `PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_READ_ONLY_OK_2026_06_09`。
- product command attempt、continuation attempt、runtime log ref、audit refs、readback ref 可追溯。
- worker report candidate 可追溯，但不自动写正式事实或正式记忆。
- `mario test` 核心文件 hash 前后一致。

## 7. B2 Workspace-Write Probe

### 7.1 授权范围

```text
sandbox: workspace-write
readback_marker: PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_WRITE_PROBE_OK_2026_06_09
allowed_project_write_path: /Users/yoyi/Documents/mario test/.workbench/pcr9/real-product-command-write-probe.md
allowed_project_write_dir: /Users/yoyi/Documents/mario test/.workbench/pcr9/
allowed_workspace_writes: host-side workbench writes only, limited to product-line evidence / handoff and existing workbench stores
codex_home_scope: minimal native Codex runtime state written by real resume only
```

host-side workbench writes include only:

- `evidence/2026-06-09-unified-product-command-routing-pcr9-level-b-real-probe-v1.md`
- `handoffs/2026-06-09-unified-product-command-routing-pcr9-level-b-real-probe-v1-result.md`
- `real-execution-product-commands.v1.json`
- `session-continuations.v1.json`
- existing runtime log / audit / readback refs generated by the workbench store layer

These host-side writes are workbench bookkeeping. B2 worker project writes are limited to `allowed_project_write_path`.

### 7.2 Prompt 合同

prompt summary：

```text
PCR9 Level B workspace-write unified product command resume probe for mario test codex-dev worker.
```

prompt ref：

```text
workbench-managed:pcr9:mario-test:codex-dev:workspace-write-unified-product-command-probe:v1
```

prompt sha256：

```text
00a85874146fc1f5928486de85e7ed1c55c8fe5ea29fefcbab56973b4f71a48c
```

canonical prompt source：

```text
You are the codex-local worker for the workbench PCR9 Level B workspace-write unified product command probe.

Scope:
- Project: /Users/yoyi/Documents/mario test
- Operation: resume only
- Sandbox: workspace-write
- Allowed write path: /Users/yoyi/Documents/mario test/.workbench/pcr9/real-product-command-write-probe.md
- Marker: PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_WRITE_PROBE_OK_2026_06_09

Rules:
- Write only the allowed probe file.
- Do not modify index.html, styles.css, game.js, README.md, .git, or any path outside .workbench/pcr9/.
- Do not run tests or start services.
- Do not read secrets, auth tokens, .env files, keychain data, OAuth credentials, provider credentials, rollout data, or full transcripts.
- Reply with the marker and a minimal structured worker report candidate.
```

sha256 口径：按上方 canonical prompt source 代码块内文本计算，不包含代码块后的额外换行。

执行线必须在真实执行前重新计算 sha256；如果 hash 不一致，停止。

### 7.3 探针文件内容要求

允许写入的探针文件必须至少包含：

```text
marker: PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_WRITE_PROBE_OK_2026_06_09
scope: pcr9_workspace_write_unified_product_command_probe
changed_files:
- .workbench/pcr9/real-product-command-write-probe.md
process_fact_candidate: codex-local worker received a unified product command resume and wrote the authorized PCR9 probe file only.
```

### 7.4 成功验收

B2 成功必须同时满足：

- B1 已通过，且主管线确认允许进入 B2。
- 真实 resume 从统一 product command 链路触发，不从 legacy/H5/direct CLI 触发。
- `prompt_sent=true`。
- `real_codex_executed=true`。
- `writes_codex_home=true`。
- `writes_project_files=true`，但只能落在 `.workbench/pcr9/`。
- readback / last message 包含 `PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_WRITE_PROBE_OK_2026_06_09`。
- 探针文件存在，内容包含 marker，hash 已记录。
- `index.html`、`styles.css`、`game.js`、`README.md` hash 前后一致。
- product command attempt、continuation attempt、runtime log ref、audit refs、readback ref 可追溯。
- worker report candidate 可追溯，但不自动写正式事实或正式记忆。

## 8. 执行前检查清单

B1 和 B2 各自执行前都必须完成：

1. 确认用户已明确授权本段真实执行。
2. 确认 PCR0-PCR8 任务包状态仍为已完成，PCR9A 状态为已通过主管复核。
3. 确认复核线没有未关闭 P0/P1。
4. 确认没有 duplicate queued / running product command。
5. 确认 diagnostics 无 blocking degraded state。
6. 确认 permission envelope / user decision / store revision / workflow revision 匹配。
7. 确认 prompt summary / ref / sha256 匹配任务包。
8. 记录 expected product command id、operation id、continuation id、runtime log refs、audit refs、readback refs。
9. 记录核心文件执行前 hash：
   - `/Users/yoyi/Documents/mario test/index.html`
   - `/Users/yoyi/Documents/mario test/styles.css`
   - `/Users/yoyi/Documents/mario test/game.js`
   - `/Users/yoyi/Documents/mario test/README.md`
10. B2 额外记录探针文件执行前状态：不存在 / 已存在及 hash。
11. B2 如探针文件已存在，必须记录 before hash，并采用覆盖写入完整探针文件；不允许追加到旧内容后再把旧内容当本轮证据。
12. 确认 `.codex` scope 只限真实 Codex 原生最小会话状态，不读完整 transcript / rollout / secret。

## 9. 失败分类

以下任一情况必须失败或阻断：

- 未获用户二次授权。
- 真实执行不是从统一 product command 链路触发。
- legacy / H5 / direct CLI 路径被用作完成证据。
- guard blocked。
- duplicate dispatch blocked。
- diagnostics blocking degraded。
- permission envelope 不匹配。
- store revision conflict。
- prompt sha256 不一致。
- exit code nonzero。
- timeout。
- readback failed / unavailable / timed_out。
- last message 缺 marker。
- `result_count` 不明时显示为 0。
- B1 修改了任何项目文件。
- B2 修改了 `.workbench/pcr9/` 之外的项目文件。
- 核心项目文件 hash 变化。
- 需要读取 full transcript / rollout / secret 才能继续。
- runtime log / audit / readback 无法写入或无法追溯。

失败时必须：

- 写 failure evidence / handoff。
- `result_count=null`。
- 不自动重试。
- 不自动 stop / kill / restart。
- 不自动回滚，除非用户另行授权。
- 不把失败包装成成功。

## 10. Evidence / Handoff

执行线必须新增：

- `evidence/2026-06-09-unified-product-command-routing-pcr9-level-b-real-probe-v1.md`
- `handoffs/2026-06-09-unified-product-command-routing-pcr9-level-b-real-probe-v1-result.md`

复核线通过后，主管线必须新增主管复核记录：

- `evidence/2026-06-09-unified-product-command-routing-pcr9-supervisor-acceptance-review-v1.md`
- `handoffs/2026-06-09-unified-product-command-routing-pcr9-supervisor-acceptance-review-v1-result.md`

本任务包当前不创建上述 evidence / handoff；只有真实执行或复核完成后才创建。

## 11. 验证要求

真实执行后必须至少验证：

```text
cargo test --lib real_execution_command
cargo test --lib h5_project_dispatch_bridge
cargo test --lib session_continuation
cargo test --lib codex_local_runner
cargo test --lib runtime_log
cargo test --lib diagnostic
cargo test --lib workflow_authorization
cargo test --lib
cargo fmt -- --check
npm run typecheck
npm run test:offline-interaction
npm run build
```

扫描要求：

```text
rg -n "execute_workflow_node_dispatch\\(|run_workflow_machine\\(|__run_workflow_machine_real|executeLegacyWorkflowNodeDispatch|runLegacyWorkflowMachine" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
rg -n "允许一次|已自动重试|自动重试|停止成功|已恢复执行|readback.*0|读回.*0|任意项目自由执行|planned adapter.*可用|provider.*已验证" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests
rg -n "\\.codex/plugins/cache|auth/token|secret|keychain|OAuth|provider credential|full transcript|rollout" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src tasks evidence handoffs
```

命中必须分类；历史 guard / 测试 fixture / 禁止词清单可接受，产品代码新增越界命中不可接受。

## 12. 分线职责

### 主管线

- 冻结本任务包。
- 派发真实探针线和复核线。
- 不在未获授权前执行真实 Codex。
- B1 通过后决定是否进入 B2。
- 最终接受或阻断 PCR9。

### 真实探针线

- 只在用户授权后执行。
- 只使用本任务包冻结的项目、session、prompt、write roots。
- 不做架构开发。
- 不扩大 `.codex` 访问范围。
- 不把 direct CLI 诊断冒充产品路径完成。

### 复核线

- 只读复核任务包、evidence、handoff、hash、readback、runtime log、audit。
- 不改文件。
- 不执行真实 Codex。
- 不读取 `/Users/yoyi/.codex` 或插件缓存。

## 13. 主管当前结论

PCR9 当前接受为“指定 `mario test` / 指定 `codex-local` session 的统一 product command Level B 真实探针完成”，结论为带 P2 通过。

记录：

- Evidence：`evidence/2026-06-09-unified-product-command-routing-pcr9-level-b-real-probe-v1.md`
- Handoff：`handoffs/2026-06-09-unified-product-command-routing-pcr9-level-b-real-probe-v1-result.md`
- 主管 evidence：`evidence/2026-06-09-unified-product-command-routing-pcr9-supervisor-acceptance-review-v1.md`
- 主管 handoff：`handoffs/2026-06-09-unified-product-command-routing-pcr9-supervisor-acceptance-review-v1-result.md`

P2：

- B1 read-only product sidecar 里 `allowed_write_roots` 仍是项目根；read-only sandbox 下该字段不代表项目写授权。
- B1/B2 warnings 仍继承底层 continuation 标签 `product_command:run_controlled_session_continuation_real_resume_phase_b`，后续需要命名收敛避免误读。

PCR9 不接受为：

- 任意项目自由执行完成。
- 通用真实 send / resume 产品化全部完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 自动 retry / stop / restart 产品化。
- 真实 Tauri / Browser / screenshot 验收完成。
- 最终蓝图完成。

下一步允许进入 PCR10 checkpoint；权威入口同步只在 PCR10 中进行。
