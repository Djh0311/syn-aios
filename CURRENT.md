# Current Authority（精简版 v3 · 2026-07-08）

> **本文是唯一「每次工作完必更」的活正本**（四块：能用 / 在做 / 下一步 / 锁着）。per-task 状态以本文为准；排布 = `docs/plans/2026-06-27-complete-workbench-phased-roadmap-v1.md`（只在阶段切换时动）；**能力细节正本 = `docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md`（产品现状说明书·逐面盘点·每阶段收口刷）**。规则见 `AGENTS.md`。完整历史：`archive/2026-07-08-current-full-history-before-slim-v3.md`（v2 全史含 Phase A/B 逐包证据流水）+ git。

## 一、现在真能用什么（验过的·细节见现状说明书）

- **交办全环（Phase A·真机验收）**：说 → 批（授权卡+批前边界意见+条件式工序图·所批即所跑）→ 干（逐格亮）→ 交货（口供上脸+黄牌+结果复核意见+[属实,沉淀]）/ 卡住（人话停因+死配对按钮·永不冻）；自动面：残料三层自愈 / 角色钳位 / flaky retry / 供给类失败人话 / 纯建议方案双保险（警条+开工口守卫）/ **超时自动打回主管重拆一次** / **重拆带「本单已完成」事实块+项目记忆**；**工作历史左栏**（按单列史九态含「批了没跑」·行内接着跑·旧单详情卡）。
- **五角色（Phase B·真机验收）**：咨询 / 主管 / worker + **全局主管两钩点**（批前边界+跑后复核·advisory·意见不是闸·词表禁「审批」）+ **秘书**（右栏摘要[与通知待办同开法] + 看板专门界面[四泳道只读] + 按需 AI 解释·零写入）。
- **记忆环**：治理钩子攒候选 → [属实,沉淀] → 确认转正 → 召回 top5 进咨询/预拆/重拆 prompt（主管 prompt 已渲染）。
- **画布 / 智能体页 / relay**：多工作流编排+全屏 HUD+任务级节点与依赖边（跑到哪亮到哪）；会话中心（虚拟化/分组/subagent 过滤/`focusedThreadId` 跳入）；manual relay 真发（唯一手动指挥通道）。
- **工程底**：`cargo test --lib` = **723/0/42**（2026-07-08）·三闸绿；测试基建：manual_relay 12-failed 级联已根治（首发抽风未定位·等自曝）；**异地备份代码半边**（私有 remote `Djh0311/syn-aios`·9 分支·恢复演练过——记忆库/线上 store 仍零副本·知情挂账）。

## 二、在做什么

- **Phase B2·执行闭环深化 = 当前阶段（2026-07-08 定）**：每任务独立对话、传递全走工作台、worker 求助通道、主管总结+终标。正本：定稿 `decisions/2026-07-08-phase-b2-execution-loop-final-v1.md` + 拆解 `docs/plans/2026-07-08-next-stage-execution-loop-breakdown-v1.md`（v1.1·两轮权威文档通读拆定）。切片：**C0 = done·核过（调研报告 `docs/research/2026-07-08-agent-collab-transfer-reference-for-b2-v1.md` 转正为 B2 设计参考正本——五发现：契约严格度待拍/codex `multi_agent`+`memories` 默认开启/求助字段三条并行互不相通[恒空·启发式·独立bool·三处亲验实锤]/C4 缺口确认[链自动completed·failed 只有结束一条路]/M4 架构性不能接）→ 主导线七拍落 `decisions/2026-07-08-b2-transfer-protocol-gap-final-v1.md`（求助=强信号不可软着陆·收敛实现A为唯一真源·双特性入不吸收+C1 运行时防护硬闸·口供摘要独立通道·C2 首务命名统一·C4 四选一·C5 词表对齐）** → **C1 = 首轮核过·收尾轮待派 `tasks/2026-07-08-phase-b2-c1-session-per-task-v1.md`（v1.2）**：核心已落地机器验过（每任务先生后绑建专属会话·失败即停不回落·target_session_id 物化·直起链切 C1·3 单测绿·725/0/42·死线全 0-diff 主导线亲验）；**收尾轮三项待做（包 §8）**：链级 3× 集成测 / 真跑耗时实数 / auto_advance 接 C1（现直起链走 C1、auto_advance 仍拐杖·两生产路径不一致不能过夜到 C2）。**memories 处置 = v1.2 观察模式（07-09 用户终拍）**：注入实锤但实害零[07-07 晚 21:38 起 7/98 会话·+3346 tok/次·三面零渗出·任务间搬运未成品·multi_agent 不自发子 agent]→ 暂不加旗·runner 保全 0-diff·先跑观察·工作台自建记忆开关记为将来候选；known-gap 不吹全隔离（会话级隔离·记忆层跨会话仍通）——见七拍修订记录。→ C2 任务包 v2 → C3 求助通道（含 dispatch cancelled 终态）→ C4 主管总结终标 → C5 上脸审计 → **C6 运行错误人话上脸（07-09 用户「顺手做」插入·包已起草待派）**。schema 正本 = `docs/workflow-task-package-design-v1.md` §3/§4/§5（C0 只出差量）；canon 演化已记：会话跟**任务**走（车间模型旧句修订·见定稿）。

## 三、下一步

1. **C1 收尾轮派出**（包 §8：链级集成测+真跑耗时+auto_advance 接 C1）→ 回交核实物 → C2 起逐片（每片真机过再下一片）。
2. **等用户真机顺手**：工作历史栏/秘书看板计数/拉窄不挤 三点；秘书两入口对调+归队右栏一眼；记忆转正加餐（转正一条→出方案见「带上 N 条」）。
3. **搁置待点火**：整台工作台原型（用户想清楚再做；资产留存：现状说明书/v1 已认元素/颗粒度四拍）｜**运行错误上脸(A=B2 尾片 C6·反馈必人话)+ 开发者工具(B·方式未定)**：提案 `docs/plans/2026-07-09-run-error-surface-plain-language-proposal-v1.md`；**A 包起草待派 `tasks/2026-07-09-run-error-plain-language-surface-A-v1.md`（两派前决定已闭:落位=B2 C6「顺手做」·fix8=收编成错误族全谱;死线精确重划=冻结核 0-diff·报告层可收编;C1 收尾轮清后派）**；B 待用户定方向。
4. **挂账**：备份剩余小件（3 记忆库+workbench store 146M+`~/.codex` 零副本·用户知情）｜「commit 后顺手 push」写不写进 AGENTS 未答｜principles 引 `tasks/README.md` 文字小口子｜旧线（画布真机对图 / 手感打磨 / A4·C default-safe 待真机 / 会话模型 P3 / 记忆中心布局重做=Phase D）。
5. **盯着的（警报器）**：manual_relay 首发抽风（级联已根治·**07-09 C1 核测头回以单具名失败现身** `manual_relay_gui_direct_running_poll...`·重跑即绿·坐标已定·定点修待排）｜tier-1 输出不稳家族三案（新接 LM 字段必配确定性兜底）｜「死锚默认」家族三前科（碰 `default_workflow_*` 一律警觉）｜prepared 148 条已裁认账（正解并入 B2）｜**codex 特性会一夜自开**（memories 07-07 晚 21:38 自启前科——观察模式下每切片收口重跑渗出三查+池内工作台条目计数,codex 升级后同步复核）。

## 四、锁着的 / 没接

- **a) 故意锁**：旧四角色机器 = blocked stub（真实现 07-08 已删·保签名保闸）；真跑**非测试真实项目**（高危#1·Phase E·用户授权那一下不可省）。
- **b) 已建+拍板暂不翻闸**：R3 真库切换（JSON→sqlite·机制全建演过·线上仍 JSON）；记忆「切 DB + 多 agent 专属门」。
- **c) 没建（按阶段排）**：审查智能体（任务包正本 §4.6 可选位）/ harness 系统 / 成熟模式候选（§26.4）= Phase D 前后；乙·自动连环/多项目 = Phase E（重档）。

---

*阶梯：甲·手动中转 ✅ → 中间·半自动 ✅ → 乙（Phase E）。**本文每次 commit 必回写**（AGENTS §五）。排布=路线图；能力细节=现状说明书；历史=archive+git。*
