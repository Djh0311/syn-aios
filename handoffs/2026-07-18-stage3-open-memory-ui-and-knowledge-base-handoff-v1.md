# 会话交接:阶段三在途·记忆中心 UI+知识库两件新任务(2026-07-18)

> 接棒人=新一代**总指导**(主导线)。读序:`CURRENT.md` → 本文 → 总执行计划 `docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md`(防跑偏总则 7 条执行任何包前重读)。规则正本 `AGENTS.md`;主导线=唯一 commit 权(问一次)、核实物不信自报、对接用户;执行线吃包不 commit。

## 一、本会话干了什么(2026-07-16 深夜→07-18,按大块)

1. **接手清桌**(07-16):真单病历二三四号入账(retry 无新信息同墙重撞/实质完成无判过出口/复核实证闸首秀=机械拒空证据 pass);重 seed 恢复 DB 主写(22:45 对账绿·观察期重计;`evidence/2026-07-16-reseed-db-primary-restored-v2.md`);P1-0/0b 驱动选型定案(主 `codex mcp-server`+A5 转交唤醒+备胎 shell resume;探针曾困执行线沙箱断网=环境根因教训)。
2. **阶段一六包全收**(07-17/18,commit `deae9ce`→`13edc99`):P1-A 主管常驻会话(私有家 MCP 白名单+threadId 续接+换代事实注入;核复拦截过 982/0 虚绿=6 败 M5 serde 回归,打回修复)/P1-B 问答接线(schema 二选一+`submit_supervisor_resident_answer`=user_reply 唯一来源+同 thread 注入续跑)/P1-C 中栏对话 UI(**四轮实物迭代**:纯对话流+唯一常驻输入框 Enter 发送+方案交货卡挂右区四视图+⚠牌退场;「方案卡=消息」纸面设计被用户实物否定=感受件必先过用户眼)/P1-D 人闸收敛(绑定停点摘除+派发审计改真话+批态四旧件退场)/P1-E 旧路退役+修宪(非测试项目诚实关门一句人话+塞纸条路退+修宪 1 号 `decisions/2026-07-18-interaction-canon-amendment-1-…`+删测预登记制)。
3. **新链真单两跑**(07-18 额度恢复后):首单 40 分钟/4 卡点撞三病(方案工具口径两皮死循环=总指导当场源头修 `allowed_tools` 改真实 shell 口径/worker 完工报成求助/幂等撞自产物);P2-A 后复跑**约 3 分钟/1 卡点**(说→带图方案秒出→批→零停点→worker→终标真跑)。
4. **P2-A 收口**(commit `51615be`):方案自带任务图(schema 九改点+lint 前移方案期=批的就是已质检的图+三处 None 清零+旧方案 fallback);批后零 LM 拆任务 E2E 实证。
5. **阶段三开工**(用户开工令):总纲=用户诊断**「病根=工作台交流方式太死板」**(现实比格子多,每个格子外情况=死结;三次「已达成无出口」实证);权限对蓝图五根柱(P3 勘察已摘录:worker 永不直通用户/主管终标裁量蓝图本来就给但必须查证据/扩权求助必升级用户/LM 不得直推状态/聊天黑板不是事实源)。**P3-A 施工中**(`tasks/2026-07-18-p3-a-chain-events-into-conversation-package-v1.md`;交接落笔时工作树已见执行线半成品代码[`WORKFLOW_CHAIN_EVENT_SOURCE_KIND` 等]——**接手时工作树的 P3-A 未提交改动=执行线领地,commit 显式列表纪律防误捞**;回传后按 10 项模板核复,基线见 §四)。

## 二、交接任务两件(用户 07-18 点名,与 P3 并行)

1. **记忆中心 UI=无待修,只欠一次真机验收**(07-18 用户纠正+翻档定案):UI=「B1 双栏定稿版」(`877d54e`)+「批2」打磨(`7b261c9`),用户认可形态;**「07-14 没法看」当天深夜就被批2 治了**(三 P0:详情死绑首项致其余记录永久点不开=根因/八处硬截断/假数 Badge)——CURRENT 里「被没法看挡住」是修复后没刷掉的残句(已刷,07-18)。**真正待办=记忆环真机验收①走一遍**(交货→[属实,沉淀]→inbox→采纳→召回,随 P3 真单自然做)+用户真机顺眼确认一下记忆中心。⚠三重教训入账:愿景稿「记忆层页」与产品「记忆中心」同名不同物勿混;CURRENT 单句也可能过期,报状态前 git+archive 二次核;交接=下一代世界观,写错=按错任务开工。
2. **知识库第一片**(用户 07-18 拍板,红灯解除):**独立 vault**(工作台自管新目录,不碰用户现有 Obsidian 库)+**浏览+用户手编**为主+**AI 写入=用户允许那一下即可写**(授权机制随片带,不做自动沉淀)+**md 渲染+编辑+[[双链]]跳转**(反链/图谱后置)。正本纪律:`docs/plans/2026-07-14-post-m5-stage-plan-v2.md` L3「设计谈话先行」(已谈,形态已拍)/`docs/memory-layer-consolidated-canon-v1.md` M8(Obsidian 兼容接口 placeholder+边界;§18.1 记忆 Markdown 展示页≠知识库笔记,别混)/总执行计划 §三 归位表红灯行(随收口更新)。**AI 写入面属 fs 写入=对齐 AGENTS 高危口径设计授权闸**(用户原话:「我允许 ai 写就可以写」=闸形态·非常开)。写包前照例勘察(知识库既有 placeholder/前端页面骨架/写入闸设计)。

## 三、桌面挂账全集(接手对账单)

- **P3-A 在途**(状态未确认,见 §一.5);P3 切包序=A→C→B(勘察拍,`p3-dataflow-recon.md` 在 scratchpad,**scratchpad 会话隔离——关键结论已入包文件,勘察笔记若丢按包内坐标重勘**);P3-B/C 包待写(勘察结论:`follow_up` 管道现成/worker 会话可 resume/黑板派生 filter 扩容=主体施工面)。
- **P2-B** 挑会话归置待议(P2-A 后语义塌缩,勘察判真后端缝)。
- **小尾巴**:P1-E 非测试项目关门脸用户未过目(不烧额度);P1-D 走查点3(常驻框改方案)/4(只读单)随后续真单顺验;病历五号=store 残锁无自愈(catch-log 07-18,exec-registry 孤儿收割先例可抄,小包待排)。
- **push**:`51615be` 及之前全部已推(origin=`Djh0311/syn-aios` 私有);**每次 push=用户明确说「push」那一下**(高危#5,classifier 也拦)。
- **DB 观察期**:07-16 22:45 重计,M6(停写 JSON)=观察期攒够+用户另授权;audit_ledger 读源切换连带。
- **额度**:07-18 恢复;曾耗尽两次(worker/终标断供的失败单先定位挂在哪一环再下结论——总指导误判前科在案)。
- **memories 渗出**:观察 a 维持;**MCP 插件面用户 07-18 拍定:codex 带真家私人 MCP=正常不当洞治**(worker 不做白名单;曾因 firecrawl token 过期崩过 worker=用户判额度问题误报,实际那次是额度)。

## 四、总指导闸清单(每次收口亲跑;数字=当前基线)

- cargo 全量 **995/0/44**(P2-A 后;删测走预登记制=回传先列名单+新基线);m5b/m5c 定向(动持久化/审计面必跑);`npm run typecheck`=0;离线交互套件全过(runner 手工登记制);**shape gate 固化命令单独一条 Bash**:`cd /Users/yoyi/workspace/product-line && node scripts/harness/workbench-shape-gate.js`,数 `[error]`=**13** 基线零净增(**ledger M-2026-07-17 三犯在案:凡 gate 数字异常先查 cwd;禁与 cd 过目录的命令串联**);`cargo fmt --check` 仅历史三文件。
- commit:显式文件列表(禁 `git add -A`)、消息必含 `catch:`(hook 强制)、**CURRENT 回写同笔**、问用户一次。
- 行数水线:types.rs 5386=贴线(P2-A 压过)/Panel 1846/launcher 2977/director 8078。serde 前科双件(`default`+`skip_serializing_if`)两案在档。

## 五、操作知识(免摸索)

- live store:`~/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`;**audit_events 双时间字段坑:按 `created_at_ms` 排序会漏最新(部分为 null),权威=`created_at` 字符串毫秒,取尾部用数组顺序 `.[-N:]`**;SQLite `production-db/workbench-state.v1.sqlite`(sqlite3 -readonly)。
- 残锁处置先例:`.workflow-state.v0.lock` 持有者 pid 死亡→核死后可手工删(07-18 案)。
- 前端改动用户看不见≠没生效:WKWebView 热更不可靠,让用户跑 `open-codex-workbench.command` 重启。
- 执行线=一次性注入 codex exec(tier-1 不自己读项目,给料注入包里;但近期执行线实际能读文件、还会自跑对抗核验=能力比交接旧述强,按回传核实物即可)。
- 真跑测试(P1-A/B 的 ignored)可总指导亲跑:`SYN_P1_A_RESIDENT_WORKBENCH_EXECUTABLE=<src-tauri>/target/debug/codex-governance-workbench cargo test --lib <name> -- --ignored --nocapture`(先 cargo build 让二进制含最新代码)。

## 六、用户风格与红线(本会话反复生效)

- **大白话**;选择题一次给全并**每个给推荐**,他说「按推荐」/单字母就落;不懂会问「什么情况/解释一下」——用比方重讲,别堆术语。
- **不讨好**:先泼冷水、错了当场撤回(本会话撤回过「走查点已实证」误判);「你觉得要做 X 吗」=要真实判断含反对(人话层案:用户拍了我仍给出「不做大层」的反对+替代,用户接受)。
- **感受件铁律**:形态类东西先给用户看(图/截图/文字描述)再施工;改完截图先过总指导看形,再请用户真机——用户只做最后一眼。
- **解释≠授权**:明确「开/批/可以/push」才动;commit 问一次;push 单独问。
- 防跑偏总则第 1 条对总指导同样生效:不新增确认点/提示牌;人闸三下之外零新闸。
- 高危 5 条(AGENTS §一)/S1 三支/guard/`final_mark` 复核实证闸(裁量设计的对手方:主管说「够了」也要有证据留痕)零碰;非测试项目=诚实关门态(P1-E)。

## 七、下一步建议序(接手后)

1. 核 P3-A 状态(发没发/跑没跑)→回传则核复收口;
2. 记忆中心 UI:出三候选形态给用户拍(先看后做);
3. 知识库第一片:勘察→包(形态已拍,见 §二.2);
4. P3-C/B 按勘察序跟上;真单三数随每单复测,及格线=小单 1-2 句/点头≤3/分钟级/零死卡(最近实测 3 分钟/1 卡点,差距在 P3 正治)。
