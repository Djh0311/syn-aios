# Control Core Second Slice Boundary Extraction v1 Evidence

日期：2026-06-01

## 本轮结论

先说薄弱点：这轮不是 `lib.rs` 瘦身完成，也不是控制核心最终版。

本轮完成的是保守第二切片：

- 新增状态文件读写边界模块。
- 新增一类审计事件构造边界模块。
- 新增一段读模型派生边界模块。
- 至少一条真实状态写入路径、一个审计构造点、一个读模型入口已经接入新 helper。
- 没有改 workflow state JSON 结构。
- 没有执行真实 Codex。
- 没有读取或写入 `/Users/yoyi/.codex`。

依据：

- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs` 存在并承载读、校验、写、备份、原子写。
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_audit.rs` 存在并承载 `work_item_state_changed` 审计构造。
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model.rs` 存在并承载 `derive_project_blackboards` 包装。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 已声明 `mod workflow_state_store;`、`mod workflow_audit;`、`mod workflow_read_model;`。
- `cargo test --lib` 通过：87 passed、1 ignored。

## 读过的权威和依据

| 文件 | 本轮使用方式 |
|---|---|
| `CURRENT.md` | 确认当前主线、上一轮控制核心命令收敛已完成、下一步为第二切片。 |
| `tasks/README.md` | 确认当前待派发任务包和历史限制。 |
| `tasks/2026-06-01-control-core-second-slice-boundary-extraction-v1.md` | 作为本轮执行范围、禁止事项、验证命令和输出要求。 |
| `decisions/2026-06-01-architecture-module-split-guardrail-v1.md` | 确认只能做无行为变化拆模块，不能碰状态机、真实 Codex、workflow state JSON、MCP canvas run 和任务包产品规则。 |
| `docs/workbench-system-architecture-v1.md` | 确认控制核心、事实层、读模型、适配器、审计的架构分层。 |
| `evidence/2026-06-01-control-core-command-convergence-v1.md` | 确认上一轮控制核心 helper 已存在，本轮继续拆状态读写、审计、读模型。 |
| `handoffs/2026-06-01-control-core-command-convergence-v1-result.md` | 确认上一轮 handoff 建议进入控制核心第二切片。 |
| `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` | 核对真实调用点和保留风险。 |
| `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs` | 核对新增状态边界模块。 |
| `prototypes/productized-desktop-shell/src-tauri/src/workflow_audit.rs` | 核对新增审计构造边界模块。 |
| `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model.rs` | 核对新增读模型边界模块。 |

## `lib.rs` 职责分层表

| 职责 | 当前主要位置 | 本轮处理 | 保留原因或风险 |
|---|---|---|---|
| 状态文件读写 | `lib.rs` wrapper + `workflow_state_store.rs` | 已把读、校验、备份、原子写的底层实现放入 `workflow_state_store.rs`。 | 业务 mutation 仍在 `lib.rs`，因为它和状态机、审计、快照回读混在一起，硬拆会扩大风险。 |
| 状态 mutation | `lib.rs` | 本轮未移动。 | `update_work_item_state_at`、派发、权限、总指导回收等函数同时改 JSON、写 audit、调用控制核心，属于高风险路径。 |
| 审计事件构造 | `lib.rs` + `workflow_audit.rs` | 已把 `work_item_state_changed` 构造放入 `workflow_audit.rs`。 | 其他审计仍散落在 mutation 函数里，字段约定多，未做大面积迁移。 |
| 读模型派生 | `lib.rs` + `workflow_read_model.rs` | 已把 `project_blackboards_from_workflows` 的集合派生包装放入 `workflow_read_model.rs`。 | `derive_workflow_read_model`、`derive_workflow_ledger_entries`、`project_workflow_summaries` 依赖大量私有 helper，完整迁移容易引入循环依赖。 |
| Tauri command 包装 | `commands.rs` + `lib.rs` | 本轮未改。 | Task B 已做保守拆分，本轮目标不是 command 包装。 |
| 控制核心校验 | `control_core.rs` + `lib.rs` 调用点 | 本轮未扩展规则，只保持上一轮行为。 | 禁止改状态机和产品规则。 |
| 适配器执行 | `lib.rs`、`codex_db.rs`、`mcp/**` | 本轮未碰。 | 真实 Codex resume、MCP canvas run、工作流机器运行逻辑都在禁止范围内。 |
| 工作流机器 | `lib.rs` | 本轮未碰。 | 会触及真实执行编排和状态推进规则，风险高。 |
| 项目黑板读模型 | `lib.rs` + `workflow_read_model.rs` | 只抽出集合派生包装，保持黑板候选仍为读模型。 | 不能做黑板持久写入，也不能让黑板直接升级正式事实或记忆。 |
| 测试 | `lib.rs` tests | 新增和更新边界测试。 | 测试仍集中在 `lib.rs`，后续可在模块稳定后再拆。 |

## 新增模块

| 模块 | 行数 | 职责 | 依据 |
|---|---:|---|---|
| `src-tauri/src/workflow_state_store.rs` | 91 | `read_value`、`validate_value`、`write_validated`、`backup_file`、`atomic_write`。 | `wc -l` 和文件内容核对。 |
| `src-tauri/src/workflow_audit.rs` | 25 | `WorkItemStateChangedAudit` 和 `work_item_state_changed`。 | 文件内容核对。 |
| `src-tauri/src/workflow_read_model.rs` | 6 | `derive_project_blackboards` 泛型集合派生包装。 | 文件内容核对。 |

## 改为调用新 helper 的函数

| `lib.rs` 函数或调用点 | 新 helper | 完成程度 |
|---|---|---|
| `read_workflow_state_value` | `workflow_state_store::read_value` | 已接入。 |
| `validate_workflow_state` | `workflow_state_store::validate_value` | 已接入，依赖 `optional_string_from` 和 `i64_value` 作为函数参数传入，避免移动更多私有 helper。 |
| `write_validated_workflow_state` | `workflow_state_store::write_validated` | 已接入，仍通过原 wrapper 保持调用面不变。 |
| `backup_workflow_state_file` | `workflow_state_store::backup_file` | 已接入。 |
| `atomic_write_json` | `workflow_state_store::atomic_write` | 已接入。真实写入路径继续调用 `atomic_write_json` wrapper，但底层原子写已进入状态边界模块。 |
| `update_work_item_state_at` 的 `work_item_state_changed` audit | `workflow_audit::work_item_state_changed` | 已接入一类真实审计构造。 |
| `project_blackboards_from_workflows` | `workflow_read_model::derive_project_blackboards` | 已接入一个读模型派生入口。 |

## 保留在 `lib.rs` 的函数和原因

| 函数或函数族 | 保留原因 |
|---|---|
| `update_work_item_state_at` | 同时承担状态转移、JSON mutation、节点状态同步、备份、审计、快照回读；本轮只抽底层写入和一类 audit，避免改变行为。 |
| `initialize_workflow_state_at` | 负责初始化完整 v0 JSON 结构；迁移可能触碰 workflow state JSON 结构，未移动。 |
| `prepare_workflow_node_dispatch_at` / `execute_workflow_node_dispatch_at` / `read_workflow_node_dispatch_result_at` | 牵涉派发状态、runner、readback、attempt、audit；真实 Codex 路径禁止触碰。 |
| `record_workflow_permission_decision_at` | 上一轮刚新增权限确认命令，本轮不改权限产品规则。 |
| `record_workflow_dispatch_director_review_at` / offline review 函数 | 牵涉总指导回收规则和状态推进，不在本轮拆分范围。 |
| `run_workflow_machine_at` | 牵涉四角色真实执行编排，属于高风险路径。 |
| `project_workflow_summaries` | 依赖大量 JSON helper 和项目/工作流类型，本轮不完整迁移。 |
| `derive_workflow_read_model` | 依赖节点、任务包、账本、子汇报、异常、状态机、验收场景等多个私有 helper，强拆会扩大范围。 |
| `derive_workflow_ledger_entries` | 依赖 audit、dispatch、review、permission 多来源，不做事件账本完整迁移。 |
| `build_snapshot` / `build_snapshot_with_session_source` | 仍是工作台总快照组装入口，迁移需要更大的读模型模块设计。 |
| `src/mcp/**` 相关运行逻辑 | 本任务禁止启动或改 MCP canvas run。 |

## 三条边界完成程度

| 边界 | 本轮完成 | 未完成 |
|---|---|---|
| 状态写入边界 | 底层读、校验、备份、原子写已进入 `workflow_state_store.rs`；`atomic_write_json` 和 `write_validated_workflow_state` 的真实调用路径会走新模块。 | 业务 mutation 仍在 `lib.rs`；状态机语义没有移动，也不该在本轮移动。 |
| 审计构造边界 | `work_item_state_changed` 已进入 `workflow_audit.rs`，字段测试覆盖 actor、source、permission、before、after、created_at、reason。 | 其他审计类型仍在 `lib.rs`，事件账本没有完整迁移。 |
| 读模型派生边界 | `project_blackboards_from_workflows` 已通过 `workflow_read_model::derive_project_blackboards` 派生。 | `derive_workflow_read_model`、账本、异常、任务包读模型仍在 `lib.rs`。 |

## 测试和结果

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell` 或其 `src-tauri` 下运行：

| 命令 | 结果 |
|---|---|
| `npm run typecheck` | 通过。 |
| `npm run test:offline-interaction` | 通过，输出 `offline interaction tests passed: 2`。 |
| `npm run build` | 通过；Vite 仍提示 JS chunk 大于 500 kB，这是既有构建警告。 |
| `rustfmt --check src/workflow_state_store.rs src/workflow_audit.rs src/workflow_read_model.rs` | 通过。 |
| `cargo test --lib` | 通过，87 passed、0 failed、1 ignored；仍有既有 warning：`JsonRpcError::invalid_params` 未使用。 |
| `cargo fmt --check` | 未通过；diff 集中在既有 `src/lib.rs` 和 `src/mcp/**` 格式差异。本任务包明确禁止为了通过该命令批量格式化这些文件，所以未修改。 |

补充：

- `git status --short` 无法使用，因为 `/Users/yoyi/workspace/product-line` 和 `/Users/yoyi/workspace` 当前都不是 git 仓库，命令返回 `fatal: not a git repository`。
- 因此本 evidence 的依据来自文件内容、搜索结果和测试输出，不来自 git diff。

## 新增或更新测试

| 测试 | 证明什么 |
|---|---|
| `workflow_state_store_helpers_preserve_write_and_backup_behavior` | 状态读、校验、备份、合法写入和非法写入拒绝保持行为。 |
| `workflow_audit_helper_preserves_work_item_state_changed_fields` | 抽出的审计 helper 没丢字段。 |
| `project_blackboard_read_model_derives_candidates_without_state_promotion` 中新增一致性断言 | `project_blackboards_from_workflows(&snapshot.project_workflows)` 与 snapshot 中项目黑板读模型一致。 |

## 禁止事项执行情况

| 禁止项 | 本轮结果 |
|---|---|
| 不执行真实 `codex exec` / `codex exec resume` | 已遵守。 |
| 不读或写 `/Users/yoyi/.codex` | 已遵守。 |
| 不读 auth、token、`.env`、密钥、授权文件 | 已遵守。 |
| 不读完整 transcript 或 rollout JSONL 正文 | 已遵守。 |
| 不手工改真实 workflow state JSON | 已遵守。 |
| 不改变 workflow state JSON 结构 | 已遵守。 |
| 不迁移数据库 | 已遵守。 |
| 不写真实业务项目目录 | 已遵守。 |
| 不做黑板持久写入 | 已遵守。 |
| 不写正式记忆 | 已遵守。 |
| 不让秘书自动改事实 | 已遵守。 |
| 不接 Obsidian、向量库、图数据库 | 已遵守。 |
| 不启动 MCP canvas run | 已遵守。 |
| 不改首页 | 已遵守。 |
| 不改任务包产品规则 | 已遵守。 |

## 当前判断

可以接受为“控制核心第二切片边界拆分完成”。

不接受为：

- 最终架构拆分完成。
- 控制核心最终版。
- 事件账本完整迁移。
- 读模型体系最终完成。
- 黑板持久写入完成。
- 正式记忆完成。
- 秘书能力完成。
- 真实业务自动编排完成。
