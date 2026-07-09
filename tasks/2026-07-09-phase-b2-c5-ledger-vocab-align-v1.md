# 实现任务包:B2·C5 账本词表对齐 + entry_type 运行时校验(B2 收官)· 主导线 → 执行线 v1

日期:2026-07-09　性质:**中等偏轻**(读模型映射改写+校验·纯读模型层)。主导线已 measure-first + 防重造 grep(见 §0)。正本:任务包设计 §3.5(13 词表)+ C0 §5.2。**B2 最后一片·落地=B2 收官。**

## 0. 接手须知(冷启即读·前提+防重造已核)

- 你是**执行线**(后端·读模型层)。**子线不 commit。** 全程中文。
- **13 词表正本(设计 §3.5)**:`task_package_created`/`subagent_started`/`permission_requested`/`permission_granted`/`permission_denied`/`tool_call_summary`/`subagent_report`/`review_result`/`node_returned`/`node_failed`/`node_passed`/`director_summary`/`user_decision`。
- **现状缺陷(主导线亲读)**:`ledger_entry_type_from_audit`(workflow_read_model_entrypoints.rs:1303)用 **`contains()` 子串启发式**——链的自定义 event_type 大半从 else 漏成原样(不在词表);**且 contains 误判**:C4c 新增的 `workflow_chain_node_failed_action_archive`(结束)和 `_failed_action_rework`(退回)**都含 "failed"→ 被错映射成 `node_failed`**(结束/退回 被当失败·审计失真)。
- **链产的全部 event_type(主导线 grep·映射输入)**:`node_started/completed/failed/skipped/needs_rework/waiting_decision/director_deterministic_completed/director_lm_completed/failed_action_archive/failed_action_rework/director_summary` + run 级 `run_started/completed/failed/stopped/superseded/waiting_decision/stop_requested`。
- **防重造 grep 结论**:无现成 entry_type 校验/枚举/词表常量(C0 属实·裸 String)——C5 新建**唯一一处**,别散落。

## 1. 拍板摘要

- **做什么**:①`ledger_entry_type_from_audit` 从 contains 启发式改**显式精确映射**(修误判)+ 对齐词表;②词表**扩纳真新态**(waiting_decision/node_skipped·B2 加的真状态·13 是 B2 前的);③`entry_type` 加**运行时校验**(∈ 词表·唯一真源常量)。
- **canon 微决策(主导线定)**:13 词表是 B2 前定的;B2 新增 `waiting_decision`(C3/C4)、`node_skipped`、`node_cancelled`(C3b)是**真审计状态**——**扩进词表**(比塞进错桶保真);记档「词表 = 13 + B2 三新态」。
- **不做**:改链产 event_type 的命名(动 append_chain_audit 多处·反而扩面)/ C6/审查智能体。

## 一句话判据

**「是不是只:ledger_entry_type_from_audit 改显式精确映射(修 failed_action 误判)+ 词表常量扩纳真新态 + entry_type 运行时校验(唯一常量)——而链产 event_type 命名 0-diff、审计写入侧 0-diff、不发明状态?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 显式映射(替 contains·修误判)

- `ledger_entry_type_from_audit` 改**精确匹配表**(按主导线给的映射·别再 contains 子串):

| 链 event_type | → entry_type | 备注 |
|---|---|---|
| `workflow_chain_node_started` | `subagent_started` | |
| `workflow_chain_node_completed` / `_director_deterministic_completed` / `_director_lm_completed` | `node_passed` | C4a 终标全算 passed |
| `workflow_chain_node_failed` | `node_failed` | |
| `workflow_chain_node_needs_rework` / `_failed_action_rework` | `node_returned` | **修误判**(rework≠failed) |
| `workflow_chain_node_failed_action_archive` | `user_decision` | **修误判**(主管选结束=决策·非失败) |
| `workflow_chain_node_waiting_decision` | `waiting_decision`(新) | 扩词表 |
| `workflow_chain_node_skipped` | `node_skipped`(新) | 扩词表 |
| `workflow_chain_director_summary` | `director_summary` | |
| 含 `task_package`/`permission_decision` 的 | `task_package_created`/`user_decision` | 保留现有正确分支 |

- **run 级 event_type**(`run_started/completed/stopped/...`):**先核它们是不是进 ledger 条目**——是则同样精确映射(run_completed→node_passed 之类·或另定)、不是(纯 run 生命周期非节点账本)则**不映射·保持原样别硬塞**;**拿不准 → 停手报回**(别猜)。

### 2.2 词表扩纳 + 唯一真源常量

- 建**唯一常量**(如 `const LEDGER_ENTRY_TYPES: &[&str] = &[...13 + waiting_decision + node_skipped + node_cancelled]`);映射与校验**都引它**(单一真源·别两处各写一遍);
- `node_cancelled`(C3b dispatch cancelled 终态)一并纳入(若审计有对应 event_type·grep 核)。

### 2.3 entry_type 运行时校验

- 加 `fn is_valid_ledger_entry_type(s: &str) -> bool`(∈ 常量);在 entry_type 落账/读出处**运行时校验**(校验失败→保守:记 warning + 归一到安全值或原样标注·**别 panic 崩读模型**·读模型是增益);
- 映射函数出口断言:`ledger_entry_type_from_audit` 返回值恒 `is_valid_ledger_entry_type`（除明确不映射的 run 级）。

### 2.4 明确不做

改 append_chain_audit 各调用点的 event_type 命名(0-diff·只在读模型侧映射)/ 改审计写入 / C6 / 审查智能体。

## 3. 安全死线

- 链产 event_type 命名(append_chain_audit 调用点)/ 审计写入侧 / director_agent 本体 / 沙箱/授权/execute — **0-diff**(C5 只动读模型映射+校验);
- **不发明状态**:词表只纳链真产的真状态(grep 为据)·不硬造;校验失败**软着陆不崩**(读模型是增益);
- fmt `cargo fmt --check`;真跑测试项目;memories 观察模式不加旗。

## 4. 验收

- **单测**:①每个链 event_type → 精确映射对(**尤其 failed_action_archive→user_decision、_rework→node_returned·证误判修了**);②真新态 waiting_decision/node_skipped 进词表且映射到自身;③entry_type 校验:合法∈词表 true、非法 false+软着陆不 panic;④词表常量是唯一真源(映射+校验都引它·grep 证没第二份词表);
- **回归**:C4c 的 751 全绿(证没碰链/审计写入);既有 ledger 读模型测按新映射调整处说明;
- 三闸绿 + 死线 0-diff(append_chain_audit 命名 git 0-diff 自证)+ 计数不降 + fmt 权威净。

## 5. 回交

- §4 证据(尤其误判修复两处 + 词表唯一常量 + run 级处置实答)+ 死线 0-diff → 主导线核实物(**重点核:误判真修了、没发明状态、没碰审计写入**)。**子线不 commit。** → **B2 收官**。

## 7. 不接受为

- 保留 contains 启发式(要显式精确·修误判)/ 词表写两份(唯一常量)/ 硬造链没产的状态 / 改 append_chain_audit event_type 命名(只映射侧改)/ 校验失败 panic 崩读模型(软着陆)/ run 级拿不准硬塞(停手报回)/ 提前做 C6 / 自报 fmt 或 ad-hoc rustfmt。
