# 当前状态

截至 2026-08-19，M1–M4 的既有具名主线与已关闭阶段事实保持不变。M5 在具名证据范围内为 `SCOPED PRODUCT-CHAIN PASS`，stage-14 已关闭。stage-15 active；M6P00、CP1、CP2 与 CP3 均已获独立 PASS，M6D01 静态合同/fixtures、M6D02 持久 Global Supervisor RoleSession、M6D03 只读跨项目 advisory 候选 `60a8e19`、M6D04 Secretary consult Handoff 候选 `ec1ba99`、M6D05 稳定成员目录候选 `a58815f` 与 M6D06 临时 agent 历史投影候选 `274cb08` 的各自具名范围成立。M6D01–M6D06 已归档，当前唯一 current leaf 为 M6D07 独立多视角会诊；M6 域层整体、stage-15、壳采纳、发布、部署与真实日用均未由此成立。

## 2026-08-18 M5 当前状态

- M5R00 候选 `99a5afc`、M5R07 修订候选 `7cab372`、M5R08 内容候选 `09e9b32` 与 M5R09 内容候选 `c91d8fc72bcbf80186736caff841cb7a9b0660d1` 已分别通过适用独立验收；M5R09 放行精确绑定记账 `8e6f59f48d2d90891d3c02396378921e4a2f5d6e`。
- M5R09 提供普通产品显式 M1 enrollment producer 与最小 UI；缺 source 可恢复启动为 `UNENROLLED` 而 M1 业务写 fail-closed。nested memory/mature owner 收敛 canonical identity，governance 测试走 authority fixture；no-follow 使用 target-family cfg；dispatch gate 与 durable duplicate-effect 反例精确分开。
- 独立验收官在 `c91d8fc` detached checkout 上复跑 `cargo check` 0；`m5r09_` 23/23，memory/mature 各 14/14，ordinary source 4/4，完整 `m5_` 188/188；前端 typecheck/default build 0；默认 bundle gate 与候选/记账 diff check 符合预期。主管原始证据位于 `.syn-gates/evidence/M5R09-c91d8fc/`。
- M5C01 closeout 内容 `de98d69a363ff82281330fb3b82de82c03a9b484` / tree `b90244a8535c829e96341d42fef39602ef499f6d` 只含 5 个 closeout/unfinished/交接路径，零产品源码变化。结构、冻结物、用户载体、authorization 与 lifecycle 检查最终全绿；日志位于 `.syn-gates/evidence/M5C01-de98d69/`。
- protected WIP 已分活动 runtime 与静态 hash 两层。30 个静态路径在 closeout 观察时 30/30 hash 不变；`commands.rs` 候选外旧 WIP仍为 59+/56-；6 个 `m6_*.rs` 仍未跟踪。用户 OSS 门面已于 `c1025ba` 精确 7 路径独立提交，不属 M5 候选。没有 reset、stash、clean、覆盖或混入候选。
- M5R09、M5C01 与 stage-14 已归档，当前没有 M5 current leaf。stage-15 已 active；M6P00、CP1、CP2 与 CP3 已获独立 PASS，M6D01–M6D06 已归档，M6D07 为唯一 current leaf，M6D08 未开始。F2/F3/F5、M7–M11、Headless Core、Primary/epoch 与壳采纳继续 `NOT_ACTIVE`。
- 上述结论只到 Linux WSL 的 detached/local/synthetic/ordinary Tauri 产品链和静态边界；没有真实个人资料/项目、真实 provider/账号/凭据、外部业务写、macOS/BSD 实机、真窗口像素、新壳运行、部署、发布或长期真实日用。

当前用户已指定 5600X WSL `/home/synadmin/workspace/syn` 为权威工作仓库。Harness 文档生命周期上 `stage-12` 仍开启，D0C04 / D0C05 保持 unfinished；`stage-13` 与 `stage-14` 已完成并归档；`stage-15` active。M6P00、CP1、CP2 与 CP3 检查点已 PASS，M6D01–M6D06 已归档；M6D07 已拉入 current 并重签 authorization，完成后同段继续 M6D08。

## 2026-08-19 stage-15 当前状态

- 内容候选 `4147454bc046d5a5d3047799725d9e77ed086179` / tree `69816100d15c449b16faef08deda1fc37af48df5` 将正式 Global Supervisor、project workflow、workflow execution/dispatch 与过程事实入口接入 M1 canonical `ProjectId`，并为 relation source owner 增加可判别类型、foreign project fail-closed 与合法 doc/tool/session 保留边界。
- detached candidate 上 `cargo check --lib --offline` exit 0；M6P00 21/21、global supervisor 33/33（2 ignored）、memory relation 19/19、project workflow 51/51（6 ignored）、workflow dispatch 14/14、offline role 3/3，`git diff --check` exit 0。原始日志在 `.syn-gates/evidence/M6P00-4147454/`。
- `conversation_transport_` 在候选与 clean HEAD 上均为 22 passed / 6 failed、exit 101，失败集合一致，作为既有基线欠账记录，不伪装成绿色回归，也不反向判 M6P00 失败。
- Cursor Opus 独立验收在修正记账写域后签发 `stage-15-m6p00-20260819-0342.verdict.md` PASS；放行只到 M6P00。
- M6D01 内容 `80ddebdf17889035bc7acde423e32ad6de6f17bb` / tree `9b9ed64be8f8cf6f02c0436ec9883631fe55b56e` 冻结跨项目 ACL/freshness/advisory join、采纳与逐项目应用、stable/temporary 成员、完整执行 envelope、多视角独立性和迁移矩阵；41 个离线 fixtures 逐例校验通过。它没有实现 service、repository、projection、runtime、Tauri command 或 UI。
- M6D02 内容 `651a8fb9329d2ff07b4befe14fb37a1811942766` / tree `8be2ac175f0aeb4027441f53883d9e7f9d5f67aa` 在既有 M3 repository 上安装 server-fixed、global、read-only 的持久 RoleSession，并由普通 AppState 与真实零身份输入 Tauri status command 消费；isolated/legacy 保持 unavailable。CP1 独立重跑为 M6D02 15/15、M4C02 14/14、cargo check exit 0；rustc 汇总 897 warnings、文本 warning 行计数 898，新 M6D02 文件为 0。
- Cursor Opus 独立验收签发 `stage-15-cp1-20260819-0521.verdict.md` PASS；欠账已路由到 M6D03、M6D08、M6S01 与 ENG-01。
- M6D03 先以 `977770f115f6a416a9466c59728ab9ecfc04b669` 关闭 canonical workflow owner exact-join 前置，再以 `60a8e198f7319c8d175754079d08c61ddb88614c` / tree `e4539f211f1c160906b4c05f41f75041a6e5134b` 建立只经 M5 ProjectSummary port 的 read-only advisory、M6 自有 DecisionRequest/receipt projection、Global RoleSession 最小 refs 与恒拒绝项目写边界。detached 证据为 M6D03 13/13、M6P00 21/21、M6D02 15/15、M5 ProjectSummary 3/3、candidate/parent cargo check 均 exit 0 且 warning delta 0。
- M6D04 内容 `ec1ba997af6c8b2418c5f1b7051f1015a5307996` / tree `685e458b3670fbc99dd57aa7f55c624d1307f271` 复用 M3 Handoff owner，把普通 M4 Secretary start/read 与 Global Supervisor accept/reject、M6D03 accepted-receipt advisory 和回执接通；detached 证据为 M6D04 4/4、M6D03 13/13、M6D02 15/15、M4C05 9/9、M3C05 43/43，cargo check/diff-check 均 exit 0。Cursor Opus 已签发 CP2 verdict `stage-15-cp2-20260819-0733` PASS；该结论仍不构成 M6 域层完成、跨项目 UI 可用、GUI/新壳验收、发布或真实系统运行。
- M6D05 内容 `a58815ff02b912003de8abcf84507c43ad7245dc` / tree `bdd45a1ca82eb85c9ce242f7b493ce72e339365f` 建立 explicit identity-only 的持久 stable member directory、heuristic ref-only quarantine、append-only lifecycle/refs、TTL availability、M3-owned non-grant contact 与生产 export/rebuild 校验，并由七个普通 Tauri command 消费。detached 证据为 M6D05 7/7、相邻 M6D04/M6D03/M6D02/M4C05/M3C05 84/84、cargo check/diff-check exit 0，warning 数保持 CP2 基线 888。该叶证据只覆盖本地离线域层；GUI/新壳、真实人员/provider/message、项目写、发布与 M6 完成均未由此成立。
- M6D06 内容 `274cb08629e09689357cd1522c1ad23f1aea9e08` / tree `b49f177d88b9f5a06306b093436fbc9728d2e5c9` 从只读 M5 完整执行 envelope 投影 TemporaryAgent、任务/结果/失败/来源引用与 REF_ONLY quarantine，严格区分 ChildRunRef、TemporaryAgent 与 StableMember；只有显式人工晋升可新建或绑定 StableMember，且原历史不变。三个普通 Tauri command 可达。detached 证据为 M6D06 8/8、相邻 M6D05/M6D04/M6D03/M6D02/M4C05/M3C05 91/91、cargo check/diff-check exit 0，warning 数为 888。
- Cursor Opus 独立验收签发 `stage-15-cp3-20260819-0924.verdict.md` PASS：在两个独立 disposable checkout 复跑 15 条命令全部 exit 0、合计 190 passed / 0 failed，两候选 warning 汇总均 888，写域、冻结物、7 个受保护载体、ordinary Tauri composition 与生命周期均通过。8 条非阻塞欠账已路由到 ENG-01、M6D08 与 M6S01。该 PASS 只覆盖 M6D05+M6D06 的本地离线合成域层，不构成 M6 域层整体、stage-15、GUI/新壳、真实系统、部署或发布结论。

## 现在分别看哪里

1. 当前用户指令：决定本轮目标和授权；
2. `docs/product/syn-product-canon-v1.md`：决定 Syn 长期是什么；
3. `docs/product/authority-register-v1.md`：决定各类文件当前有什么效力；
4. `docs/workbench-system-architecture-v1.md`：决定现行系统边界；
5. 本文件、源码与新鲜验证：说明已实现事实、未知和证据上限；
6. `AGENTS.md`、`docs/harness/plan.md`、唯一 current leaf（若存在）与 `docs/harness/done/2026-08/`：确认当前工作投影和历史收口；leaf / stage / 旧授权不能扩大当前用户原话。

验收报告、交接、历史任务、研究和旧决定只按登记状态提供证据或来源，不自行成为产品定义、当前计划或持续授权。

## M4 已进入主线及 stage-07 修正后的事实

- M4 实施合同 `docs/contracts/m4-secretary-attention-daily-resolution-v1.md` 已冻结并保持 SHA-256 `4e4d6251d53e1b9b156fb2fd1266d73d6beace38be2086e83e0f05694dec4e51`；M1 四份合同和 M3 实施补充合同均未被 M4 改写。
- 普通产品 `AppState` 已安装后端构造的 Secretary RoleSession、PersonalScope、daily channel 与权限快照；身份不再来自固定项目 cwd、路由或 renderer 自报字段，错误 scope 继续 fail closed。
- M4 自有 SQLite schema/repository/UoW 已持久化 source-first Inbox、OpenLoop、Decision projection、watermark、去重、排序理由、receipt/event/audit/checkpoint；不同 source owner 不合并，未知、敏感、过期或无法精确绑定的输入 quarantine。
- read、dismiss、snooze、acknowledge、close、reopen、carry-over、Notification、Reminder 与显式 standalone `PersonalAction` 已有 CAS、幂等、重启和审计语义。协调状态不反写 owner；OpenLoop、日报或模型解释不会自动创建 Todo。M4R02 已接普通产品来源与个人对象组合入口，M4R03 已接服务端到期时钟与恢复链。
- Secretary 应用服务基于持久上下文提供确定性 brief、只读查询、模型增强 ledger 和 M3 Handoff 状态处理；M6D04 候选已把普通产品的 Secretary consult start/read 接到 M3-owned Handoff，并把 Global Supervisor accept/reject 与 M6D03 advisory 返回接通，且该段已获 CP2 PASS。结论仍只覆盖本地离线域层，不代表真实 provider、消息、GUI 或新壳可用。
- 首页已消费后端 typed read model，展示来源、owner、优先理由、最后变化、状态和 source descriptor，并提供协调动作；React 不拥有协调真值。M4R04 已接注册 owner 的精确回源，M4R05 已接复用 M3 RoleSession / Turn / transport 的持续 Secretary 对话与恢复。
- Daily scheduler 已实现 OS IANA timezone、本地自然日窗口、最多 7 个窗口 catch-up、同窗幂等、版本纠正、重建和确定性 report；空事件窗口的 agent turn 与 model invocation 均机械证明为 0。M4R03 已把 snoozed OpenLoop 与 Reminder 到期推进接入普通服务端时钟链。
- 旧 secretary/right rail/runtime attention/pending action/memory daily 五类读面已有 inventory、comparator、compatibility read-only 边界和 quarantine；M4R06 已接实际 server-owned shadow reader、parity/quarantine 与受守卫 fallback，未物理删除旧面。
- C09 使用隔离 profile、两个 synthetic source owner 与 fake model 完成首启、SIGKILL、同 profile 重启、生命周期恢复、日报重跑、deep link、模型失败和零事件验收；证据只到机械层与隔离产品 App，不等于真实日常使用。
- C10 将全部 M4 前端测试挂入 44-entrypoint 离线 runner，并以运行时等价源码修复 C09 与旧 R4/M3 source-string 静态契约碰撞；修复提交为 `9e97120`。

## 2026-08-11 独立总线复核（修正前历史基线）

独立复核确认 Git/Harness 程序性收口成立，M4 底层 repository、状态机、日报、去重和隔离合成证据可继续依赖；但以下五项阻断产品验收：

1. 普通产品没有 M4 内部 source ingress 的生产调用者；C09 直接注入 synthetic source。
2. snoozed OpenLoop 与 Reminder 到期没有生产 scheduler 驱动。
3. source link 只进入通用 Projects 页面，没有精确定位原对象。
4. 首页持续 Secretary 消息输入、M3 Turn 写入与跨重启历史恢复未接入。
5. 五类 legacy compatibility 仍是 inventory-only quarantine，没有实际 adapter parity。

完整事实、证据上限和当时验收决定见 `docs/harness/reports/M4-independent-bus-review-2026-08-11.md`。以上五项是 M4R01–M4R06 的修正输入，不再代表 2026-08-13 的当前产品链状态，也不需要重新拍板 M4 核心产品需求。

## 2026-08-13 M4R07 v2 产品链收口证据

- 仓库可携带 receipt `docs/harness/reports/M4R07-isolated-product-reacceptance-closeout-behavior-receipt.json` 使用 `syn.m4.isolated-product-reacceptance.behavior-receipt.v2`，结果为 `PASS`，固定 12 次、实际 12 次，`retry=0`。
- 第 8 次仍执行普通 `recovery_timer`，真实等待 98 秒并保留后端恢复验证；其 receipt SHA 被 `launch_8_ui_validation` 范围对象绑定。取消的只是 UI / Computer Use / PNG / attestation gate。
- `launch_8_ui_validation` 明确记录 `required_by_current_contract=false`、`execution_status=NOT_EXECUTED`、`acceptance_result=NOT_APPLICABLE`、Computer Use 次数 0、截图/attestation/capture signal 均未写。这既不是视觉失败，也不是页面、Accessibility、截图质量或 Computer Use 的 `PASS`。
- `docs/harness/reports/M4R07-isolated-product-reacceptance-evidence/manifest.json` 使用 `syn.m4r07.closeout-evidence-manifest.v2`，精确绑定 portable receipt SHA 与 `launch_8_ui_validation` canonical SHA。当前完成标记只证明该 v2 后端/普通产品链范围。
- M4R07 已归档，`stage-07` Harness 生命周期已经关闭；该关闭事实不自动激活 M5–M11，也不扩大上述 v2 产品链证据范围。

## M3C01–M3C07 已证实的主线事实

- M3C01 冻结了 RoleSession、Turn、ProviderHandle、ConversationContext 与 Handoff 的实施解释、迁移矩阵、默认 fail-closed 规则和回切边界；M1 四份冻结合同保持原样。
- M3C02 在 provider spawn 前为 shared Agent existing、legacy raw existing 和 GUI-direct existing 接入 server index、canonical thread owner、Station 3b 与固定 permission envelope 守卫。
- M3C03 建立了 M3 自有、版本化的 RoleSession / Turn / ProviderHandle / ConversationContext repository、schema 与只读 shadow import 分类；raw transcript 与前端 cache 不成为迁移真源。
- M3C04 建立 start / continue / poll / stop / resume transport port 与 fake provider 的重启、receipt 和不重复 effect 语义。
- M3C05 建立显式 Handoff 状态机、幂等 receipt 与由 source owner 回写结果的边界。
- M3C06 建立 server-owned RoleSession read model；Agent Center 与 Jiaoban 的 React / module cache 已退为兼容显示 fallback，不再决定恢复身份、role、scope 或 permission 真值。
- M3C07 在 debug build、隔离 profile 和 fake provider 的三重 gate 内完成 Agent / Jiaoban synthetic host 的分层验收。旧路保留受守卫的回切入口；但 M3C07 隔离子进程的全局 `invoke_handler` allowlist 会在 wrapper、binding、provider 与 effect 之前拒绝 legacy Agent / Jiaoban transport。该隔离限制不等于普通模式的产品退役。

M3C07 的已归档命令、分层结果、六份 launcher receipt SHA-256、P0/P1/P2 和桌面观察边界见 `docs/harness/reports/M3C07-isolated-desktop-layered-acceptance.md`。这些是 M3C07 隔离证据，不是 M3C08 的新鲜主线回归结果。

## 冻结、迁移与证据边界

- 2026-08-10 静态核对确认 M1 四合同 SHA-256 仍为 `77c829…b2ca`、`3378f0…86bf`、`3cb007…3ea4`、`15a24d…8e99`，且相对 `29085cc` diff exact；M3 合同 SHA-256 为 `946c75…ac48`，M4 合同为 `4e4d62…4e51`，两者相对 `530ab41` diff exact。完整值见 M4C10 报告。
- 最终 Rust 聚焦回归为 C09 3/3、M4 98/98；完整 `cargo test --lib` 在主机权限环境为 1639 passed / 0 failed / 45 ignored。受限 sandbox 首跑的 5 个 launcher source-string collision 与 1 个 PID `lstart` EPERM 作为红灯保留；前五项已做等价源码消歧，PID exact 主机复跑 1/1 后完整套件全绿。
- `cargo check --lib`、M4 新增 Rust 文件定向 `rustfmt --check`、TypeScript typecheck、44-entrypoint offline interaction、3 个 launcher `node --check` 和 production build 均 exit 0。build 仍有既有 `>500k` chunk 提示；`cargo check` 仍报告仓库既有 warning debt，不写成零 warning。
- C09 仓库回执 SHA-256 为 launcher `036d002…d1eb`、runtime `5371773…210`、UI `669f4b1…23c6`；截图只在本任务可见记录中，以 hash 留痕，不在仓库。详细证据和完整 hash 见 `docs/harness/reports/M4C09-isolated-product-app-layered-acceptance.md` 与 M4C10 报告。
- 回切只能选择受守卫的 legacy read-only 展示或关闭 M4 ingestion/scheduler/read projection；必须保留 M1/M3 守卫、M4 已提交协调状态、event/audit/receipt/quarantine/report version，不能重放 effect、反写 owner 或物理删除旧面。

## 尚未成立或未进入

- M5 已在 `c91d8fc` 的具名 product-chain 范围通过独立验收并由 M5C01 关闭 stage-14；M6P00、CP1、CP2 与 CP3 已获独立 PASS。尚未成立的是 M6D07–M6D08、M6 域层整体、stage-15、UI、发布、真实资料/项目、真实 provider/账号/凭据、macOS/BSD 实机、真窗口像素、新壳运行与长期日用。
- M6 Global Supervisor consult 已在 `ec1ba99` 候选的普通 Tauri/M4/M3/M6 本地链路实现并以 fake provider、合成 summary 验证，且 CP2 已放行；真实 provider/模型/消息、renderer consumption 与新壳交互均未进入。
- M7 对 `DailyWindowClosed` / `DailyReportVersioned` 的消费、正式记忆、PersonalFact、个人模型与 Skill 未实现；M4 只产出 source-backed event/ref，不写 M7 对象。
- M8 真实 connector、credential 与外部 source 未进入；M9 旧路 command unregister/物理退役、M10 全日真实试点与发布硬化、M11 受治理自升级也未进入。
- 真实个人资料、真实用户项目写入、真实模型/provider、真实 Codex 消息、真实账号/凭据/connector、网络外部写入、真实数据迁移、push、merge、rebase、部署和发布均未进入。
- M4R07 v2 只证明本机隔离普通产品的后端/产品链；第 8 次 UI / Computer Use / PNG / attestation 为 `NOT_EXECUTED / NOT_APPLICABLE`，没有视觉验收结论。它也不证明长期运行、真实日常节奏、真实数据/provider/connector、发布包或生产结论。
- 5600X WSL 已成为用户指定的权威源码工作仓库，D0D01 有 `SOURCE_BYTES_MATCH` 证据；长期 SSH 当前启动周期与重启后稳定性仍分别由 D0C04 / D0C05 结算，不能从当前可连接反推 `PERSISTENT_SSH_RESTART_STABLE`。
- 2026-08-16 已完成 DSH 官方研究、Syn 原生治理核心 / 可替换 Agent Runtime / 受治理自升级方向决定、架构与 M5–M11 计划校准；这些是文档和计划事实，不是产品代码、DSH 集成或自升级实现。

## 当前开发状态与停止点

M4 与 stage-06/stage-07 的历史关闭事实保持不变。WSL 迁移 `stage-12` 仍开，D0C04 / D0C05 不因本轮恢复；`stage-13`、`stage-14` 已归档，`stage-15` active。M6P00、CP1、CP2 与 CP3 已独立 PASS；M6D01–M6D06 已归档，M6D07 为唯一 current leaf。M6D07 自复核收口后同段继续 M6D08，再到阶段交包；不提前宣布阶段或 M6 完成。

stage-15 的明确激活只授权当前计划内 M6 域层连续推进，不自动激活 F2/F3/F5、壳采纳、OSS-01 push/申请、真实数据 / 模型 / provider / connector / 账号凭据 / 外部业务写、部署或发布；这些均没有发生。

## 保全

- 5600X WSL `/home/synadmin/workspace/syn` 是当前权威工作仓库；其中既有 dirty WIP 继续保全，不 reset、clean、stash、覆盖或归责。
- `/Users/yoyi/workspace/product-line-syn-fnd-002` 与 `/Users/yoyi/workspace/product-line-syn-m2-closeout` 只作为旧 Mac 审查锚点保留，不再作为当前主线写入位置。
