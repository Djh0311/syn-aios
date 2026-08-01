# Syn M0 文档对齐至 FND-001 指导交接 v1

- 日期：2026-08-01
- 状态：**CURRENT GUIDANCE HANDOFF / TEMPORARY / NO EXECUTION AUTHORITY**
- task-id：`SYN-M0-TO-FND-001-GUIDANCE-HANDOFF`
- from：当前 Syn M0 文档对齐主导线
- to：下一位 Syn 主导线或 `SYN-FND-001` 执行线
- summary：M0 已完成文档、决定和路线冻结；下一任务只能先显式激活合同切片 `SYN-FND-001`。
- 仓库：`/Users/yoyi/workspace/product-line`
- Git 基线：`main@8f63076ff4ac`
- 当前 Harness：`mode=PLAN`、`work-state=READY`、`active-id=NONE`

> 本交接只压缩 M0 的已确认结果、当前事实、边界和下一动作。它不是新的产品正本、任务包或执行授权。发生冲突时，依次服从当前用户指令、`../docs/harness/AUTHORITY.md`、`../docs/harness/CURRENT.md`、当前决定和当前计划。

## 1. 交接结论

M0“文档、决定与当前路线冻结”已经完成，可以在这里结束长上下文。当前没有进行到一半的工程任务，也没有 active package。下一任务应从 M1 的首个文档切片 `SYN-FND-001` 开始，不得续跑旧 Jiaoban、conversation repair、R7、知识线或历史 workflow 计划。

当前准确状态：

- 产品方向、全工作台运行模型和 M0—M10 长期路线已经对齐；
- 产品代码仍暂停，没有因为计划完成而自动获准施工；
- `SYN-FND-001` 是唯一首个可激活切片，只写合同和迁移矩阵；
- `SYN-FND-002`—`SYN-FND-005` 涉及安全闸、作用域或授权判断，后续必须逐包取得用户明确授权；
- 当前没有 Stage 1 的外部设计阻塞。

## 2. 已确认的 Syn 产品模型

1. Syn 是只服务当前用户的全能个人 AI 工作台，不是单纯的项目管理器或工作流工具。
2. 顶层入口是“秘书”和“全局主管”；“贾维斯 / Jarvis”只是不再使用的秘书历史别名。项目主管常驻明确项目内部，不成为会猜项目的万能顶层入口。
3. 项目是复杂工作的事实、权限和执行边界；个人范围与项目范围并存，个人事项不必强行建项目。
4. 日常与开发是每轮会话或工作项明确携带的执行通道，不是全局模式开关，也不是两套产品。
5. 整个工作台采用事件驱动：有新事件才处理；没有事件时不得持续调用 Agent 或模型。
6. 秘书负责汇总、解释、提醒和看住与用户有关的未闭环事项，但不擅自改变项目事实、任务、优先级或授权。
7. 普通对话不自动创建项目、待办、方案、工作流、审批或想法。正式业务对象只来自用户明确语义、已批准规则或既有授权；正式记忆按单独的记忆治理策略处理。
8. 记忆允许由秘书和记忆层主动捕获；每日整理可以形成日报、记忆结果和 Skill 候选，但必须可回源、可纠正、可审计、不泄密、不扩权。
9. 外部软件按 `view / index / sync / action / secret` 分权；读取权不等于写入权。OpenConnector 只是参考，不是已经选定或接入的实现。
10. 凭据只通过受保护的 `CredentialRef` 使用，不得进入聊天、记忆、事件、普通数据库字段、日志或 prompt。
11. 已与他人约定和时间敏感事项优先。稳定成员必须可搜索、可直接联系；临时 Agent 必须可追踪和审计。
12. 现有前端壳、项目页和专业能力入口暂时保留；后续首页改成秘书情境简报加持续对话。内部“一人 AI 公司”默认在后台运行，不强迫用户每天面对组织图。

正式依据：

- `../decisions/2026-08-01-whole-workbench-event-driven-operating-model-amendment-v1.md`
- `../decisions/2026-08-01-memory-self-capture-daily-consolidation-and-skill-governance-amendment-v1.md`

## 3. M0 实际完成了什么

### 3.1 建立唯一当前路线

- 权威索引：`../docs/harness/AUTHORITY.md`
- 当前状态：`../docs/harness/CURRENT.md`
- 唯一总开发计划：`../docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`
- 当前阶段计划：`../docs/plans/2026-08-01-syn-stage-1-contracts-and-security-foundation-plan-v1.md`

### 3.2 全面文档对齐

- 对齐了 README、AGENTS、长期工作线、架构、前端边界、记忆正本、组织愿景、现状库存和 backlog；
- 把项目交互、conversation-first、角色、编排治理、共享 transport、知识和记忆决定重新接入当前索引；
- 把旧 E/F/G/H/I、J/K/L/R、Phase C、旧 workflow、旧 UI、旧 memory slice 等路线标为历史、停止或 HOLD；
- 08-01 Jiaoban 有界重建路线保持停止，不得借“项目对话仍重要”恢复施工；
- 现有能力统一按 `KEEP / EXTRACT / REWRITE / RETIRE / HOLD` 处置，采用模块化单体内的绞杀式迁移，不做一次性推倒重来。

### 3.3 建立可施工的 M0—M10 路线

- M1—M3：合同、安全、事实、事件、事务、角色会话和显式交接；
- M4—M8：秘书、项目主管、全局主管、内部成员、记忆、个人模型、Skill、Connector 和 CredentialRef；
- M9：读模型切换、数据迁移和旧路退役；
- M10：真实完整一天试用与发布硬化。

## 4. 当前实现事实，不得扩大解释

- 当前源码仍是多个分散垂线，不等于目标模块已经实现；现状底账只看 `../docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md` 和直接源码核对。
- 普通项目 conversation 已存在持久 transport 方向的工作树改动，但只有离线检查，不是当前桌面端到端验收。
- 工作流、方案、审批、运行、知识和记忆有部分既有能力；这些是待提取资产，不证明已经形成 Syn 的日常协同闭环。
- 通用个人收件箱、正式日报实体、个人模型、统一 Connector 运行面和凭据生命周期仍未完成。
- M0 只修改和校正文档；没有启动 App、发送真实消息、调用真实 store、运行 workflow、连接外部软件、读取真实凭据或验收产品运行态。

## 5. 下一任务：`SYN-FND-001`

### 5.1 激活前提

必须先由当前用户指令明确激活 `SYN-FND-001`，并建立与 v0.5 active node 匹配的 active package。仅阅读本交接、master 或 Stage 1 计划不能取得执行权。

### 5.2 唯一允许的工作目标

1. 在 `docs/contracts/**` 冻结十份版本化合同：
   - `identity-scope-v1`
   - `command-v1`
   - `event-audit-outbox-v1`
   - `role-session-v1`
   - `handoff-v1`
   - `attention-decision-v1`
   - `project-orchestration-v1`
   - `memory-personal-model-v1`
   - `connector-capability-v1`
   - `object-ref-navigation-v1`
2. 建立旧对象 / command 到新对象 / port / retire 的逐项迁移矩阵。
3. 盘点当前 store、表、sidecar、projection 的 owner、join key 和迁移去向。
4. 给出 M1 测试矩阵，以及 M2 shadow-write、parity、rollback 的输入。
5. 把所有未决定项写成显式 HOLD，不允许在实现代码中临时拍板。
6. 冻结 `OpenLoop` 与既有 `Todo` 的语义和物理关系，避免建立第二套重叠真源。

### 5.3 明确禁止

- 不改任何产品源码或产品测试；
- 不启动 App、Vite、真实 Codex 消息、store、workflow、connector 或真实凭据；
- 不创建第二套总设计、总路线或与 AUTHORITY 竞争的入口；
- 不激活或复用任何历史任务包；
- 不进入 FND-002—FND-006；
- 不执行 Git stage、commit、push、reset、clean、stash 或批量归因。

### 5.4 `SYN-FND-001` 验收重点

- 每个合同都有唯一 owner、真源、状态、跨 scope 规则、事件、审计、敏感字段、幂等、失败、回滚、兼容和明确不做；
- 合同之间没有环形 owner；
- 每个正式动作都能回答 command、policy、state、event、audit、outbox；
- secret、raw transcript 和完整 tool output 的禁止边界可机械测试；
- 每个旧对象和入口都有 `keep / extract / rewrite / retire / hold` 去向；
- 完成后由主导线独立对照当前源码和两份 2026-08-01 修订验收，再决定是否申请 FND-002 / FND-003 授权。

## 6. 当前 HOLD，不阻塞 FND-001

- 真实凭据最终采用 Keychain、加密文件还是其他存储；
- 第一个真实外部 Connector 选择及其正文、引用、同步、删除和保留合同；
- 自动记忆策略矩阵的具体阈值；
- SkillCandidate 何时升级、验证、启用和回滚；
- 当前 dirty 工作树上 DB-primary / JSON reconcile 和真实 App 的运行状态。

这些问题必须在各自阶段形成合同或单独授权，不能由 `SYN-FND-001` 偷偷实现；但它们不妨碍先定义接口、状态和 HOLD 槽位。

## 7. 必须停止并回交

下一任务遇到以下任一情况必须停止，不得自行扩大范围：

- 当前用户指令、AUTHORITY、CURRENT、任务节点和 active package 无法同时一致；
- 需要修改产品源码、测试、安全闸、作用域判断或授权逻辑才能完成合同；
- 无法给旧对象确定唯一 owner、真源、scope 或迁移去向；
- 需要启动 App、使用真实 store / message / workflow / connector / credential 才能得出结论；
- 写面撞上无法归属的 dirty WIP，或需要 Git stage / commit / push / reset / clean / stash；
- 只能靠猜测、历史 task 或旧 handoff 填补当前合同事实。

后续负责人：`SYN-FND-001` 执行线只负责合同和迁移矩阵；主导线负责对照当前源码与两份 2026-08-01 修订独立验收，并决定是否向用户申请 FND-002 / FND-003 的高风险授权。

## 8. 验证与 Git 现实

M0 文档收口已经通过：

- `git diff --check`
- `node scripts/harness-v2/config-check.js --target . --strict`
- `node scripts/harness-v2/active-path-audit.js --target . --strict`
- `node --test scripts/harness-v2/active-path-audit.test.js`
- 当前文档相对 Markdown 链接检查
- `node scripts/harness-v2/project-context.js --target . --diagnostic`

2026-08-01T21:06:50+08:00 的交接诊断为：`PLAN / READY`、active package=`none`、diagnostics=`OK`、Code Map=`OK`；Git 为 `main@8f63076ff4ac`，staged=`0`、unstaged=`100`、untracked=`64`、conflicts=`0`。工作树有大量本轮之前就存在的源码、文档和未跟踪 WIP，必须整体保留，不能从 `git status` 把它们全部归到 M0，也不能为了获得“干净基线”执行清理。

本轮没有 stage、commit 或 push。上述验证只证明文档与 Harness 路由一致，不证明产品代码、真实 App、外部软件或发布通过。

## 9. 下一任务阅读顺序

1. `../AGENTS.md`
2. `../docs/harness/AUTHORITY.md`
3. `../docs/harness/CURRENT.md`
4. 本交接
5. `../docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`
6. `../docs/plans/2026-08-01-syn-stage-1-contracts-and-security-foundation-plan-v1.md`
7. 两份 2026-08-01 正式修订
8. `../docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md`
9. 仅在 `SYN-FND-001` 所需范围内核对源码，不按历史 task / handoff 续跑。

新任务可直接使用下面的启动指令：

> 按当前 AUTHORITY、CURRENT、master 和 Stage 1，正式激活并执行 `SYN-FND-001`。只建立合同包与旧能力迁移矩阵，不修改产品源码；先核对现状，再逐份冻结合同，完成后独立验收并回写 CURRENT。保留全部 dirty WIP，不运行 App、真实消息、store、workflow、connector、凭据或 Git 写入。
