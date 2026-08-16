# 当前状态

截至 2026-08-16，M1–M3 已完成各自具名主线范围。M3 内容提交 `fa8e392`，状态为 `COMPLETED / MAINLINE / STAGE-05 CLOSED`。M4C01–M4C10 已进入主线，`stage-06` 已程序性关闭；2026-08-11 独立总线复核发现的五项普通产品 P1 已由 M4R01–M4R06 修正，M4R07 的 v2 可携带 receipt 在当前后端/普通产品链合同范围内为 `PASS`（12/12）。M4R01–M4R07 均已完成并归档，`stage-07` 已关闭，当前状态是 `M4R07 V2 PRODUCT-CHAIN PASS / STAGE-07 CLOSED`。

## 2026-08-16 M5 事实重整启动（当前状态）

- M5 与 M6 现有实现统一定级为 `M5/M6 CANDIDATE WIP / UNIT-LEVEL PROTOTYPE / NOT_ACCEPTED / NOT_MAINLINE`；57（M5 定向）/ 33（M6 定向）单测只作为候选原型单测基线，不绑定阶段退出。此前 `docs/harness/plan.md` 手工勾选的 stage-14/15 完成标记已撤销，不构成阶段历史。
- 用户 2026-08-16 明确“按计划开始 M5”；`docs/plans/2026-08-16-syn-m5-m6-fact-reconciliation-and-product-closure-plan-v1.md` 成为 M5 现行实施计划。真实 `stage-14` 已建立，唯一 current leaf 为 REC-00（事实恢复门），`authorization.json` 为精确 closed 两字段。
- M5 候选 WIP 与迁移/Harness Lite 0.8/架构文档混合仍在本机 uncommitted 工作树（HEAD 仍为 main@9103c3b，index 相对 HEAD 无 staged delta）；R0 恢复载体（tar + SHA-256 manifest + 分层归责）在 REC-00 中形成。
- M6 保持未激活：须在 M5 独立验收后才另建真实 stage-15。M7–M11、Headless Core、Primary/epoch 继续 `PLANNED / NOT_ACTIVE`。

当前用户已指定 5600X WSL `/home/synadmin/workspace/syn` 为权威工作仓库。D0D01 只证明 `main@9103c3b26b060e854be119a8cedaa856a2a900ce` 与冻结 WIP 的 `SOURCE_BYTES_MATCH`；它不证明依赖安装、产品运行、Headless Core、Primary / Edge、部署或发布。Harness 文档生命周期上 `stage-12` 仍开启，D0C04 / D0C05 保持 unfinished；本轮文档校准 `stage-13` 已完成并归档，当前无 current leaf，`authorization.json` 保持 closed。

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
- Secretary 应用服务基于持久上下文提供确定性 brief、只读查询、模型增强 ledger 和 M3 Handoff 状态处理；普通产品的 M6 recipient 仍显式 `UNAVAILABLE`，不伪造全局主管成功结果。
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

- M5 ProjectSummary 合同和 owner 尚未激活，M4 项目摘要来源保持 `HOLD / UNAVAILABLE`；不得据此声称“全部用户相关 open loops”已经接入。
- M6 Global Supervisor 成功 consult 未实现；M4 只持 M3 Handoff 请求/回执边界，普通产品 recipient 显式 unavailable。
- M7 对 `DailyWindowClosed` / `DailyReportVersioned` 的消费、正式记忆、PersonalFact、个人模型与 Skill 未实现；M4 只产出 source-backed event/ref，不写 M7 对象。
- M8 真实 connector、credential 与外部 source 未进入；M9 旧路 command unregister/物理退役、M10 全日真实试点与发布硬化、M11 受治理自升级也未进入。
- 真实个人资料、真实用户项目写入、真实模型/provider、真实 Codex 消息、真实账号/凭据/connector、网络外部写入、真实数据迁移、push、merge、rebase、部署和发布均未进入。
- M4R07 v2 只证明本机隔离普通产品的后端/产品链；第 8 次 UI / Computer Use / PNG / attestation 为 `NOT_EXECUTED / NOT_APPLICABLE`，没有视觉验收结论。它也不证明长期运行、真实日常节奏、真实数据/provider/connector、发布包或生产结论。
- 5600X WSL 已成为用户指定的权威源码工作仓库，D0D01 有 `SOURCE_BYTES_MATCH` 证据；长期 SSH 当前启动周期与重启后稳定性仍分别由 D0C04 / D0C05 结算，不能从当前可连接反推 `PERSISTENT_SSH_RESTART_STABLE`。
- 2026-08-16 已完成 DSH 官方研究、Syn 原生治理核心 / 可替换 Agent Runtime / 受治理自升级方向决定、架构与 M5–M11 计划校准；这些是文档和计划事实，不是产品代码、DSH 集成或自升级实现。

## 当前开发状态与停止点

M4 / stage-06 已程序性关闭；M4R01–M4R07 已完成并归档，M4R07 v2 产品链证据为 PASS，`stage-07` 已关闭。WSL 迁移 `stage-12` 仍开着但没有 current leaf，D0C04 / D0C05 不因本轮文档收口恢复；文档校准 `stage-13` 已归档。`USER-SYN-M4-AUTONOMOUS-STAGE-06-20260810` 只作为历史授权记录，不延续到 M5 或其他工作。

后续任何实现都需要新的当前用户指令和匹配的 Harness 工作投影。若需要真实数据 / 模型 / provider / connector / 设备写 / 发布，或要进入 M5–M11，必须停在该事实，不扩大本次 M4R07 v2、D0D01 或文档校准结论。

## 保全

- 5600X WSL `/home/synadmin/workspace/syn` 是当前权威工作仓库；其中既有 dirty WIP 继续保全，不 reset、clean、stash、覆盖或归责。
- `/Users/yoyi/workspace/product-line-syn-fnd-002` 与 `/Users/yoyi/workspace/product-line-syn-m2-closeout` 只作为旧 Mac 审查锚点保留，不再作为当前主线写入位置。
