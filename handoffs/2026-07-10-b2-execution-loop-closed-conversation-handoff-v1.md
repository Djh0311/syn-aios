# 对话交接:B2 执行闭环深化收官 · 主导线 → 下一条主导线对话 v1

日期:2026-07-10。**入口正本 = `CURRENT.md`(35 行·已梳)**;本文只补「CURRENT 之外、新对话该知道」的东西。树净·无在飞的活·最近 commit `38efa72`。

## 一句话

**Phase B2·执行闭环深化 = 整个做完(C0-C5 逐片核过·2026-07-10 收口)**,蓝图 §12 项目执行闭环落全。当前无在研阶段,等用户定下一步。

## 接手第一步

1. 读 `CURRENT.md`(四块:①能用②在做③下一步④锁着)——**§三「B2 收官剩两件」= 现在的活**:① 端到端真机走一单(用户在真机过·B2 真验收)② 用户定下一步(C6/整台原型/开新阶段·别催);
2. 读 `AGENTS.md`(轻档纪律·高危 5 条)+ 自动加载的记忆;
3. roadmap `docs/plans/2026-06-27-...roadmap-v1.md`:B2 ✅·Phase C 已标焦点(**但用户没拍开·别自作主张开新阶段**)。

## 这对话干了什么(2026-07-09~10)

B2 全程:C0 调研 → C1 每任务独立会话(含 prepare C1-aware 收官·退 S0 死结重拍) → C2 命名统一 → C3 求助通道 → C4 主管终标+总结候选+failed 四选一 → C5 账本词表。**每片主导线核实物**(重跑测/扫 diff/亲读命根)。外加:防重造两普查包(逮到 `humanize_consult_error`×2 真重复)+ 能力地图 v2 收编为正本。

## 碰过的坑 / 纪律(多数已进记忆·这里点名)

- **执行线 = codex**(用户确认·完整模式·能自己读/grep/写)。kickoff 写 **§0 自包含 + 关键 file:line 当向导**(不是它读不了)。别再瞎贴模型标签(这对话贴错过)。见 [[tier1-codex-exec-no-ondemand-read-inject]](已补两模式区分)。
- **写"加新能力"包前 grep 全仓 + 对照能力地图 v2 概念反查**(`docs/2026-07-09-codebase-capability-map-v2.md`)——防重造轮子(C0 逮过 SubagentReport 两套并行·这对话逮过 humanize×2)。见 [[package-red-lines-need-source-read]]。
- **触及核心数据流(prepare/dispatch/binding)的包·尤其红线·先读透产/消/依赖**——这对话 S0 退役包没读 prepare 就下"退干净"红线→执行线撞 needs_binding 死结·2 包 1 轮。
- **核实物不信自报**:执行线假报过 fmt 净×2;**验 fmt 用权威 `cargo fmt --check`**(别 ad-hoc rustfmt·配置不符会误报几百行·这对话误报过 commands.rs 35 块还写进档案)。见 [[rustfmt-recurses-mod-children-breaks-0diff]]。
- **主导线 commit 显式列文件**(执行线共树·别 add -A 扫它 WIP)。子线不 commit。
- **codex `memories` 观察模式(未了)**:C1 撞出 codex 记忆注入实锤但实害零→不加旗先观察;每切片收口重跑渗出三查(测试项目/store/记忆池)。known-gap:C1 是会话级隔离·记忆层跨会话仍通。

## 挂账(CURRENT §三/④有全的)

allowed_write fail-open(改名前先修)｜manual_relay 首发抽风(反复单具名失败·重跑即绿·B2 后定点修)｜备份剩余小件｜记忆转正加餐。

## 别人的线(不是本交接范围)

untracked 3 份 research(self-evolution / spec-gate-atdd / syn-measurement-layer)= 主导线另一条 session 的自进化研究·未 commit·跟 B2 无关。
