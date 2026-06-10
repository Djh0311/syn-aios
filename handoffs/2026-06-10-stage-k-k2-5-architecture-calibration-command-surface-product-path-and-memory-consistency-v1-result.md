# Stage K / K2.5 Architecture Calibration Handoff v1

日期：2026-06-10

结论：`accepted`。

K2.5 已完成。Stage K 原目标不变：自由操控 Codex + 自动化工作流 + 记忆层记录。K2.5 只作为 K2 与 K3 之间的架构校准 gate，已确认 K3 可以继续，但 K2.5 本身不等于 K3/K4/K5/K6 完成。

## 已完成

- 删除 MCP canvas legacy real runner：`prototypes/productized-desktop-shell/src-tauri/src/mcp/codex_runner.rs`。
- `mcp/orchestrator.rs` 的 `start_run` / `tick` 已封存，不能绕过 Product Command spawn Codex。
- App 普通路径不再调用 legacy workflow dispatch / workflow machine Tauri wrapper。
- ProjectsView 旧节点派发 / 旧闭环按钮已改为 disabled，并标注旧入口封存。
- 新增 `memory_consistency.rs` 只读跨 sidecar scanner，并接入 store integrity。
- `codex_local_runner.rs` 不再用 `sandbox != "read-only"` 推断 `writes_project_files=true`；缺 hash / manifest 时只给 warning。
- offline interaction 测试已覆盖旧入口封存按钮和禁用状态。

## 验证

已通过：

- `cargo fmt -- --check`
- `cargo test --lib memory_consistency`
- `cargo test --lib mcp`
- `cargo test --lib codex_local_runner`
- `cargo test --lib real_execution_command`
- `cargo test --lib`：325 passed / 14 ignored
- `npm run typecheck`
- `npm run test:offline-interaction`：14 passed
- `npm run build`：通过，仅既有 Vite chunk-size warning

扫描：

- 裸 `Command::new("codex")` / `mod codex_runner` / `spawn_director` / `spawn_subagent` / `RealCodexResumeRunner`：无命中。
- legacy frontend wrapper：普通 App / ProjectsView 路径不再调用；仅 compatibility exports / sealed boundary / tests。

## 边界

本轮没有执行新的真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或完整 rollout。

本轮没有真实 Tauri / Browser / Chrome / screenshot 验收。

## 下一步

进入 K3：项目工作流真实自动化编排产品化。

K3 必须遵守：

- RunUnit 真实派发走 `RunUnit -> ProductCommand -> WorkerReport -> ProcessFact -> MemoryCapture`。
- 旧 dispatch / workflow-machine 只能作为历史 compatibility，不作为普通产品路径。
- MCP canvas runner 不能作为产品执行旁路。
- workspace-write 必须补 hash / manifest / diff evidence，不能只靠 sandbox 推断。
- memory consistency finding 只解释和阻断，不自动修正式记忆。

## 不可冒领

K2.5 不接受为：

- K3 项目工作流真实自动化编排完成。
- K4 记忆捕获体验完成。
- K5 retry / stop / restart / cancel 完成。
- K6 dogfood 或真实 Tauri UI 全量验收完成。
- 任意项目无限制自由控制台完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- FormalMemory 自动写入完成。
