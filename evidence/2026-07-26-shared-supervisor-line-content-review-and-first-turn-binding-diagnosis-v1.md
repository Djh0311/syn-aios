# 对话/主管线内容复核 + 首句 binding 断点诊断（只读）v1

- 日期：2026-07-26
- 任务包：`tasks/2026-07-26-shared-supervisor-line-content-review-and-first-turn-binding-diagnosis-package-v1.md`
- 范围：**零产品写入、零真机、零 Codex CLI/MCP、零真实 store/vault**。只读代码 + 机械普查 + 写 evidence。
- raw：`evidence/raw/2026-07-26-shared-supervisor-line-review-and-binding-diagnosis/`

## 0. 结论

- **`PARTIAL_D1`** —— 内容复核只覆盖了 A 层 6 个文件里的 3 个（1 个逐行、2 个定点），B/C 层未展开。未覆盖清单见 §2.3。
- **`PASS_D2_OBSERVABILITY_PLAN`** —— 折叠点普查完成；离线未复现真机失败，但**沿链读出一条比"判不出来"更强的结论**（§3.3），并据此给出最小可观测性改造方案（§4）。
- 整包：**`NEEDS_GUIDANCE_REVIEW`**。不得据此声称 binding 已修或对话线可重验。

**本轮最重要的一条**：§1.2 的既有认知需要修正——07-23 evidence 说"公开命令把 binding 内部错误折叠、前端收敛成通用失败"，**在 `a13599e` 的代码上只有后半句的一半成立**：相位（stage）其实是端到端暴露的，而 07-23 真机看到的那句通用文案，在当前代码里**没有生产来源**。详见 §3.3。

## 1. 开工实核

| 项 | 包内要求 | 实测 |
| --- | --- | --- |
| HEAD | `a13599e` | ✅ `a13599e` |
| `git status --short` | 空 | ⚠️ 3 行：` M CURRENT.md`（**非本线所改**，提交后由指导线动过）+ I5 与本包两个任务包文件（未跟踪） |
| Syn / cargo-tauri / Vite 残留 | 无 | ✅ 无 |
| `5173` | 空闲 | ✅ 空闲 |

`CURRENT.md` 的改动不是本包产生的，本包全程零写入代码面；收口 `git status` 见 §6。

## 2. D1 内容复核（PARTIAL）

### 2.1 覆盖情况

| 文件 | 计划层 | 本轮实际 |
| --- | --- | --- |
| `mcp/supervisor_conversation_binding.rs`（800） | A 逐行 | ✅ **逐行读完** |
| `commands.rs` 首句链段（`:198-206`、`:400-530`、`:650-710`） | A/B | ✅ **该段逐行读完**（全文 6794 行未通读） |
| `mcp/supervisor_orchestrator.rs` binding 段（`:1040-1150`、`:1211-1300`） | B 定点 | ✅ **该段逐行读完**（全文 3715 行未通读） |
| `manual_relay/conversation_transport.rs`（1457） | A 逐行 | ❌ **未读**（仅机械普查计数） |
| `mcp/supervisor_orchestrator_db_primary.rs`（370） | A 逐行 | ❌ 未读 |
| `mcp/supervisor_orchestrator_submit_proposal.rs`（345） | A 逐行 | ❌ 未读 |
| `mcp/storage.rs`（462） | A 逐行 | ❌ 未读 |
| `mcp/capability_registry.rs`（504） | A 逐行 | ❌ 未读 |
| B 层其余（`manual_relay.rs` 8108、`supervisor_resident_oneshot_session.rs` 3382、`supervisor_session_launcher.rs` 2990 等） | B | ❌ 未展开 |
| C 层（其余约 20 个文件 + 测试） | C 扫描 | ❌ 未展开 |
| 前端 10 个文件 | — | 仅 `useJiaobanConversationState.ts` 与 `conversationTransport.ts` 的相关段 |

**不假装浏览过等于读过**：上表"未读"的一律没有结论，也没有进发现清单。

### 2.2 已读部分的发现清单

| # | 文件:行 | 层 | 严重度 | 发现 |
| --- | --- | --- | --- | --- |
| 1 | `supervisor_conversation_binding.rs` 全文 | A | **无需动** | 写路：**零文件/进程操作**（模块 doc 声明属实，全文无 `fs::`/`Command`）。失败语义：12 个 typed variant，全部向上传递，无吞错。只读声明属实。 |
| 2 | `supervisor_conversation_binding.rs:452,467` | A | 无需动 | 两处 `unwrap_or_default()` 是 map 查不到时返回 `NotRequested`，语义正确，不是吞错。 |
| 3 | `commands.rs:455-460` | A | **应修** | `establish_supervisor_read_only` 的 12 变体 typed error 被 `Err(_)` 整个丢弃，压成单一 `BindingConstruct`。这是链上**信息损失最大**的一处。 |
| 4 | `commands.rs:400,433,419` | A | **应修** | 另外 3 处不同原因（resolve 失败 / run_id 生成失败 / 既有 thread 校验失败）同样压成 `BindingConstruct`。**同一个 stage 至少并进 5 种不同原因**。 |
| 5 | `supervisor_orchestrator.rs:1110-1112` | B | **应修** | `DbPrimaryStoreUpdateError::Update(_)` → `BindingConstruct`。但 update 闭包的失败包含 `:1070` 的"run 已绑定其他 turn，已拒绝覆盖"——**业务冲突被报成构造非法**，语义碰撞。 |
| 6 | `supervisor_orchestrator.rs` 生产段 | B | 记账 | 8 处 `map_err(|_|`、4 处 `let _ =`、5 处 `unwrap_or`（普查见 raw）。逐处判定未完成（未通读）。 |
| 7 | `useJiaobanConversationState.ts:36-85` | — | **应修（死码/空承诺）** | `reconcileResidentMessageSubmission` 自述"legacy…no longer selects the client command or controls the visible conversation"，实测**生产零调用点，只有测试在调**（7 处全在 `tests/jiaoban-conversation-center.test.tsx`）。见 §3.3。 |
| 8 | `tests/jiaoban-conversation-center.test.tsx:589-595` | — | 记账 | 该测试锁的是一个生产已不接线的 helper 的真值表——**测试全绿不代表生产该行为存在**。属包 §5 问题 5「测试是否真的锁住语义」的实例。 |

### 2.3 明确未覆盖

上表 2.1 标 ❌ 的全部文件、`cargo check` 的 598 个 warning（"never used" 族逐个判定）、C 层安全面扫描 —— **本轮一条都没做**。不得把本 evidence 当作 `a13599e` 欠账已还清。

## 3. D2 首句 binding 断点诊断

### 3.1 链路实读（host 收到首句 → … → transport start）

```
commands.rs:~390  resolve 项目/工作流
        :407      target_project_root != resolved.project_root      → BindingConstruct
        :413-426  既有 thread 校验 verify_supervisor_existing_thread → BindingConstruct
        :428-439  supervisor_run_id_for                              → BindingConstruct
        :441-461  ConversationTurnBinding::establish_supervisor_read_only
                     └ 成功即 lifecycle = Starting（binding.rs:257）  → 失败 BindingConstruct
        :462-472  supervisor_orchestrator::establish_…_binding       → 4 个 stage（见下）
        :474-489  activate_…_binding（仅当已有 thread_id）           → BindingActivate
        :491-517  knowledge_open_relay 缺失 / issue_grant 失败        → TransportStart
        :518+     transport start
```

持久化侧（`supervisor_orchestrator.rs:1093-1123`）：

```
workflow_state_path 失败                        → BindingStorePrepare
primary_repository_for_write Ok(Some) → DB 主写 → Store→StorePrepare / Update→BindingConstruct
                                                  / PersistDb→BindingPersistDb / ProjectJson→BindingProjectJson
primary_repository_for_write Ok(None)  → JSON-only 分支（:1125-1150，同样 4 个映射）
primary_repository_for_write Err(_)             → BindingStorePrepare
```

### 3.2 折叠点普查（生产路径，已剔除 `#[cfg(test)]` 之后正文）

原始输出：`raw/folding-point-census.txt`

| 文件 | `map_err(\|_\|` | `.ok()` | `unwrap_or` | `let _ =` | `Err(_)` | `.is_err()` |
| --- | --- | --- | --- | --- | --- | --- |
| `supervisor_conversation_binding.rs` | 0 | 0 | 2 | 0 | 0 | 0 |
| `conversation_transport.rs` | 3 | 0 | 1 | 1 | 1 | 2 |
| `supervisor_orchestrator_db_primary.rs` | 0 | 0 | 0 | 0 | 0 | 0 |
| `supervisor_orchestrator_submit_proposal.rs` | 3 | 0 | 2 | 0 | 1 | 2 |
| `mcp/storage.rs` | 0 | 0 | 3 | 0 | 1 | 0 |
| `mcp/capability_registry.rs` | 0 | 0 | 0 | 0 | 0 | 0 |
| `supervisor_orchestrator.rs` | 8 | 0 | 5 | 4 | 1 | 0 |
| `commands.rs` | 6 | 3 | 3 | 0 | 7 | 3 |

**折叠后外部能否分辨是哪一相**（本轮逐处判定的部分）：

| 折叠点 | 折叠后还能分辨吗 |
| --- | --- |
| `commands.rs:400/419/433/455` → `BindingConstruct` | **不能**——5 种原因同一个 stage，且 typed error 全丢 |
| `supervisor_orchestrator.rs:1106-1119` 四路映射 | **能分相**（StorePrepare/PersistDb/ProjectJson 各自独立），但**相内原因不能**（内层 payload 丢弃） |
| `:1110` `Update(_)` → `BindingConstruct` | **不能**，且**误导**：业务冲突伪装成构造非法 |
| `commands.rs:482/494/511` → `BindingActivate` / `TransportStart` | 相位能分；相内原因不能 |

### 3.3 本轮最硬的一条：07-23 的症状在当前代码里没有生产来源

包 §1.2 把"前端把 start rejection 收敛为通用失败"当既定事实。实读结果**部分推翻**：

1. **后端确实暴露相位**：`SupervisorConversationBindingStage` 有 7 个值（`commands.rs:198-206`），并且 `supervisor_start_failure_receipt`（`:650-663`）把 `binding_stage: Some(stage)` **放进了 receipt**，还带各相独立的人话文案（`:685-703`，例如"主管对话绑定没有写入主存储；运输没有启动。"）。
2. **前端确实接了**：`conversationTransport.ts:87` 有该字段类型；`:244-245` 用 `supervisorStartFailureMessage(receipt.transport.binding_stage)` 作为展示文案；`:249-250` 在 `failed` 时构造 `{ turn_id, stage }`；`:400` 原样透传。
3. **而 07-23 真机记录的那句"消息已送到主管，但主管这次没回上来"**——全仓只出现在 `useJiaobanConversationState.ts:37`（常量）与其测试。产出它的 `reconcileResidentMessageSubmission` **生产零调用点**，且函数自己的注释写明已不再控制可见对话。

**推论（只到证据允许的程度）**：若首句真的走到了 binding 链并失败，UI 应当显示某个相位文案，而不是那句通用文案。07-23 观察到的是后者。因此三者必居其一：

- (a) 07-23 那轮跑的是**更早的构建**，当时该 legacy helper 仍在接线（transport 改造后来才落）；
- (b) 07-23 evidence 里的引文是**转述**而非逐字 UI 抓取；
- (c) 该文案另有来源，而本轮 grep 未覆盖（本轮只 grep 了 `src` 与 `tests`）。

**本轮无法在离线判定是哪一种**，因为没有那轮的 receipt 原件或 UI 逐字抓取。但无论哪一种，结论一致：

> **07-23 的失败刻画不能直接搬到 `a13599e` 上当作待修 bug 的现状描述。** 下一次真机首句应按**全新观测**对待，而不是"确认既有结论"。

这一条不推翻 §1.2 的任何**已排除项**（R3 两个假设、R4a 对账、R3-binding 相位语义仍然成立），它推翻的是"当前失败长什么样"的那个前提。

### 3.4 离线复现：未做

包 §6.2 要求在离线夹具上走一遍。**本轮没做**，如实记录，不补猜。原因是链路实读先撞上 §3.3 那条前提问题——在"现状症状无法对应到当前代码"没澄清之前，离线复现只能复现出一个与真机无法对照的结果。建议顺序调整为：先按 §4 让相位可归因，再做真机一次，然后才谈离线对照。

`registry entries=0` 是否为前置条件（包 §6.2 问题 2）——**未查**。

## 4. 最小可观测性改造方案（§6.3）

目标：下一次真机首句**一次就能归因**。不改行为、不改写路，只让失败可分辨。

| # | 改哪里 | 改什么 | 动没动已验收合同 |
| --- | --- | --- | --- |
| 1 | `supervisor_conversation_binding.rs` | 给 `ConversationTurnBindingError` 加 `fn family(&self) -> &'static str`，12 变体各返回稳定短名（`missing_field` / `invalid_project_root` / `project_id_mismatch` / …） | 否，纯新增 |
| 2 | `commands.rs:455` | `Err(_)` → `Err(error)`，把 `error.family()` 随 stage 一起放进 receipt | 否（receipt 加字段，既有字段不动） |
| 3 | `commands.rs:400/419/433` | 各自给一个稳定 family（`project_root_mismatch` / `existing_thread_unverified` / `run_id_unavailable`），不再共用裸 `BindingConstruct` | 否 |
| 4 | `supervisor_orchestrator.rs:1106-1119` | 四路映射保留，但把内层 payload 转成 family 字符串一并上报；特别把 `Update(_)` 从 `BindingConstruct` 拆出来，给 `binding_conflict` | **需注意**：`SupervisorConversationBindingEstablishmentError` 是既有 enum，加变体要同步其现有断言 |
| 5 | receipt 结构 | `transport` 层加 `binding_failure_family: Option<String>`（与既有 `binding_stage` 并列） | 否，新增可选字段 |
| 6 | `conversationTransport.ts` | 透传新字段；`supervisorStartFailureMessage` 在有 family 时附加 family 原文（或至少 `console.warn` 落进可读日志） | 否 |
| 7 | `useJiaobanConversationState.ts` | **删掉或明确标注** `reconcileResidentMessageSubmission` 与那三条常量（生产已不接线），避免下一次又拿它的文案当现状证据；同步处理其测试 | 需改 `tests/jiaoban-conversation-center.test.tsx` |
| 8 | 新断言 | ①每个 stage 至少一条 family 断言；②`binding_conflict` 与 `binding_construct` 不得互相冒充；③receipt 序列化含新字段 | 新增 |

**只出方案，本包未动手。**

## 5. 与 §1.2 已排除项的对照

| §1.2 条目 | 本轮是否推翻 |
| --- | --- |
| R3 假设 A/B 均不成立 | **未推翻**，本轮也未把任何源码候选当根因 |
| R4a DB/JSON 对账已排除缺字段/空 tasks/归属差异 | **未推翻**，本轮未触及 |
| 07-23 binding 七相语义已固定 | **未推翻**——七相在 `commands.rs:198-206` 实读为 7 个值，与记载一致 |
| 前序 `25/0 → 26/1` 只证明 `Starting` 写入 | **未推翻**；本轮补一条：`Starting` 由 `binding.rs:257` 在构造时即赋值，落盘在 `supervisor_orchestrator.rs:1078` |
| 真机 binding=0 即连 `Starting` 都没落 | **未推翻**，但见 §3.3——该轮症状与当前代码对不上，需重新观测 |

## 6. 收口

- `git status --short` 收口应只多出：本 evidence、`raw/` 目录、本任务包 §11 回填。**零代码改动。**
- 新 catch：**一条**——"legacy 助手生产已不接线，但其文案仍被当作真机现状证据引用"，属"测试锁住了一个生产不存在的行为"的实例。是否入账由指导线定（本轮已在 §2.2 #7/#8 与 §3.3 完整留证）。

## 7. 未完成（必须由下一包接手）

1. D1 的 A 层剩余 5 个文件、B 层全部、C 层全部、598 个 warning 的逐个判定；
2. D2 §6.2 的离线复现与 `registry entries=0` 前置条件排查；
3. §3.3 三种可能性的证据收敛——需要 07-23 那轮的 receipt 原件或逐字 UI 抓取；若都拿不到，就按"全新观测"重跑。

## 8. 指导线复核与裁决（2026-07-26 · guidance）

裁决：**D1 `PARTIAL` 接受（3 条发现留用）；D2 头号结论 `REJECTED`，退回返工。** 整包 `NEEDS_D2_REWORK`。

### 8.1 指导线逐条核过、判定成立的部分

| 项 | 指导线独立核验 |
| --- | --- |
| 零代码改动 | `git diff -- prototypes/` 为空 ✅ |
| D1 发现① | `commands.rs:455` 确为 `Err(_) => supervisor_start_failure_receipt(.., BindingConstruct)`，typed error 整体丢弃 ✅ |
| D1 发现② | `commands.rs:400 / 419 / 433` 三处不同原因（context 解析 / 项目根 `.is_err()` / run_id 生成）确实汇进同一失败回执 ✅ |
| D1 发现③ | `supervisor_orchestrator.rs:1110` 确为 `DbPrimaryStoreUpdateError::Update(_) => BindingConstruct`，存储更新失败被报成构造失败，语义碰撞 ✅ |
| 后端已暴露相位 | `commands.rs` 的 `supervisor_start_failure_receipt` 确实带 `binding_stage` + 每相人话文案 ✅ |
| 前端确实接了 | `conversationTransport.ts:87 / 244-250 / 400` 确实读取并使用 `binding_stage` ✅ |
| 前端 helper 是死码 | `useJiaobanConversationState.ts:45` 只有定义，`src/` 内零调用点，仅测试调用 ✅ |
| 自述未完成项 | 与实际一致，没有把浏览说成读过 ✅ |

### 8.2 被推翻的头号结论（附实证）

本 evidence §3 断言「那句 UI 文案在当前代码里没有生产来源」。**该断言为假。** 指导线全仓复搜（含后端）结果：

同一句 `消息已送到主管，但主管这次没回上来——可以再发一次。` 在 **生产区**出现 5 次：

```
src-tauri/src/supervisor_resident_oneshot_session.rs:2773  （fn recorded_resident_reply_outcome，2727）
src-tauri/src/supervisor_resident_oneshot_session.rs:2889  （fn submit_supervisor_resident_answer_with_parts，2847）
src-tauri/src/supervisor_resident_oneshot_session.rs:2950  （同上）
src-tauri/src/supervisor_resident_oneshot_session.rs:2968  （同上，分支前置为 .is_err()）
src-tauri/src/supervisor_resident_oneshot_session.rs:2990  （同上，分支前置为 .is_err()）
```

可达性链（逐层核过，非推断）：`command_registry.rs:166` 注册 `supervisor_session_launcher::submit_supervisor_resident_answer` → `submit_supervisor_resident_answer_with`（`:2829`）→ `_with_parts`（`:2838` 调用点）→ 上述 5 个 return。文件内 `#[cfg(test)]` 均为**单项属性**（`:2378` 起的 thread_local / struct / impl），**不构成测试模块边界**；这 5 处不在任何测试块内。

**推论反转**：07-23 真机看到的那句文案与当前代码**对得上**，最可能就是这条 resident-answer 路吐出来的。§3.3「三种可能性 / 下次真机按全新观测对待」建立在假前提上，**作废**。

### 8.3 更要命的一点：漏掉的正是本包要找的东西

那 5 个分支返回的 `status` 完全相同（`message_recorded_supervisor_incomplete`），`message` 也完全相同，仅 `reply_injected`、`thread_id` 有微差；其中 `:2968`/`:2990` 两处的前置是 `.is_err()`——错误被检测到之后**错误值本身被丢弃**。

**这就是本包 §6.1 要普查的折叠点**，而且是与实际观测症状直接对应的那一处。`raw/folding-point-census.txt` 覆盖 8 个文件，**其中不含 `supervisor_resident_oneshot_session.rs`**——普查因此不完整。

### 8.4 指导线自认的责任

本包 §6.1 把链路预先写死为 host → construct → store prepare → DB persist → JSON projection → activate → transport start（即 binding 那条路），§1.2 又把「后端折叠 + 前端收敛」当既定事实。执行线照图施工，就没有从「这句话到底是谁产的」反查。**诊断包不该替执行线画路线图**；应当要求"从观测到的症状反查来源"。这条计入账本，返工包已按此改写。

### 8.5 留用与返工边界

- **留用**：D1 的 3 条发现、`binding_stage` 双端已接线的事实、前端 helper 是死码的事实（该事实成立，但**不支持**它原来的推论）。
- **返工（D2-R1）**：补 resident-answer 路的折叠点普查、区分那 5 个分支、重写可观测性方案覆盖两条路。
- **仍开口**：D1 的 A 层剩余 5 文件、B/C 层、598 warning 逐个判定——**不在返工包内**，继续挂账；本 evidence 不等于 `a13599e` 欠账已还清。
