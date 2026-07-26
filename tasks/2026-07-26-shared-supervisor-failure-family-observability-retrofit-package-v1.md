# 任务包：首句失败可观测性改造（failure_family 落地 · 只加名字不改行为）v1

- 日期：2026-07-26
- 状态：**待用户授权派发（DRAFT_AWAITING_DISPATCH）**
- 负责人：独立对话底座线执行线
- 指导/验收：当前总指导对话
- 上游：`evidence/2026-07-26-shared-supervisor-first-turn-symptom-traceback-v1.md`（`ACCEPTED_D2R1_TRACEBACK` + 指导线 §11 裁决与订正）
- 目标 evidence：`evidence/2026-07-26-shared-supervisor-failure-family-observability-retrofit-verification-v1.md`

## 0. Kickoff

D2-R1 把病灶量清楚了：首句失败时，**五个完全不同的分支对外只有三种签名**，`:2950`（首句没接上）和 `:2889`（重放找不到记录）逐字同形；binding 那条路也有三处并流（12 变体压成一个、3 原因共用一 stage、`Update(_)` 报成构造失败）。所以 07-23 那轮"归因不了"不是运气差，是**外面根本没有可分辨的信息**。

本包只做一件事：**给每种失败一个自己的名字**。不修 bug、不改行为、不动 UI 文案。做完之后，下一次真机首句才能一次归因。

### 0.1 指导线拍定的三条默认（执行线不得自选）

1. **行为零变化**：控制流、成功/失败的判定、写入面、审计事件全部不动。只改"失败对外怎么自报家门"。
2. **新字段一律可选加法**：`Option<String>` / serde default / `skip_serializing_if`，保证既有消费者与既有存量记录仍然合法。
3. **UI 文案不改**：`failure_family` 只进回执与可读日志，用户可见那句话保持原样。理由：改文案属产品可见变更，且真机重验读回执就够。

### 0.2 明确不做

- **不修 bug**：为什么失败仍未知，本包只让它可分辨；
- **不接线孤儿路径**：`submitSupervisorResidentAnswer`（`src/lib/tauri.ts:1899`，前端零调用者）保持现状，不接也不删；
- **不处置死码**：`reconcileResidentMessageSubmission` 与其 7 处测试断言原样保留（删/标属产品决策，且会污染"行为零变化"的证明）；
- **不进真机**：本包全部离线；真机首句重验是下一包，需用户在场并单独授权。

## 1. 已确立事实（直接施工依据，不得重查）

1. resident-answer 路五分支：`supervisor_resident_oneshot_session.rs:2773 / 2889 / 2950 / 2968 / 2990`，`status` 与 `message` 五处逐字相同、`supervisor_reply` 全 `None`，对外仅 3 种签名（`(false,None)`×2、`(false,Some)`×2、`(true,Some)`×1）。指导线已独立复读确认。
2. `:2937` `let _ = append_resident_delivery_diagnostic(...)`：诊断写入自身的失败被整体丢弃。
3. binding 路三处并流：`commands.rs:455`（`Err(_)` 丢弃 12 变体）、`commands.rs:400/419/433`（3 原因共用一 stage）、`mcp/supervisor_orchestrator.rs:1110`（`Update(_)` 报成 `BindingConstruct`）。
4. binding 路已有 `binding_stage` 端到端接线（回执 + 前端 `conversationTransport.ts:87/244-250/400`），本包在其上补充，不重造。
5. `append_resident_delivery_diagnostic` 只属 resident-answer 路（唯一生产调用点 `:2937`）——**这是下一包真机取证的抓手，本包必须保证它失败时也能被看见（A7）**。
6. **反例**：`recorded` 事件名两路共用（`commands.rs:115-116`），不可用于推断路径。

## 2. Authority 与边界

- authority_chain：`AGENTS.md` → `CURRENT.md` → 07-23 真机停点 → D2-R1 evidence（含指导线 §11）→ 本包。
- capabilities_touched：**none**。不新增/改名/删除任何 Tauri command，不改 payload 语义、vault root、写路、capability registry。
- 并发：本包只写下列文件；期间不得有其他包写同一批文件；不得启动真机。

## 3. 冻结与基线副本

- HEAD：`a13599e`（代码区干净；工作树另有未提交的 evidence/tasks 文档，属正常）。
- **基线副本（硬规矩）**：改动前把全部窄写目标逐字节复制到 `evidence/raw/2026-07-26-shared-supervisor-failure-family-observability-retrofit/baseline/` 并写 `baseline-manifest.txt`；收口给出逐文件 diff 摘要（改动行数 + 每个 hunk 的函数名）。**无基线副本不得开工。**

| 文件 | 派发 SHA-256 | 权限 |
| --- | --- | --- |
| `src-tauri/src/supervisor_resident_oneshot_session.rs` | `5b6696e2eb804d2a18e31b22a7373877fc815911c2179b6f2850f44324afd413` | 窄写（A1–A8） |
| `src-tauri/src/supervisor_resident_oneshot_tests.rs` | `ee74fae6c2021189fe7780301e951284e529254268e5189a38698857c035c23d` | 窄写（A9 红测） |
| `src-tauri/src/commands.rs` | `e9f98ea7c340c8f871e227505a962905298f345aebb7d5ddbccf904a78005126` | 窄写（B2/B3/B5） |
| `src-tauri/src/mcp/supervisor_conversation_binding.rs` | `d9f066d2cb99b0707357ff633e3cf73e58eeb9f9498a868e9b3cb232590a57f9` | 窄写（B1 `family()`） |
| `src-tauri/src/mcp/supervisor_orchestrator.rs` | `7238b2f0c229483b8a6bc8f43128319568df19de345a27bd59fcde635fc7c0bf` | 窄写（B4，仅 `:1110` 一带） |
| `src/lib/tauri.ts` | `95587bdd68c7e207e18d6ecdc2c862a260706c9aa7f5c3085b7dcf95d8dc14ee` | 窄写（类型加可选字段） |
| `src/lib/conversationTransport.ts` | `7f0a7cd82f1d814f13ba3e8d4cff88e6958c1abcc343b0726355fd7b81c15e96` | 窄写（B6 透传，不改文案） |
| `src/views/projects/jiaoban/useJiaobanConversationState.ts` | `b86a1dff8b75e8dcb72c746cb3876473ed09a1ad0f551ee472e2a433d33ca071` | **冻结只读**（死码不动） |
| `tests/shared-conversation-transport.test.tsx` | `0debae5bb479e24a3498c0c2265c386914dd01c034943a69cf278e3ec0acde7f` | 窄写（仅在类型变更确需时） |

其余一律冻结只读。任一 hash 漂移即按 §10 停止。

## 4. 精确写入白名单

### 4.1 D1 resident-answer 路（A1–A9，来自 D2-R1 §6.1）

| # | 位置 | 内容 |
| --- | --- | --- |
| A1 | `SupervisorResidentAnswerOutcome` | 加 `failure_family: Option<String>`（可选、默认 `None`） |
| A2 | `:2773` | `resident_reply_missing_after_injection` |
| A3 | `:2889` | `replay_recorded_without_injection` |
| A4 | `:2950` | `consult_failed` + 把 `failure` 的**稳定分类**一并带出（禁止只带自由文本） |
| A5 | `:2968` | `injected_event_append_failed` |
| A6 | `:2990` | `supervisor_reply_append_failed` |
| A7 | `:2937` | 诊断写入失败也要能被看见（不得再 `let _ =` 静默） |
| A8 | `:2867 / 2880 / 2913` | 三处 `delivery_unknown` 各给子 family |
| A9 | `supervisor_resident_oneshot_tests.rs` | 五分支各断言其 family；**并断言任意两分支 family 互不相等** |

### 4.2 D2 binding 路（B1–B6）

| # | 位置 | 内容 |
| --- | --- | --- |
| B1 | `supervisor_conversation_binding.rs` | 给 `ConversationTurnBindingError` 加 `family()`（稳定字符串，覆盖全部 12 变体） |
| B2 | `commands.rs:455` | 不再 `Err(_)` 丢弃：把 typed error 的 `family()` 带进回执 |
| B3 | `commands.rs:400/419/433` | 三处各给自己的 family（不再共用一 stage 的语义） |
| B4 | `supervisor_orchestrator.rs:1110` | `Update(_)` 拆出 `binding_conflict`，不再报成 `BindingConstruct` |
| B5 | 回执结构 | 加 `binding_failure_family: Option<String>`（`binding_stage` 保留不动） |
| B6 | `conversationTransport.ts` | 原样透传该字段到已有诊断出口；**不改任何用户可见文案** |

### 4.3 新 evidence

`evidence/raw/2026-07-26-shared-supervisor-failure-family-observability-retrofit/**`（含 baseline）、目标 evidence、本包 §12 回填、`docs/harness-catch-log.md`（有真 catch 才追加）。

除此之外一律不写；禁止 stage / commit / push。

## 5. 硬合同

1. **行为零变化的证明**：改前改后，`cargo test --lib` 的**通过数与失败数必须完全一致**（除 A9 与 B 系新增断言带来的增量，须逐条列出）。任何既有断言因本包改动而失败 → 说明行为变了 → 按 §10 停止上交。
2. **family 稳定且互不相等**：所有 family 是固定字符串常量（不是格式化出来的自由文本）；A9 的两两不等断言必须覆盖五个分支；binding 路 12 变体的 family 也必须两两不等。
3. **可选性**：新字段在 JSON 中缺失时反序列化仍成功（给出针对既有存量记录形状的断言）。
4. **UI 文案逐字不变**：给出改前改后用户可见文案的逐字对照（应完全相同）。
5. **不得顺手**：不得重命名既有 stage、不得调整既有 `binding_stage` 取值、不得删除死码、不得接线孤儿命令。

## 6. Red-first

先写 A9 与 B 系断言并跑一次——它们**必须先红**（当前五分支 family 不存在/相同、binding 12 变体无 family）。红测输出留档（断言名 + 失败值）。**不得先改实现再补断言。**

## 7. 必跑验证

从 `src-tauri`：

1. `cargo check --lib`（AGENTS 硬规矩：含 Rust 生产路径不得只跑 test）；
2. `cargo test --lib` **全量**，报改前/改后 passed/failed/ignored 三个数；

从 `prototypes/productized-desktop-shell`：

3. `npm run typecheck`；
4. `npm run test:offline-interaction`（37 入口全过）；

从仓库根：

5. shape gate `--mode baseline` 与 `--mode check`：**finding 集合须逐条相同、零新增**。注意 `commands.rs`(6794)、`supervisor_orchestrator.rs`(3715)、`supervisor_resident_oneshot_session.rs`(3382) **本来就在 `file_over_limit_not_in_ratchet` 之列**——行数会涨，属同一条既有 finding，**须在 evidence 明写三者改前/改后行数**，不得当作新增，也不得为压行数做无关重构；
6. 两个 selftest；
7. `git diff --check`、`git diff --cached --name-only` 为空；
8. 回算全部冻结 hash + 给出窄写文件相对基线副本的 diff 摘要。

## 8. 必须回传

1. 开工实核 + 基线副本 manifest；
2. red 输出（A9/B 系断言先红的证据）；
3. A1–A9、B1–B6 的逐项落点（文件:行 + family 常量名）；
4. §5 五条硬合同的逐条证明，特别是**改前/改后 cargo test 三数对照**与**UI 文案逐字对照**；
5. §7 全部门禁输出（含三个大文件的行数变化）；
6. 与 §1 已确立事实的对照（有无推翻）；
7. 新 catch；没有则明写零新 catch；
8. 结论。

## 9. 结论枚举

- `PASS_FAMILY_RETROFIT`（两路 family 全部落地、红转绿、行为零变化已证）
- `PARTIAL_FAMILY_RETROFIT`（说明哪几项未落、为什么）
- `BLOCKED_FAMILY_RETROFIT_<原因>`（尤其：发现某项无法在不改行为的前提下落地）
- 整包 `NEEDS_GUIDANCE_REVIEW`；**不得**声称 binding bug 已修、不得声称对话线可重验。

## 10. 立即停止条件

- 任何既有断言因本包改动而失败（说明行为变了）；
- 必须改控制流 / 写入面 / 审计事件 / command 名或 payload 才能落地某项；
- 必须动 `useJiaobanConversationState.ts`、必须删死码或接孤儿命令；
- 冻结 hash 漂移、staged 非空；
- 需要启动真机 / Codex CLI / MCP / 真实 store/vault。

## 11. 下一步（不在本包内）

本包收口且经指导线接受后，下一包是**一次真机首句重验**：读回执里的 `failure_family` 与 `binding_failure_family` 一次归因，并核 `append_resident_delivery_diagnostic` 计数是否变化（§1.5 的抓手）。**需用户在场并单独授权**。

## 12. 实际执行回填

- 状态：**已施工并回交指导线复核**（2026-07-26，用户明确"开工"）。evidence：`evidence/2026-07-26-shared-supervisor-failure-family-observability-retrofit-verification-v1.md`。
- 结论：**`PARTIAL_FAMILY_RETROFIT`** —— A 路 A1–A7、A9 落地并绿；**A8 与 B1–B6 未做**。整包 `NEEDS_GUIDANCE_REVIEW`。
- 基线：9 项冻结 hash 零漂移；基线副本 9 文件 + manifest 齐备。改前基准 `cargo check --lib` exit 0、`cargo test --lib` **1200/0/45**。
- 实际只动 2 个文件：`supervisor_resident_oneshot_session.rs`(+116/-2, 8 hunk)、`supervisor_resident_oneshot_tests.rs`(+110/-0, 1 hunk)。**其余 7 个冻结文件回算零漂移**。
- **§5 硬合同全过**：① 行为零变化——`cargo test --lib` **1200/0/45 → 1203/0/45**，+3 全是 A9 新测试（逐条列名），既有断言零失败零变动；② family 全是 `&'static str` 常量、A9 机械断言两两不等（4 分支 + 13 consult，且互不撞名、相位覆盖数恰为 13）；③ 可选性——`None` 时 JSON 整键省略，有断言；④ **UI 文案逐字未改**（patch 里所有 `message:` 均为上下文行）；⑤ 未重命名 stage、未动 `binding_stage`、未删死码、未接孤儿命令、未动 `useJiaobanConversationState.ts`。
- A4 比原方案省：既有 `stable_error_family()` 已提供 13 个一一对应的稳定分类，直接接，不用新造。
- 门禁：typecheck 通过、37 入口全过、shape gate `17/5/5` 零新增、`git diff --check` 干净、staged 空。三个大文件行数 `commands.rs` 6794→6794（未动）、`supervisor_orchestrator.rs` 3715→3715（未动）、`supervisor_resident_oneshot_session.rs` 3382→**3496**，属同一条既有 finding，未做压行数重构。
- **主动上报流程违规**：§6 红先行被做反——先落实现再补断言。补做的红证据是把 session 换回基线副本后的**编译错误**（`has no field named failure_family` 等 5 条），已在 evidence §3 明确区分"编译错误"与"断言以失败形式跑一遍"两者强度不同，不包装。
- **未完成**：**A8**（三处 `delivery_unknown` 子 family）——那三处返回 `Err(String)` 非 `Outcome`，落 family 需改错误类型/返回形状，可能触碰"行为零变化"，本轮未评估充分故留下一包；**B1–B6 全部未做**（会话上下文预算耗尽，优先做完病灶所在的 A 路并证明行为零变化）；`cargo check` warning 598→599 的具体来源**未查清**，如实记为未查项、不猜。
- **重要**：因交办页首句实际走 binding 路（D2-R1 §5 已证）而 B 路未落，**本包不构成"下一次真机一次归因"的完整前置**。下一包须先补 B 路。
- 与 §1 六条已确立事实对照：**一条未推翻**；§1.5 的抓手已按 A7 处理（诊断写入失败不再静默）。
- 新 catch：**一条（本线自查）**——红先行被做反；"加字段"类改动最容易让人觉得红先行是形式主义，恰恰这时更要先让断言以失败形式跑一遍。
