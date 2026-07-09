# 实现任务包:B2·C3b 求助收敛(读模型只投影真源)+ dispatch cancelled 终态· 主导线 → 执行线 v1

日期:2026-07-09　性质:**中等**(改读模型派生+状态机·不碰真源写侧/契约)。主导线已 measure-first 亲读(见 §0)。正本:七拍 `decisions/2026-07-08-b2-transfer-protocol-gap-final-v1.md` 拍②+附带裁定 + C0 §5.3。上承:**C3a 已核过**(实现A=worker_report 契约链已成求助唯一真源·`worker_structured_report_recorded` 审计事件带真求助字段)。

## 0. 接手须知(冷启即读·前提已核到底)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **C3a 立的真源(直接投影它)**:`record_worker_structured_report_at`(c4_c6:374)把 worker 结构化报文(含 C3a 填的真 open_issues/permission_requests/direction_risks/follow_up_suggestions)记成审计事件 **`event_type="worker_structured_report_recorded"`**(c4_c6:440)。**这是求助的唯一真源。**
- **要退役的启发式**:`derive_subagent_reports`(workflow_read_model_entrypoints.rs:903)现从 **dispatch warnings 猜**——`direction_risks` 靠 `warning.contains("direction")||contains("risk")`(922)、`open_issues=warnings.clone()`(通用告警非自述)。**它本来就收 `audit_events: &[Value]` 参数(906)**——真源数据现成、只是没用。
- **独立 bool 现状**:`unresolved_direction_risk` **只有读侧**(workflow_read_model:1240 `bool_value(artifact,...)` → 1253 生成 exception;1458/1551/1564 查该 exception),**grep 找不到写侧**=大概率恒空死读(C0 已疑)。
- **dispatch 终态现状**:completed(1267)/failed(director:781)/running;已有 `"accepted"|"cancelled"` match 臂(director:473·别处语义)。cancelled 终态随本片落(七拍附带裁定·配 waiting_decision 主管取消)。

## 1. 拍板摘要

- **做什么**:①`derive_subagent_reports` 改从 `worker_structured_report_recorded` 真源投影求助字段(退役 contains 启发式);②`unresolved_direction_risk` 接真源或删死读;③dispatch cancelled 终态(配 C3a 的 waiting_decision·主管取消一个待决策任务时落 cancelled)。
- **canon(拍②)**:**单一真源·读模型只投影不发明**;过渡期派生数据标注「派生·非自述」。
- **不做**:碰 worker_report.rs(C3a 已定·0-diff)/ 真源写侧 record 函数本体 / 契约。

## 一句话判据

**「是不是只:derive_subagent_reports 求助字段改投影 `worker_structured_report_recorded` 真源(退 contains 启发式)+ unresolved_direction_risk 接真源或删死读+dispatch cancelled 终态——而 worker_report.rs/c4_c6 record 本体/沙箱/授权/execute 0-diff、不发明真源没有的数据?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 derive_subagent_reports 投影真源(workflow_read_model:903·核心)

- 求助字段(`open_issues`/`permission_requests`/`direction_risks`/`follow_up_suggestions`)改从**本 work_item/node 对应的 `worker_structured_report_recorded` 审计事件**取(现成 `audit_events` 参数里 filter event_type + 关联 work_item_id/node_id);
- **退役 922 的 `contains("direction")||contains("risk")` 启发式**(direction_risks 不再从 warnings 猜);`open_issues` 不再无脑 `warnings.clone()`;
- **真源没有该字段 → 空**(不回退 warnings 猜·拍②「不发明」);真源事件缺失(旧数据/未上报)→ 字段空 + 可留一条「派生·非自述」标注区分;
- **dispatch 的通用 warnings 仍可留在 SubagentReport.warnings**(那是结构性告警·非求助自述·语义不混)。

### 2.2 unresolved_direction_risk 接真源或删(workflow_read_model:1240)

- 先 grep 全仓确认 `artifact["unresolved_direction_risk"]` **写侧到底有没有**;
- **无写侧(恒空死读)**→ 删 1240 那段死读+其驱动的 exception 生成(1253),连带 1458/1551/1564 的查该 exception 处一并清理(别留悬空引用);
- **有写侧**→ 改成接 C3a 真源(worker 报的 direction_risks 非空 → 生成该 exception);
- **拿不准写侧 → 停手报回**(别猜删)。

### 2.3 dispatch cancelled 终态

- dispatch 状态机加 `"cancelled"` 终态:**语义=主管取消一个 waiting_decision 待决策任务**(C3a 停在 waiting_decision 的任务·主管选「取消」时该 dispatch/节点落 cancelled·可逆终态);
- 复用现有终态落法(completed/failed 同款写点)+ 审计;waiting_decision→cancelled 的合法迁移进状态机允许表(workflow_read_model 的 NODE_ALLOWED_TRANSITIONS 类);
- **只加 cancelled 这一态**,不碰 completed/failed/waiting_decision 现有语义。

### 2.4 明确不做

worker_report.rs 任何改动(C3a 已定)/ 真源 record 函数本体 / C4 主管终标 / C5 词表对齐 / failed 四选一(归 C4)。

## 3. 安全死线

- `worker_report.rs`(C3a)/ `c4_c6` record_worker_structured_report_at 本体+validate / 沙箱/path-lock/授权/execute/runner/relay/commands — **0-diff**;
- **不发明真源没有的数据**(拍②·读模型只投影);cancelled 只加不改现有终态;真跑圈测试项目;memories 观察模式不加旗。

## 4. 验收

- **单测**:① worker 报 direction_risks=["方向A 可能错"] → derive_subagent_reports 投影出该值(**来自真源·非 warnings 猜**);② worker 没报方向风险但 dispatch warnings 含 "direction" 字样 → direction_risks **空**(证启发式真退役·不再误报);③ unresolved_direction_risk:删了则相关 exception 不再恒生成/接真源则真源驱动;④ dispatch cancelled:waiting_decision 任务被取消 → dispatch/节点落 cancelled·迁移合法;
- **回归**:C3a 的 735 测全绿(证没碰 C3a);读模型既有测不破;
- **真跑**(可选·`#[ignore]`):一条链 worker 求助 → 读模型呈现的求助来自真源;
- 三闸绿 + 死线 0-diff 自证 + 计数不降 + fmt **`cargo fmt --check` 真跑**(别 ad-hoc rustfmt·会假报;只看新增块)。

## 5. 回交

- §4 证据(尤其「真源投影 vs 启发式退役」两侧+unresolved_direction_risk 写侧实答)+ 死线 0-diff 自证 + 落点清单 → 主导线核实物(**我重点核:真源真投影了、启发式真退役了、没发明数据、cancelled 只加不改**)。**子线不 commit。**

## 7. 不接受为

- 保留 contains 启发式当回退(拍②要退役)/ 真源没有的字段用 warnings 猜补(不发明)/ 猜删 unresolved_direction_risk 写侧没查清(拿不准停手)/ 碰 worker_report.rs 或 record 本体 / cancelled 改到 completed/failed/waiting_decision 现有语义 / 提前做 C4/C5 / 自报 fmt 或 ad-hoc rustfmt 核。
