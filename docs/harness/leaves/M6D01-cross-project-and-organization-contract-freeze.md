# M6D01 跨项目与成员合同冻结（ORG-001，只写合同）

阶段：stage-15 M6 全局主管与内部组织（域层先行，UI 验收载体为新壳）

状态：`CURRENT` / `NOT_STARTED`。stage-15 检查点 CP1 的第一叶。前置已满足：M6P00 内容 `4147454`、记账 `cf1cb25` 获独立 verdict `stage-15-m6p00-20260819-0342.verdict.md` PASS。

来源收据：`docs/plans/2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md` 第 4 节 SYN-ORG-001 与第 3 节对象/owner/不变量表；执行引用字段集固定依 `handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md` 第 1 节；用户 2026-08-18 22:41 要求总指导一次排完 M6 并设中间检查点。

目标：把 M6 跨项目与内部组织的对象边界、精确 join、生命周期与迁移矩阵一次冻结成可引用的合同，让后面各叶有唯一判据。本叶**只写合同与 fixtures，不写任何源码，不实现任何 service**。

做完的标准：

1. 新增合同 `docs/contracts/m6-cross-project-and-organization-v1.md`（新建，不改 M1–M5 冻结合同正文与旧 hash）。冻结跨项目侧：ProjectSummary 消费侧 ACL、watermark、freshness 状态机（fresh / stale / missing / denied / degraded 的判定与不可互相降级）；`CrossProjectAdvisory` 的精确 join 字段集（AdvisoryId、global RoleSession、ConsultHandoff、每个 summary 的 id + version + watermark + hash、policy decision、generated_at），任一字段缺失即拒；
2. 冻结采纳与应用边界：用户采纳只生成 `DecisionRequest`；逐项目应用只引用各项目 owner 的 authoritative command / receipt；`AdvisoryApplicationProjection` 只投影 `applied / failed / rolled-back / unknown` 并引用 receipt，不拥有执行结果、不改 advisory lifecycle；partial apply 与回滚语义写明；
3. 冻结成员侧：stable `MemberId` 与 `TemporaryAgentId` 严格分型；membership lifecycle；scope / role assignment；capability / permission 只存 `source + revision + observed_at` 的只读 ref（目录永不是授权真源）；availability 的 source / observed_at / TTL 与"陈旧即 unknown 且不参与授权"；contact receipt；retention 与停用保留 refs；同名冲突与人工晋升 `promoted_from`；
4. 冻结 TemporaryAgent 的执行引用为 M5 完整 envelope，逐字段列出：`project_id + orchestration_id + workflow_run_id + work_item_id + node_id + dispatch_id + attempt_id + grant_id + worker_role_session_id + authoritative receipt + trusted actor + hashes`；明文禁止从 report 自报、缺字段兼容或 runtime trace 推导执行身份；`ChildRunRef` 只作执行引用，不生成 StableMember / TemporaryAgent / 组织层级；
5. 冻结多视角会诊独立性：相同、最小、有来源的问题包；各咨询方独立 RoleSession / Workcell / context packet；提交前互不读取对方结论；并排共识 / 分歧 / 证据索引；成本上限与超时是显式状态；结果只进用户待决定链；
6. 写明迁移矩阵：旧 single-project review 只作 legacy adapter / history 不自动升格为 cross-project advisory；既有 Agent Center session 不自动成 StableMember；TemporaryAgent 从 immutable execution refs 重建且不复制报告正文；summary version / watermark 变化把 advisory 标 stale 而非静默重算覆盖历史；无法精确映射的 legacy 记录进 quarantine；
7. `docs/contracts/fixtures/` 下给出正反例 fixtures，至少覆盖：stale summary、denied scope、foreign project owner、缺 watermark、temporary 冒充 stable、availability 陈旧被当权限、执行 envelope 缺字段。反例必须是"应当被拒"的期望，不是被容忍；
8. 先核实 `docs/contracts/manifest.v1.json` 是否为合同索引；是则按既有惯例登记本合同，不是则不动它；
9. 合同正文明写"本叶只冻结合同，未实现任何 service / repository / projection"，报告不得声称任何实现已成立；
10. 零源码改动：`git show --stat` 里不出现任何 `src-tauri/src/*.rs`；`git diff --check` 通过；独立内容提交，写域精确；
11. 本叶做完不停，直接进 M6D02（同属 CP1）。

证据：合同与 fixtures 的自洽性检查；如仓库已有 fixture 校验脚本则运行并记录真实退出码。本叶不需要 `cargo` 证据，也不做 GUI、不接真实 provider 或账号。

允许动：

- `docs/contracts/m6-cross-project-and-organization-v1.md`（新建）、`docs/contracts/fixtures/`（仅新增 M6 fixtures）、`docs/contracts/manifest.v1.json`（仅在其确为合同索引时按惯例登记）
- `tasks/2026-08-*`、`tasks/2026-08-19-*`
- `docs/harness/stages/stage-15.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/done/2026-08/`（仅本叶收口归档）、`docs/harness/authorization.json`、`docs/harness/audit/2026-08.jsonl`、`docs/harness/reports/M6D01-*`、`docs/harness/plan.md`、`docs/current-state.md`

不许动：

- 任何 `src-tauri/src/*.rs`（本叶零源码）
- M1–M5 冻结合同正文与旧 hash；解释或扩展只能新建合同
- M5 已接受语义：ExecutionGrant、WorkerReport、receipt / audit / quarantine 不得放宽；`m5_runner_entry_registry` 的 `new-grant / guarded-legacy / blocked` 分类不得改判
- 6 个未跟踪 `m6_*.rs`（含 `.bak`）与 `gen/schemas/linux-schema.json`：只读保全，不暂存、不清理、不恢复、不作实现输入，也不得被同名新文件覆盖
- `syn-shell` 仓库、F2/F3/F5、壳采纳、旧壳 UI 与页面布局
- stage-12、`unfinished/D0C04`、`unfinished/D0C05`、`OSS-01`、用户自有载体
- 真实凭据 / provider / 模型 / 账号 / 个人资料 / 外部网络业务写
- 自行关闭 stage-15、宣布 M6 完成、跳过检查点
- 伪造 receipt、authorization、stage/leaf、测试或合同 fixtures
- push、merge、rebase、部署、发布
