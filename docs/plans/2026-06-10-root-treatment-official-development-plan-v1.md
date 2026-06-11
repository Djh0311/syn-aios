# Root Treatment Official Development Plan v1

日期：2026-06-10

状态：正式开发计划已创建，用户已要求按全计划开发推进；R4-A30 Session Continuation Store Fixture Helper Extraction 已完成并通过复核线 `STATUS: CLEAR`，implementation commit 为 `8b1bb5b8773f9a4d39515a40feec9eaf9fc10ee9`。R4-A30 接受为 R4-6 Session Continuation Store 相关离线 fixture cluster 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、视觉重做、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。R-Preflight、R0、R1、R2-B1 到 R2-B10、R2 closing / R3 preflight review、R3-P0 SQLite schema / importer / rollback contract freeze、R3-A1 SQLite schema file + temp DB initializer + idempotent dry-run importer + fixtures、R3-A2 temp DB apply importer / schema hardening / transaction failure injection / DB -> JSON export dry-run、R3-A3 fixture-only dual-write transaction rehearsal、R3-A4 fixture-only read-cut DB / JSON fallback / rollback recovery dry-run rehearsal、R3-A5 fixture-only observation / export / rollback verification rehearsal、R3-A6 production cutover / rollback operator contract freeze、R3-A7 production preflight scanner / report、R3-A8 copied snapshot temp DB apply / export / rollback boundary、R3-A9 production DB initializer + apply with backup manifest / no read-cut Level A、R3-A10 limited read-cut planning / feature flag fallback Level A、R3-A11 production observation / export verification Level A、R3-A12 stop-write JSON decision / rollback drill Level A、R3-A13 transaction acceptance / cutover gap matrix Level A、R4-A1 到 R4-A30 均已完成。R3 Level B 未执行，真实 workbench state root 未读取，真实 workbench-owned production DB 未创建，不切 app startup / Tauri command / UI / 产品全局读写路径，不停写 JSON / sidecar。当前下一步是准备 R4-A31：继续中等粒度离线交互测试按域拆分；如要执行任何 R3 Level B，必须另写 execution record、allowed roots、rollback strategy 和 fresh verify。本文承接 `2026-06-10-root-treatment-plan-v1.md` 和 `handoffs/2026-06-10-root-treatment-plan-claude-to-codex-kickoff-v1.md`，用于把“冻结新功能，集中治理”的治本方案转成 Codex 全局主管可派发、可复核、可验收的开发计划。

本文不是任务包，不授权真实 `codex exec` / `codex exec resume`，不授权读写 `/Users/yoyi/.codex`，不授权 Stage L 的 K3-B1 retry / K3-B2，不授权 planned adapters 真实接入，不授权 backlog 解冻后功能开工。

## 0. 全局主管复核结论

已知事实：

- 用户已确认治本方案的核心决策：冻结新功能，集中治理；R3 SQLite 收口前不开多 agent 并行真实执行；lib.rs 最终目标 `<= 3,000` 行；新文件上限 Rust `3,000` 行、TS / TSX `2,000` 行；蓝图正本后续迁入 product-line；R4 按解冻后目标布局区块拆分。
- R-Preflight 前，当前入口曾把 Stage L / L1 写成下一步，Stage K 后续开发冻结在 K6 / `accepted_with_deferred_items`；这正是本轮需要同步的口径冲突。
- Stage L 的 L1-L6 还没有执行，且 L1 属于产品路径 / UI / 状态恢复工作，不是纯治理任务。
- 交接审查指出当前 `product-line` 没有 git 版本控制；R2 拆 `lib.rs` 和 R3 存储迁移若没有 diff / commit / rollback，会让 agent 审查和回滚风险过高。
- 治本方案第 10 节边界与当前 AUTHORITY / CURRENT 的硬约束不冲突：都禁止未授权真实 Codex、禁止读写 `.codex`、禁止绕过用户确认、禁止 planned adapters 真实接入。

主管复核结论：

```text
治理阶段 R 插队执行。
Stage L 剩余 L1-L6 暂挂为 deferred，不在治理冻结期内开工。
Stage L 中纯文档、权威入口、蓝图口径同步项可在 R5 中并入处理。
治理收口后，再回到 Stage L / Stage K 继续处理 K3-B1、K3-B2、真实恢复和日常硬化。
```

原因：

- 治理期目标是修正“任务包只管行为、不管代码形状”的制度缺口；这个缺口会继续放大 Stage L / Stage K 的实现风险。
- L1 虽然重要，但它会新增产品状态、UI、runtime / audit / memory capture 接入；如果在 shape gate 和存储锁之前推进，仍会继续奖励加法。
- K3-B1 / K3-B2 的真实执行仍被安全边界阻断；治理期不会让这个风险变小，反而应先把代码形状、写入锁和证据口径收紧。

执行前置：

- “治理阶段 R 插队、Stage L 暂挂 deferred”已写入 `decisions/2026-06-10-stage-l-root-treatment-freeze-relationship-v1.md`。
- R-Preflight 已同步 `CURRENT.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / `tasks/README.md` / `README.md`。
- R-Preflight 已建立 git baseline commit：`ed01c6f281e3fd7a38548da948046e8366cc368d`。
- 不关闭 L1 任务包，只把它标为治理期暂停 / deferred。

## 1. 总目标

治理阶段 R 的目标：

```text
补上工作台开发制度的“形状治理”能力，让后续 agent 开发不再继续把巨石文件、无事务 sidecar 和整包读模型越堆越大。
```

治理阶段 R 不推翻：

- 方案授权制。
- 任务包制度。
- evidence / handoff。
- `accepted_with_deferred_items` 的真实边界文化。
- 全局主管 / 项目主管 / worker / 复核线分工。
- 当前安全边界。

治理阶段 R 新增：

- shape gate。
- 任务包“形状影响”必填节。
- 治理任务包类型。
- 解冻后治理配额：每 3 个功能任务包至少配 1 个治理任务包，跑一个 Stage 后复盘可调。
- 版本控制前置：R2/R3 前必须有 git baseline、可审查 diff 和可回滚提交。
- lib.rs / 大文件 / sidecar / command surface 棘轮水位线。
- workflow state 写入锁和备份保留策略。
- SQLite 统一存储迁移路线。
- 按页读模型和前端瘦身路线。
- 蓝图正本和矛盾清理路线。

## 2. 不做项

治理期不做：

- 不执行 Stage L / L1-L6。
- 不启动 K3-B1 retry。
- 不启动 K3-B2。
- 不新增真实 `codex exec` / `codex exec resume` 执行点。
- 不读写 `/Users/yoyi/.codex`。
- 不做 planned adapters 真实接入。
- 不做 provider credential store、真实 token 读取或 model verification。
- 不实现 backlog 中的解冻后用户功能：前端整体布局重做、无限画布、秘书型 AI、多 agent / adapter 产品能力等都不开工。
- backlog 中能强化治理制度、减少返工或只做设计预留的条目，可以按第 9 节分类并入 R0 / R4 / R5，但不得把设计预留冒充功能完成。
- 不借治理引入向量库、图数据库、GraphRAG、Obsidian 原生同步或自动技能化。
- 不把 gate 上线说成旧债已还。
- 不把建库建表说成 R3 完成。

紧急缺陷例外：

- 只有影响当前可运行性或数据安全的紧急缺陷可以插队。
- 紧急缺陷也必须过 shape gate。
- 紧急缺陷不得夹带新功能。

## 3. 基线指标

来自 `2026-06-10-root-treatment-plan-v1.md` 的当前基线：

| 指标 | 当前值 | 治理目标 |
| --- | --- | --- |
| `src-tauri/src/lib.rs` | 25,925 行 | 最终 `<= 3,000` 行，理想 `<= 1,500` 行 |
| `real_execution_command.rs` | 8,763 行 | 进入棘轮清单，只降不升 |
| `ProjectsView.tsx` | 6,069 行 | R4 拆分 |
| 离线测试主文件 | 9,369 行 | R4 拆分 |
| 生产 sidecar JSON 种类 | 15 种 | R3 后统一进 SQLite，sidecar 只作导出 / 备份 |
| `workflow-state.v0.json` | 全局 2MB 单文件 | R3 表化 |
| `backups/` | 227 份 / 184MB | R1 加保留策略 |
| `WorkbenchSnapshot` | 18 个顶层字段整包加载 | R4 拆为按页查询 |
| Tauri command 总数 | 96 个 | R0 后新增 command 不进 lib.rs |

## 4. 阶段顺序

### R-Preflight：治理插队决策确认

目标：

- 用户确认“治理阶段 R 插队，Stage L 剩余 L1-L6 暂挂 deferred”。
- 写入 decision，避免后续对 Stage L / Stage K / R 阶段优先级产生歧义。
- 同步 `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md`。

建议 decision：

- `decisions/2026-06-10-stage-l-root-treatment-freeze-relationship-v1.md`

接受标准：

- 当前入口明确：Stage R 治理正在执行；Stage L L1-L6 暂挂；治理后恢复 Stage L / Stage K。
- 不把 Stage L 暂挂说成 Stage L 完成。
- 不把治理插队说成取消 K3-B1 / K3-B2。

### R0：制度升级

R0 可与 R1 并行，但 R0 是所有后续任务的制度前置。

R0-1 shape gate：

- 建议新建通用 gate：`scripts/harness/workbench-shape-gate.js`。
- 现有 `scripts/harness/stage-k-architecture-gate.js` 保持为 Stage K 历史 gate，不直接改造成通用 gate。
- R0 可复用 Stage K gate 的扫描经验，但新 gate 的权威名称应是 workbench shape gate。

shape gate 最小规则：

- `lib.rs` 水位线只降不升，初始水位线为 25,925 行。
- 新增 `#[tauri::command]` 不得写进 `lib.rs`。
- 禁止新增生产 sidecar JSON 种类；确需新增必须用户确认并写 decisions。
- 新建文件上限：Rust 3,000 行，TS / TSX 2,000 行。
- 存量超限文件进入棘轮清单，只降不升。
- gate 豁免必须有 decisions 记录，不允许沉默豁免。

R0-2 任务包模板：

- 新增“形状影响”必填节。
- 每个任务包必须说明新增代码落点、是否触碰棘轮文件、预计行数变化、是否新增 command、是否新增 sidecar。

R0-3 治理任务包类型：

- 验收 = 行为不变 + 形状指标改善。
- evidence 必须写前后指标。
- 治理任务包也走 C1-C6 / evidence / handoff，不另起制度。
- 解冻后治理配额写入制度：每 3 个功能任务包至少配 1 个治理任务包，跑一个 Stage 后复盘可调。
- 配额例外必须写进 decision，不允许沉默跳过。

R0 验收：

- workbench shape gate 可运行。
- 至少能输出当前基线指标和 fail / pass 结果。
- 任务包模板已新增形状影响节。
- 第一条治理任务包按新模板创建并通过复核。
- R0 任务包模板 / gate 文档已写入解冻后 `1:3` 治理配额。
- evidence / handoff 模板能记录 git commit hash；若 git 仍未初始化，必须明确标记 R2 / R3 blocked。

### R1：立即止血

R1 可与 R0 并行，但代码合入必须受 R0 gate 约束；如 R0 尚未完全上线，R1 evidence 至少手工记录形状指标。

R1-1 workflow-state 写入锁：

- 先核实 `workflow_state_store.rs` 上层是否已有进程内互斥。
- 如没有，补 StoreLock。
- 实现模式参考 `session_continuation_store.rs` / 现有 sidecar store 的 StoreLock。
- 保持 temp + rename 原子替换。
- 增加并发写测试。

R1-2 备份保留策略：

- 给 workflow state backups 加保留策略。
- 默认策略：保留最近 30 份 + 每日 1 份。
- 初次任务只实现策略和测试，不直接对用户真实历史备份做不可逆清理；如需要清理真实备份，必须在任务包里列 dry-run 输出和用户确认。

R1 验收：

- 并发写测试通过。
- corrupt JSON / revision conflict / lock busy 不覆盖原文件。
- 备份数量在测试夹具中按策略收敛。
- 不新增 sidecar 种类。
- 不读写 `/Users/yoyi/.codex`。

### R2：lib.rs 解体

R2 按批次治理，每批一个治理任务包，禁止一次性大拆。

推荐批次：

1. 命令注册 / 分发出 `lib.rs`，按领域归入 `commands/` 子模块。已由 R2-B1 完成。
2. workflow state JSON helper、workflow state lifecycle 和 task package 写入链分批抽出。已由 R2-B2 / R2-B3 完成。
3. workflow run check / binding / legacy dispatch entrypoints 分批抽出。已由 R2-B4 完成。
4. workflow 读模型派生逻辑与 `workflow_read_model.rs` 汇合。已由 R2-B5 完成。
5. workflow dispatch execution control、offline role dispatch、workflow machine 逻辑抽出。已由 R2-B6 完成。
6. 记忆领域与 formal memory / candidate / observation / lint / mature pattern 等模块汇合。R2-B7 已完成 memory command bridge / observation bridge / task memory packet preview bridge / context guard 抽出；更深的 memory store / lifecycle 内部重构仍是后续治理。
7. runtime / diagnostics 领域出 `lib.rs`。
8. 剩余清扫。

水位线：

- 第一阶段：25,925 → 15,000。
- 第二阶段：15,000 → 8,000。
- 第三阶段：8,000 → 3,000。
- 理想目标：`lib.rs <= 1,500`，只保留应用装配。

R2 验收：

- 每批测试矩阵全绿。
- `lib.rs` 行数下降。
- 新文件未超过上限，若超过必须写 decision。
- command surface 没有新增到 `lib.rs`。

### R3：SQLite 统一存储

R3 是治理阶段的硬门槛。R3 未完成前，不解锁多 agent 并行真实执行。

迁移五步：

1. 建 schema + 导入器：JSON → DB，幂等可重跑。
2. 双写期：写 DB + 写 JSON，读仍走 JSON。
3. 读切 DB：JSON 降级为导出格式。
4. 观察期：停写 JSON，归档旧 sidecar。
5. 回滚演练：记录恢复流程和失败边界。

R3 必须表化：

- workflow state 11 个顶层数组。
- 记忆相关正式 / 候选 / observation / lint / entity / mature pattern。
- runtime log。
- audit。
- plan authorization / proposal。
- continuation / product command attempt。

R3 验收：

- 至少一次跨记忆 + 审计的候选采纳在单事务内完成。
- 中断注入测试不产生半写状态。
- DB 导入幂等。
- JSON 导出可重建。
- 不把建库建表冒充 R3 完成。

### R4：读模型与前端瘦身

R4 排在 R3 后，避免在旧 JSON / 整包 snapshot 上继续堆 UI。

R4 做：

- `WorkbenchSnapshot` 拆为按页查询：项目页、智能体页、运行中页、记忆页、知识库页、设置页。
- 前端 `types.ts` 分域。
- `ProjectsView.tsx` / `AgentView.tsx` 按解冻后目标布局区块拆分。
- `styles.css` 提炼水墨风格资产清单：颜色、字体、质感、间距、阴影、动效边界。
- 离线测试主文件按领域拆分。
- 任务包上下文加入代码地图。

R4 不做：

- 不重做前端布局。
- 不做无限画布。
- 不做 UI 视觉反馈 MCP 工具。
- 不复制 Xuanji 源码、命名、图标、视觉资产。

R4 拆分基准：

- 按解冻后目标布局区块切：四周窄栏、中央主界面、右侧详情面板、底部状态栏、悬浮秘书 / 入口。
- 不按当前页面外观切，避免布局重做时二次重拆。

### R5：文档与蓝图对齐

R5 做：

- 蓝图 §17 / §22 / §26.4 的经验沉淀口径统一为 M12 已实现路径。
- 两份吸收建议文档与 M1-M13 / C1-C6 查重。
- 已实现项标注完成，剩余有效项登记 backlog。
- 蓝图正本迁入 `product-line/docs/architecture/` 或在 AUTHORITY 标注唯一正本路径。
- codexbridge 侧 mission-control README 标注 superseded by product-line。
- Stage L 中纯文档 / 口径项可在 R5 并入处理。

R5 不做：

- 不启动 Stage L 产品代码。
- 不改 UI 布局。
- 不实现 backlog 解冻项。

## 5. 执行级拆解

本节用于让后续任务包可以直接拆派。每一步都必须保持“行为不变、形状改善、证据可回收”的治理口径。

### 5.1 R-Preflight 执行步骤

| 步骤 | 做什么 | 产物 | 验收 |
| --- | --- | --- | --- |
| RP-1 | 复核 Stage L / Stage K / Root Treatment 三条口径是否冲突 | 复核摘要 | 明确治理插队不等于 Stage L 完成、不等于取消 K3-B1/K3-B2 |
| RP-2 | 写 Stage L 与治理冻结关系 decision | `decisions/2026-06-10-stage-l-root-treatment-freeze-relationship-v1.md` | decision 写清 Stage L L1-L6 暂挂、R 后恢复 |
| RP-3 | 同步权威入口 | `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md` | 入口不再同时指向 L1 执行和 R0/R1 执行 |
| RP-4 | 写 R0 / R1 任务包 | 两个任务包草案 | 任务包含形状影响、边界、不做项、验证矩阵 |
| RP-5 | 建立版本控制前置 | git 初始化 / baseline commit 或替代方案 decision | R2/R3 动工前必须有 diff、commit hash 和可回滚点 |

版本控制规则：

- 默认方案是在 `product-line` 根目录初始化 git，创建治理前 baseline commit。
- R0 / R1 如果只改文档、脚本、小范围存储代码，仍应尽快进入 git；R2 / R3 动工前 git baseline 是硬门槛。
- 每个治理任务包完成后都应形成独立提交，handoff / evidence 记录 commit hash。
- 如果用户确认不能使用 git，必须先写替代方案 decision，至少具备文件快照、hash manifest、diff 产物和回滚步骤；否则 R2 / R3 阻断。

RP-Preflight 禁止：

- 不改产品代码。
- 不执行 R0 / R1。
- 不执行真实 Codex。
- 不关闭 L1 任务包，只把它标为治理期 deferred / paused。

### 5.2 R0 执行步骤：制度升级

R0 目标是让后续所有任务包先过“形状守门”，停止继续长歪。

| 步骤 | 做什么 | 产物 | 验收 |
| --- | --- | --- | --- |
| R0-1 | 建立当前形状基线 | shape baseline JSON / Markdown 摘要 | 记录 `lib.rs`、棘轮文件、command 数、sidecar 种类、超限文件 |
| R0-2 | 设计 gate CLI 契约 | gate 使用说明 | 明确 `check` / `baseline` / `strict` 模式和退出码 |
| R0-3 | 实现 `workbench-shape-gate.js` | `scripts/harness/workbench-shape-gate.js` | 能扫描行数、command、sidecar、超限文件 |
| R0-4 | 接入棘轮规则 | gate 配置或内置基线 | `lib.rs` 水位线只降不升，新增 command 不进 lib.rs |
| R0-5 | 更新任务包模板 | 模板新增“形状影响”节 | 新任务包必须声明新增代码落点、行数、sidecar、command |
| R0-6 | 定义治理任务包类型 | 文档 / 模板说明 | 验收口径 = 行为不变 + 形状改善 |
| R0-7 | 写入解冻后治理配额 | 模板 / gate 文档 | 每 3 个功能任务包至少配 1 个治理任务包 |
| R0-8 | 接入版本控制元信息 | evidence / handoff 字段 | 每批记录 git commit hash；无 git 时标记 R2/R3 blocked |
| R0-9 | dry-run 当前仓库 | R0 evidence | 当前债务可报告；非治理任务不被误判为已完成 |
| R0-10 | 复核线审查 | R0 handoff / supervisor review | 无 P0/P1，P2 可带入后续 |

R0 最小实现范围：

- 新增或更新脚本。
- 更新任务包模板 / 计划文档。
- 不改业务 Rust / TS 逻辑。

R0 验证：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- 文档扫描：确认任务包模板含“形状影响”。
- 文档扫描：确认治理配额 `每 3 个功能任务包至少 1 个治理任务包` 已写入模板 / gate 文档。
- evidence 字段扫描：确认 commit hash / no-git blocked 字段存在。
- 如只改脚本 / 文档，可不跑全量 `cargo test`，但 evidence 必须说明原因。

R0 禁止：

- 不借 gate 改业务。
- 不删除历史代码。
- 不把 gate 当前 fail 解释为治理失败；初期 fail 代表债务可见。

### 5.3 R1 执行步骤：workflow state 写入止血

R1 目标是先降低当前 JSON 写入的并发和备份膨胀风险。

| 步骤 | 做什么 | 产物 | 验收 |
| --- | --- | --- | --- |
| R1-1 | 审计 `workflow_state_store.rs` 当前写路径 | 写路径清单 | 找到所有 backup / atomic write / validated write 入口 |
| R1-2 | 审计是否已有上层互斥 | 审计结论 | 若已有互斥，说明覆盖范围；若无，进入 StoreLock |
| R1-3 | 设计 StoreLock | lock path / write id 规则 | lock 文件路径稳定，lock busy 可分类 |
| R1-4 | 实现 StoreLock | Rust 代码 | 写入前 acquire，Drop 释放，异常不覆盖原文件 |
| R1-5 | 设计备份保留策略 | retention 函数 | 最近 30 份 + 每日 1 份，测试可控 |
| R1-6 | 实现备份 prune | Rust 代码 | 只清理测试夹具；真实备份清理需另行确认 |
| R1-7 | 补单测 | Rust tests | 并发写、lock busy、corrupt JSON、backup retention 都覆盖 |
| R1-8 | 跑验证 | evidence | `cargo test` 相关项、`cargo fmt -- --check` 通过 |

R1 最小实现范围：

- `workflow_state_store.rs`。
- 必要测试。
- 如 helper 已在 `lib.rs` 测试区，允许只迁移本任务新增测试，不顺手大拆。

R1 禁止：

- 不迁移 SQLite。
- 不改 workflow state 顶层 schema。
- 不读写 `/Users/yoyi/.codex`。
- 不对真实历史 backups 做不可逆清理，除非任务包单列 dry-run、影响面和用户确认。

### 5.4 R2 执行步骤：lib.rs 解体

R2 只做行为保持型拆分，每批都必须让 `lib.rs` 行数下降。

当前 R2 checkpoint：

- R2-B1 已完成并由主管线收口为 `accepted_with_p2`：`tasks/2026-06-10-root-treatment-r2-b1-command-registry-extraction-v1.md`，completion commit `13016917442070fc2f59a130b2748eb0cba06a34`。接受为 command registry 从 `lib.rs::run()` 物理拆出，不接受为 R2 完成。
- R2-B2 已完成并由主管线收口为 `accepted_with_p2`：`tasks/2026-06-11-root-treatment-r2-b2-lib-map-and-workflow-state-helper-extraction-v1.md`，completion commit `76ed0ef46d9b0a2a83f6e77ce533d6c8741c93cf`。接受为补 R2 代码地图并抽出 workflow state JSON helper，不接受为 R2 完成。
- R2-B3 已完成并由主管线收口为 `accepted_with_p2`：`tasks/2026-06-11-root-treatment-r2-b3-workflow-state-lifecycle-and-task-package-chain-extraction-v1.md`，completion commit `208fabaa4cae8aeda45cdce4c66cbe7f2cf8e6c3`。接受为抽出 workflow state 生命周期入口和 task package 写入链，不接受为 R2 完成。
- R2-B4 已完成并由主管线收口为 `accepted_with_p2`：`tasks/2026-06-11-root-treatment-r2-b4-workflow-run-binding-and-legacy-dispatch-entrypoints-extraction-v1.md`，completion commit `66a0cff5a4fb94101c1830a174dc908448ec8dba`。接受为抽出 workflow run check、work item state、session binding 和 legacy workflow node dispatch 入口，不接受为 R2 完成。
- R2-B5 已完成并由主管线收口为 `accepted_with_p2`：`tasks/2026-06-11-root-treatment-r2-b5-workflow-read-model-dispatch-summary-and-readback-stats-extraction-v1.md`，completion commit `35cacc22ec813152e9357a42bc82e7ef581d2509`。接受为抽出 workflow read model、dispatch summary 和 readback stats 相关派生逻辑，不接受为 R2 完成。
- R2-B6 已完成并由主管线收口为 `accepted_with_p2`：`tasks/2026-06-11-root-treatment-r2-b6-workflow-execution-control-offline-role-and-machine-extraction-v1.md`，completion commit `2dd766be84e977d75e77f31ec2dbf9d463f45690`。接受为抽出 workflow dispatch execution control、offline role dispatch 和 workflow machine 相关逻辑，不接受为 R2 完成。
- R2-B7 已完成并由主管线收口为 `accepted_with_p2`：`tasks/2026-06-11-root-treatment-r2-b7-memory-command-bridge-and-context-guard-extraction-v1.md`，completion commit `9cd10bb51fe828ae5b2b72501414b5cf025b77a9`。接受为抽出 memory command bridge、observation bridge、task memory packet preview bridge 和 context binding guard，不接受为 R2 完成。
- R2-B8 已完成并由主管线收口为 `accepted_with_p2`：`tasks/2026-06-11-root-treatment-r2-b8-diagnostics-provider-continuation-adapter-boundary-extraction-v1.md`，completion commit `9935dac822ab41bce2391b8f6a54d6b42eeb4f95`。接受为抽出 diagnostics、store integrity、provider availability、session continuation preview / guard、agent adapter descriptors 和 session operation descriptors，不接受为 R2 完成。
- R2-B9 已完成并由主管线收口为 `accepted_with_p2`：`tasks/2026-06-11-root-treatment-r2-b9-index-host-app-assembly-extraction-v1.md`，completion commit `bd63d7f5a12a29443d4d0c97713c1c6b1921cf20`。接受为抽出 index parsing、allowed paths、host OS helper 和 Tauri app assembly 尾段，不接受为 R2 完成。
- R2-B10 已完成并由主管线收口为 `accepted_with_p2`：`tasks/2026-06-11-root-treatment-r2-b10-c4-c6-automation-workflow-governance-extraction-v1.md`，completion commit `d5f423d97c1f2dac4bca33f84c34e46b0b4716a6`。接受为抽出 C4-C6 自动化工作流治理连续区块，并确认 `lib.rs` 已从 16,457 行降到 13,949 行，达成第一阶段 `lib.rs <= 15,000` 水位线；不接受为 R2 完成。
- R2 closing / R3 preflight review 已完成，结论为 `DONE_WITH_CONCERNS`：只读复核剩余 `lib.rs` 结构、inline tests 巨石、R3 SQLite 前置风险和后续拆分/迁移顺序；R3-P0、R3-A1、R3-A2 和 R3-A3 均已完成。
- R3-A4 已完成并由主管线收口为 `accepted_with_p2`：`tasks/2026-06-11-root-treatment-r3-a4-fixture-only-read-cut-db-and-rollback-rehearsal-v1.md`，completion commit `d1343e87f2e62fe959f622f68037714218ed6c13`。接受为 fixture-only read-cut DB / JSON fallback / rollback recovery dry-run 演练，不接受为生产 DB、生产读切、JSON / sidecar 停写或多 agent 并行真实执行解锁。
- R3-A5 已完成并由主管线收口为 `accepted_with_p2`：`tasks/2026-06-11-root-treatment-r3-a5-fixture-only-observation-export-and-rollback-verification-v1.md`，implementation commit `0e8255a8248601caf7b1d513131f43e4bb157589`。接受为 fixture-only observation period、export verification、two-sample stability 和 rollback recovery dry-run 演练，不接受为生产 DB、生产读切、JSON / sidecar 停写或多 agent 并行真实执行解锁。
- R3-A6 已完成，结论为 `DONE_WITH_CONCERNS`：`tasks/2026-06-11-root-treatment-r3-a6-production-cutover-contract-and-rollback-operator-freeze-v1.md`。接受为 production cutover contract、rollback operator contract、allowed roots / denied paths、backup / recovery 和 dry-run / apply 分界冻结，不接受为生产 DB、生产读切、JSON / sidecar 停写或多 agent 并行真实执行解锁。
- R3-A7 已完成，结论为 `DONE_WITH_CONCERNS`：`tasks/2026-06-11-root-treatment-r3-a7-production-preflight-scanner-and-report-v1.md`，implementation commit `7949253c91c8e688dc48e03c47a952f00fcd6fda`。接受为 production preflight scanner / report 模块和 temp fixture validation 完成；真实 production root scan 未执行，不接受为生产 DB、production apply、生产 read-cut、JSON / sidecar 停写或多 agent 并行真实执行解锁。
- R3-A8 已完成，结论为 `DONE`：`tasks/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1.md`，implementation commit `ce631c1cd23dadb367288885d61a331b88b83511`，主管验收回填 commit `81815be171899bca8e98cd70cd9ea9464c5f2556`。接受为 Level A fixture / temp copied snapshot apply、temp DB、export verification 和 rollback dry-run boundary 完成；Level B 未执行，真实 workbench state root 未读取，真实 production snapshot 未复制，不接受为 production DB、production apply、生产 read-cut、JSON / sidecar 停写或多 agent 并行真实执行解锁。
- R3-A9 Level A 已完成，结论为 `DONE`：`tasks/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1.md`，implementation commit `52d6b4b73dcb49e4ffc582dac500d9ad6a8ee4df`。接受为 fixture / temp production DB initializer + apply with backup manifest / export verification / rollback boundary 合同完成；Level B 未执行，真实 workbench state root 未读取，真实 workbench-owned production DB 未创建，不接受为 production read-cut、JSON / sidecar stop-write、rollback production workflow、R3 完成或多 agent 并行真实执行解锁。
- R3-A10 Level A 已完成，结论为 `DONE`：`tasks/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1.md`，implementation commit `b18424c38bf0f36f8c9b8ee783a0010598ca9683`。接受为 `workflow_state_summary` 单一低风险 read model 的 fixture / temp limited read-cut 合同、feature flag、verified JSON fallback、blocked matrix、recovery dry-run 和 A10 专用 projection path guard 完成；Level B 未执行，真实 workbench state root 未读取，真实 production DB 未创建，不切 app startup / Tauri command / UI / 产品全局读路径，不停写 JSON / sidecar，不接受为 production read-cut、rollback production workflow、R3 完成或多 agent 并行真实执行解锁。
- R3-A11 Level A 已完成，结论为 `DONE`：`tasks/2026-06-11-root-treatment-r3-a11-production-observation-export-verification-contract-v1.md`，implementation commit `a7d715c49888b9d3ec67c36c3e431f07e14af12a`。接受为 `workflow_state_summary` 单一低风险 read model 的 production observation / export verification Level A fixture / temp 合同、feature flag、DB observation、verified JSON fallback、export verification、rollback readiness、blocked matrix、safety flags 和 redaction policy 完成；Level B 未执行，真实 workbench state root 未读取，真实 workbench-owned production DB 未创建，不切 app startup / Tauri command / UI / 产品全局读路径，不停写 JSON / sidecar，不接受为 production observation Level B、production read-cut、rollback production workflow、R3 完成或多 agent 并行真实执行解锁。
- R3-A12 Level A 已完成，结论为 `DONE_WITH_P2`：`tasks/2026-06-11-root-treatment-r3-a12-stop-write-json-decision-and-rollback-drill-v1.md`，implementation commit `eacfad7c4a916f1307e633a37a6084a9fc2927e6`。接受为 `workflow_state_summary` 单一低风险 read model 的 stop-write JSON / sidecar supervisor decision contract 和 fixture / temp rollback drill Level A 完成；复核线结论为 `CLEAR_WITH_P2`，P2 仅作为 Level B 前加固建议，不阻断当前 checkpoint；Level B 未执行，真实 workbench state root 未读取，真实 workbench-owned production DB 未创建，不切 app startup / Tauri command / UI / 产品全局读写路径，不停写 JSON / sidecar，不接受为 JSON / sidecar stop-write 完成、production read-cut、production observation Level B、rollback production workflow、R3 完成或多 agent 并行真实执行解锁。
- R3-A13 Level A 已完成，结论为 `DONE`：`tasks/2026-06-11-root-treatment-r3-a13-transaction-acceptance-and-cutover-gap-matrix-v1.md`，implementation commit `d96ed042341fa816e62b149f0ea451516f0e5ad2`。接受为 fixture / temp SQLite transaction acceptance 和 cutover gap matrix Level A 完成；已验证 memory candidate adoption、formal memory record、formal memory version、formal memory audit event 和 workflow audit event 可在同一 SQLite transaction 内提交，before-commit failures 不留下 half-adopted state，after-commit-before-report 分类为 `committed_but_report_failed`；Level B 未执行，真实 workbench state root 未读取，真实 workbench-owned production DB 未创建，不切 app startup / Tauri command / UI / 产品全局读写路径，不停写 JSON / sidecar，不接受为 R3 全量完成、production apply、production read-cut、production observation、JSON / sidecar stop-write、rollback production workflow 或多 agent 并行真实执行解锁。

| 批次 | 做什么 | 主要落点 | 验收 |
| --- | --- | --- | --- |
| R2-0 | 建代码地图 | docs 或 generated map | 标出 lib.rs 内领域块、调用关系、测试落点 |
| R2-1 | 命令注册 / 分发拆出 | `commands/` 或 app assembly 模块 | `lib.rs` command 相关行数下降，无新增 command 进 lib.rs |
| R2-2 | workflow 读模型拆出 | `workflow_read_model.rs` 等 | 工作流 snapshot / 派生逻辑不再堆 lib.rs |
| R2-3 | 记忆领域拆出 | memory domain modules | formal / candidate / observation / lint 相关逻辑归域 |
| R2-4 | runtime / diagnostics 拆出 | runtime / diagnostic modules | G1/G2/K5 派生逻辑归域 |
| R2-5 | 剩余清扫 | app assembly | `lib.rs <= 3,000` 或进入明确下降轨道 |

每个 R2 批次都要：

- 先跑 shape baseline。
- 只搬一个领域，不改行为。
- 保留公开 command / type 契约。
- 跑相关 `cargo test --lib ...` 和必要前端测试。
- 记录 `lib.rs` 前后行数。

R2 禁止：

- 不趁拆分新增功能。
- 不重命名用户可见概念。
- 不改 storage schema。
- 不把测试失败标记为“只是不相关”跳过。
- 没有 git baseline / commit hash / 可审查 diff 时，不启动 R2。

### 5.5 R3 执行步骤：SQLite 统一存储

R3 是最大风险阶段，必须一小步一回收。

| 步骤 | 做什么 | 产物 | 验收 |
| --- | --- | --- | --- |
| R3-0 | schema 设计冻结 | `docs/plans/...sqlite-schema...md` 或 migration docs | workflow / memory / runtime / audit / continuation / product command 映射完整 |
| R3-1 | 建 DB 和导入器 | Rust storage module / importer | JSON → DB 幂等，可重跑 |
| R3-2 | 导入完整 fixture | fixture DB | 导入后数量、hash、引用关系一致 |
| R3-3 | 双写期 | 写 DB + 写 JSON | 写入成功 / 失败一致，失败不半写 |
| R3-4 | 读切 DB | read model 读 DB | JSON 降为 fallback / export |
| R3-5 | 停写 JSON | sidecar 归档策略 | 新写入只进 DB，JSON 可导出 |
| R3-6 | 回滚演练 | rollback evidence | DB 损坏 / 中断 / 迁移失败可恢复 |
| R3-7 | 事务验收 | integration test | 候选采纳跨记忆 + 审计单事务完成 |

R3 禁止：

- 不把建表当 R3 完成。
- 不一次切完所有读写路径。
- 不删除原 JSON 历史，除非已导出、备份、回滚演练通过。
- R3 未收口前，不解锁多 agent 并行真实执行。
- 没有 git baseline / commit hash / 可审查 diff 时，不启动 R3。

### 5.6 R4 执行步骤：读模型和前端瘦身

R4 只拆读模型和前端结构，不改布局风格。

| 步骤 | 做什么 | 产物 | 验收 |
| --- | --- | --- | --- |
| R4-0 / R4-A1 | 页面数据需求盘点 | 页面读模型矩阵 + `WorkbenchSnapshot.page_read_model_inventory` 合同 skeleton | 已完成；9 个页面合同已冻结，设置页开发者区只读展示 |
| R4-1 | 后端按页查询 | Tauri commands / read models | R4-A2 已完成只读 selector contract skeleton；真实页面仍未迁移，后续还需逐页接入 |
| R4-A3 | Projects / Agents selector 分域 | 前端纯 selector / tests | 已完成并通过复核线 `STATUS: CLEAR` |
| R4-A4 | Projects / Agents 页面消费 selector | `ProjectsView.tsx` / `AgentView.tsx` 最小 diff 接线 | 已完成并通过复核线 `STATUS: CLEAR`；页面仍未迁移真实数据源 |
| R4-A5 | Running / Memory selector 分域和页面消费 | `RunningWorkflowsView.tsx` / `MemoryCenterView.tsx` 首屏摘要最小 diff 接线 | 已完成并通过复核线 `STATUS: CLEAR`；页面仍未迁移真实数据源 |
| R4-A6 | Knowledge / Settings selector 分域和页面消费 | `KnowledgeBaseView.tsx` / `SettingsView.tsx` 首屏摘要最小 diff 接线 | 已完成并通过复核线 `STATUS: CLEAR`；页面仍未迁移真实数据源 |
| R4-2 / R4-A7 | TS 类型分域 | frontend type modules | 已完成并通过复核线 `STATUS: CLEAR`；`types.ts` 从 5,149 行降到 4,998，保持 re-export 兼容 |
| R4-3 / R4-A8 | ProjectsView 拆分 | project page components | 已完成并通过复核线 `STATUS: CLEAR`；`ProjectsView.tsx` 从 6,069 行降到 5,897，项目入口和资料 / 资源面板抽出，页面行为不变 |
| R4-4 / R4-A9 | AgentView 拆分 | agent page components | 已完成并通过复核线 `STATUS: CLEAR`；`AgentView.tsx` 从 3,360 行降到 3,118，transcript 展示组件抽出，对话工作区行为不变 |
| R4-5 / R4-A10 | styles.css 风格资产提炼 | ink style tokens / docs | 已完成并通过复核线 `STATUS: CLEAR`；新增 `docs/design/2026-06-11-root-treatment-r4-a10-ink-style-assets-v1.md`，只接受为风格资产清单和后续 CSS 治理依据，不接受为 CSS 源码拆分或 UI 重做 |
| R4-6 / R4-A11 到 R4-A30 | 离线测试拆分 | tests by domain | R4-A11 到 R4-A30 已完成；A11/A12/A19/A21 通过复核线 `STATUS: CLEAR_WITH_P2` 且 P2 均已处理或进入 hash backfill，A13/A14/A15/A16/A17/A18/A20/A22/A23/A24/A25/A26/A27/A28/A29/A30 通过复核线 `STATUS: CLEAR`；主测试通用 helper、权限弹层场景 runner、任务字段 / 派发准备 helper、runtime / diagnostic fixture helper、worker protocol fixture helper、workflow state 变体 fixture helper、authorization workflow fixture helper、基础 workflow state / project workflow fixture helper、derived workflow fixture helper、C6 result summary fixture helper、run queue fixture helper、candidate governance fixture helper、memory center core store fixture helper、memory center governance fixture helper、memory pattern fixture helper、KnowledgeBase / Secretary fixture helper、Transcript / Session fixture helper、Workbench base snapshot fixture helper、Real Execution Product Command / Project Workflow Automation fixture helper、Session Continuation Store fixture helper 已抽出，`offline-permission-dialog.test.tsx` 从 9,369 行降到 4,872 行，测试仍绿；R4-6 后续仍需继续按域拆分 |
| R4-7 | UI 边界复核 | evidence / screenshots if needed | 不实际重做布局，不动视觉风格 |

R4 必须为解冻后 UI 预留：

- 按 Xuanji 报告的目标布局区块拆组件边界。
- 保留当前水墨古风，不引入 Xuanji 视觉风格。
- 为 React Flow 深化无限画布保留读模型映射。
- 为 UI 视觉反馈工具保留“验证入口”而非实现工具。

R4 禁止：

- 不做“顺手美化”。
- 不把页面结构改成新布局。
- 不实现 MCP 看图工具。

### 5.7 R5 执行步骤：文档与蓝图对齐

R5 是治理阶段的文档收束，不是功能开发。

| 步骤 | 做什么 | 产物 | 验收 |
| --- | --- | --- | --- |
| R5-1 | 蓝图正本迁入 / 标注 | `docs/architecture/` 或 AUTHORITY 唯一路径 | 不再有多个正本 |
| R5-2 | 清理蓝图矛盾 | 蓝图 patch / decision | §17 / §22 / §26.4 与 M12 口径一致 |
| R5-3 | 吸收建议查重 | 对照矩阵 | 已实现 / 未实现 / 废弃条目分类 |
| R5-4 | backlog 分类回填 | backlog / plan 更新 | 治理吸收、只预留、解冻后三类清楚 |
| R5-5 | Stage L 文档项并入 | CURRENT / STAGE_PLAN / AUTHORITY | 纯文档项合并，产品代码项仍 deferred |
| R5-6 | 治理阶段验收 | final evidence / handoff | R0-R5 accepted / deferred / blocked 冻结 |

R5 禁止：

- 不把研究参考当执行事实。
- 不开启 UI / 画布 / MCP / 记忆时效代码实现。
- 不改产品运行时代码，除非是文档路径或入口索引必要修正。

## 6. 分线职责

全局主管线：

- 维护冻结边界、任务包、入口文档、最终验收。
- 复核 R0 / R1 并行回交。
- 防止治理任务夹带功能。

Gate / 模板线：

- 负责 R0 shape gate、任务包模板、治理任务包类型。
- 不改业务逻辑。

存储止血线：

- 负责 R1 workflow state StoreLock、备份保留策略、并发写测试。
- 不迁移 SQLite，不改 UI。

重构线：

- 负责 R2 lib.rs 分批解体。
- 每批只搬一个领域，保持行为不变。

SQLite 迁移线：

- 负责 R3 schema、导入器、双写、读切、回滚演练。
- 不在 R3 前授权多 agent 并行真实执行。

前端瘦身线：

- 负责 R4 按页查询、组件拆分、样式资产清单、测试拆分。
- 不改视觉风格，不重做布局。

文档 / 蓝图线：

- 负责 R5 蓝图正本、矛盾清理、吸收建议查重。
- 不把研究条目升级成执行事实。

复核线：

- 只读复核 P0/P1/P2、形状指标、行为不变、安全边界、evidence / handoff。
- 不承担开发写入，不替主管线做最终接受。

## 7. 首批任务包

用户确认本文后，先写两个任务包，可并行执行：

### R0 任务包

建议路径：

- `tasks/2026-06-10-root-treatment-r0-shape-gate-task-template-and-governance-package-type-v1.md`

范围：

- workbench shape gate。
- 任务包模板“形状影响”节。
- 治理任务包类型。
- 解冻后 `1:3` 治理配额。
- evidence / handoff 的 git commit hash / no-git blocked 字段。
- R1 也可使用的手工 shape metrics 表。

不做：

- 不改业务代码。
- 不拆 lib.rs。
- 不改 storage。

### R1 任务包

建议路径：

- `tasks/2026-06-10-root-treatment-r1-workflow-state-lock-and-backup-retention-v1.md`

范围：

- `workflow_state_store.rs` StoreLock。
- workflow state backups 保留策略。
- 并发写 / lock busy / corrupt JSON / backup retention 测试。

不做：

- 不迁移 SQLite。
- 不改真实 Codex。
- 不清理真实历史备份，除非任务包列 dry-run 并用户确认。

## 8. 验证矩阵

每个治理任务包至少记录：

- shape gate 结果。
- `lib.rs` 行数前后值。
- 触碰棘轮文件前后行数。
- 是否新增 command。
- 是否新增 sidecar。
- 是否触碰 workflow state schema。
- 是否读写 `/Users/yoyi/.codex`。
- 是否执行真实 Codex。
- git commit hash；如无 git，必须写 `no_git_blocked_for_r2_r3`。
- 解冻后功能任务包是否触发 `1:3` 治理配额。

常规验证按改动范围选择：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib`
- 相关 Rust 单测
- `cargo fmt -- --check`
- 文案 / 禁止项扫描

R0 可接受不跑全量 cargo，但必须说明原因；R1 改 Rust 存储逻辑时必须跑相关 Rust 单测和格式检查。

## 9. 治理冻结期间的 backlog 分类规则

治理期不是把 backlog 全部丢到一边，也不是趁治理夹带功能。判断标准如下：

- 能减少治理风险、强化开发制度、降低后续返工的，可以并入 R0-R5。
- 只适合先明确 schema / decision / 设计预留的，可以在 R5 或对应阶段记录，但不能实现运行时。
- 会新增用户功能、扩大执行能力、改变产品体验的，必须冻结到治理后。

### 9.1 治理期可吸收

这些条目可以并入治理期，因为它们服务制度和代码形状，不是产品功能：

| backlog 条目 | 并入位置 | 治理期做什么 | 不做什么 |
| --- | --- | --- | --- |
| 决策影响标注 | R0 | 进入任务包模板 / decision 模板，要求新 decision 写影响面 | 不回填所有历史 decision |
| 引入 spike 类型任务 | R0 | 增加 spike / governance spike 模板，限定时间盒、问题、产出归档 | 不把 spike 变成绕过任务包的开发入口 |
| 走通骨架先丑后美 | R0 | 写入治理任务包原则：先保行为，再改形状 | 不借此降低验收标准 |
| outbox JSON schema | R5 | 做 schema / decision 归档，作为后续 worker 回收接口预留 | 不接运行时，不改 workflow state |

### 9.2 治理期只设计 / 预留

这些条目和治理强相关，但治理期只能做边界、映射或预留，不实现产品功能：

| backlog 条目 | 并入位置 | 治理期允许 | 治理期禁止 |
| --- | --- | --- | --- |
| 前端整体观感重做 | R4 | 按解冻后目标布局区块拆组件边界；保留水墨风格资产清单 | 不实际重做布局，不改视觉风格 |
| 工作流画布重做为 AIGC 式无限画布 | R4 / R5 | 保留 React Flow 深化决策和读模型映射预留 | 不实现无限画布，不做节点交互重写 |
| UI 视觉反馈 MCP 工具 | R4 后评估 | 可作为治理辅助工具 spike 候选，只评估截图 / DOM 结构回传路线 | 不实现 MCP 产品工具，不接 agent 自动验收 |
| 记忆召回时效标注 | R5 | 写清 schema / 注入链路预留，登记到 M4 / Stage L 后续 | 不改 M4 运行时代码，不自动生成正式记忆 |

其中“记忆召回时效标注”的当前口径：

- 它不是“修正错误记忆”，而是解决召回时缺少写入时间和来源，导致读者分不清这是现状还是旧况。
- 解冻后实现时，每条被注入的记忆都应带 `createdAt` / `updatedAt` 推导出的时效提示，超过阈值时提示“建议复查”。
- 它最适合和 M4 任务记忆包 / 后续主管视野注入链路合并处理。

### 9.3 治理后解冻

这些条目仍然冻结到治理后：

- CEO 秘书型工作台 AI。
- 角色技能派发规则。
- 四角色工作流模板。
- 技能注册表。
- 记录员 agent。
- 咨询 agent。
- 方案 + 反对双 agent。
- Adapter 化每种 agent。
- 前端整体布局实际重做。
- React Flow 深化无限画布实际开发。
- UI 视觉反馈 MCP 工具实际实现。
- 记忆召回时效标注代码实现。

解冻顺序建议：

1. UI 视觉反馈 MCP 工具评估。
2. 前端整体布局重做。
3. React Flow 深化无限画布。
4. 记忆召回时效标注。
5. 多 agent / adapter / 技能相关路线。

该顺序只是解冻后建议，不进入 R0-R5 的实现范围。

## 10. 最终验收

治理阶段 R 可接受为：

- shape gate 强制执行至少一个完整 Stage。
- `lib.rs <= 3,000`，或全部棘轮文件处于明确下降轨道并冻结剩余 deferred。
- 生产持久化收敛为 1 个 SQLite + 可重建索引，sidecar JSON 只作导出 / 备份。
- 跨域写有事务，中断注入不产生半写。
- 至少 4 个主页面使用按页查询。
- 蓝图正本唯一，§17 / §22 / §26.4 矛盾清除。
- R2 / R3 启动前存在 git baseline、可审查 diff 和可回滚提交。
- 解冻后 `1:3` 治理配额写入模板、gate 文档和验收口径。
- R0-R5 evidence / handoff 完整。

治理阶段 R 不接受为：

- 最终蓝图完整工作台。
- Stage L 完成。
- Stage K 完成。
- K3-B1 retry 成功。
- K3-B2 可开始。
- planned adapters 真实接入。
- provider credential / model verification。
- backlog 解冻后功能完成。
- 多 agent 并行真实执行已解锁。

## 11. 下一步

当前按用户要求继续 Root Treatment / Stage R，下一步：

1. 准备 R4-A31：继续中等粒度离线交互测试按域拆分任务包。
2. R4 必须按页查询和前端结构瘦身，不改视觉风格、不实际重做布局、不实现 MCP 看图工具。
3. 如要执行 R3 Level B，必须先写单独 execution record，明确 allowed source root、production DB path、backup / report / rollback manifest、before / after source hashes、rollback / recovery，不得跳过 fresh verify。

下一步准备期间：

- 不执行真实 Codex。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 不启动 Stage L / K3-B1 retry / K3-B2。
- 不解冻 backlog 功能。
- 不切产品读写路径，不停写 JSON / sidecar，不把 R3-A8 copied snapshot rehearsal、R3-A9 Level A fixture / temp rehearsal、R3-A10 Level A limited read-cut contract、R3-A11 Level A observation contract、R3-A12 stop-write JSON decision contract、R3-A13 transaction acceptance、R4-A4 selector consumption、R4-A5 selector consumption、R4-A6 selector consumption、R4-A7 type split、R4-A8 ProjectsView component extraction、R4-A9 AgentView transcript component extraction、R4-A10 styles.css 风格资产清单、R4-A11 离线测试 helper 抽离、R4-A12 权限弹层场景 helper 抽离、R4-A13 任务字段 / 派发准备 helper 抽离、R4-A14 runtime / diagnostic fixture 抽离、R4-A15 worker protocol fixture 抽离、R4-A16 workflow state 变体 fixture 抽离、R4-A17 authorization workflow fixture 抽离、R4-A18 基础 workflow state / project workflow fixture 抽离、R4-A19 derived workflow fixture 抽离、R4-A20 C6 result summary fixture 抽离、R4-A21 run queue fixture 抽离、R4-A22 candidate governance fixture 抽离、R4-A23 memory center core store fixture 抽离、R4-A24 memory center governance fixture 抽离、R4-A25 memory pattern fixture 抽离、R4-A26 KnowledgeBase / Secretary fixture 抽离、R4-A27 Transcript / Session fixture 抽离、R4-A28 Workbench base snapshot fixture 抽离、R4-A29 Real Execution Product Command / Project Workflow Automation fixture 抽离或 R4-A30 Session Continuation Store fixture 抽离冒充为真实 production apply、生产读切、production observation、JSON / sidecar 停写、R3 全量完成、R4 完成、离线测试全部按域拆分完成、CSS 源码拆分、UI 重做、真实 Codex 执行或页面真实数据来源迁移完成。
