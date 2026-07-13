# 任务包:audit_events natural-key 法证判定(纯只读·轻档·零代码)v1

日期:2026-07-13 · 档位:**轻档·纯只读法证**(零代码改动·零 tracked 文件改动·产出=一份 evidence 报告)· 基线 commit `cc31c8a`。
上承:谓词修复包第二战果(catch-log 07-13)——M3 真实往返被 natural-key conflict 闸拦:live `audit_events` **16 组重复 event_id(51 条·内容确不同)+ 4 条 event_id 缺失**。本包只查清真相+摆修法,**修哪条由用户拍,本包一行代码不改**。

> 总指导预勘(2026-07-13 亲测,执行线复核并深挖):
> ① **出血未止**:最近 30 条事件 20 条 id 中段=96 字符顶格;撞号组时间跨 07-03→**07-11**。生成器=`format!("audit:{kind}:{}:{timestamp}", stable_id(实体))` 家族(已见 `workflow_state_lifecycle_task_package.rs:405`、`project_workflow_automation.rs:1884/:1990`、`workflow_execution_entrypoints.rs:418/:544` 等);与 3a 已修的 binding_id 96 字符截断同病(评审§五预言)。
> ② **4 条"无号"实为旧代 schema**:字段是 `audit_event_id`+`event_type`(2026-05-31 期),非数据损坏——natural-key 合同不认旧字段名。
> ③ **存在按前缀引用事件的消费方**:`memory_daily_loop.rs:191/:385` 的 `audit_refs`(`audit:worker-report:…`/`audit:final-review:…`)——「修数据改号」可能断引用,法证 D 必须清点。

## 所属开发线

执行线(架构/存储线)。纯只读;不 commit;回传总指导核收。

## 目标(四份法证 + 一份决策备忘录,全落一份 evidence 文档)

**法证 A·撞号 16 组逐组画像**:每组列 event_id、条数、kind、时间戳(同毫秒?)、所属 workflow/project、内容差异性质(是否同一操作批次里**不同实体**的事件被折叠同 id);判定撞号机理(96 字符前缀相同+同毫秒?)并给每组证据。

**法证 B·旧代 schema 事件清点**:live 主 store 里**全部**旧代事件(有 `audit_event_id`/`event_type` 而无 `event_id` 的)总数——不止已知 4 条,可能有更多只是没进 conflict;它们在 importer 的 natural_key/分类路径上的真实走向(file:line);两代 schema 字段对照表。

**法证 C·生成器全清单**:grep 全仓所有 audit event_id 构造位点,逐个列:file:line、id 模板、实体源、是否经 `stable_id` 截断(确认 `stable_id`(lib.rs:1081 起)的截断长度与行为)、**是否仍活跃产新**(对照最近事件);同毫秒多事件的发生场景(哪些操作一批发多条)。

**法证 D·消费方清单(修法爆炸半径)**:谁按 event_id/audit_refs 读、查、关联——`memory_daily_loop` 的 audit_refs 全部引用点、前端/命令层有无按 event_id 反查、exporter/importer 合同依赖;结论:改历史 id 会断什么、断多少。

**决策备忘录·修法三选一+止血(利弊+推荐,决定权在用户)**:
- **a) 修数据**:历史 51 条改号+4 条补 `event_id`——动真实根=重档+维护窗口;若 D 查出引用则需同步改引用;
- **b) 修合同**:迁移面 natural key 复合化(如 event_id+record_hash)+ 旧代字段 fallback(`audit_event_id`)——只动迁移机器=轻档;历史数据原样进库;代价=SQLite 侧 natural key 语义弱化,永久背着撞号史;
- **c) 保持 fail-closed**:不切库,直到人工裁决——最保守,M5 无限期挂;
- **止血(独立于三选一,基本必做)**:活跃生成器照 binding_id 3a 修法改全量 SHA——动活写路径,**另开包**,本包只评估位点与风险。

## 允许读取

src/ 全部;live 根**只读**(copy-out/解析/统计,严禁写);docs/evidence/ 全部。

## 允许写入

**仅** `evidence/2026-07-13-audit-events-natural-key-forensics-v1.md`(+scratch 临时文件不落仓)。

## 禁止事项(红线)

1. **零代码改动**:任何 .rs/.ts/.tsx/配置一行不改——`git status` 前后 tracked 文件零新 M 是硬验收;
2. live 根/生产 DB/真实状态零写;不跑任何会写的演练;
3. 不修 conflict 逻辑、不改谓词、不动迁移机器;
4. 法证不到位不下结论——每条判定必须带 file:line 或数据证据,推断标「推断」。

## 验收(预写死)

- 四份法证全带证据坐标;旧代事件**总数**清点(≠只看那 4 条);16 组逐组机理判定;生成器清单标活跃/停用;引用面给出「改号会断 N 处」的实数;
- 备忘录三选一+止血,各自档位(轻/重档)、前置、风险写明,附推荐+理由;
- `git status` 前后对比:唯一新增=evidence 报告一个文件;
- 回传 10 项,第 10 项无也写「无」。

## 总指导回收动作

抽查法证坐标(逐条可复核)+ 亲核 git status 零码改 → 接受后连 evidence 一起 commit(问一次);把备忘录三选一摆给用户拍板;止血包视 C 的结论另立。
