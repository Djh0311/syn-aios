# Evidence: Stage H / H5 Supervisor Acceptance Review v1

日期：2026-06-08

## 1. 结论

全局主管复核 H5 开发线回交后，H5-Level-A 接受为：

```text
accepted_as_h5_level_a_non_real_product_path_after_supervisor_review
```

接受范围：

- H5-Level-A 非真实产品路径集成完成。
- 后端 `h5_project_dispatch_bridge` 能只读生成 / 校验 `prepared dispatch -> permission envelope -> CodexLocalExecutionRequest -> guard preview -> runtime/audit preview -> readback boundary -> worker report candidate -> process fact handoff -> C6 handoff status`。
- H5 preview 固定 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`、`writes_workbench_state=false`。
- `readback_boundary.result_count = null`；`not_attempted`、readback failed / unavailable / timed out 等 unknown-result 不能显示成真实 0 条。
- stale memory packet、duplicate active attempt、diagnostics blocking degraded、non-prepared dispatch、missing prompt ref/hash、resume target missing、new_session 缺 H3-B / Level-B 授权会阻断 Level B。
- 前端只补 TS 类型和 Tauri wrapper；未新增可见 UI、执行按钮或误导状态。

不接受为：

- H5 整体完成。
- H5-Level-B 真实项目工作流派发已授权或已执行。
- 真实 worker 已执行。
- 真实 Codex 已执行。
- `codex exec` / `codex exec resume` 已执行。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- H3-B retry 已授权、已执行或已成功。
- H4-Level-B 真实失败 / 超时探针已完成。
- worker report 已成为正式事实。
- observation / candidate 已成为正式记忆。
- planned adapters 已接入。
- provider credential / model 已验证。
- 阶段 H 已完成。

## 2. 主管复核动作

本轮全局主管线完成：

- 读取 H5 task / evidence / handoff。
- 复核 H5 新增后端 bridge、Tauri command、Rust / TS 类型和 Tauri wrapper。
- 复核 H5 开发线记录的验证结果。
- 派发 H5-Level-A 只读辅助复核线并读取其回交进度 / 结论。
- 复跑 H5 定向 Rust 测试。
- 扫描真实执行、`.codex`、冒领口径和 UI 误导文案。

多线程协作记录：

- H5 开发线：`019ea3a3-2677-7ce2-bcb0-08324fb0368e`，已回交。
- H5-Level-A 只读辅助复核线：`019ea394-3cc3-7b01-b278-0af41a8e05fb`，只读复核，无文件修改、无真实执行。

## 3. 代码复核摘要

复核文件：

- `prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`

复核结论：

- `preview_h5_project_workflow_dispatch` 只调用 `h5_project_dispatch_bridge::preview_h5_project_workflow_dispatch_at`。
- H5 bridge 只读 workflow state 和工作台自有 `session-continuations.v1.json` sidecar，用于 prepared dispatch、task package artifact、memory snapshot 和 duplicate scope 校验。
- H5 bridge 构造 `CodexLocalExecutionRequest` 后只调用 `inspect_codex_local_execution_guard`，没有调用真实 runner。
- H5 bridge 没有 `Command::new("codex")`、没有 `spawn()`、没有 prompt stdin 写入、没有工作台 state 写入、没有 `.codex` 读写。
- 测试中的 `fs::write` 只写临时 fixture 和测试 sidecar，不是产品路径。
- TS 侧只有 `H5ProjectWorkflowDispatchPreview*` 类型和 `previewH5ProjectWorkflowDispatch(...)` wrapper，没有可见 UI 调用或执行按钮。

## 4. 验证依据

沿用并复核 H5 开发线验证记录：

```text
cargo test --lib h5_project_dispatch_bridge -- --nocapture
cargo test --lib
rustfmt --check src/h5_project_dispatch_bridge.rs src/types.rs src/commands.rs src/lib.rs
npm run typecheck
npm run test:offline-interaction
npm run build
```

开发线记录结果：

- H5 定向 Rust：3 passed。
- `cargo test --lib`：257 passed / 3 ignored。
- `rustfmt --check`：passed。
- `npm run typecheck`：passed。
- `npm run test:offline-interaction`：12 passed。
- `npm run build`：passed；保留既有 Vite chunk size warning。
- Rust 保留既有 warning：`JsonRpcError::invalid_params` dead code。

本轮主管线补充验证：

```text
cargo test --lib h5_project_dispatch_bridge -- --nocapture
```

结果：

- 3 passed，0 failed。
- 既有 `JsonRpcError::invalid_params` unused warning 仍存在，非 H5 新增阻断。

本轮主管线补充扫描：

```text
rg -n 'Command::new|spawn\(|write_all|codex exec|exec resume|\.codex|read_to_string|fs::write|save_|atomic_write|record_worker|record_process|prepare_authorized|runner|resume_with_options' ...
rg -n 'H5 草案|Level A 待设计|H5-Level-A 待|Level A 不修改产品代码|H5 已完成|H5-Level-B.*已完成|H5-Level-B.*已授权|真实项目工作流派发.*已执行|真实 worker 已执行|真实 Codex 已执行|阶段 H 已完成|H3-B 成功|H3-B retry 已授权|H4-Level-B.*已完成' ...
rg -n 'Codex 已收到任务|worker 执行中|真实派发已开始|worker 已执行|系统已记住|真实 0 条|0 条结果|result_count: 0|result_count\s*=\s*Some\(0\)|result_count\s*=\s*0' ...
```

分类结果：

- H5 bridge 无真实 runner 调用、无 prompt 写入、无 `.codex` 读写、无产品路径 workflow state 写入。
- 口径扫描命中均为“不接受为 / 禁止声称 / 边界说明”等负向文案，未发现 H5-Level-B 已授权 / 已执行或阶段 H 已完成的冒领。
- UI 误导文案命中主要来自任务包禁止项、evidence 禁止项和既有测试黑名单；H5 没有新增可见 UI 或执行按钮。
- `ProjectsView.tsx` 中既有“真实 0 条结果”逻辑属于历史路径，不是 H5 新增；H5 readback preview 本身保持 `result_count=null`。

## 5. 边界确认

本轮主管线没有：

- 改产品代码。
- 改可见 UI。
- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送 prompt。
- 创建真实 Codex session。
- 创建真实项目派发 run。
- 读写 `/Users/yoyi/.codex`。
- 读取 auth、token、secret、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 启动 Tauri 或 GUI。

## 6. 下一步

H5-Level-A 可作为已通过主管复核的非真实预览 / 校验链路进入后续决策。

下一步可选路径仍需全局主管逐项授权：

- H3-B retry final approval / real new session fixture run。
- H4-Level-B 真实失败 / 超时探针。
- H5-Level-B 真实项目工作流派发授权包和执行点确认。
- H6 真实执行 UI 产品化和 Tauri 验收准备。

未获执行点明确授权前，不能执行新的真实 `codex exec` / `codex exec resume`，不能发送真实 prompt，不能读写 `/Users/yoyi/.codex`，不能把 H5-Level-A 冒充为真实项目工作流派发完成。
