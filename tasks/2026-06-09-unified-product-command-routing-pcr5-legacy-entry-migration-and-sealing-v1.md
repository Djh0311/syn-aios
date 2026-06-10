# Unified Product Command Routing PCR5 Legacy Entry Migration And Sealing v1

日期：2026-06-09

状态：已完成。

后端 / 前端 wrapper 小切片任务。本文用于在 PCR1-PCR4 已完成后，收束旧真实执行入口的普通产品可达面，避免旧 `execute_workflow_node_dispatch` / `run_workflow_machine` / `read_workflow_node_dispatch_result` / `__run_workflow_machine_real` / canvas real run 被误读为统一 Product Command Routing 的正式入口。

## 0. 前置事实

- PCR0 冻结方向：真实执行必须归口统一 Product Command Routing；旧 workflow / machine / canvas 入口保持 legacy / sealed / blocked。
- PCR1 已建立 `real-execution-product-commands.v1.json` 类型、store skeleton 和 read model。
- PCR2 已完成 preview / prepare，只写 command + preview snapshot。
- PCR3 已完成 decision / confirmation，只写 decision + audit ref。
- PCR4 已完成 Phase A no-op / fake runner，写 product attempt、continuation、runtime log、audit、readback refs，但不执行真实 Codex。
- 入口文档仍留到 PCR8 或 PCR10 checkpoint 同步；PCR5 不更新 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

## 1. 目标

PCR5 目标：

1. 普通前端代码不再 import / call 容易误读的旧 alias：`executeWorkflowNodeDispatch`、`runWorkflowMachine`。
2. TS wrapper 只保留显式 legacy 名称：`executeLegacyWorkflowNodeDispatch`、`runLegacyWorkflowMachine`。
3. 旧 Tauri command 继续返回 `legacy_product_command_blocked`，不能直达 runner。
4. `read_workflow_node_dispatch_result` 继续 blocked，不能冒充 H/H5 execution readback。
5. `__run_workflow_machine_real` CLI 继续 blocked。
6. MCP canvas `canvas_start_run` / `canvas_tick_run` 继续 sealed。
7. 不接 UI 新执行按钮，不做 PCR6 UI 产品链路接入。

## 2. 非目标

PCR5 不做：

- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 不调用 Phase B real runner 或 H3-B real runner。
- 不新增 `Command::new("codex")`。
- 不删除历史内部 helper；测试和历史路径可继续用 `_at` helper。
- 不做 PCR6 UI。
- 不做 PCR8 checkpoint 文档同步。
- 不做 PCR9 Level B。

## 3. 文件范围

允许修改：

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`，仅允许补 legacy alias / no direct UI call 断言。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`，仅当旧 command wrapper 未 blocked 时补 guard。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`，仅当 CLI guard 未 blocked 时补 guard。
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/commands.rs`，仅当 canvas run 未 sealed 时补 guard。
- 本任务包。

默认不修改：

- `CURRENT.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `tasks/README.md`
- `src-tauri/src/codex_local_runner.rs`
- `src-tauri/src/mcp/codex_runner.rs`
- `src/views/*`
- `src/components/*`

## 4. 实现要求

### 4.1 前端 alias 收束

- `App.tsx` 不得 import `executeWorkflowNodeDispatch` / `runWorkflowMachine`。
- `App.tsx` 若仍需要处理历史 pending action，必须显式调用 `executeLegacyWorkflowNodeDispatch` / `runLegacyWorkflowMachine`，并依赖后端 blocked guard。
- `src/lib/tauri.ts` 不再导出 `executeWorkflowNodeDispatch` / `runWorkflowMachine` 这两个易误读 alias。
- `executeLegacyWorkflowNodeDispatch` / `runLegacyWorkflowMachine` 保留，并用注释说明 legacy backend guarded / not unified product command。

### 4.2 后端旧入口确认

必须确认：

- `execute_workflow_node_dispatch` Tauri command 返回 `legacy_product_command_blocked`。
- `run_workflow_machine` Tauri command 返回 `legacy_product_command_blocked`。
- `read_workflow_node_dispatch_result` Tauri command 返回 `legacy_product_command_blocked`。
- `run_workflow_machine_cli` / `__run_workflow_machine_real` 返回 `legacy_product_command_blocked`。
- `canvas_start_run` / `canvas_tick_run` 返回 `mcp_canvas_real_execution_blocked`。

如当前已满足，只补测试 / 扫描记录，不做无意义重构。

### 4.3 测试要求

至少补齐或确认：

- 普通前端代码不再引用 `executeWorkflowNodeDispatch` / `runWorkflowMachine`。
- `src/lib/tauri.ts` 不再导出旧 alias。
- 旧 Tauri command / CLI / MCP canvas 仍 blocked。
- 旧 internal `_at` helper 可保留给测试 / 历史路径，但不得被普通 command 直接调用。

## 5. 验证命令

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
npm run typecheck
npm run test:offline-interaction
npm run build
```

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri
cargo test --lib real_execution_command
cargo test --lib
cargo fmt -- --check
```

## 6. 扫描要求

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
rg -n '\bexecuteWorkflowNodeDispatch\b|\brunWorkflowMachine\b' src/App.tsx src/views src/components tests
```

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
rg -n 'export const executeWorkflowNodeDispatch|export const runWorkflowMachine' src/lib/tauri.ts
```

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri
rg -n 'fn execute_workflow_node_dispatch|fn read_workflow_node_dispatch_result|fn run_workflow_machine|run_workflow_machine_cli|canvas_start_run|canvas_tick_run|legacy_product_command_blocked|mcp_canvas_real_execution_blocked' src
```

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri
rg -n 'Command::new\("codex"\)|codex exec|codex exec resume|run_real_resume_phase_b|run_real_new_session_h3_b' src
```

第一组和第二组应无命中。第三组应命中旧入口 guard。第四组会命中既有 runner adapter / Phase B / H3-B / fixture / 文案，必须分类，不得新增 PCR5 普通入口命中。

## 7. 验收标准

PCR5 可接受为完成，当且仅当：

- 普通 UI 不再 import / call `executeWorkflowNodeDispatch` / `runWorkflowMachine`。
- `src/lib/tauri.ts` 不再导出两个旧 alias。
- 显式 legacy wrapper 仍存在，且后端 command blocked。
- CLI `__run_workflow_machine_real` blocked。
- MCP canvas real run sealed。
- 验证命令通过。
- 扫描完成并分类。
- 未同步权威入口。

## 8. 不接受条件

出现以下任一情况，PCR5 不接受：

- 打开真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 新增 UI 执行按钮。
- 删除历史 helper 导致既有测试大面积失效。
- 把 legacy wrapper 说成统一 product command。
- 同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

## 9. 回交格式

开发线完成后中文回交：

1. 修改文件。
2. alias 收束结果。
3. 旧入口 guard 核对。
4. 验证命令结果。
5. 扫描分类。
6. 不能声明完成事项。

## 10. 本线执行结果草稿，待复核

执行时间：2026-06-09

执行人：全局主管线。

状态：复核线只读审查通过，已标记完成。

### 10.1 修改文件

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `tasks/2026-06-09-unified-product-command-routing-pcr5-legacy-entry-migration-and-sealing-v1.md`

未改 Rust 后端、runner、MCP runner、views/components、权威入口文档。

### 10.2 alias 收束结果

- `App.tsx` 不再 import / call `executeWorkflowNodeDispatch`。
- `App.tsx` 改为显式 import / call `executeLegacyWorkflowNodeDispatch`。
- `App.tsx` 不再 import / call `runWorkflowMachine`。
- `App.tsx` 改为显式 import / call `runLegacyWorkflowMachine`。
- `src/lib/tauri.ts` 移除了 `export const executeWorkflowNodeDispatch = executeLegacyWorkflowNodeDispatch`。
- `src/lib/tauri.ts` 移除了 `export const runWorkflowMachine = runLegacyWorkflowMachine`。
- `executeLegacyWorkflowNodeDispatch` / `runLegacyWorkflowMachine` 继续保留，并仍调用后端 blocked Tauri command。

### 10.3 旧入口 guard 核对

- `execute_workflow_node_dispatch` 仍返回 `legacy_product_command_blocked`。
- `read_workflow_node_dispatch_result` 仍返回 `legacy_product_command_blocked`。
- `run_workflow_machine` 仍返回 `legacy_product_command_blocked`。
- `run_workflow_machine_cli` / `__run_workflow_machine_real` 仍返回 `legacy_product_command_blocked`。
- `canvas_start_run` / `canvas_tick_run` 仍返回 `mcp_canvas_real_execution_blocked`。
- internal `_at` helper 仍保留给测试 / 历史路径，普通 command 不直达 runner。

### 10.4 验证结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，13 passed。
- `npm run build`：通过，仅保留既有 Vite chunk size warning。
- `cargo test --lib real_execution_command`：通过，27 passed。
- `cargo test --lib`：通过，296 passed / 5 ignored。
- `cargo fmt -- --check`：通过。

### 10.5 扫描分类

- `rg -n '\bexecuteWorkflowNodeDispatch\b|\brunWorkflowMachine\b' src/App.tsx src/views src/components tests`：无命中。
- `rg -n 'export const executeWorkflowNodeDispatch|export const runWorkflowMachine' src/lib/tauri.ts`：无命中。
- 旧入口 guard 扫描命中 `commands.rs`、`lib.rs`、`mcp/commands.rs` 中的 blocked / sealed wrapper 和测试，符合 PCR5 预期。
- 真实 Codex 关键词扫描命中既有 `mcp/codex_runner.rs`、`lib.rs` runner、`session_continuation_store.rs` Phase B/H3-B 授权路径、worker protocol command preview、runtime log test fixture 和历史边界文案；PCR5 没有新增真实 runner 或普通入口命中。

### 10.6 边界确认

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Browser/Chrome/Tauri/Vite dev/screenshot。
- 未调用 Phase B real runner 或 H3-B real runner。
- 未新增 UI 执行按钮。
- 未同步权威入口。

### 10.7 不能声明完成事项

- 不能声明统一 Product Command Routing 全链路完成。
- 不能声明 PCR6 UI 产品链路完成。
- 不能声明 PCR8 checkpoint 完成。
- 不能声明 PCR9 Level B 真实执行开放。
- 不能声明旧 internal helper 已删除或完全迁移；本轮只收束普通 UI alias 和确认旧 command guard。

### 10.8 主管复核结论

复核线 `019ea33a-23c4-7c10-8db3-95b8cf910fe7` 已只读回交：

- P0：无。
- P1：无。
- P2：无必须修补项。
- 结论：PCR5 可由主管线标记完成。

主管线接受该结论。PCR5 只接受为 legacy alias / 入口可达面收束完成，不接受为统一 Product Command Routing 全链路完成，不接受为 PCR6 UI 产品链路完成，不接受为 PCR9 Level B 真实执行开放。
