# 当前状态

截至 2026-08-11，M1–M3 已完成各自具名主线范围。M3 内容提交 `fa8e392`，状态为 `COMPLETED / MAINLINE / STAGE-05 CLOSED`。M4C01–M4C10 已进入主线，C09 隔离验收内容提交为 `c823986c`，C10 launcher 回归修复提交为 `9e97120`，`stage-06` 已程序性关闭；但独立总线复核发现五项普通产品 P1，M4 产品状态改为 `CORRECTIVE CLOSURE PLANNED / NOT_ACTIVE`。当前没有活动 stage、leaf 或有效工程授权；M5–M10 继续 `PLANNED / NOT_ACTIVE`。

## 现在分别看哪里

1. 当前用户指令：决定本轮目标和授权；
2. `docs/product/syn-product-canon-v1.md`：决定 Syn 长期是什么；
3. `docs/product/authority-register-v1.md`：决定各类文件当前有什么效力；
4. `docs/workbench-system-architecture-v1.md`：决定现行系统边界；
5. 本文件、源码与新鲜验证：说明已实现事实、未知和证据上限；
6. `AGENTS.md`、`docs/harness/plan.md` 与 `docs/harness/done/2026-08/`：确认 M3/M4 已关闭的阶段和 leaf 归档；历史授权只证明当时施工有依据，不延续为新的实现权限。

验收报告、交接、历史任务、研究和旧决定只按登记状态提供证据或来源，不自行成为产品定义、当前计划或持续授权。

## M4 / stage-06 已进入主线的事实

- M4 实施合同 `docs/contracts/m4-secretary-attention-daily-resolution-v1.md` 已冻结并保持 SHA-256 `4e4d6251d53e1b9b156fb2fd1266d73d6beace38be2086e83e0f05694dec4e51`；M1 四份合同和 M3 实施补充合同均未被 M4 改写。
- 普通产品 `AppState` 已安装后端构造的 Secretary RoleSession、PersonalScope、daily channel 与权限快照；身份不再来自固定项目 cwd、路由或 renderer 自报字段，错误 scope 继续 fail closed。
- M4 自有 SQLite schema/repository/UoW 已持久化 source-first Inbox、OpenLoop、Decision projection、watermark、去重、排序理由、receipt/event/audit/checkpoint；不同 source owner 不合并，未知、敏感、过期或无法精确绑定的输入 quarantine。
- read、dismiss、snooze、acknowledge、close、reopen、carry-over、Notification、Reminder 与显式 standalone `PersonalAction` 已有 CAS、幂等、重启和审计语义。协调状态不反写 owner；OpenLoop、日报或模型解释不会自动创建 Todo。普通产品到期时钟和个人对象入口仍待修正。
- Secretary 应用服务基于持久上下文提供确定性 brief、只读查询、模型增强 ledger 和 M3 Handoff 状态处理；普通产品的 M6 recipient 仍显式 `UNAVAILABLE`，不伪造全局主管成功结果。
- 首页已消费后端 typed read model，展示来源、owner、优先理由、最后变化、状态和 source descriptor，并提供协调动作；专业模块入口保留，React 不拥有协调真值。当前 source link 只到通用项目面，持续 Secretary 输入明确 disabled，产品消息与历史恢复尚未接入。
- Daily scheduler 已实现 OS IANA timezone、本地自然日窗口、最多 7 个窗口 catch-up、同窗幂等、版本纠正、重建和确定性 report；空事件窗口的 agent turn 与 model invocation 均机械证明为 0。该 scheduler 目前没有调用 OpenLoop/Reminder 到期推进。
- 旧 secretary/right rail/runtime attention/pending action/memory daily 五类读面已有 inventory、comparator、compatibility read-only 边界和 quarantine。普通产品目前没有 legacy tuple adapter，因此五类 inventory 全部 fail closed；实际 shadow/parity/fallback 仍待接入。
- C09 使用隔离 profile、两个 synthetic source owner 与 fake model 完成首启、SIGKILL、同 profile 重启、生命周期恢复、日报重跑、deep link、模型失败和零事件验收；证据只到机械层与隔离产品 App，不等于真实日常使用。
- C10 将全部 M4 前端测试挂入 44-entrypoint 离线 runner，并以运行时等价源码修复 C09 与旧 R4/M3 source-string 静态契约碰撞；修复提交为 `9e97120`。

## 2026-08-11 独立总线复核

独立复核确认 Git/Harness 程序性收口成立，M4 底层 repository、状态机、日报、去重和隔离合成证据可继续依赖；但以下五项阻断产品验收：

1. 普通产品没有 M4 内部 source ingress 的生产调用者；C09 直接注入 synthetic source。
2. snoozed OpenLoop 与 Reminder 到期没有生产 scheduler 驱动。
3. source link 只进入通用 Projects 页面，没有精确定位原对象。
4. 首页持续 Secretary 消息输入、M3 Turn 写入与跨重启历史恢复未接入。
5. 五类 legacy compatibility 仍是 inventory-only quarantine，没有实际 adapter parity。

完整事实、证据上限和验收决定见 `docs/harness/reports/M4-independent-bus-review-2026-08-11.md`；当前修正入口为 `docs/plans/2026-08-11-syn-m4-independent-remediation-and-reacceptance-plan-v1.md`。这些是施工漏项，不需要重新拍板 M4 核心产品需求。

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
- M8 真实 connector、credential 与外部 source 未进入；M9 旧路 command unregister/物理退役、M10 全日真实试点与发布硬化也未进入。
- 真实个人资料、真实用户项目写入、真实模型/provider、真实 Codex 消息、真实账号/凭据/connector、网络外部写入、真实数据迁移、push、merge、rebase、部署和发布均未进入。
- C09 只证明本机 debug App + synthetic fixture + fake model + isolated profile；没有长期运行、真实日常节奏、发布包或生产结论。npm 回归使用仓库既有 ignored `node_modules`，不等于 clean install 已通过。

## 当前开发状态与停止点

M4 / stage-06 已程序性关闭，当前工作树停在无活动 stage/leaf 的本地主线。独立修正计划已经建立但尚未激活；拟议下一 Harness 生命周期为新的 `stage-07` 和 `M4R01…M4R07`，不得重开 stage-06。`USER-SYN-M4-AUTONOMOUS-STAGE-06-20260810` 只作为历史授权记录，不延续到修正阶段、M5 或其他工作。

后续任何实现都需要新的当前用户指令、匹配 stage、唯一 leaf 与授权。若复核发现冻结合同冲突、owner/revision 无法精确绑定、协调动作反写 owner、空事件触发模型、敏感正文进入 M4 store、普通模式依赖隔离 gate/fixed cwd，或需要真实数据/模型/provider/connector/远端/发布，必须停在该事实，不扩大本次 M4 结论。

## 保全

- `/Users/yoyi/workspace/product-line-syn-fnd-002` 继续只读保留既有战略开发中工作；它不是当前主线事实。
- `/Users/yoyi/workspace/product-line-syn-m2-closeout` 继续保留 M2 审查锚点。
- 不从这些工作树清理、覆盖、暂存或归责既有改动。
