# M6D03 只读跨项目 query 与 advisory（ORG-002，域层）

阶段：stage-15 M6 全局主管与内部组织（域层先行，UI 验收载体为新壳）

状态：`PLANNED` / `NOT_STARTED`。stage-15 检查点 CP2 的第一叶。前置：CP1 获总指导 PASS。

来源收据：stage-6 计划第 4 节 SYN-ORG-002、第 3 节 `ProjectSummary` 与 `CrossProjectAdvisory` / `AdvisoryApplicationProjection` 不变量、第 7 节关键验收（write-spy / hash baseline）；ProjectSummary 输入固定依 `handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md` 第 1 节与 `docs/contracts/m5-project-summary-projection-v1.md`；判据以 M6D01 冻结合同为准。

目标：让全局主管只读地消费多个项目的最小 ProjectSummary，产出可回源的风险 / 依赖 / 冲突 / 优先级建议，并且对项目零写入。这是 M6 的核心能力叶。

做完的标准：

1. 新增 `m6_org_cross_project_advisory.rs`，跨项目读取**只**经 M5 `ProjectSummaryQueryPort`，一次消费 ≥2 个版本化 summary；不得直读项目 store / projection / project root，不得复制项目原始事实；须有测试证明不存在绕过路径；
2. 实现 freshness / missing / denied / degraded 四态，保留 M5 侧的 stale 与 foreign 拒绝；consumer RoleSession / scope / expiry / policy gate 全程携带并校验，缺 watermark 或缺 owner 的 summary 一律拒（反例测试）；
3. 冲突规则确定性：同输入同输出；每条结论可回源到具体 summary 的 id + version + watermark，source links 齐全；
4. Advisory 与用户采纳本身零项目 mutation：采纳只生成 `DecisionRequest`，不写任何项目、不创建 workflow、不批准 grant、不执行 action；
5. `AdvisoryApplicationProjection` 只引用各项目 owner 的 authoritative command / receipt，投影 `applied / failed / rolled-back / unknown`，不拥有执行结果、不改 advisory lifecycle；partial apply 与回滚按 M6D01 合同；
6. summary version / watermark 变化把相关 advisory 标 stale，不静默重算覆盖历史；
7. 模型侧只增强解释，不得绕过 source / ACL；本叶只用 fake provider / runtime，不接真实模型；
8. **写入面证明**：对项目 domain store、event / audit / outbox、sidecar / compatibility projection、相关文件设 write-spy 或 hash baseline，证明本叶运行期只动 M6 自有 advisory / audit owner，项目侧零变化；
9. **真实生产消费者**：真实 Tauri command 在 `commands.rs` 注册、在 `lib.rs` 接入 `AppState`，普通启动路径可达；报告须给出完整调用链，禁止只有测试能触发；
10. 定向测试覆盖：两项目冲突发现与回源、stale、missing、denied、degraded、缺 watermark 拒绝、越权 scope 拒绝、零项目写、采纳只生成 DecisionRequest；
11. `cargo check --lib --offline` 与本叶定向测试在 disposable checkout 上通过，记录真实数字与退出码，证据绑定候选 SHA；
12. 独立内容提交，写域精确，`git diff --check` 通过；本叶做完不停，直接进 M6D04（同属 CP2）。

证据：只在 disposable checkout 上产出定向证据，绑定候选 SHA。summary 只用隔离 app-data 与 scratch projects 的合成数据。本叶不做 GUI、不接真实 provider / 项目 / 账号。

允许动：

- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_cross_project_advisory.rs`（新建）
- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_schema.rs`、`m6_org_store.rs`、`m6_org_dto.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_global_role_session.rs`（仅本叶所需的会话消费接线）
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（仅 `mod` 声明、`AppState` 接线与 command 注册）
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`（仅本叶 command 接线）
- `m5_project_summary.rs`：**仅**可见性调整与新增 trait 实现，不改 `ProjectSummaryQueryPort` 语义、不放宽 stale / foreign 拒绝、不改 watermark 与 hash 判定；每处改动在报告里逐条说明
- `docs/contracts/`（仅新增增补合同）
- `tasks/2026-08-*`、`tasks/2026-08-19-*`
- `docs/harness/stages/stage-15.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/done/2026-08/`、`docs/harness/authorization.json`、`docs/harness/audit/2026-08.jsonl`、`docs/harness/reports/M6D03-*`、`docs/harness/plan.md`、`docs/current-state.md`

不许动：

- 直读项目 store / projection / project root；跨项目共享业务数据库；复制项目原始事实
- 让 Global Supervisor 直接写项目、创建 workflow、批准 grant 或执行 action；把 advisory / summary 变成项目正式事实
- 放宽 M5 `ProjectSummaryQueryPort` 的 ACL、watermark、stale / foreign 拒绝或只读不可反写
- M1–M5 冻结合同正文与旧 hash；M5 已接受执行语义（ExecutionGrant、WorkerReport、receipt / audit / quarantine 不放宽，`m5_runner_entry_registry` 分类不改判）
- 成员目录、临时 agent、会诊（分属后续叶）
- 6 个未跟踪 `m6_*.rs`（含 `.bak`）与 `gen/schemas/linux-schema.json`：只读保全，不暂存、不清理、不恢复、不作实现输入，不得被同名新文件覆盖
- 前端源码、页面布局、旧壳 UI、`syn-shell` 仓库、F2/F3/F5、壳采纳
- stage-12、`unfinished/D0C04`、`unfinished/D0C05`、`OSS-01`、用户自有载体
- 真实凭据 / provider / 模型 / 账号 / 个人资料 / 真实项目 summary / 外部网络业务写
- 自行关闭 stage-15、宣布 M6 完成、跳过检查点
- 伪造 receipt、authorization、stage/leaf、测试或 App 证据
- push、merge、rebase、部署、发布
