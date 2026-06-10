# Evidence：Stage E / E4 Session Continuation Protocol And Permission Preview v1

日期：2026-06-06

## 1. 结论

E4 已完成，接受为：

- 会话继续 / resume 的只读协议和权限预览。
- `SessionContinuationRequest` / `SessionContinuationPreview` / `SessionContinuationGuardResult` 等价模型落地。
- `WorkbenchSnapshot.session_continuation_previews[]` 可从 adapter descriptor、session operation、provider availability 和 workflow session binding 派生。
- 智能体页可显示 target session、project / workflow / node binding、cwd、allowed write roots、sandbox、prompt summary、readback expectation、failure handling、audit impact 和 guard result。
- guard 能阻断 missing binding、cwd 越界、敏感路径、planned adapter、缺 readback strategy，并把用户确认停在 preview / confirmation boundary。

E4 不接受为：

- 真实发消息完成。
- 通用 `codex exec resume` 完成。
- prompt 已发送或 Codex 已收到任务。
- attempt / dispatch / readback / runtime log 已写入。
- worker / agent 已执行。
- planned adapters 具备继续会话能力。
- 阶段 G 真实 Tauri 全面验收完成。

## 2. 范围、边界和偏差记录

E4 产品实现没有新增真实 `codex exec` 或 `codex exec resume` 执行路径，没有发送真实 prompt，没有调用外部 agent / provider，没有新增 execution store / credential store / adapter sidecar / provider sidecar，没有迁移数据库，没有改 `workflow-state.v0.json` 顶层结构，没有写真实 attempt / dispatch / readback / worker report / runtime log。

实现只把 E2 的 operation boundary 和 E3 的 provider availability 接入 E4 guard 输入，不把它们升级为真实执行能力。

偏差记录：

- 收尾文档残留检查中，有一条 `rg` 命令把包含 Markdown 反引号的搜索模式放进 shell 双引号，导致 shell 将反引号内的 `codex exec resume` 当作命令替换执行。
- 该偏差不是 E4 产品代码路径、不是 UI 路径、不是 workflow dispatch 路径；但它确实触发了一次 Codex CLI 命令尝试。
- 当时输出显示：`Reading prompt from stdin...`、`No prompt provided via stdin.`，并提示访问 `/Users/yoyi/.codex/state_5.sqlite` 时因 readonly database 失败。
- 因此本轮不能严格声称“完全未执行 Codex 命令 / 完全未触碰 `/Users/yoyi/.codex`”。只能确认没有发送 prompt，且从命令输出看没有完成 resume。
- 后续搜索含反引号文本必须使用单引号或 `rg -F`，不能使用 shell 双引号包住未转义反引号。

## 3. send_message 与 workflow dispatch resume 差异

| 项 | `send_message` / 会话继续 | `workflow dispatch resume` / 项目派发 |
| --- | --- | --- |
| 目标 | 继续一个已绑定会话的下一轮项目意图 | 在项目工作流授权范围内派发任务包 |
| 必须绑定 | project / workflow / node / session | project / workflow / node / task package / session |
| E4 状态 | 只派生 preview 和 guard | 只参考既有安全经验，不启动派发 |
| prompt | 只显示 summary，不发送 raw prompt | 由任务包生成，E4 不触发 |
| 权限 | 缺用户确认时停在 `needs_user_confirmation` | C1-C4 guard 负责 prepared dispatch |
| readback | 只定义 expectation | C5 / 后续 runtime 读模型负责结果可见化 |
| 写入 | E4 不写执行态 | 既有 dispatch 才会写状态，E4 不调用 |
| 风险 | 自由聊天绕过任务包、cwd 越界、readback 缺失 | 授权范围、任务包、记忆包、readback、audit |

证据：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中 E4 的 `derive_session_continuation_previews` / `inspect_session_continuation_guard` 只生成预览和 guard。
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx` 的 E4 面板文案明确“不是执行入口”。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx` 的 E4 scenario 断言没有发送按钮、没有误导文案、没有秘书执行 proposal。

## 4. 最终模型字段

后端类型位于 `prototypes/productized-desktop-shell/src-tauri/src/types.rs`，前端类型位于 `prototypes/productized-desktop-shell/src/lib/types.ts`。

`SessionContinuationRequest` 字段：

- `adapter_id`
- `operation_id`
- `project_id`
- `project_root`
- `workflow_id`
- `node_id`
- `session_id`
- `target_cwd`
- `allowed_write_roots`
- `sandbox`
- `prompt_source_kind`
- `prompt_summary`
- `readback_strategy`
- `requested_by`
- `user_confirmation_state`

`SessionContinuationPreview` 字段：

- `preview_id`
- `adapter_id`
- `operation_id`
- `target_session_id`
- `target_session_title`
- `project_id`
- `project_root`
- `workflow_id`
- `node_id`
- `binding_id`
- `work_item_id`
- `target_cwd`
- `allowed_write_roots_summary`
- `sandbox_summary`
- `prompt_source_kind`
- `prompt_summary`
- `readback_expectation`
- `failure_handling`
- `audit_impact`
- `provider_availability_summary`
- `guard_result`
- `request`
- `user_visible_warnings`

`SessionContinuationGuardResult` 字段：

- `status`: `allowed_preview` / `needs_user_confirmation` / `blocked` / `requires_future_task`
- `severity`
- `blocks_execution`
- `allows_preview`
- `requires_user_confirmation`
- `reasons`
- `required_fixes`
- `warnings`

E4 的 `ContinuationAuditImpact` 明确：

- `writes_attempt_in_e4: false`
- `writes_dispatch_in_e4: false`
- `writes_readback_in_e4: false`
- `impact_kind: preview_only_no_execution`

## 5. Guard 矩阵结果

后端单测：`session_continuation_guard_covers_e4_boundary_matrix`。

| 场景 | 最终结果 |
| --- | --- |
| `codex-local` + 完整 project/workflow/node/session binding + cwd 在项目范围 + readback required + 未确认 | `needs_user_confirmation`，`blocks_execution: true` |
| 同上，但用户确认 | `allowed_preview`，仍 `blocks_execution: true` |
| 缺 project binding | `blocked`，reason 包含 `missing_project_binding` |
| cwd 越出 project root / allowed roots | `blocked`，reason 包含 `cwd_out_of_scope_blocked` |
| target cwd 命中 `.env` 等敏感路径 | `blocked`，reason 以 `sensitive_path_blocked` 开头 |
| 缺 readback strategy | `blocked`，reason 包含 `readback_strategy_required` |
| planned adapter，如 `claude-code` | `blocked`，reason 包含 `planned_adapter_blocked` |
| provider availability 为 planned / credential_missing / external_call_blocked | 作为 guard 输入产生 future-task / warning，不等于授权 |

前端离线测试还覆盖：

- 5 个 adapter 派生 10 条 send_message / resume preview。
- `codex-local` 两条预览携带 target session 和 project root。
- planned adapters 保持 blocked 或 future task。
- UI 不出现真实发送按钮。
- 秘书只给风险和查看建议，不给 send / resume / approve / retry action proposal。

## 6. UI 和秘书证据

UI 位置：

- 仍使用既有 `智能体` 页面。
- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不改项目工作流画布主区域。

前端文件：

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx` 新增局部“会话继续预览 / 权限预览”面板。
- `prototypes/productized-desktop-shell/src/lib/sessionContinuation.ts` 提供前端 fallback preview / guard。
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts` 新增 `session_continuation_boundary` risk 和 `inspect_session_continuation_preview` suggestion。
- `prototypes/productized-desktop-shell/src/App.tsx` 在空 snapshot 中补 `session_continuation_previews: []` 并传入 `AgentView`。

UI 明确显示：

- “E4 预览协议，不是执行入口”
- “不会发送 prompt”
- “不会执行 resume”
- “不会写 Codex 原生状态”
- “不会写 attempt、dispatch 或 readback”

PermissionDialog：

- E4 没有接入真实 PermissionDialog 执行路径。
- 测试断言会话继续预览区域没有 `发消息`、`发送`、`resume`、`申请确认`、`执行`、`重试` 等按钮文本。
- 因此不存在“确认后触发真实 `codex exec resume`”的 E4 路径。

秘书：

- 秘书只读模型能解释 E4 风险和建议查看预览。
- 测试断言秘书 `action_proposals` 不包含发送、resume、批准、确认预览或重试。

## 7. 扫描结果

禁止误导文案扫描：

```text
rg -n "已发送|已 resume|Codex 已收到任务|自动派发已开始|worker 执行中|readback 已完成|Claude Code 可继续会话|OpenClaw 可 resume|OpenCode 已支持发送|真实 Codex 已执行" prototypes/productized-desktop-shell/src
```

结果：无命中，命令 exit code 1。

真实执行 / 敏感路径扫描：

```text
rg -n "Command::new\\(\"codex\"\\)|codex exec resume|\\.codex|read_to_string\\(.*auth|read_to_string\\(.*token|read_to_string\\(.*secret|read_to_string\\(.*\\.env|keychain|oauth|provider credential" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
```

结果：有历史 / 边界命中。分类如下：

- E4 新增合理命中：`src/lib/sessionContinuation.ts` 和 `src-tauri/src/lib.rs` 中的敏感路径 guard，用于阻断 `.codex`、`.env`、auth/token/secret/keychain、OAuth、provider credential 等路径。
- 既有真实 workflow / MCP 路径：`src-tauri/src/lib.rs` 顶部的 `Command::new("codex")` workflow dispatch runner、`src-tauri/src/mcp/codex_runner.rs` 的 MCP runner。E4 没有新增或调用这些路径。
- 既有 UI 权限说明：`ProjectsView.tsx`、`PermissionDialog.tsx`、`OfflineRoleOrchestrationPanel.tsx`、`adapterCapabilities.ts` 等历史文案说明哪些旧动作会或不会执行 Codex / 写 `.codex`。
- 会话读取 / 测试 fixture：`codex_db.rs`、`codex_transcript.rs`、`src-tauri/src/lib.rs` 的测试 fixture 中存在 `.codex` 路径字符串；本轮没有读取真实 `/Users/yoyi/.codex`。

结论：扫描没有发现 E4 新增真实 Codex runner、secret 读取、provider credential 读取或 `.codex` 写入路径。

额外偏差说明：后续“文档残留检查”命令误触发了 shell 反引号命令替换，详见第 2 节。这不改变代码扫描结论，但改变本轮过程合规结论。

## 8. 验证命令

前端：

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：通过，`offline interaction tests passed: 11`。

```text
npm run build
```

结果：通过。Vite 仍有既有 chunk-size warning。

Rust：

```text
cargo test --lib session_continuation
```

结果：通过，1 passed。

```text
cargo test --lib session_operation
```

结果：通过，1 passed。

```text
cargo test --lib provider_availability
```

结果：通过，1 passed。

```text
cargo test --lib workflow_authorization
```

结果：通过，1 passed。

```text
cargo test --lib
```

结果：通过，224 passed，1 ignored。

```text
rustfmt --check src/types.rs src/lib.rs src/control_core.rs src/commands.rs
```

结果：通过。

备注：cargo 输出仍有既有 `JsonRpcError::invalid_params` unused warning，不是 E4 新增失败。

## 9. 未完成 / 不接受项

- 未做真实窗口 / 截图验收，因此不能接受为阶段 G 验收。
- E4 产品代码未实现真实 `codex exec` / `codex exec resume`，但收尾检查中发生一次误触发 Codex CLI 的操作偏差。
- 未发送 prompt。
- 未通过 E4 产品代码写 `/Users/yoyi/.codex`；但偏差命令尝试访问 `/Users/yoyi/.codex/state_5.sqlite` 并因只读失败。
- 未写真实 attempt / dispatch / readback。
- 未新增 continuation execution store。
- 未新增 credential store、provider sidecar、adapter sidecar。
- 未迁移数据库。
- 未让 planned adapters 具备继续会话能力。

## 10. 后续建议

下一步应单独拆 E5：`codex-local` controlled send / resume minimal loop。E5 如涉及真实 `codex exec resume`、prompt 发送或 `/Users/yoyi/.codex` 写入，必须在任务包里列清权限、读写范围、回滚和用户确认，且先取得用户明确授权。
