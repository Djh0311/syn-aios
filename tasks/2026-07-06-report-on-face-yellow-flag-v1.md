# 实现任务包:刀A·口供上脸 + 黄牌(worker 自述进交货脸;自报没干完的不许装全绿)· 主导线 → 执行线 v1

日期:2026-07-06　性质:**轻档**(一处后端加法字段 + 前端呈现;单线双面·文件边界见 §2.4;死线 0-diff)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(本包特批单线双面:后端小加法 + 前端呈现,当前无并行线,文件边界照 §2.4 走)。**子线不 commit。** 全程中文。
- **背景**:回程契约已落(d2dba24)——worker 交结构化口供、链解析落库,`DirectorChainStep` 已有 `report_summary`/`report_warning`(serde 加法·随 `chain_outcome` 回到前端)。**但**:① 交货脸不显示自述——上周真机逮到的假完成("无法启动浏览器,未完成手动验收")躺在库里,界面照样全绿;② 口供的 `status`(done|partial|failed)没有独立字段,前端无法可靠判黄牌。
- **一句话**:后端给 step 补一个 `report_status` 字段(从已解析的口供取,加法);前端交货脸/失败脸按任务显示自述行,**status≠done 或没交口供 → 黄牌**,任何黄牌时标题改"做好了(有 N 项要看一眼)"——自报没干完的不许装全绿。

## 1. 拍板摘要

- **要做的事**:口供从档案柜走上台面;"完成"和"自称完成"从此在界面上分得开。
- **为什么**:Phase A"闭环后半段·worker 汇报前端接"的正主;也给 Phase B 全局主管复核铺数据面。
- **代价**:一轮。后端 ~15 行加法 + 前端一个任务行组件。

## 一句话判据

**「是不是只:step 加 `report_status` 字段(消费已解析口供·加法)+ 前端按 steps 渲染自述行与黄牌——而链的成败/重试/状态迁移零改动、口供仍只归档不驱动(黄牌是呈现不是判决)、死线 0-diff?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 后端(小·加法)
- `DirectorChainStep` 加 `report_status: Option<String>`(serde 加法);在链消费缝(worker_report::consume 返回处)从已解析 `WorkerReport.status` 取值填入;没交口供/解析失败 → None(与现 report_summary 同语义)。
- **不改**:成败判定/report_warning 既有语义/落库路径。测试进 worker_report 自己的 mod(照旧不进 lib.rs):status 透传三态(done/partial/缺失)。

### 2.2 前端(主体·交货脸 + 失败脸同组件)
- 新 `JiaobanStepReportList`(或等价):吃 `outcome.chain_outcome.steps`,每任务一行——**任务标题 + 自述一句(report_summary)+ 徽章**:
  - `report_status=="done"` → 绿✓;
  - `=="partial"` / `=="failed"` → **黄牌⚠**,徽章文案「自述:没干完」/「自述:失败」;
  - `report_summary==None`(没交口供)→ **黄牌⚠**「没交汇报」;`report_warning` 有值 → 黄牌⚠ 显其人话(「落库失败…」等);
  - 链 step 本身 `state=="failed"/"skipped"` → 照现有红/灰,不被口供徽章覆盖(执行状态优先于自述)。
- **标题联动**:交货脸有任何黄牌 → 标题「✓ 做好了(有 N 项要看一眼)」;全绿照旧「✓ 做好了」。失败脸(fail-stop)也渲染同一列表(死在哪个任务、之前几个的自述一目了然)。
- **词表**:用户可见 = 任务标题 + 自述 + 人话徽章;不露 planned_task_id/node_id/dispatch;"口供"是内部黑话,界面用「自述/汇报」。
- 无 steps(老数据/异常)→ 整块不渲染,零回退。

### 2.3 明确不做(§5 同)
黄牌**不驱动任何行为**(不重试/不拦确认/不改链)——"读了并反应"到全局主管(Phase B)才判;历史口供浏览页;事实确认按钮(刀B);redo 幂等。

### 2.4 文件边界(单线双面·越界即停)
- 允许:`director_agent.rs`(仅 step 字段+填值缝)/ `worker_report.rs`(status 透传+自测)/ `ProjectJiaobanPanel.tsx` / `projectWorkflowSidePanel.css` / `lib/types/workflow.ts`(前端类型加字段)。
- **0-diff**:commands / codex_local_runner / c4_c6 / control_core / chain controller / 两 store / manual_relay / lib.rs / 其余一切前端文件。

## 3. 安全死线

- 口供**只归档只呈现不驱动**;链成败/重试/迁移零改动;黄牌是信息不是闸。
- 渲染类**必须真机过**;fmt 只本包文件(skip_children)。

## 4. 验收

- **单测**(worker_report mod):status 三态透传。
- **离线 DOM**(现有 harness):steps fixtures 三态(全绿/含 partial/含缺口供)→ 行与徽章与标题联动断言;无 steps 零渲染。
- **真机**(额度在):跑一单 → 交货脸每任务自述行;若真 codex 又交出 partial(如浏览器验收那类)→ 黄牌自然出现 = 最佳实证(没有也不硬造,离线断言已覆盖);失败脸(可用历史 failed 数据或真造)同列表在。
- 三闸绿;死线 0-diff 自证;计数不降。

## 5. 回交

- §4 证据(真机截图必带:自述行 + 黄牌或全绿)+ 字段/组件落点 + 0-diff 自证 → 主导线核实物。**子线不 commit。**
