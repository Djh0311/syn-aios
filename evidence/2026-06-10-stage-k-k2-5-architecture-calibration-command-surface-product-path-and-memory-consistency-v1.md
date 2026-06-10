# Stage K / K2.5 Architecture Calibration Evidence v1

日期：2026-06-10

结论：`accepted`。

K2.5 接受为 Stage K 在 K2 与 K3 之间的架构校准 gate 完成：真实执行命令面、legacy / probe / MCP 实验路径、Product Command 主路径、workspace-write proof 口径和记忆跨 sidecar consistency scanner 已收敛到可继续 K3 的状态。

K2.5 不接受为 K3 项目工作流真实自动化编排完成，不接受为 K4 记忆捕获体验完成，不接受为 K5 failure / retry / stop / restart 完成，不接受为 K6 dogfood / 真实 Tauri 全量验收完成，也不接受为任意项目无限制自由控制台完成。

## 1. 执行边界

本轮没有执行新的真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或完整 rollout。

本轮没有启动真实 Tauri、Browser、Chrome、Vite preview 或截图工具；因此不能把 K2.5 声称为真实窗口 / 截图验收完成。

本轮执行范围是架构校准和低风险修补：

- 封存旧真实执行入口。
- 删除 MCP canvas 旁路 runner。
- 增加只读 memory consistency scanner。
- 校准 workspace-write proof 口径。
- 补前端 legacy UI 封存状态和测试。

## 2. Command Surface 校准

命令面结论：

- 统一 Product Command 仍是 Stage K 真实执行归口。
- 旧 `execute_workflow_node_dispatch` / `run_workflow_machine` 普通 UI 路径不再发起 legacy Tauri wrapper。
- MCP canvas `start_run` / `tick` 已封存为 sealed experiment，不再能绕过 Product Command spawn Codex。
- transcript viewer 保持 viewer / session history 边界，不被当作 execution readback。

关键实现：

- `prototypes/productized-desktop-shell/src/App.tsx` 新增 `legacyProductCommandBlockedNotice`，普通 UI 触发旧入口时本地阻断。
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx` 将旧节点派发和旧闭环按钮改为禁用态，并显示“旧入口已封存”。
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/orchestrator.rs` 的 `start_run` / `tick` 返回 `mcp_canvas_real_execution_blocked`，并新增 `mcp_orchestrator_real_run_entries_are_sealed` 测试。
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/codex_runner.rs` 已删除，`mcp/mod.rs` 不再声明 `mod codex_runner`。

扫描结果：

```text
rg -n "Command::new\\(\"codex\"\\)|mod codex_runner|spawn_director|spawn_subagent|RealCodexResumeRunner" prototypes/productized-desktop-shell/src-tauri/src
```

结果：无命中。

## 3. Product Path / Workflow Dispatch 校准

K3 前置主路径固定为：

```text
RunUnit -> ProductCommand -> WorkerReport -> ProcessFact -> MemoryCapture
```

校准结论：

- K2/J2 的固定真实探针继续作为 fixture / 历史执行点，不再作为 K3 普通产品逻辑入口。
- J2-B bridge 不升级为 K3 主路径；K3 必须从 run unit 进入统一 Product Command。
- 前端普通项目页不再暴露可点击 legacy dispatch / workflow-machine 执行按钮。
- `src/lib/tauri.ts` 仍保留 deprecated legacy exports 作为兼容边界，但 App / ProjectsView 普通路径不再调用。

前端 legacy wrapper 扫描：

```text
rg -n "executeLegacyWorkflowNodeDispatch|runLegacyWorkflowMachine|canvasStartRun|canvasTickRun" prototypes/productized-desktop-shell/src
```

分类：

- `src/lib/tauri.ts`：deprecated wrapper / compatibility export。
- App / ProjectsView 普通产品路径：无真实调用；旧入口本地阻断或禁用。

## 4. Memory Consistency 校准

新增只读 scanner：

- `prototypes/productized-desktop-shell/src-tauri/src/memory_consistency.rs`

接入位置：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 的 `derived_store_integrity_findings`。

覆盖 sidecar：

- ProductCommand
- SessionContinuation
- RuntimeLog
- MemoryCapture
- Observation
- MemoryCandidate
- FormalMemory

finding taxonomy 覆盖：

- product command 缺 continuation / runtime log 链接。
- capture event 缺 downstream observation / candidate。
- observation 缺 capture link。
- candidate 缺 observation / capture link。
- formal memory 缺 candidate adoption link。
- sidecar load error / corrupt JSON。

边界：

- scanner 只生成 `StoreIntegrityFinding`。
- 不迁移 sidecar。
- 不自动修复链路。
- 不自动写 FormalMemory。

## 5. Workspace Write Proof 校准

`RealCodexLocalPhaseBProcessRunner` 不再通过 `sandbox != "read-only"` 推断 `writes_project_files=true`。

当前行为：

- 真实 Phase B process runner 仍可记录 `real_codex_executed=true`、`writes_codex_home=true`。
- `writes_project_files` 默认为 `false`，除非后续有 hash / manifest / diff evidence 支撑。
- 当 prompt 已发送且 sandbox 非 read-only 时，warnings 增加 `writes_project_files_unverified_requires_hash_manifest`。

这避免把“允许写工作区”错误解释成“已经证明写了项目文件”。K3 后续若需要 workspace-write 自动化，必须补 allowed path manifest、前后 hash 或 diff evidence。

## 6. Readback / Transcript / Secret 边界

readback 口径：

- `readback_unavailable` / `readback_failed` / `readback_timed_out` 保持 `result_count=null` 或 unknown 语义。
- 测试中存在 `Some(0)` fixture 用于验证 null conversion / 历史兼容，不代表生产逻辑把 unavailable 解释成真实 0 条。

transcript viewer 口径：

- `viewer_boundary` 明确 session history viewer 不是 execution readback。
- viewer 不等于 K/H execution readback，不作为 worker 结果事实来源。

敏感信息扫描分类：

- `.codex`、secret、token、`.env`、provider credential、keychain、OAuth、full transcript、rollout 等命中主要来自 guard 文案、测试 fixture、边界说明和历史记录。
- 本轮没有新增读取 secret / token / `.env` / full transcript / rollout 的产品代码路径。

## 7. 验证

Rust：

```text
cargo fmt -- --check
cargo test --lib memory_consistency
cargo test --lib mcp
cargo test --lib codex_local_runner
cargo test --lib real_execution_command
cargo test --lib
```

结果：

- `cargo fmt -- --check` 通过。
- `cargo test --lib memory_consistency` 通过。
- `cargo test --lib mcp` 通过。
- `cargo test --lib codex_local_runner` 通过。
- `cargo test --lib real_execution_command` 通过：36 passed / 7 ignored。
- `cargo test --lib` 通过：325 passed / 14 ignored。
- 保留既有 warning：`mcp/protocol.rs invalid_params is never used`。

前端：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过：offline interaction tests passed: 14。
- `npm run build` 通过，仅保留既有 Vite chunk-size warning。

架构扫描：

```text
rg -n "Command::new\\(\"codex\"\\)|mod codex_runner|spawn_director|spawn_subagent|RealCodexResumeRunner" prototypes/productized-desktop-shell/src-tauri/src
```

结果：无命中。

```text
rg -n "executeLegacyWorkflowNodeDispatch|runLegacyWorkflowMachine|canvasStartRun|canvasTickRun" prototypes/productized-desktop-shell/src
```

分类：仅 compatibility wrapper / test / sealed boundary 命中，普通产品路径不再调用。

## 8. Deferred To K3 / K4 / K5

K3 继续：

- 把项目工作流 run unit 真实派发接入 Product Command 主路径。
- 生成 worker report、process fact、runtime log、readback 和 memory capture refs。
- 不复用 legacy dispatch / workflow-machine 作为普通执行路径。

K4 继续：

- 将用户操作、Codex output、worker report、final review 进入 observation / candidate UX。
- FormalMemory 仍必须用户确认、版本、来源和审计。

K5 继续：

- failure / retry / stop / restart / cancel 的真实产品化。
- workspace-write proof 的 hash / manifest / diff evidence。

K6 继续：

- dogfood 和真实 Tauri UI 验收。

## 9. 最终结论

K2.5 架构校准 gate 通过。下一步可以进入 K3 项目工作流真实自动化编排产品化。

进入 K3 的硬边界：

- 真实执行只能走 Product Command。
- legacy dispatch / workflow-machine / MCP canvas runner 不能作为普通产品执行入口。
- workspace-write 不能只靠 sandbox 推断写入结果。
- memory consistency finding 只能解释和阻断，不自动写正式记忆。
- prompt body、secret、完整 transcript、rollout 不得进入普通 sidecar / runtime log / memory。
