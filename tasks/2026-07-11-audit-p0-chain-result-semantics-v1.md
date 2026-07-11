# 实现任务包:审计 P0——链结果四分+待决定脸+同击续跑 · 总指导 → 执行线 v1(可派)

日期:2026-07-11 · 性质:**轻档**(结果语义+前端脸;人闸 0-diff;同击续跑按重档决策的限定执行)。上承:审计报告 `docs/2026-07-11-state-machine-failure-path-audit-report-v1.md` §8 P0×2+P0.5(总指导抽样核过·三处证据属实)+ 用户拍「P0+P1 全修」+ **P0.5 已重档授权** `decisions/2026-07-11-same-click-boundary-review-auto-restore-v1.md`(限定条件=授权正文·超出即违规)。

## 0. 接手须知

- 执行线(后端结果语义+前端脸)。**子线不 commit。** 全程中文。
- **变更辐射面(模板新节·必读)**:改 `stage:"ran"` 的假设=「所有消费 outcome.stage 的地方」——已知消费者:前端翻脸正则(ProjectJiaobanPanel.tsx:1156-1165)、archived 处置人为构造 ran(:1094-1115)、run cache/翻脸缓存、可能的测试 fixture。**grep 全仓 `"ran"` 消费点逐个核**,漏一个=旧语义残留。
- **五态旅程走查(模板新节)**:本变更后——说(不涉及)/批(不涉及)/干(running 照旧)/交货(**只 completed 进「做好了」**)/卡住(failed→卡住脸·stopped→「已停下可接着跑」·waiting_decision→**新「待你决定」脸**)。

## 1. 建什么

### 1.1 链结果四分(P0 主项)
- `run_auto_advance…`(director_agent.rs:4060-4091)返回 stage 按 outcome/链态四分:`completed / interrupted / failed / waiting_decision`(替「ran 一把抓」);
- 前端(:1156-1165 正则)改为:**仅 completed → done 脸**;failed → 卡住脸(死配对·停因人话);stopped/interrupted →「已停下·可接着跑」;waiting_decision → 1.2 的新脸;
- 「永不冻」不许倒退:每个新脸都有可行动按钮(卡住脸死配对现成·停下脸=[接着跑]+[重新出方案]);
- archived 处置构造 ran 处(:1094-1115)同步改真实终态语义;done 脸标题只在 completed 时用「✓ 做好了」。

### 1.2 waiting_decision 处置脸(P0)
- 链/节点 waiting_decision → 交办主区「**待你决定**」脸:worker 求助原文上脸(求助内容/权限请求字段现成·07-10 实单有真数据)+ 受控动作:[让它继续(按现状态)] [换个新会话重做] [退回主管重拆] [结束这单]——**不自动重跑**;
- 后端:失败处置命令现只收 failed/needs_rework(:1060-1086)→ 白名单加 waiting_decision(转移合法性沿 workflow_read_model_entrypoints.rs:1491-1504 现成表·不旁路);
- 画布节点 waiting_decision 显「待你决定」不再回落「等待」(与 P1 包画布补态衔接·本包先做此一态)。

### 1.25 worker 工具箱事实注入(热修·案发追加 07-11)
- **实案**:主管拆任务连续两单臆造工具名(指定 codex 不存在的 `read_file`)并禁 shell(codex 唯一读手段)→ worker 被双重死约束锁死·「按约束停止」两次(第二次 worker 无辜);
- **修**:拆任务/重拆注入模板加「worker 工具箱事实」段——worker=codex exec·唯一工具=shell·读文件用 cat/ls/sed·**只读由沙箱保证(任务文本不许禁 shell·不许指定不存在的工具名·不许「仅限注入原文」类禁读约束)**;主管产出任务后确定性 lint:任务文本含「不得运行 shell」或引用未知工具名 → 打回重拆(与角色钳位同型的钳位);
- 验收:两单实案原文做负样本(lint 拦截断言)+ 注入段渲染断言。

### 1.3 同击续跑(P0.5·按决策限定)
- 合流 post_confirm 内边界批准步瞬时失败(锁类)→ **同击内**调用现成恢复器(`restore_pending_global_boundary_review_after_confirm` 安全筛选复用:3527-3607),幂等补记后继续推进;
- review summary 写「同一次[允许并开始]内自动补记」(决策条款 5);补记失败 → 卡住脸(条款 4);**决策六条限定逐条实现并各配一条单测**(超出条件不补=负测试)。

## 2. 安全死线

- 人闸 0-diff([允许并开始]语义不变);审批规则除 P0.5 决策限定内的补记外零改;冻结核/runner/h5/包1-4 已收资产零碰;死配对不倒退(零按钮=违规)。

## 3. 验收

- 单测:四分结果各一条(completed/interrupted/failed/waiting_decision)+ archived 真终态 + P0.5 六限定(五正一负起步)+ 处置命令收 waiting_decision;
- 前端离线:四脸断言(failed 不再出「做好了」标题=**负断言必须有**)+ 待决定脸四按钮+求助原文上脸;
- `grep '"ran"'` 生产码零残留(或仅兼容读取·列清单);双测全量(基线 784/0/43·+N 不降);
- 真机:用户造一次失败/求助看新脸(待真机口径)。

## 4. 回交

四分落点+消费点清单(辐射面核对结果)+ §3 证据 → 总指导核实物。**子线不 commit。**

## 5. 不接受为

- stage 语义半改(留 ran 兼容分支不清)/ failed 仍进 done 脸 / 待决定脸自动重跑 / P0.5 超决策限定(跨点击/跨方案/不幂等)/ 碰人闸或包1-4 资产 / 零按钮脸。
