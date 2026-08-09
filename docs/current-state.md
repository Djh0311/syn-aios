# 当前状态

截至 2026-08-10，已提交主线 HEAD 为 `0a6507c8ce2dddee6a0f99b2d806c6e2bdf0b4a1`。M3C01–M3C07 已分别由 `29085cc`、`17933ea`、`0769dc5`、`089d36e`、`b953939`、`3472262`、`0a6507c` 进入主线；唯一活动叶仍是 `stage-05 / M3C08-integration-regression-closeout`。M3C08 主线回归已通过，状态为 `MAINLINE_REGRESSION_PASS / TERMINAL_CLOSEOUT_BY_MAIN_THREAD`：M3C08 的产品 / 内容终态提交、最终 HEAD / clean tree 核对和 Harness stage close 由主线程随后执行，本文件不把它们写成已发生。

## 现在分别看哪里

1. 当前用户指令：决定本轮目标和授权；
2. `docs/product/syn-product-canon-v1.md`：决定 Syn 长期是什么；
3. `docs/product/authority-register-v1.md`：决定各类文件当前有什么效力；
4. `docs/workbench-system-architecture-v1.md`：决定现行系统边界；
5. 本文件、源码与新鲜验证：说明已实现事实、未知和证据上限；
6. `AGENTS.md`、`docs/harness/plan.md`、`docs/harness/stages/stage-05.md`、唯一活动叶及 `docs/harness/authorization.json`：决定本轮如何施工。

验收报告、交接、历史任务、研究和旧决定只按登记状态提供证据或来源，不自行成为产品定义、当前计划或持续授权。

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

- 2026-08-10 的静态核对确认四份 M1 冻结合同仍与 M3C01 frozen inputs 相符：`role-session-v1`、`handoff-v1`、`identity-scope-v1`、`event-audit-outbox-v1`。完整 SHA-256 记录在 `docs/harness/reports/M3C08-mainline-integration-and-acceptance.md`。
- M3C01 冻结的 M3 计划快照是 `9403851ece470c32bac5071e2613495a6f0e525214dbd6990a1cd2d28d1ce013`。该快照并非第四个 M1 合同；在 M3C08 前，现行 M3 计划文件已因状态型回写而为 `d584cc19592095cbeb521b483319ab77b61ecc3276351220ef5b94c0c9dae25c`。M3C08 的状态型回写还会再次改变现行计划文件，不得声称“所有 frozen hash 均相等”。
- 迁移只允许 shadow / provenance / bounded references；无法精确绑定的记录保留为 orphaned 或 ambiguous，不自动分配项目。前端 cache 只可显示，不能升格为真源。
- rollback 只可切换旧 UI / read fallback 或关闭新 M3 read projection；不得移除 M1 thread-owner、scope 或 Station 3b 守卫，不得重放 provider effect，不得恢复跨项目 bypass，也不得删除未解决 orphan。

## 尚未成立或未进入

- M1 四份 frozen contract SHA 和相对 `29085cc` 的 diff 均 exact；Rust `m3c07_` 为 exit 0、11/11 通过（Cargo 4.92s），`m3c0` 为 exit 0、123/123 通过（Cargo 40.29s），`m3c07_ --no-run` 为 exit 0。
- 完整 `--lib` 的受限 sandbox 首跑为 exit 101、1520 通过 / 4 失败 / 45 忽略：3 个失败是 M3C07 launcher source-string 与既有 `acceptance_runtime_profile` helper 的静态契约 collision，不是 runtime 或 sandbox 行为失败；第 4 个是 resident-session test 读取 PID `lstart` 的 EPERM 环境差异。该首跑保留为红灯证据，不从记录中抹去。
- current leaf 已按 stage 范围只扩充 `prototypes/productized-desktop-shell/scripts/run-r4-isolated-app-preflight.mjs`，以运行时等价源码消歧修复 3 个静态 collision；`node --check` 通过、launcher focused 5/5 通过、`m3c07_` 11/11 通过。resident-session exact test 在主机权限环境为 exit 0、1/1 通过、3.27s。最终完整 `--lib` 在主机权限环境为 exit 0、1524 通过 / 0 失败 / 45 忽略、72.83s；仅保留 141 条既有 `unused` / `dead_code` / `private_interfaces` warning，无新增失败。
- 启动器纠偏后主线程再次直接复跑前端 / 构建：`npm run typecheck`、`npm run test:offline-interaction`、`node --check scripts/run-r4-isolated-app-preflight.mjs` 和 `npm run build` 均 exit 0；offline runner 实际遍历 39 个 entrypoint，脚本摘要为 `offline interaction tests passed: 15`；build 转换 306 modules、955ms 完成，只有既有 Vite `>500k` chunk warning。npm 使用既有 ignored `node_modules`，未将曾因缺少 `zustand` 失败的 `npm ci --offline` 写成 clean install 成功。
- 真实 provider、真实 Codex 消息、真实用户项目、真实账号、凭据、外部 connector、部署、发布、merge、push 与真实数据迁移均未进入。
- 桌面证据只覆盖 Agent / Jiaoban synthetic host；窗口截图只保存在 Codex 主任务 `019fe53e-c4c2-7ab0-a965-0e231075df57` 的线程内，仓库持久证据只有 `docs/harness/reports/M3C07-isolated-desktop-evidence/` 的 6 份脱敏 launcher JSON receipt。
- 完整知识检索、同步、记忆治理、connector、Secretary、项目主管和全局主管实现均不属于 M3 已完成范围。

## 当前开发状态与停止点

M4、M5、M6 及之后阶段均为 `PLANNED / NOT_ACTIVE`。M3C08 的主线回归已通过，但本任务不自动激活 M4/M5，也不授予 M4/M5/M6+ 的实现权限。主线程完成终态提交、最终工作树核对和 stage close 后任务队列必须停止；任何下一步均需新的明确用户指令、匹配的活动阶段 / 唯一叶和授权。

## 保全

- `/Users/yoyi/workspace/product-line-syn-fnd-002` 继续只读保留既有战略开发中工作；它不是当前主线事实。
- `/Users/yoyi/workspace/product-line-syn-m2-closeout` 继续保留 M2 审查锚点。
- 不从这些工作树清理、覆盖、暂存或归责既有改动。
