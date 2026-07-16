# 会话交接:方向拍板·对话优先改造开局(2026-07-16)

> 接棒人=新一代**总指导**(主导线)。读序:`CURRENT.md`(38 行瘦身版)→ 本文 → 总执行计划。规则正本 `AGENTS.md`;主导线=唯一 commit 权(问一次)、核实物、对接用户;执行线吃包不 commit。

## 一、本会话干了什么(按时间)

1. **存储降级修复包收口**(commit `193c373`):A 勘察判 **B 无修**——live 真相=JSON 领先非 DB 领先(总指导包前提截断误读,ledger `M-2026-07-16:包前提未核原件` 二犯立案);M5 双桥(15:57/22:14)晚于案发写入无法回填;降级留痕事件 fail-safe 只写 JSON=每启必再降,**安全语义不是 bug**;唯一恢复路=包外重 seed。
2. **真单三层修复入库**(commit `bb878d3`):方案开放问题上批卡+主按钮降级「答完问题出新方案」+空单卡住脸 classifyBlocked 2b 支路→replan+人话。(注:这批件按新方向属"止血件",P1-D/P1-E 时退场。)
3. **大事:根因对谈 → 方向拍板**(commit `d5d08ad`,已推):用户裁定病根=规矩太多流程太重、产品不够智能;整场对谈演进=场景对照→角色表(对话面=**项目主管**,非秘书)→单向阀诊断→**MCP=Syn 能力总插座**(用户核实点:mcp/ 模块已存在、主管试点 7 动作含 RequestUserDecision、launcher 往 codex config 写 mcp_servers)→与蓝图逐条对账(=蓝图 §9/§12 的执行)→MCP 承载力(四层合一,resume=发条)。
4. **计划收敛**(同 `d5d08ad`):CURRENT §二/§三 收敛成瘦指针(旧巨型叙事原文迁 `archive/2026-07-16-current-sections-2-3-before-consolidation.md`,警报器全文也在里面);**唯一计划入口=总执行计划**。
5. **P1-0 选型勘察包已写好**:`tasks/2026-07-16-p1-0-codex-drive-mode-probe-package-v1.md`(**未 commit**,随下次收口);开工句已交用户。

## 二、正本硬指针(全部已入库)

- 方向:`docs/plans/2026-07-16-conversation-first-direction-and-execution-plan-v1.md`(场景判据/三阶段/刀法/及格线/§十 蓝图对账/§十一 MCP 承载力)
- **执行:`docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md`(唯一计划入口——防跑偏总则 7 条执行任何包前重读;P1-0..P3-D 拆包;旧待办全量归位表;决策点日历)**
- 决策记录:`decisions/2026-07-16-conversation-first-direction-ratified-v1.md`
- 角色表一句话:你只跟**项目主管**说话(常驻会话·换人成本低);工人后台;全局主管留证;秘书=用户私人助理不进项目;控制核心=制度非人。人闸三下(干吧/对的/记住)。及格线:小单 1-2 句话/点头 ≤3/分钟级/零死卡,每真单落三数进 CURRENT。

## 三、接手第一眼·桌面状态

1. **用户真单(mario cp)已终结**:19:02 用户四选一 retry 重跑任务一→worker 二次索授权原话「未收到新增的明确复制执行授权…立即停止」(授权闸被顾问设计进任务一;乙型回话框 07-15 甲案不通电,「我授权」无通道进链;retry 不携带新信息=同墙重撞);19:23 用户 archive 归档(合法 cancelled 终态)。=单向阀病历二号(catch-log 在案)。**三验收(交货卡/复核实证闸首秀/记忆环①)改由新单白捡**:同项目重新下单、目标句自带授权(模板已给用户:「…我在此明确授权:本单允许且只允许创建 index.agent-copy.html…方案里不要再设授权确认环节…」),批卡核「怎么算做好」无授权闸再确认。**接手先核 live 现况再说话**(读法见 §五)。
2. **重 seed**:用户拍「这单走完就做」,用户在场几分钟;做完观察期重开重计。现况:json_only 自 07-14 21:55,数据无损,每启再降属设计。
3. **P1-0**:开工句可能已发/未发执行线。回传后按 10 项模板核(第 7 项 shape gate 三数**原文**,基线 13/5/5 仓根跑;含糊报回按假动作禁令打回;不信自报,闸亲跑)。核复后拍驱动方式 → 写 **P1-A 常驻主管会话包**(写包前必读透 supervisor_session_launcher.rs / mcp/supervisor_orchestrator.rs / consult 数据流——「先读数据流」纪律,ledger 二犯在案;运行时 reason 引用必须整段原文,截断串不得进包题/前提/红线)。
4. **未提交**:仅 `tasks/2026-07-16-p1-0-...-package-v1.md`(+若干历史 untracked 研究稿,一直不 commit)。

## 四、总指导闸清单(每次收口亲跑)

- `npm run typecheck`(shell 目录)=0;`npm run test:offline-interaction`=24 套件全过;`node scripts/harness/workbench-shape-gate.js` 仓根跑,数 `[error]/[warn]/[info]`=**13/5/5 基线零净增**(exit=1 属历史债正常);cargo 全量=`cd` 绝对路径同调用+输出落盘+直取 `$?`+读 `test result:` 行,**976 口径**(登记 flaky solo 复跑即准);`cargo fmt --check` 仅历史三文件(codex_db/codex_local_runner/mcp/storage)。
- commit:显式文件列表(共树**禁 `git add -A`**)、消息必含 `catch:`(hook 强制)、**CURRENT 回写同笔**、问用户一次;push 顺手(origin=`Djh0311/syn-aios` 私有)。

## 五、操作知识(免摸索)

- live store 只读窥探:`~/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`(audit_events:event_type/reason/created_at 毫秒串,jq `localtime` 转)+同目录 sidecars(project-proposals/global-supervisor-reviews/plan-authorizations 等);SQLite=`…/CodexGovernanceWorkbench/production-db/workbench-state.v1.sqlite`(**sqlite3 -readonly**)。
- 前端改动用户看不见≠没生效:WKWebView css 热更不可靠,让用户跑 `open-codex-workbench.command` 脚本重启(四缓存窝全清)。
- 实渲量尺铁律(ledger M-2026-07-16):从最外层真组件链入、抄真机栏宽、报告注明入层。
- 执行线是一次性注入的 codex exec(tier-1 不自己读项目)——给料必须注入包里,别假设它会读。

## 六、用户风格与红线(本会话反复生效)

- **大白话场景版**讲方案(公司/劳务市场比方是本会话的通用语);裸结构化文档会锁死注意力,发散是你的义务;决策 ≤5 次/轮,他说"不知道/没偏好"就按推荐落、别再问。
- **不讨好**:先泼冷水、承认边界、错了主动撤回(本会话撤回过「授权凭证/打扰预算」两条立规矩式药方——**不许再立规矩**,智能还给模型)。
- **解释≠授权**:明确"开干/批"才动;写文档=纸面活可先做,commit/开工必问。
- 防跑偏总则第 1 条对你同样生效:**任何包不得新增用户确认点或提示牌**。
- 高危 5 条(AGENTS §一)/执行闸/渗出观察零碰;知识库 fs 面、agent 层、M6=红灯照旧。

## 七、观察项与警报器

memories 渗出观察模式(用户拍 a 维持;下轮真实派发后立即复巡②面;后手 b=`--disable memories`);flaky 家族 solo 复跑即准;其余全文见 archive 同文件(CURRENT §三.5 有压缩清单)。
