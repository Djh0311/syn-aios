# Unified Product Command Routing PCR8 Test Matrix And Safety Scan Checkpoint v1

日期：2026-06-09

状态：已完成。

PCR8 是统一 Product Command Routing 的测试矩阵和安全扫描 checkpoint。它接在 PCR0-PCR7 之后，用来证明 Level A 链路的后端合约、旧入口封口、UI 产品口径、秘书只读建议、失败 / 停止 / 重试状态和敏感边界都经过统一验证。PCR8 不新增产品功能，不授权真实 `codex exec` / `codex exec resume`，不发送真实 prompt，不读写 `/Users/yoyi/.codex`，不启动 Tauri / Browser / Chrome / 截图工具。

## 0. 前置事实

- PCR0 已冻结统一 Product Command Routing 决策：真实执行必须归口统一 product command，旧入口只能 legacy / sealed / blocked。
- PCR1 已建立 product command sidecar、store skeleton 和 `WorkbenchSnapshot.real_execution_product_commands` 读模型。
- PCR2 已完成 preview / prepare 服务；prepare 只写 product command sidecar，不真实执行。
- PCR3 已完成用户 decision / confirmation 服务；approved 只是用户决定，不等于已发送或已完成。
- PCR4 已完成 Phase A no-op / fake runner；可写 attempt / continuation / runtime log ref / audit ref，但不执行真实 Codex。
- PCR5 已完成 legacy entry migration / sealing。
- PCR6 已完成 UI product linkage，并经复核线确认 P2 关闭。
- PCR7 已完成 failure / stop / retry 产品状态，并经复核线确认 P2 关闭。
- 入口文档按计划只在 PCR8 或 PCR10 checkpoint 同步；PCR8 默认先不改权威入口，除非本任务最终明确作为 checkpoint 同步收口。

## 1. 目标

PCR8 必须完成：

1. 跑完统一 Product Command Routing 的 Rust / 前端验证矩阵。
2. 做旧入口、误导 UI、真实 Codex 可达性、`.codex`、secret / token / `.env`、full transcript / rollout、planned adapter 可用性等安全扫描。
3. 分类所有扫描命中：产品代码真实风险、测试禁用词、边界说明、历史 legacy guard、开发者详情或计划文档。
4. 复核 PCR0-PCR7 的任务包状态和关键结论，确认 PCR9 前置是否满足。
5. 输出 PCR8 checkpoint 结论：`accepted` / `accepted_with_deferred_items` / `blocked`。
6. 明确 PCR8 不等于 PCR9 Level B、不等于真实执行产品化全部完成、不等于任意项目自由执行。

## 2. 非目标

PCR8 不做：

- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 `.codex/plugins/cache`、auth、token、secret、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 不启动 Browser / Chrome / Tauri / Vite dev / 截图工具。
- 不新增 runner、stop、kill、restart、retry 真实进程控制。
- 不新增 product command command / Tauri wrapper / sidecar schema。
- 不迁移数据库，不修改 `workflow-state.v0.json` 顶层结构。
- 不做 PCR9 Level B 真实探针。
- 不同步阶段 H / I 最终权威结论。

## 3. 文件范围

允许修改：

- 本任务包。
- 如验证发现测试断言或误导文案需要极小修补，可修改对应产品代码或测试，但必须先写明原因并重新跑相关验证。

默认不修改：

- `CURRENT.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `tasks/README.md`
- `docs/plans/*`
- `src-tauri/src/*`
- `src/*`
- `workflow-state.v0.json`

如果发现 P0 / P1 需要代码修补，PCR8 应转为“阻断并回交修补任务”，不得在 checkpoint 内扩大成新开发任务。

## 4. 验证矩阵

### 4.1 Rust 必跑

在 `prototypes/productized-desktop-shell/src-tauri` 下运行：

```bash
cargo test --lib real_execution_command
cargo test --lib h5_project_dispatch_bridge
cargo test --lib session_continuation
cargo test --lib codex_local_runner
cargo test --lib runtime_log
cargo test --lib diagnostic
cargo test --lib workflow_authorization
cargo test --lib
cargo fmt -- --check
```

### 4.2 前端必跑

在 `prototypes/productized-desktop-shell` 下运行：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

`npm run build` 的既有 Vite chunk size warning 可记录为非阻断；新的 TypeScript、测试或构建失败必须阻断。

## 5. 安全扫描矩阵

### 5.1 真实 Codex / runner 可达性

```bash
rg -n 'Command::new\("codex"\)|codex exec|codex exec resume' \
  prototypes/productized-desktop-shell/src-tauri/src \
  prototypes/productized-desktop-shell/src \
  prototypes/productized-desktop-shell/tests
```

分类要求：

- `Command::new("codex")` 只允许存在于受 guard / ignored real probe /明确内部 runner 边界保护的历史路径中。
- 普通 UI / wrapper / App action handler 不得新增真实 Codex 调用。
- `codex exec` 文案命中必须分类为权限说明、禁止项、历史边界或测试 fixture；不得写成已执行成功。

### 5.2 product command wrapper 普通 UI 调用

```bash
rg -n 'runRealExecutionProductCommandPhaseA|confirmRealExecutionProductCommand|recordRealExecutionProductCommandDecision|prepareRealExecutionProductCommand|previewRealExecutionProductCommand' \
  prototypes/productized-desktop-shell/src/App.tsx \
  prototypes/productized-desktop-shell/src/views \
  prototypes/productized-desktop-shell/src/components
```

普通 UI 不得直接调用这些 wrapper。若有命中，必须解释是否仅类型引用 / 测试 fixture；否则 P1。

### 5.3 误导 UI 文案

```bash
rg -n '已自动重试|自动重试已完成|已停止 agent|已重启 agent|真实派发已完成|真实 prompt 已发送|Codex 已收到任务|真实 readback 已完成|readback 0 条|结果数：0|失败已自动恢复|允许一次|H5 命令|H6 真实执行状态|启动实验画布运行|已启动实验画布运行' \
  prototypes/productized-desktop-shell/src \
  prototypes/productized-desktop-shell/tests
```

允许命中：

- 测试 forbidden text 断言。
- canvas / boundary 禁止词常量。
- 权限弹层的明确风险说明。

不允许命中：

- 普通产品 UI 成功态。
- 自动重试 / 自动恢复成功态。
- `result_count=null` 被显示成 0。

### 5.4 敏感路径和凭据

```bash
rg -n '/Users/yoyi/\.codex|\.codex/plugins/cache|auth|token|secret|\.env|keychain|OAuth|provider credential|full transcript|rollout' \
  prototypes/productized-desktop-shell/src-tauri/src \
  prototypes/productized-desktop-shell/src \
  prototypes/productized-desktop-shell/tests \
  tasks/2026-06-09-unified-product-command-routing-*.md
```

分类要求：

- `/Users/yoyi/.codex` 命中必须是边界说明、禁止项、权限说明或已授权历史 probe 记录。
- `.codex/plugins/cache` 不得作为当前任务读取来源。
- secret / token / `.env` / keychain / OAuth / provider credential 不得被产品代码读取。
- full transcript / rollout 不得在普通 UI 铺开；只允许受边界的 catalog / viewer / fixture 文案。

### 5.5 planned adapters 可用性

```bash
rg -n 'Claude Code 已接管|OpenClaw 已运行|OpenCode 已 resume|planned adapter 已可执行|provider 已验证|模型已验证|credential 已配置' \
  prototypes/productized-desktop-shell/src \
  prototypes/productized-desktop-shell/tests
```

planned adapters 仍必须显示 planned / unavailable / no credential / model unverified；不得误写成真实可用。

## 6. PCR0-PCR7 状态核对

必须核对这些任务包状态均为已完成：

- `tasks/2026-06-09-unified-product-command-routing-pcr0-entry-matrix-and-supervisor-decision-freeze-v1.md`
- `tasks/2026-06-09-unified-product-command-routing-pcr1-backend-contract-and-read-model-v1.md`
- `tasks/2026-06-09-unified-product-command-routing-pcr2-prepare-preview-service-v1.md`
- `tasks/2026-06-09-unified-product-command-routing-pcr3-decision-confirmation-service-v1.md`
- `tasks/2026-06-09-unified-product-command-routing-pcr4-execute-phase-a-noop-fake-runner-v1.md`
- `tasks/2026-06-09-unified-product-command-routing-pcr5-legacy-entry-migration-and-sealing-v1.md`
- `tasks/2026-06-09-unified-product-command-routing-pcr6-ui-product-linkage-v1.md`
- `tasks/2026-06-09-unified-product-command-routing-pcr7-failure-stop-retry-product-state-v1.md`

任一任务包仍为待执行 / 待复核时，PCR8 不得通过。

## 7. 分线职责

主管线：

- 创建 PCR8 任务包。
- 跑验证矩阵。
- 分类扫描结果。
- 只在 PCR8 最终接受时决定是否做 checkpoint 同步。

复核线：

- 只读复核 PCR8 结果和扫描分类。
- 不改文件，不跑真实 Codex，不读写 `/Users/yoyi/.codex`，不启动 GUI / 服务。

开发线：

- 默认不参与。
- 只有 PCR8 发现 P0 / P1 代码缺陷时，才由主管线拆出小修任务。

## 8. 验收标准

PCR8 可接受为完成，当且仅当：

- 4.1 / 4.2 验证矩阵通过。
- 5.1-5.5 扫描已完成并分类。
- PCR0-PCR7 任务包均为已完成。
- 未发现 P0 / P1。
- 复核线只读确认扫描分类可信。
- 没有执行真实 Codex，没有读写 `/Users/yoyi/.codex`，没有启动 GUI / Tauri / Browser / Chrome。

## 9. 不接受条件

出现以下任一情况，PCR8 不接受：

- 发现普通 UI 或 App action handler 可直接触发真实 Codex。
- 发现 `result_count=null` 被普通 UI 显示为 0。
- 发现自动重试 / 自动恢复 / 已停止 agent / 已重启 agent 成功态。
- 发现 planned adapters 被写成真实可执行。
- 发现未授权读写 `/Users/yoyi/.codex` 或读取 `.codex/plugins/cache`。
- 发现 secret / token / `.env` / credential 被产品代码读取。
- 发现 PCR0-PCR7 任一任务包未完成。

## 10. 回交格式

PCR8 完成后回交：

1. 验证矩阵结果。
2. 扫描结果和分类。
3. PCR0-PCR7 状态核对表。
4. 复核线结论。
5. 是否允许进入 PCR9 Level B 授权准备。
6. 不能声明完成事项。

## 11. 主管线执行结果

状态：已完成。

主管线已按 PCR8 矩阵完成本地验证和扫描分类。

### 11.1 验证矩阵结果

Rust：

- `cargo test --lib real_execution_command`：通过，28 passed。
- `cargo test --lib h5_project_dispatch_bridge`：通过，4 passed。
- `cargo test --lib session_continuation`：通过，17 passed / 4 ignored。ignored 均为需要显式真实执行授权的 Level B 探针。
- `cargo test --lib codex_local_runner`：通过，11 passed。
- `cargo test --lib runtime_log`：通过，6 passed。
- `cargo test --lib diagnostic`：通过，4 passed。
- `cargo test --lib workflow_authorization`：通过，1 passed。
- `cargo test --lib`：通过，297 passed / 5 ignored。ignored 包含真实执行授权探针和一个确认型写任务包测试。
- `cargo fmt -- --check`：通过。

前端：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，offline interaction tests passed: 13。
- `npm run build`：通过；仅保留既有 Vite chunk size warning。

### 11.2 扫描分类

真实 Codex / runner 可达性：

- `Command::new("codex")|codex exec|codex exec resume` 扫描有命中。
- 命中分类为：既有 MCP runner / internal runner 路径、权限弹层高风险说明、preview command 文案、测试 fixture、历史 Level B 探针 guard、任务包禁止项。
- 未发现 PCR8 新增真实 Codex 调用；本轮未执行真实 Codex。

product command wrapper 普通 UI 调用：

- `runRealExecutionProductCommandPhaseA|confirmRealExecutionProductCommand|recordRealExecutionProductCommandDecision|prepareRealExecutionProductCommand|previewRealExecutionProductCommand` 在 `App.tsx / views / components` 无命中。

误导 UI 文案：

- `已自动重试|自动重试已完成|已停止 agent|已重启 agent|真实派发已完成|真实 prompt 已发送|Codex 已收到任务|真实 readback 已完成|readback 0 条|结果数：0|失败已自动恢复|允许一次|H5 命令|H6 真实执行状态|启动实验画布运行|已启动实验画布运行` 扫描有命中。
- 命中分类为：`tests/offline-permission-dialog.test.tsx` 的 forbidden text 断言、`src/lib/canvasSurfaceBoundaries.ts` 的禁止词常量。
- `src` 普通产品 UI 未见这些成功态或误导态文案。

敏感路径 / 凭据：

- 宽扫命中很多授权、authority、token limit、secret fixture、rollout fixture 和任务包边界说明，已分类为测试 fixture / 边界说明 / 诊断白名单 / 历史授权语义。
- 窄扫 `.codex/plugins/cache` 只命中任务包中的历史偏差记录和禁止项，未命中产品代码。
- 窄扫文件读取调用与 `auth|token|secret|.env|keychain|OAuth|credential|/Users/yoyi/.codex|plugins/cache` 交集无命中，未发现产品代码直接读取这些敏感材料。

planned adapters 可用性：

- `provider 已验证|模型已验证|credential 已配置|Claude Code 已接管|OpenClaw 已运行|OpenCode 已 resume` 在 `src` 无命中。
- 测试命中均为 forbidden text 断言。

旧入口 / sealed 状态：

- `executeLegacyWorkflowNodeDispatch` / `runLegacyWorkflowMachine` 仍在 `App.tsx` 处理历史 pending action，但后端 wrapper 返回 `legacy_product_command_blocked`。
- `__run_workflow_machine_real` CLI 仍 blocked。
- MCP `canvas_start_run` / `canvas_tick_run` 仍返回 sealed / blocked。
- 这类命中为历史兼容和 guard，不是 PCR8 新增真实执行入口。

### 11.3 PCR0-PCR7 状态核对

- PCR0：已完成。
- PCR1：已完成。
- PCR2：已完成；另有主管 fresh verify 与复核线只读审查通过记录。
- PCR3：已完成。
- PCR4：已完成；另有复核线只读审查通过记录。
- PCR5：已完成；复核线只读审查通过。
- PCR6：已完成；复核线确认 P2 关闭。
- PCR7：已完成；复核线确认 P2 关闭。

### 11.4 当前结论

主管线最终结论：PCR8 可接受为 `accepted_with_deferred_items`。

可进入 PCR9 前置讨论的条件基本满足，但 PCR9 仍必须单独任务包、单独用户授权、明确测试项目、session、prompt summary、prompt hash、allowed write roots 和 `.codex` 读写范围。PCR8 本身不授权 PCR9，也不执行真实 Codex。

### 11.5 本轮边界

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`，未读取 `.codex/plugins/cache`。
- 未启动 Browser / Chrome / Tauri / Vite dev / 截图工具。
- 未同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

### 11.6 复核线结论

复核线只读审查结论：通过。

- P0 / P1：无。
- P2：无必须修补项。
- 复核线确认 PCR8 任务包足以支撑 checkpoint 验收，主管线扫描分类可信，PCR0-PCR7 顶层状态均为“已完成”。
- 复核线确认可进入 PCR9 单独授权准备，但 PCR8 本身不授权真实 Codex 执行。

复核线边界确认：未改文件，未跑测试，未执行真实 `codex exec` / `codex exec resume`，未发送 prompt，未读写 `/Users/yoyi/.codex` 或插件缓存，未启动 Browser / Chrome / Tauri / Vite / 截图工具，未同步权威入口。

### 11.7 最终口径

PCR8 已完成，结论为 `accepted_with_deferred_items`。

PCR8 接受为统一 Product Command Routing 的 Level A 测试矩阵和安全扫描 checkpoint 完成；可作为 PCR9 单独授权准备的前置之一。

PCR8 不接受为 PCR9 Level B 真实探针完成，不接受为真实 Codex 执行授权，不接受为任意项目自由执行，不接受为 planned adapters 真实接入，不接受为 provider credential / model verification 完成，也不接受为最终蓝图完成。
