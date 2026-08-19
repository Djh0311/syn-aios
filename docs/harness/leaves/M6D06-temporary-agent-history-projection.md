# M6D06 临时 agent 历史投影（ORG-006，域层）

阶段：stage-15 M6 全局主管与内部组织（域层先行，UI 验收载体为新壳）

状态：`CURRENT` / `IN_PROGRESS`。stage-15 检查点 CP3 的第二叶；M6D05 内容 `a58815f` 已主管自复核 PASS 并归档。本叶做完即到 CP3，必须先收口交包并取得独立 PASS。

来源收据：stage-6 计划第 4 节 SYN-ORG-006、第 3 节 `TemporaryAgent` 与 `ChildRunRef` 不变量；执行引用字段集固定依 `handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md` 第 1 节；判据以 M6D01 冻结合同为准。

目标：把临时 agent 的工作历史从 M5 不可变执行事实投影出来，能查任务、结果、失败与来源，并与稳定成员严格分型、不自动晋升。

做完的标准：

1. 新增 `m6_org_temporary_agent_projection.rs`（**不要用被保全占用的 `m6_temporary_agent_history.rs` 名字**），从 M5 WorkItem / Attempt / RuntimeReceipt / Report / audit 投影临时 agent；
2. 执行引用必须是 M5 完整 envelope 的精确 join：`project_id + orchestration_id + workflow_run_id + work_item_id + node_id + dispatch_id + attempt_id + grant_id + worker_role_session_id + authoritative receipt + trusted actor + hashes`。任一字段缺失即拒，**不得**从 report 自报、缺字段兼容或 runtime trace 推导（反例测试逐类覆盖）；
3. runtime child session 只有在能精确绑定 attempt / grant / actor / receipt 时才成为来源；禁止按 session 名称或 parent/child 关系猜（反例测试）；`ChildRunRef` 只是执行引用，不生成成员或组织层级；
4. temporary 与 stable 严格分型：重复使用不自动晋升；类型混淆在编译期或运行期 fail-closed；
5. 人工晋升只能新建或绑定 StableMember 并保留 `promoted_from`，**不修改原历史的类型与来源**（定向测试证明原记录不被改写）；
6. 可搜索任务、结果、失败与来源；**不复制报告正文**，只持引用；
7. 无法精确映射的 legacy 记录进 quarantine，不进目录、不被静默丢弃；
8. **真实生产消费者**：真实 Tauri command 在 `commands.rs` 注册、在 `lib.rs` 接入 `AppState`，普通启动路径可达；报告须给出完整调用链，禁止只有测试能触发；
9. `cargo check --lib --offline` 与本叶定向测试在 disposable checkout 上通过，记录真实数字与退出码，证据绑定候选 SHA；
10. 独立内容提交，写域精确，`git diff --check` 通过；
11. 本叶做完即到 **CP3 检查点**：主管自复核放行并收口后，authorization 打回精确 closed，在 `/home/synadmin/workspace/.syn-gates/open/` 写 `stage-15-cp3-<YYYYMMDD-HHMM>.md` 交包（含 M6D05 与 M6D06 两叶），由同一长驻 Codex 前台阻塞启动独立 Cursor Opus 验收并每两分钟报活。PASS 前不得进入 M6D07；FAIL 只按 verdict 点名范围返修。

证据：只在 disposable checkout 上产出定向证据，绑定候选 SHA。执行事实只用隔离 app-data 与合成 attempt / receipt。本叶不做 GUI、不接真实 runner、真实 provider 或真实账号。

允许动：

- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_temporary_agent_projection.rs`（新建）
- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_member_directory.rs`（仅晋升边界与分型接线）
- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_schema.rs`、`m6_org_store.rs`、`m6_org_dto.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（仅 `mod` 声明、`AppState` 接线与 command 注册）
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`（仅本叶 command 接线）
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`（只允许把 `refresh_global_supervisor_temporary_agent_history`、`search_global_supervisor_temporary_agent_history`、`promote_global_supervisor_temporary_agent` 这 3 个精确 command 加入普通 `generate_handler!`；不得改其他 registry、gate 或 command）
- `m5_orchestration_identity.rs`、`m5_runtime_receipt.rs`、`m5_execution_grant.rs`、`m5_agent_runtime.rs`：**仅**可见性调整与新增只读 trait 实现，不改执行语义、不放宽 receipt / audit / quarantine、不改 `m5_runner_entry_registry` 分类；每处改动在报告里逐条说明
- `docs/contracts/`（仅新增增补合同）
- `tasks/2026-08-*`、`tasks/2026-08-19-*`
- `docs/harness/stages/stage-15.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/done/2026-08/`、`docs/harness/authorization.json`、`docs/harness/audit/2026-08.jsonl`、`docs/harness/reports/M6D06-*`、`docs/harness/plan.md`、`docs/current-state.md`

不许动：

- 从 report 自报、缺字段兼容或 runtime trace 推导执行身份；按 session 名或父子关系猜成员
- 让 temporary 自动晋升为 stable；改写原历史类型或来源；复制报告正文
- 放宽 ExecutionGrant / WorkerReport / receipt / audit / quarantine 边界；改判 `m5_runner_entry_registry` 的 `new-grant / guarded-legacy / blocked`；把 guarded legacy 升格
- 直读项目 store / projection / project root；写项目事实
- M1–M5 冻结合同正文与旧 hash
- 会诊（属 M6D07）
- 6 个未跟踪 `m6_*.rs`（含 `.bak`，其中 `m6_temporary_agent_history.rs` 尤其易被同名覆盖）与 `gen/schemas/linux-schema.json`：只读保全，不暂存、不清理、不恢复、不作实现输入，不得被同名新文件覆盖
- 前端源码、页面布局、旧壳 UI、`syn-shell` 仓库、F2/F3/F5、壳采纳
- stage-12、`unfinished/D0C04`、`unfinished/D0C05`、`OSS-01`、用户自有载体
- 真实 runner / 凭据 / provider / 模型 / 账号 / 个人资料 / 外部网络业务写
- 自行关闭 stage-15、宣布 M6 完成、跳过 CP3、越过检查点继续下一叶
- 伪造 receipt、authorization、stage/leaf、测试或 App 证据
- push、merge、rebase、部署、发布
