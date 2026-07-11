# Current Authority（精简版 v3 · 2026-07-08）

> **本文是唯一「每次工作完必更」的活正本**（四块：能用 / 在做 / 下一步 / 锁着）。per-task 状态以本文为准；排布 = 阶段语义 `docs/plans/2026-06-27-complete-workbench-phased-roadmap-v1.md` + **当前排布 `docs/plans/2026-07-11-orchestrator-fast-path-five-stations-plan-v1.md`（五站快车道·07-11 拍）**；**能力细节正本 = `docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md`（产品现状说明书·逐面盘点·每阶段收口刷）**。规则见 `AGENTS.md`。**对话交接（2026-07-10·B2 收官）**：`handoffs/2026-07-10-b2-execution-loop-closed-conversation-handoff-v1.md`（新对话接手先读本文+它+记忆）。完整历史：`archive/2026-07-08-current-full-history-before-slim-v3.md`（v2 全史含 Phase A/B 逐包证据流水）+ git。

## 一、现在真能用什么（验过的·细节见现状说明书）

- **交办全环（Phase A·真机验收）**：说 → 批（授权卡+批前边界意见+条件式工序图·所批即所跑）→ 干（逐格亮）→ 交货（口供上脸+黄牌+结果复核意见+[属实,沉淀]）/ 卡住（人话停因+死配对按钮·永不冻）；自动面：残料三层自愈 / 角色钳位 / flaky retry / 供给类失败人话 / **纯建议=只读单**（07-10 升级:写根空一步批·沙箱锁 read-only 零 --add-dir·交货「只读单·未改文件」·守卫按 :4202 先见注释随分流升级）/ **超时自动打回主管重拆一次** / **重拆带「本单已完成」事实块+项目记忆**；**工作历史左栏**（按单列史九态含「批了没跑」·只读浏览+旧单详情卡；[接着跑]在**卡住脸**不在历史栏——主导线 07-09 误述已纠）。
- **五角色（Phase B·真机验收）**：咨询 / 主管 / worker + **全局主管两钩点**（批前边界+跑后复核·advisory·意见不是闸·词表禁「审批」）+ **秘书**（右栏摘要[与通知待办同开法] + 看板专门界面[四泳道只读·「打开秘书」chip 开·浮钮留作桌宠占位] + 按需 AI 解释·零写入·07-09 真机验过）。
- **执行闭环深化（Phase B2·2026-07-10 收口·机器验过·**同日端到端真机验收过**[说→批→干→口供落账→主管终标打回→[接着跑]复跑→交货·主链全通·用户拍「算过」；抓 4 bug 见 §三.1·浏览器验收能力不可用=worker 无浏览器·老毛病用户判不管]）**：**每任务独立会话**（会话跟任务走·四路统一·prepare C1-aware·派发读活绑定防空会话）→ 任务包 v2（三层命名统一·新字段可配）→ **worker 求助通道**（契约 blocked 求助字段·求助=强信号不软着陆·进 waiting_decision 待主管·实现A 唯一真源）→ **主管七查终标**（确定性初筛全绿零 LM+黄牌 LM 判过/退回预算制/断供保守）+ **主管总结→记忆候选**（一次链·候选态不转正）+ **failed 四选一**（重试/退回/换会话/结束·复用现成机器·重跑走人闸不自动连环）→ **账本词表对齐+entry_type 校验**。人闸没动·死线全 0-diff。
- **运行错误人话上脸（C6·B2 尾片·2026-07-10 机器验过·主导线核实物过）**：worker/codex 跑挂的原始诊断层不再零呈现——新模块 `run_error_translation.rs` 七族全谱分类器（结构化 `{family,human,raw_snippet}`·unknown 保守兜底带原文）收编散在四处的翻译器成**单一真源**（供给类判据搬入·`humanize`×2 消成薄委托·`codex_provider_unavailable:`/`consult_last_message_read_failed:` 两 retry 承重信号不断）；run-history 详情位失败单出「人话摘要+族标+`<details>` 下钻原文」两层脸，替掉「跑挂了（去工作流看详情）」死胡同。冻结核 byte-0-diff·成败判定没动·呈现不阻断（黄牌哲学）。回交 `handoffs/2026-07-09-run-error-plain-language-surface-A-handoff-v1.md`。
- **记忆环**：治理钩子攒候选 → [属实,沉淀] → 确认转正 → 召回 top5 进咨询/预拆/重拆 prompt（主管 prompt 已渲染）。
- **画布 / 智能体页 / relay**：多工作流编排+全屏 HUD+任务级节点与依赖边（跑到哪亮到哪）；会话中心（虚拟化/分组/subagent 过滤/`focusedThreadId` 跳入）；**智能体页会话列表并显工作台绑定会话**（A·07-09·has_user_event=0 的 exec 任务会话经 store 绑定捞出+「工作台任务」徽标·用户真机验过 OK）；manual relay 真发（唯一手动指挥通道）。
- **工程底**：`cargo test --lib` = **809/0/43**（2026-07-11·①.75 收口·+4 收割器测试+1 runner 超时实杀测试）·三闸绿；**exec 僵尸收割已上线**（登记表制·只杀登记且启动时间+命令行双匹配的孤儿·未登记绝不杀·app 启动自动收）；测试基建：manual_relay 12-failed 级联已根治（首发抽风未定位·等自曝）；**fmt 核法**:验 fmt 用**权威 `cargo fmt --check`**（非 ad-hoc `rustfmt --edition 2021`——后者设置不符项目 rustfmt 配置·会误报 commands.rs/测试文件几百行假阳性·我先前「commands.rs 35 块」即此误报·已纠）；真历史漂移只在 `codex_db.rs`(9)/`codex_local_runner.rs`(4)/`mcp/storage.rs`(1)·全量 rustfmt 会造无关 diff 故不动；**异地备份代码半边**（私有 remote `Djh0311/syn-aios`·9 分支·恢复演练过——记忆库/线上 store 仍零副本·知情挂账）。

## 二、在做什么

- **Phase B2·执行闭环深化 = 整个做完·收口 2026-07-10**（C0-C5 逐片主导线核实物·蓝图§12 执行闭环落全·**C6 尾片[运行错误上脸]亦 2026-07-10 收编上脸·核实物过**·人闸没动·全程死线 0-diff·764/0/43）。正本：roadmap（B2 ✅）+ 定稿 `decisions/2026-07-08-phase-b2-execution-loop-final-v1.md` + 七拍 `decisions/2026-07-08-b2-transfer-protocol-gap-final-v1.md` + canon `decisions/2026-07-09-session-mode-drives-per-task-creation-v1.md`（C1 三转全记）+ schema `docs/workflow-task-package-design-v1.md`。canon 演化：会话跟**任务**走。**B2 真机验收 2026-07-10 过（Gate 0 ✅·用户拍「算过」）。方向三拍全落档**：① 不做第二入口·项目内强化（`decisions/2026-07-10-entry-direction-project-internal-no-second-entry-v1.md`）② Jarvis=全局能力非入口 + **五类业务版图采纳**（游戏/Agent/企业/个人/市场·远景定义域非排期·`decisions/2026-07-10-jarvis-global-capability-and-five-domain-vision-v1.md`）。**修包群全清；07-10 晚新一波（真机绑定面板体验后用户定向）**：①纯建议→只读单 = **✅ 已收**（守卫从拒改限·派发层空写根→read-only 收紧·runner argv 案发断言·执行线勘明只读路径不经 h5[fail-open 挂账风险面更小但仍挂]）②交办·画布合一 = **提案三点已拍**（分片 **M2 布局→M1 绑定前置→M3 工序图退役**·工作流 tab 留一版过渡·简单活单节点）`docs/plans/2026-07-10-jiaoban-canvas-merged-page-proposal-v1.md`·**M2/M1/M3+页面清理全 ✅ 已收（07-11·真机过·合一改造收官）**：M1=预演节点上画布+节点选会话+批准打包映射零停点（映射「稳定id→同序→不猜」）·M3=卡内工序图退役+授权卡瘦身·清理七项（「去工作流tab」遗产话术→「看右侧画布」等）。**第二波已定向（用户 07-11 真机反馈）**：①删全部教学/标签文案（清单在包·含「两句并存」漏）②历史栏→悬浮覆盖（已拍）③**上下改左右分栏**（已拍·左交办窄栏右画布大区·相位主区切换机制退役）。**
- **对话线编制（07-10 立·两条·用户拍「以后归属由总指导判断」）**：会话爆了可换代（新会话读本登记+该线最近回交 handoff 接续·线身份不灭）。
  - **总指导**（主导线对话）：统筹·拆包·**派发调度与归属判断**·回交核实物·正本落档·**唯一 commit 权**·对接用户。
  - **执行线**（单条常设·全栈实现域·串行吃包）：**在手 = 空**；**下一件 = 站1·MCP 工具面（包已拆可派 `tasks/2026-07-11-orchestrator-station1-mcp-toolface-v1.md`·⚠️ 勘察=先核硬闸[ledger M-2026-07-11]：单独回交、核复前不得实现）**。还债改支线：manual_relay 小包空窗吃；allowed_write fail-closed 挪站3a 前置。已收库：修包群四包+微件+只读单+M2/M1/M3+页面清理+界面二三四波+**审计 P0/P1（07-11 总指导逐项核过·合批 commit）**。
  - **规则**：串行为默认（共树防写回互踩）；确需并行由总指导按文件级地盘零交集临时开第二线、干完即收；执行线不 commit、不维护本登记。〔研究/设计稿=用户另一 session 产出·候选不动码·不归编制·commit 避开其 untracked 文件〕
- **仍活的观察项·memories 观察模式（07-09 终拍·未了）**：C1 撞出 codex `memories` 注入实锤但**实害零**[07-07 晚 21:38 起·7/98 会话·三面零渗出·任务间搬运未成品·multi_agent 不自发子 agent]→ **不加旗先观察**；**known-gap 不吹全隔离**（C1 是会话级隔离·codex 记忆层跨会话仍通）；**巡检**：每切片收口重跑渗出三查+池内工作台条目计数·codex 升级重跑探针（见七拍修订记录）；工作台自建记忆开关=将来候选。**07-11 复巡（补 C6/修包群/合一欠账·`evidence/2026-07-11-memories-leak-triple-check-rerun-v1.md`）：①测试项目②store 两面净；③池内现工作台条目=判据触发——但全部来自执行线开发会话（cwd=product-line·管发 worker 零渗出·mario 0 命中）；新风险面=工作台内部知识将随 memories 注入未来 worker（破 C1 干净上下文本意）→ 议题按规程回用户重拍，**用户 07-11 拍 a：维持观察**（下轮真派发后立即复巡②面；b[worker argv 加 `--disable memories`]留作后手，②面一出现回声即升级 b）。**

## 三、下一步

1. **主线 = 五站快车道(07-11 拍·排布正本 `docs/plans/2026-07-11-orchestrator-fast-path-five-stations-plan-v1.md`·取代本行旧序列[历史见 git])**:站1 工具面+契约(现在·包已拆 `tasks/2026-07-11-orchestrator-station1-mcp-toolface-v1.md`[含勘察先核硬闸]·契约**已核字**[07-11·第3稿·对齐 codex base 格式·随站2包下发]`docs/plans/2026-07-11-supervisor-contract-v1-draft.md`)→ 站2 主管上岗试点(只读单·测试项目·新旧对照) → 站3 账本达标两扩:3a 写单(前置=allowed_write fail-closed 重档)/ **3b 只读单进真实项目(重档·到站拍·候选=crazytown)** → 站4 Phase D 按痛点拉动 → 站5 Phase E。治理姿势=**两分法**(`decisions/2026-07-11-machine-ruling-dichotomy-v1.md`:确定性越权=拦·LM 意见=留证·否决权在人;契约一页纸死线·防复发条款·双轨开关永久保留)。**重心声明:3b=真正验收(工作台第一次干真活),站1-2 只是最短路径。**
2. **防重造资产（07-09 立·B2 副产）**：能力地图 v2 = 正本 `docs/2026-07-09-codebase-capability-map-v2.md`（**概念→在哪反查索引**·写"加新能力"包前查它定位现有实现；⚠️8 处已判[humanize×2 真重复→**C6 已消**/终标误报已排/余候选待细看]·已知漏概念4·便宜模型索引非权威关键处仍 grep）。**流程补丁**:写"加新能力"包前 grep 全仓+对照 v2（C4c 已用它挡住退回/换会话重造）。真重复 `humanize_consult_error`×2 **已随 C6 收编消成薄委托**（2026-07-10·`strip_prefix("codex_provider_unavailable:")` 单一真源现只在 `run_error_translation.rs`）。
3. **搁置**：整台工作台原型 = **永久暂停（用户 07-11 拍·用户提起才重开）**——合一改造已吸收其主要诉求·实机迭代路线胜出（资产留档:现状说明书/颗粒度四拍）｜**开发者工具 B**（方式未定·C6 提案 §B 占位·`docs/plans/2026-07-09-run-error-surface-plain-language-proposal-v1.md`）。〔C6-A 运行错误上脸已收口 → §一〕
4. **挂账**：**allowed_write fail-open**（h5_project_dispatch_bridge:44 缺键→project_root·改名前先修·单独一步）｜备份剩余小件（3 记忆库+store 146M+`~/.codex` 零副本·知情）｜「commit 后顺手 push」未答｜principles 引 `tasks/README.md` 小口子｜记忆转正加餐（欠着）｜旧线（画布真机对图/手感/会话模型 P3/记忆中心布局重做=Phase D）。
5. **盯着的（警报器）**：manual_relay 首发抽风（反复以单具名失败现身·重跑即绿·**B2 后定点修**）｜tier-1 输出不稳家族（新接 LM 字段必配确定性兜底）｜「死锚默认」家族（碰 `default_workflow_*` 警觉）｜**codex 特性会一夜自开**（memories 前科+**07-11 实锤第二发:reasoning effort 被外部改 ultra**[非用户·oh-my-codex/app 嫌疑·致拆任务假死十几分钟·已调回 xhigh(用户授权)]·观察模式巡检渗出三查）。

## 四、锁着的 / 没接

- **a) 故意锁**：旧四角色机器 = blocked stub（真实现 07-08 已删·保签名保闸）；真跑**非测试真实项目**（高危#1·Phase E·用户授权那一下不可省）。
- **b) 已建+拍板暂不翻闸**：R3 真库切换（JSON→sqlite·机制全建演过·线上仍 JSON）；记忆「切 DB + 多 agent 专属门」。
- **c) 没建（按阶段排）**：审查智能体（任务包正本 §4.6 可选位）/ harness 系统 / 成熟模式候选（§26.4）= Phase D 前后；乙·自动连环/多项目 = Phase E（重档）。

---

*阶梯：甲·手动中转 ✅ → 中间·半自动 ✅ → 乙（Phase E）。**本文每次 commit 必回写**（AGENTS §五）。排布=路线图；能力细节=现状说明书；历史=archive+git。*
