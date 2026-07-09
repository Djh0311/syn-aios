# Current Authority（精简版 v3 · 2026-07-08）

> **本文是唯一「每次工作完必更」的活正本**（四块：能用 / 在做 / 下一步 / 锁着）。per-task 状态以本文为准；排布 = `docs/plans/2026-06-27-complete-workbench-phased-roadmap-v1.md`（只在阶段切换时动）；**能力细节正本 = `docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md`（产品现状说明书·逐面盘点·每阶段收口刷）**。规则见 `AGENTS.md`。完整历史：`archive/2026-07-08-current-full-history-before-slim-v3.md`（v2 全史含 Phase A/B 逐包证据流水）+ git。

## 一、现在真能用什么（验过的·细节见现状说明书）

- **交办全环（Phase A·真机验收）**：说 → 批（授权卡+批前边界意见+条件式工序图·所批即所跑）→ 干（逐格亮）→ 交货（口供上脸+黄牌+结果复核意见+[属实,沉淀]）/ 卡住（人话停因+死配对按钮·永不冻）；自动面：残料三层自愈 / 角色钳位 / flaky retry / 供给类失败人话 / 纯建议方案双保险（警条+开工口守卫）/ **超时自动打回主管重拆一次** / **重拆带「本单已完成」事实块+项目记忆**；**工作历史左栏**（按单列史九态含「批了没跑」·只读浏览+旧单详情卡；[接着跑]在**卡住脸**不在历史栏——主导线 07-09 误述已纠）。
- **五角色（Phase B·真机验收）**：咨询 / 主管 / worker + **全局主管两钩点**（批前边界+跑后复核·advisory·意见不是闸·词表禁「审批」）+ **秘书**（右栏摘要[与通知待办同开法] + 看板专门界面[四泳道只读] + 按需 AI 解释·零写入）。
- **记忆环**：治理钩子攒候选 → [属实,沉淀] → 确认转正 → 召回 top5 进咨询/预拆/重拆 prompt（主管 prompt 已渲染）。
- **画布 / 智能体页 / relay**：多工作流编排+全屏 HUD+任务级节点与依赖边（跑到哪亮到哪）；会话中心（虚拟化/分组/subagent 过滤/`focusedThreadId` 跳入）；**智能体页会话列表并显工作台绑定会话**（A·07-09·has_user_event=0 的 exec 任务会话经 store 绑定捞出+「工作台任务」徽标·机器核过·待真机一眼）；manual relay 真发（唯一手动指挥通道）。
- **工程底**：`cargo test --lib` = **731/0/43**（2026-07-09）·三闸绿；测试基建：manual_relay 12-failed 级联已根治（首发抽风未定位·等自曝）；**fmt 核法**:验 fmt 用**权威 `cargo fmt --check`**（非 ad-hoc `rustfmt --edition 2021`——后者设置不符项目 rustfmt 配置·会误报 commands.rs/测试文件几百行假阳性·我先前「commands.rs 35 块」即此误报·已纠）；真历史漂移只在 `codex_db.rs`(9)/`codex_local_runner.rs`(4)/`mcp/storage.rs`(1)·全量 rustfmt 会造无关 diff 故不动；**异地备份代码半边**（私有 remote `Djh0311/syn-aios`·9 分支·恢复演练过——记忆库/线上 store 仍零副本·知情挂账）。

## 二、在做什么

- **Phase B2·执行闭环深化 = 当前阶段（2026-07-08 定）**：每任务独立对话、传递全走工作台、worker 求助通道、主管总结+终标。正本：定稿 `decisions/2026-07-08-phase-b2-execution-loop-final-v1.md` + 拆解 `docs/plans/2026-07-08-next-stage-execution-loop-breakdown-v1.md`（v1.1·两轮权威文档通读拆定）。切片：**C0 = done·核过（调研报告 `docs/research/2026-07-08-agent-collab-transfer-reference-for-b2-v1.md` 转正为 B2 设计参考正本——五发现：契约严格度待拍/codex `multi_agent`+`memories` 默认开启/求助字段三条并行互不相通[恒空·启发式·独立bool·三处亲验实锤]/C4 缺口确认[链自动completed·failed 只有结束一条路]/M4 架构性不能接）→ 主导线七拍落 `decisions/2026-07-08-b2-transfer-protocol-gap-final-v1.md`（求助=强信号不可软着陆·收敛实现A为唯一真源·双特性入不吸收+C1 运行时防护硬闸·口供摘要独立通道·C2 首务命名统一·C4 四选一·C5 词表对齐）** → **C1 = 全闭·核过（会话跟任务走·四路统一·2026-07-09）**：直起链 / 独立[接着跑] / 合流-new 三条自动路 = 每任务先生后绑建专属会话+失败即停不回落+target_session_id 物化；existing 手动挡保留。经四包落地（首轮+收尾轮①②链级集成测·[接着跑]mode-aware·prepare C1-aware 架构收官），主导线逐包核过。关键架构（收官包）：**prepare 加 `chain_binds_per_task` mode**——C1 路跳 needs_binding 产 prepared·thread 延迟由链每任务 create_and_bind 补·**派发读活节点绑定不读 deferred null·两道兜底防空会话·安全闸[path-lock/沙箱/审批]全 0-diff**；副产品合流-new 退 S0 变干净。727/0/43·fmt 净。**协作账（值得记）**：主导线拦下执行线假报 fmt×2、认账 S0 包设计错（没吃透 prepare 就下红线·measure-before-guessing）→ 用户三选拍 3；执行线纠主导线 6565/6667 过度指令×1、两次停手报回不一锅端。正本：C1 定稿 v1.2 + canon 决策 `decisions/2026-07-09-session-mode-drives-per-task-creation-v1.md`（三转全记）。**memories 处置 = v1.2 观察模式（07-09 用户终拍）**：注入实锤但实害零[07-07 晚 21:38 起 7/98 会话·+3346 tok/次·三面零渗出·任务间搬运未成品·multi_agent 不自发子 agent]→ 暂不加旗·runner 保全 0-diff·先跑观察·工作台自建记忆开关记为将来候选；known-gap 不吹全隔离（会话级隔离·记忆层跨会话仍通）——见七拍修订记录。→ C2 任务包 v2 → C3 求助通道（含 dispatch cancelled 终态）→ C4 主管总结终标 → C5 上脸审计 → **C6 运行错误人话上脸（07-09 用户「顺手做」插入·包已起草待派）**。schema 正本 = `docs/workflow-task-package-design-v1.md` §3/§4/§5（C0 只出差量）；canon 演化已记：会话跟**任务**走（车间模型旧句修订·见定稿）。

## 三、下一步

1. **C4b 起（C4a 已核过）= 主管总结→记忆候选**（§4.8+七查⑥⑦:工作流末主管出总结→上货脸主导位+进记忆候选[架构§8.2 候选来源·转正走确认·与现行[属实]确认制零冲突]）。→ 拆包派出 → C4c(failed 四选一) → C5。
   - **C4a（主管七查+终标）= done·核过**：完成分支从「解析即自动 completed」改为主管终标——确定性初筛①-⑤全绿直过 completed(**零 LM**·1749 不调 final_marker)/黄牌走主管 LM 判过(completed)或退回(needs_rework·预算制 attempts 累计·耗尽→waiting_decision)/LM 断供→waiting_decision 保守不蒙混;确定性初筛拿不准全走黄牌(缺报文/status≠done/acceptance≠completed/evidence 空/required_checks 配了无真源/direction_risks 有);C3 求助路 1647 短路早于 1727 完成分支不受影响。744/0/43·2 文件死线全 0-diff·fmt 权威净·主导线亲读六条命根坐实。
   - **C3（worker 求助通道）= 整块 done·核过**：C3a 立真源（契约加 blocked+四求助字段+consume 求助分支→waiting_decision 早 return 不计 completed+疑似求助保守升级+实现A 成唯一真源）；**C3b 收敛**（derive_subagent_reports 改投影 `worker_structured_report_recorded` 真源·多键关联防串源·无真源→空不猜；退役 922 contains 启发式生产零残留；unresolved_direction_risk+unresolved_conflict 死读删净；dispatch cancelled 终态·仅 project_director 可取消）。738/0/43·死线全 0-diff·fmt 权威净·主导线亲读核过。
   - **C3a（worker 求助通道核心）= done·核过**：契约加 blocked 求助路径·WorkerReport 加四求助字段 serde default·consume 加求助分支（`help_signal_from_raw`:可解析走结构化判定[status=blocked/字段]·不可解析才走 17 词表 suspected·**完成路无假阳性**）·链 help→`waiting_decision` 早 return[不计 completed·停后续·主管必看·fallback「suspected_blocked」]·激活 blocked·填真源=实现A 成唯一真源。735/0/43·2 文件·死线全 0-diff·fmt 权威净·完成汇报软着陆逐字未动（主导线亲读三条行为属性坐实）。
   - **C2（命名统一）= done·核过**（objective→task_goal+serde alias·物化三键归正·读方全切·731/0/43·死线 0-diff·fmt 权威闸净）。**allowed_write 隔离记债**:h5_project_dispatch_bridge:44 缺键 fallback→project_root=fail-open→改名前先修·单独一步。
   - **A（工作台会话可见）+ C1 四会话徽标 = 用户真机验过·OK**（只秘书有问题·见挂账）。
2. **秘书入口 = done·用户真机验过（07-09）**：「打开秘书」chip 接看板 + 浮钮撤开看板行为留作桌面宠物占位——用户真机确认 OK。记忆转正加餐（另项·未做）。
3. **搁置待点火**：整台工作台原型（用户想清楚再做；资产留存：现状说明书/v1 已认元素/颗粒度四拍）｜**运行错误上脸(A=B2 尾片 C6·反馈必人话)+ 开发者工具(B·方式未定)**：提案 `docs/plans/2026-07-09-run-error-surface-plain-language-proposal-v1.md`；**A 包起草待派 `tasks/2026-07-09-run-error-plain-language-surface-A-v1.md`（两派前决定已闭:落位=B2 C6「顺手做」·fix8=收编成错误族全谱;死线精确重划=冻结核 0-diff·报告层可收编;C1 收尾轮清后派）**；B 待用户定方向。
4. **挂账**：备份剩余小件（3 记忆库+workbench store 146M+`~/.codex` 零副本·用户知情）｜「commit 后顺手 push」写不写进 AGENTS 未答｜principles 引 `tasks/README.md` 文字小口子｜旧线（画布真机对图 / 手感打磨 / A4·C default-safe 待真机 / 会话模型 P3 / 记忆中心布局重做=Phase D）。
5. **盯着的（警报器）**：manual_relay 首发抽风（级联已根治·**07-09 C1 核测头回以单具名失败现身** `manual_relay_gui_direct_running_poll...`·重跑即绿·坐标已定·定点修待排）｜tier-1 输出不稳家族三案（新接 LM 字段必配确定性兜底）｜「死锚默认」家族三前科（碰 `default_workflow_*` 一律警觉）｜prepared 148 条已裁认账（正解并入 B2）｜**codex 特性会一夜自开**（memories 07-07 晚 21:38 自启前科——观察模式下每切片收口重跑渗出三查+池内工作台条目计数,codex 升级后同步复核）。

## 四、锁着的 / 没接

- **a) 故意锁**：旧四角色机器 = blocked stub（真实现 07-08 已删·保签名保闸）；真跑**非测试真实项目**（高危#1·Phase E·用户授权那一下不可省）。
- **b) 已建+拍板暂不翻闸**：R3 真库切换（JSON→sqlite·机制全建演过·线上仍 JSON）；记忆「切 DB + 多 agent 专属门」。
- **c) 没建（按阶段排）**：审查智能体（任务包正本 §4.6 可选位）/ harness 系统 / 成熟模式候选（§26.4）= Phase D 前后；乙·自动连环/多项目 = Phase E（重档）。

---

*阶梯：甲·手动中转 ✅ → 中间·半自动 ✅ → 乙（Phase E）。**本文每次 commit 必回写**（AGENTS §五）。排布=路线图；能力细节=现状说明书；历史=archive+git。*
