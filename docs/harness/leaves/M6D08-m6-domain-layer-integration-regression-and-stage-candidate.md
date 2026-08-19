# M6D08 M6 域层集成回归与阶段候选

阶段：stage-15 M6 全局主管与内部组织（域层先行，UI 验收载体为新壳）

状态：`CURRENT` / `IN_PROGRESS`。stage-15 最后一叶，做完即阶段交包。前置 M6D01–M6D07 已全部真实通过；M6D07 内容 `15bd053cccbaee1302d244afc84eb05578e7fa1c` 已主管自复核 PASS 并归档。

来源收据：stage-6 计划第 6 节迁移与回滚、第 7 节验证矩阵与关键验收、第 9 节阶段退出条件；载体修订依 `decisions/2026-08-17-syn-lightcode-fork-desktop-shell-direction-v1.md`（UI 与隔离 App 验收改新壳）；用户 2026-08-18 22:41 要求一次排完 M6 并设中间检查点。

目标：把 M6 域层各叶合起来跑集成回归，证明它们组合后仍然对项目零写入、legacy 仍可回滚，并如实登记 M6 只到域层、UI 与双项目 App 验收欠在新壳。本叶不宣布 M6 完成。

做完的标准：

1. 域层端到端集成回归（非 GUI）覆盖：两项目摘要发现冲突并回源、consult handoff 幂等、稳定成员查找与联系、temporary 不冒充 stable、runtime child 不冒充组织成员、stale availability 不参与能力判定、采纳只生成 DecisionRequest；
2. **全量写入面证明**：对项目 domain store、event / audit / outbox、sidecar / compatibility projection、相关文件与 spawn 设置 write-spy 或 hash baseline，证明 M6 全部叶子组合运行后，允许变化的只有 M6 自身 advisory / directory / audit owner，项目侧零变化；
3. legacy 兼容与回滚有测试：旧 single-project global review 与 Agent Center 显示仍可用；目录可导出 / 重建；回滚路径不恢复跨项目 raw read；
4. `cargo check --lib --offline` 与 M6 全量定向测试在 disposable checkout 上通过，记录真实 passed / failed 与退出码，证据绑定候选 SHA；与各叶自报数字不一致时以本轮重跑为准并指出差异；
   - 按 M6P00 PASS verdict 欠账 5，把 `m1_project_index_restart_restores_same_project_id` 及适用的 `m1_` restart/replay filter 纳入本轮回归，证明 canonical alias 在重启后仍解析为同一 ProjectId；不得只引用 M6P00 或历史 M1 数字。
   - 按 CP1 PASS 欠账 2，明确裁定 ordinary `AppState` 遇到 Global Supervisor RoleSession ambiguous/quarantined/corrupt/install failure 时，是有意让整个普通产品启动 fail-closed，还是降级成显式 `Unavailable` slot；两种口径只能选一套并与合同/产品可用性一致，必须覆盖真实 ordinary startup 调用链、稳定错误和无 legacy/default fallback 的集成反例，不得维持未决行为。
   - 按 CP2 PASS 欠账 1，增加集成反例，明确断言 ordinary `record_project_director_process_fact_decision` 不能进入任何 M6 跨项目 query 输入面；本叶只验证隔离边界，不实施其 ownerless legacy command cutover，后者仍归 `ENG-01`。
   - 按 CP2 PASS 欠账 2，增加 handoff 在 decide 前已推进 revision 的 accept/reject 用例，确认返回稳定、可读且有意 fail-closed 的 revision conflict，而不是把 `expected_handoff_revision: 1` 的当前前提静默冒充通用语义。
   - 按 CP3 PASS 欠账 4，统一 TemporaryAgent 三个 ordinary command 的 Global Supervisor RoleSession authorization gate：在 M5 store 可用而 global role session unavailable 的真实 composition 反例里，`refresh`、`search`、`promote` 必须在读取/幂等重放或写 M6 store 前以同一稳定错误 fail-closed；可用明确集成断言证明既有 composition 永不解耦，或在入口补同一 gate，但不得让只有 promotion 的后半段间接过门。
   - 按 CP3 PASS 欠账 5–6，集成断言真实形状 M5 store 的 11 张投影依赖表及 receipt/event/audit/durable-operation 字符串约定；“没有执行历史”与“schema/载体约定不匹配”必须成为不同、可观察的显式状态，缺 `m5_durable_operations` 也不得静默退成无 ChildRunRef。仍须 fail-closed，不得以兼容默认值放宽完整 envelope。
   - 按 CP3 PASS 欠账 7，用两个真正独立的 fake provider/runtime 实例重跑 stable identity/rebuild 不漂移反例；若 M6D07/M6D08 引入任何 provider 派生字段，必须同步证明它不进入 StableMember 身份、权限或 memory ref 真值。
5. **如实划界**：报告与 `docs/current-state.md` 明写 stage-15 只到 M6 域层；ORG-007 双项目隔离 App 验收、顶层入口 UI、真实窗口像素证据均欠在新壳（F2/F3/F5），M6 未完成、未发布。不得用域层证据声称跨项目能力已在产品上可用；
6. 把 ORG-007 与 UI 侧欠账写成 `docs/harness/unfinished/` 文件（若 M6S01 已存在则据实更新），并把 advisory / member / source-ref 事件合同交 M7 的输入写成 `handoffs/` 交接文档——只写交接，不激活 M7；
7. 逐条列出本阶段所有欠账与它们的去处；
8. 独立内容提交，写域精确，`git diff --check` 通过；
9. **阶段交包**：authorization 打回精确 closed，在 `/home/synadmin/workspace/.syn-gates/open/` 写 `stage-15-<YYYYMMDD-HHMM>.md`，含各叶候选与记账 SHA / tree、每叶做了什么、主管自复核结论与原始证据（命令、退出码、日志路径）、仍未完成事项与欠账、请求阶段验收的确切范围、全阶段实际写域清单；然后停下等总指导阶段验收。不自行关闭 stage-15、不宣布 M6 完成、不进入 F2/F3/F5 或壳采纳。

证据：只在 disposable checkout 上产出定向与集成证据，绑定候选 SHA。只用隔离 app-data、scratch projects、fake roles / provider / runtime 与白名单合成动作。本叶不做 GUI、不做窗口截图、不接真实 provider / 项目 / 账号。

允许动：

- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_*.rs`（仅集成回归所需的收敛性修补，不新增能力）
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`、`commands.rs`（仅接线修补）
- `docs/contracts/`（仅新增增补合同）
- `handoffs/2026-08-*`（新增 M6 → M7 输入交接）
- `tasks/2026-08-*`、`tasks/2026-08-19-*`
- `docs/plans/2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md`（仅如实记进度与载体口径）
- `docs/harness/stages/stage-15.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/done/2026-08/`、`docs/harness/authorization.json`、`docs/harness/audit/2026-08.jsonl`、`docs/harness/reports/M6D08-*`、`docs/harness/plan.md`、`docs/current-state.md`

不许动：

- 用域层证据声称 UI、隔离 App、真实窗口像素或跨项目产品能力已成立
- 关闭 stage-15、宣布 M6 完成、勾选阶段完成标记（那是总指导阶段验收后的事）
- 新增本阶段未排的能力；把欠账顺手做成新叶
- 直读项目 store / projection / project root；写项目事实
- M1–M5 冻结合同正文与旧 hash；M5 执行语义不放宽
- 6 个未跟踪 `m6_*.rs`（含 `.bak`）与 `gen/schemas/linux-schema.json`：只读保全，不暂存、不清理、不恢复、不作实现输入，不得被同名新文件覆盖
- 前端源码、页面布局、旧壳 UI、`syn-shell` 仓库、F2/F3/F5、壳采纳；激活 M7–M11、Headless Core、Primary / authority epoch
- stage-12、`unfinished/D0C04`、`unfinished/D0C05`、`OSS-01`、用户自有载体
- 真实凭据 / provider / 模型 / 账号 / 个人资料 / 外部网络业务写
- 伪造 receipt、authorization、stage/leaf、测试或 App 证据
- push、merge、rebase、部署、发布
