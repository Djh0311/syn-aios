# Control Core Second Slice Boundary Extraction v1 Result

日期：2026-06-01

## 本轮完成

完成 `tasks/2026-06-01-control-core-second-slice-boundary-extraction-v1.md` 的保守切片。

先说限制：这轮只是把边界抽出一小段，不是把 `lib.rs` 拆干净。

已完成：

- 新增 `workflow_state_store.rs`，承载 workflow state 文件读、校验、备份、合法写入和原子写。
- 新增 `workflow_audit.rs`，承载 `work_item_state_changed` 审计事件构造。
- 新增 `workflow_read_model.rs`，承载项目黑板集合派生包装。
- `lib.rs` 的 `read_workflow_state_value`、`validate_workflow_state`、`write_validated_workflow_state`、`backup_workflow_state_file`、`atomic_write_json` 已改为调用状态边界 helper。
- `update_work_item_state_at` 的 `work_item_state_changed` 审计构造已改为调用审计 helper。
- `project_blackboards_from_workflows` 已改为调用读模型 helper。
- 新增或更新测试，覆盖状态边界、审计字段、读模型派生一致性。
- 新增 evidence：`evidence/2026-06-01-control-core-second-slice-boundary-extraction-v1.md`。

## 不接受为

不接受为：

- 最终架构拆分完成。
- 控制核心最终版。
- 事件账本完整迁移。
- 读模型体系最终完成。
- 黑板持久写入完成。
- 正式记忆完成。
- 秘书能力完成。
- 真实业务自动编排完成。

原因：

- `lib.rs` 仍有 12347 行，状态 mutation、派发、总指导回收、工作流机器、读模型大头仍在里面。
- 本轮没有迁移任何 schema，也没有改变 workflow state JSON。
- 本轮没有真实 Codex 执行验证，任务包也禁止这么做。

## 改动文件

| 文件 | 内容 |
|---|---|
| `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs` | 新增状态文件读写边界 helper。 |
| `prototypes/productized-desktop-shell/src-tauri/src/workflow_audit.rs` | 新增 `work_item_state_changed` 审计构造 helper。 |
| `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model.rs` | 新增项目黑板读模型集合派生 helper。 |
| `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` | 接入三个新模块，更新 wrapper 和调用点，新增/更新测试。 |
| `evidence/2026-06-01-control-core-second-slice-boundary-extraction-v1.md` | 新增执行证据。 |
| `CURRENT.md` | 标记本任务完成，更新下一步建议。 |
| `tasks/README.md` | 标记当前待派发任务完成，记录 evidence/handoff。 |

## 测试结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `rustfmt --check src/workflow_state_store.rs src/workflow_audit.rs src/workflow_read_model.rs`
- `cargo test --lib`

结果摘要：

- 离线交互测试：`offline interaction tests passed: 2`。
- Rust：87 passed、0 failed、1 ignored。
- Vite 仍有既有 chunk 大小 warning：构建后 JS chunk 大于 500 kB。
- Rust 仍有既有 warning：`JsonRpcError::invalid_params` 未使用。

未通过但已记录：

- `cargo fmt --check` 未通过，原因是既有 `src/lib.rs` 和 `src/mcp/**` 格式 diff 很多。
- 任务包明确禁止为了通过该命令批量格式化既有 `src/lib.rs` 或 `src/mcp/**`，所以本轮没有做大范围格式化。

## 仍然存在的架构风险

- 状态 mutation 还留在 `lib.rs`，尤其是派发、权限、总指导回收、工作流机器，后续拆分仍要小步做。
- 审计事件只抽出了一类，其他 audit 仍散落在写入函数里。
- 读模型只抽出项目黑板集合派生包装，`derive_workflow_read_model` 和账本派生仍在 `lib.rs`。
- 测试仍集中在 `lib.rs`，后续模块稳定后再考虑拆测试。
- `cargo fmt --check` 的历史格式债还在，后续若要解决，应单开格式化任务，不和业务改动混在一起。

## 下一步建议

建议下一步不要直接上秘书、正式记忆或黑板写入。

可选下一步：

- 第三切片：继续迁移低风险审计构造，例如 `workflow_permission_decision_recorded` 或派发 prepared audit。
- 读模型切片：只迁移 `derive_workflow_ledger_entries` 这种纯派生函数，并补一致性测试。
- 格式债任务：单开一次只格式化 `src/lib.rs` 和 `src/mcp/**` 的任务，避免和架构拆分混在一起。
- 黑板候选持久状态：先写 schema/迁移计划，再实现写入命令。

## 是否需要用户验收

需要。

验收时建议只看这些点：

1. 是否接受这轮只做小边界抽出，不把大 mutation 函数硬拆。
2. 是否接受 `cargo fmt --check` 因历史格式债失败但不在本轮修。
3. 是否接受下一轮继续做更小的审计或读模型切片。

## 明确未做

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未读取或写入 `/Users/yoyi/.codex`。
- 未读取 auth、token、`.env`、密钥、完整 transcript 或 rollout JSONL 正文。
- 未改 workflow state JSON 结构。
- 未手工写真实 workflow state。
- 未迁移数据库。
- 未写真实业务项目目录。
- 未启动 MCP canvas run。
- 未做黑板持久写入。
- 未写正式记忆。
- 未让秘书自动改事实。
