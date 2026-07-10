# 实现任务包:跑后复核输入修正(bug①口供匹配 + bug②按轮过滤)· 主导线 → 执行线 v1(可派)

日期:2026-07-10 · 性质:**轻档**(复核输入构造纯读层·advisory 面·不碰人闸/执行/冻结核)。来源:B2 真机验收(2026-07-10)实单抓获,`CURRENT.md` §三.1 bug①②。

## 00. 主导线勘察记录(派前核实物·坐标为当日磁盘)

- **案发实单**:mario-test 2026-07-10 16:04 轮。复核(16:10:56)报「四任务均未交口供,任务1需返工」;实况=任务1口供**已落账**、任务2/3/4**本轮没跑**。
- **bug① 根因=1 毫秒竞态(铁证)**:停链事件 `created_at=1783671034718`,口供事件 `worker_structured_report_recorded created_at=1783671034719`;`load_review_input` 第 2 步(`global_supervisor_agent.rs:174-210`)按时间窗 `started ≤ created_at ≤ ended` 捞口供,窗尾=停链毫秒 → **口供晚 1ms 被滤掉**。终标不走此窗(直接消费 last-message)故读得到——「终标引用了口供、复核说没口供」全解释。
- **bug② 根因=零轮次过滤(铁证)**:第 3 步(`:211-231`)按前缀 `{wf}:node:task:` **全量**捞任务节点+当前 state 喂 LM;planned-task 节点跨轮复用(created 07-03),task-2/3/4 的 `state=completed` 是 **07-09 旧轮**遗留(`updated_at=07-09 17:0x` 铁证)→ 复核把昨天的状态当本轮结果点评。而第 1 步(:143-173)明明已读**本轮** `run.nodes`(node_id+state)——第 3 步是旧信息污染源。
- **可用匹配键(比时间窗稳)**:口供事件自带 `dispatch_id` / `work_item_id` / `node_id`(实单全字段核过);本轮 run 记录在 `workflow_chain_runs`(精确按 workflow_id+project_id+started_at 找,第 1 步现成)。**执行线先核 `run.nodes` 元素有哪些字段**(chain_nodes 投影只取了 node_id/state·run 里节点是否带 work_item_id/dispatch_id 待你亲核)再定匹配键。
- 复核幂等键=(workflow_id, chain_started_at)(:~695 注释)·复跑轮(17:02 起 started=1783674174155)的口供两条(...4395164/...4564087)按新窗判**均在窗内**——bug① 只在「口供与停链同毫秒竞态」时现形,flaky 型,修按轮匹配根治。

## 0. 接手须知(自包含)

- 你是执行线(纯后端·一个文件为主)。**子线不 commit。** 全程中文。
- 改的是**全局主管跑后复核的输入构造**(`load_review_input`)——advisory 面,意见不是闸;**不碰**终标(director)、不碰链驱动、不碰人闸。
- 词表死线:复核输出词表(pass/needs_rework/needs_human_check)与「意见不是闸·禁『审批』」照旧,本包不动 prompt 契约段语义,只修输入数据。

## 1. 拍板摘要

- **做什么**:① 口供捞取从「时间窗」改「按轮标识匹配」(时间窗最多留作 fallback);② 任务节点态从「全量前缀扫」改「本轮 run.nodes 投影」——复核只看本轮实况。
- **为什么**:复核喂了错输入 → LM 只能如实报错话 → 用户被误导「亲自核验」。B2 真机验收实单撞出,每单都会复发(旧轮污染必现·口供竞态偶现)。
- **不做**:不改复核 prompt 契约/词表;不改终标;不改口供落账侧(worker_report/audit 写入);不加新 audit event_type(C5 词表);不动幂等键。

## 一句话判据

**「是不是只改 `load_review_input`(及其直接辅助)让复核拿到『本轮真实口供 + 本轮真实节点态』,而落账侧/终标/链驱动/人闸/prompt 契约 0-diff?」** 是 → 做;否 → 停、报回。

## 2. 建什么

### 2.1 bug① 口供按轮匹配(替时间窗)

- 首选:**按本轮标识精确匹配**——先亲核 `workflow_chain_runs[].nodes` 元素字段;若带 `work_item_id`(或 dispatch_id),口供事件按 `work_item_id ∈ 本轮集合` 匹配(事件自带该字段·勘察已核);
- 若 run.nodes 无可用标识 → **fallback:窗尾放宽**为 `ended_at + 容差`(容差给 60_000ms·注释写明防同毫秒竞态·并留 TODO 指向标识匹配),**禁止**只把 `<=` 改 `<` 之类的碰运气修;
- 无论哪种:同 workflow 旧轮口供**不得混入**(匹配键必须含轮次语义·纯放宽窗不许把上轮口供放进来——上轮口供 created_at < 本轮 started_at 天然挡住,自证即可)。

### 2.2 bug② 任务节点态按本轮投影

- 第 3 步 `task_nodes` 改为**从本轮 `run.nodes` 投影**(第 1 步已读·复用),不再全量前缀扫全 store;
- 语义约束:本轮没跑到的任务**如实呈现为本轮态**(pending/未派发),不得显示旧轮 completed;prompt 组装处(`build_supervisor_prompt`)如需字段名调整随之最小改;
- 若 run.nodes 与旧 task_nodes 字段形状不同,以「复核只该知道本轮」为准绳取舍,别为兼容把全量扫留成第二真源。

### 2.3 案发实单回归(必做)

- 用 00 节实单形状造回归测试:同毫秒竞态(口供 created_at = 停链 ended_at + 1)→ 修后复核输入**含**该口供;旧轮 completed 节点 + 本轮未派发 → 修后输入**不含**旧轮 completed。

## 3. 安全死线

- **0-diff**:终标机器(director_agent 处置/七查)/ 链驱动 / 人闸 / worker_report 落账侧 / runner 全家 / 沙箱 / 冻结核;
- 复核仍 advisory:输出词表/幂等/「意见不是闸」一字不动;
- 只许动 `global_supervisor_agent.rs` 的 `load_review_input`+直接辅助(投影结构/prompt 拼接对应字段)+ 测试;越界 → 停手报回。

## 4. 验收

- 单测:2.3 两条回归 + 现有复核测试(幂等/unavailable/词表)逐条不回归;
- 口供匹配含正例(本轮口供入)反例(旧轮口供不入·同毫秒口供入);节点态含反例(旧轮 completed 不入);
- 三闸绿 + `cargo test --lib` 计数不降(基线 764/0/43·2026-07-10)+ **权威 `cargo fmt --check`** 新改行净(预存债 codex_db9/runner4/storage1 别碰);
- 0-diff 自证:`git diff --name-only` 只命中允许文件。

## 5. 回交

改动清单 + §4 证据 + 匹配键选型(走了首选还是 fallback·run.nodes 字段实况)→ 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 只放宽时间窗不建轮次语义(旧轮口供可能混入=换个方向的错) / 保留全量前缀扫当第二真源 / 改 prompt 契约或词表 / 碰终标/落账侧/链驱动 / 新造 event_type / 为测试便利改生产幂等键。
