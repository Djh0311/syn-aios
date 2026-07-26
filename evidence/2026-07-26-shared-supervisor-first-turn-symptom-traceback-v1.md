# 首句失败——从症状反查来源（D2-R1 返工）v1

- 日期：2026-07-26
- 任务包：`tasks/2026-07-26-shared-supervisor-first-turn-symptom-traceback-rework-package-v1.md`
- 范围：**只读**。零产品写入（`src` / `src-tauri` / `tests` 一个字节未动）、零真机、零 Codex CLI/MCP、零真实 store/vault、不修 bug。
- raw：`evidence/raw/2026-07-26-shared-supervisor-first-turn-symptom-traceback/`

## 0. 结论

**`PASS_D2R1_TRACEBACK`** —— 症状反查完成、5 分支可区分性判明、两条路的关系说清。整包 `NEEDS_GUIDANCE_REVIEW`。

**三条主结论**：

1. **上一包被否的那条，我确认自己错了**，并且错因不是"方法没给到"——我在自己的证据里明写了"(c) 另有来源，本轮只 grep 了 `src` 与 `tests`"，**却仍然把强结论顶到标题**。知道搜索不完整还下断言，这是执行线的错。
2. **本轮又抓到我上一包的第二个错，而且更严重**：那张折叠点普查表的统计方法是坏的，`commands.rs` 真实折叠点是 **93** 处，我上次报的是 22 处（详见 §2）。
3. **§3.2 有一条新的、证据更强的结论**：交办页首句走的是 **binding/transport 那条路**；产出那句 UI 文案的 resident-answer 路，其命令 `submit_supervisor_resident_answer` 在前端**零调用点**。所以 07-23 的 `binding=0` 与那句文案**仍然是两条路的产物**——但这次的依据是可达链，不是"搜不到"。

## 1. 开工实核

| 项 | 实测 |
| --- | --- |
| HEAD | `a13599e` ✅ |
| Syn / cargo-tauri / Vite 残留 | 无 ✅ |
| `5173` | 空闲 ✅ |
| `git status --short` | 7 行：` M CURRENT.md`、` M docs/harness-catch-log.md`（**两条均非本线所改**）+ 上一包 evidence/raw + 三个任务包文件（未跟踪） |

## 2. 先自查：上一包那张普查表是错的

上一包 `raw/folding-point-census.txt` 用的是

```
awk '/#\[cfg\(test\)\]/{intest=1} !intest' "$f"
```

——在文件里**第一个** `#[cfg(test)]` 处整体截断。但那个属性多数是**单项属性**（只作用于紧跟的一个 item），不是模块边界。实测 8 个文件里只有 2 个（`supervisor_conversation_binding.rs`、`capability_registry.rs`）第一处后面真是 `mod`：

| 文件 | 首个 `#[cfg(test)]` 行 | 总行 | 后面是 `mod`？ | v1 静默丢弃 |
| --- | --- | --- | --- | --- |
| `commands.rs` | 1002 | 6794 | 否 | **85%** |
| `supervisor_orchestrator.rs` | 1152 | 3715 | 否 | **69%** |
| `supervisor_orchestrator_db_primary.rs` | 141 | 370 | 否 | **62%** |
| `conversation_transport.rs` | 746 | 1457 | 否 | **49%** |

**修正后的普查（v2）**：脚本 `raw/folding-point-census-v2.py`，输出 `raw/folding-point-census-v2.json`。v2 只剔真正的测试项/模块（单项属性按花括号配对只剔一个 item）。

| 文件 | 生产行/总行 | 折叠合计 | v1 报的 | 明细（map_err / ok / unwrap_or / let _ / Err(_) / is_err） |
| --- | --- | --- | --- | --- |
| `commands.rs` | 4949/6794 | **93** | 22 | 9 / 5 / 57 / 4 / 14 / 4 |
| `supervisor_session_launcher.rs` | 2014/2990 | **44** | 未测 | 1 / 7 / 15 / 20 / 0 / 1 |
| `supervisor_orchestrator.rs` | 2816/3715 | **38** | 18 | 8 / 2 / 21 / 6 / 1 / 0 |
| `supervisor_resident_oneshot_session.rs` | 3323/3382 | **27** | 未测 | 6 / 2 / 8 / 6 / 0 / 5 |
| `conversation_transport.rs` | 901/1457 | **13** | 8 | 7 / 0 / 1 / 1 / 1 / 3 |
| `supervisor_orchestrator_submit_proposal.rs` | 345/345 | 8 | 8 | 3 / 0 / 2 / 0 / 1 / 2 |
| `mcp/storage.rs` | 462/462 | 4 | 4 | 0 / 0 / 3 / 0 / 1 / 0 |
| `supervisor_conversation_binding.rs` | 585/800 | 2 | 2 | 0 / 0 / 2 / 0 / 0 / 0 |
| `mcp/supervisor_orchestrator_db_primary.rs` | 324/370 | 1 | 0 | 0 / 0 / 1 / 0 / 0 / 0 |
| `mcp/capability_registry.rs` | 373/504 | 0 | 0 | 全 0 |

**上一包 evidence §3.2 那张表作废，以本表为准。**

## 3. 症状 → 产出点反查

搜索一律覆盖 `src/` + `src-tauri/` + `tests/`。实际命令与命中行留档在 `raw/symptom-01-ui-copy-traceback.txt`、`raw/symptom-02-which-path.txt`。

### 3.1 症状①：UI 文案「消息已送到主管，但主管这次没回上来——可以再发一次。」

命令：`grep -rn "主管这次没回上来" src src-tauri tests` → **11 命中**

| 位置 | 性质 | 是否生产区 |
| --- | --- | --- |
| `supervisor_resident_oneshot_session.rs:2773 / 2889 / 2950 / 2968 / 2990` | **产出点 ×5** | **是**（该文件 `#[cfg(test)]` 均为单项属性，非模块边界；实测 3382 行里生产 3323 行） |
| `supervisor_resident_oneshot_tests.rs:1755/1818/2781/2972` | 断言 | 否 |
| `useJiaobanConversationState.ts:37` | 前端常量（死码，见 §3.4） | 否 |
| `tests/jiaoban-conversation-center.test.tsx:595` | 断言 | 否 |

**指导线 §1.1 所述属实，我上一包漏搜 `src-tauri` 是硬错。**

### 3.2 症状②：`status = message_recorded_supervisor_incomplete`

同 5 个产出点（`:2769 / 2885 / 2946 / 2964 / 2986`），另有 15 处测试断言。

## 4. §3.1 五分支可区分性（表已填满）

调用方（IPC 返回值 `SupervisorResidentAnswerOutcome`）能读到的字段只有五个：`status`、`reply_injected`、`thread_id`、`supervisor_reply`、`message`。其中 `status` 与 `message` 五处**逐字相同**，`supervisor_reply` 五处**都是 `None`**。因此可区分性只能靠 `reply_injected` + `thread_id`。

| 分支 | 触发条件 | `reply_injected` | `thread_id` | 外部可区分？ |
| --- | --- | --- | --- | --- |
| `:2773` | 幂等重放：找到 `user_message_injected` 事件，但没有对应的 `supervisor_message_recorded` | `false` | `Some`（取自 injected 事件） | ❌ 与 `:2968` 同签名 |
| `:2889` | 幂等重放：`client_request_id` 命中已记录消息，但 `recorded_resident_reply_outcome` 返回 `None`（连 injected 都没有） | `false` | `None` | ❌ 与 `:2950` 同签名 |
| `:2950` | **首次发送：`consult_supervisor_resident_with_parts` 失败**（主管没接上） | `false` | `None` | ❌ 与 `:2889` 同签名 |
| `:2968` | consult 成功，但 `append_resident_user_message_injected` 写失败 | `false` | `Some(turn.thread_id)` | ❌ 与 `:2773` 同签名 |
| `:2990` | 全链成功，但 `append_resident_supervisor_message_recorded` 写失败 | **`true`** | `Some(turn.thread_id)` | ✅ **唯一可分**（唯一 `reply_injected=true`） |

**结论：5 个分支对外只有 3 种签名，其中 2 组各并进 2 个分支。**

最要命的是 `:2889` 与 `:2950` 不可分——**`:2950` 正是"首次发送、主管没接上"，也就是 07-23 那类失败最可能落的分支**，而它与"幂等重放找不到记录"在外部完全同形。

**该路上的错误丢弃点（逐处）**：

- `:2905` `append_resident_user_message_recorded(...).is_err()` —— 错误值丢弃，只当布尔用
- `:2937` `let _ = append_resident_delivery_diagnostic(...)` —— **诊断写入失败被整个丢掉**（这条尤其讽刺：唯一为排查而写的诊断，它自己失败了没人知道）
- `:2961` `append_resident_user_message_injected(...).is_err()` —— 同上
- `:2983` `append_resident_supervisor_message_recorded(...).is_err()` —— 同上
- `:2867 / :2880 / :2913` `map_err(|_| "supervisor_resident_message_delivery_unknown")` —— 三处不同的底层失败压成同一个字符串

## 5. §3.2 两条路的关系（本轮最有价值的一条）

| 路 | 入口命令 | 前端调用点 |
| --- | --- | --- |
| **binding / transport 路** | `start_supervisor_conversation_transport`（`tauri.ts:1744`） | ✅ 交办页 `useJiaobanConversationState.ts` 的 `transportController` 在用 |
| **resident-answer 路** | `submit_supervisor_resident_answer`（`tauri.ts:1903`，包装函数 `submitSupervisorResidentAnswer`） | ❌ **`src/` 内零调用点** |

搜索为闭合式，专门补上了上次漏掉的两个口子（留档 `raw/symptom-02-which-path.txt`）：

- 搜包装函数名 `submitSupervisorResidentAnswer` → `src/` 内除定义处外 **0 命中**；
- 搜 **raw command 名** `submit_supervisor_resident_answer` 在整个 `src/` → 只有 `tauri.ts:1903` 定义处本身；
- 搜**动态 invoke**（`invoke(\`…\`)` / `invoke(cmd)` / `invoke(command)` / `invoke(name)`）→ **0 命中**，排除"命令名由变量拼出来所以搜不到"。

**结论**：交办页发首句走的是 binding/transport 路，**不经过** resident-answer 路；resident-answer 路的命令在 Rust 侧已注册、IPC 可达，但当前前端没有任何代码调用它。

**这对 07-23 那轮意味着什么**（严格限定在证据允许的范围）：

- `conversation_turn_binding=0` 若发生在 binding 路上，**是 binding 相关失败**，且该路已端到端可归因（回执带 `binding_stage`），本该能看到相位文案；
- 那句 UI 文案只能来自 resident-answer 路的 5 个分支之一，而该路当前前端不可达；
- 两者在**当前构建**上无法同时成立。

因此上一包"07-23 刻画不能直接当现状"的**方向**仍成立，但**理由必须换掉**：不是"全仓无来源"（错），而是**"产出该文案的那条路，当前前端没有调用点"**。这两个理由的强度和可证伪性完全不同，前者被一次 grep 就推翻了，后者是可达链事实。

**我不据此断言 07-23 那轮就是旧构建**——那仍然需要那轮的 receipt 原件或逐字 UI 抓取。本轮**判不出来**，如实写明。

## 6. §3.3 可观测性方案（按两条路重写）

### 6.1 resident-answer 路（新增，本轮重点）

| # | 改哪里 | 改什么 | 碰既有断言？ | 改行为？ |
| --- | --- | --- | --- | --- |
| A1 | `SupervisorResidentAnswerOutcome` | 加 `failure_family: Option<String>` | 否（新增可选字段） | 否 |
| A2 | `:2773` | `resident_reply_missing_after_injection` | 否 | 否 |
| A3 | `:2889` | `replay_recorded_without_injection` | 否 | 否 |
| A4 | `:2950` | `consult_failed`，并把 `failure` 的稳定分类一并带出（**别只带自由文本**） | 否 | 否 |
| A5 | `:2968` | `injected_event_append_failed` | 否 | 否 |
| A6 | `:2990` | `supervisor_reply_append_failed` | 否 | 否 |
| A7 | `:2937` | `let _ =` → 把诊断写入失败也记成 family（诊断自己失败必须可见） | 否 | 否 |
| A8 | `:2867/2880/2913` | 三处 `delivery_unknown` 各给子 family | 否 | 否 |
| A9 | 新断言 | 5 个分支各断言其 family；并断言**任意两分支 family 不相等** | 新增 | — |

### 6.2 binding 路（沿用已有 `binding_stage`，只补并流）

沿用上一包 §4 的 1–6 项（`ConversationTurnBindingError::family()`、`commands.rs:455` 不再丢 typed error、`:400/419/433` 各给 family、`supervisor_orchestrator.rs:1110` 把 `Update(_)` 拆出 `binding_conflict`、receipt 加 `binding_failure_family`、前端透传）。这 6 项在本轮复核后**仍然成立**。

### 6.3 死码处置

`useJiaobanConversationState.ts:36-85` 的 `reconcileResidentMessageSubmission` 与三条常量：`src/` 内零调用点（该事实经本轮复核仍成立）。建议删除或明确标注，并同步 `tests/jiaoban-conversation-center.test.tsx` 的 7 处断言——否则下次又会有人拿它的文案当现状证据（我上次就是）。

**只出方案，本包未动手。**

## 7. §3.4 离线复现：未做

§3.2 的结论出来后，"沿真正的那条路复现 `message_recorded_supervisor_incomplete`"这件事本身出现了前置问题：那条路**当前前端不可达**，离线夹具即使能触发 `:2950`，也无法回答"交办页首句为什么失败"——因为交办页根本不走它。

**判不出来。需要什么才能判**：① 07-23 那轮的 receipt 原件或逐字 UI 抓取（用来确定那轮到底走的哪条路）；或 ② 先落 §6 的 family，再做一次真机首句。

**不补猜。**

## 8. 与 §1 已确立事实的对照

| §1 条目 | 本轮是否推翻 |
| --- | --- |
| 1. 文案 5 处生产来源 + 可达链 | **未推翻，且已独立复核确认**（含 `#[cfg(test)]` 为单项属性） |
| 2. binding 路已端到端可归因 | **未推翻** |
| 3. 前端 helper 是死码 | **未推翻**（`src/` 内零调用点，本轮复搜确认） |
| 4. 上一包 D1 的 3 条发现 | **未推翻** |
| 5. R3 两假设 / R4a 对账 / 七相语义 | **未推翻**，本轮未触及 |

**新增一条 §1 未涵盖的事实**：`submit_supervisor_resident_answer` 在前端零调用点（含 raw 命令名与动态 invoke 两种搜法均为空）。这不与 §1.1 冲突——§1.1 说的是"Rust 侧可达链通到已注册命令"，成立；本条说的是"前端有没有去调它"，不成立。

## 9. 收口

- `git status --short` 应只多出本 evidence、`raw/` 目录、本包 §8 回填。**零代码改动**。
- **新 catch：一条**（本线自查）——机械普查用"首个 `#[cfg(test)]` 处截断"当剔除测试的手段，会把单项属性后面的整片生产代码静默丢掉；`commands.rs` 因此少算 71 处折叠点。教训：**任何"机械统计"在拿去支撑结论前，必须先验证它的剔除边界；统计口径错了，比没统计更糟——因为它看起来像证据。**

## 10. 未完成（继续挂账）

1. D1 的 A 层剩余 5 文件、B 层、C 层、598 warning 逐个判定（本包 §4 明确不在范围）；
2. 07-23 那轮走哪条路的实证收敛；
3. 任何代码修复与真机验证。

## 11. 指导线复核与裁决（2026-07-26 · guidance）

裁决：**`ACCEPTED_D2R1_TRACEBACK`（synthetic/只读范围）**，附一处证据订正。零代码改动经核属实（`git diff -- prototypes/` 为空）。

### 11.1 指导线独立复算 / 复读，判定成立

| 项 | 指导线独立结果 |
| --- | --- |
| 旧普查表作废的理由 | 成立。`commands.rs` 首个 `#[cfg(test)]` 在第 `1002` 行、全文 `6794` 行 → 按首个标记截断确实丢约 85%，算术相符；指导线在**全文件**上数折叠模式得 `104` 行命中，与其生产区 `93` 相容。旧值 `22` 属严重少算。 |
| 5 分支 → 3 种外部签名 | 成立。指导线自读五处：`(reply_injected=false, thread_id=None)` ×2（`:2889`/`:2950`）、`(false, Some)` ×2（`:2773`/`:2968`）、`(true, Some)` ×1（`:2990`）；`status` 与 `message` 五处逐字相同、`supervisor_reply` 全 `None`。**`:2950`（首句未接上）与 `:2889`（重放找不到记录）对外完全同形**。 |
| `:2937` 诊断自身失败被吞 | 成立。`Err(failure)` 分支内 `let _ = append_resident_delivery_diagnostic(...)`，该诊断写入的失败被整体丢弃。 |
| resident-answer 路前端不可达 | **结论成立**（见 11.2 的订正）：包装函数存在但 `src/` 内零调用者，指导线已核。 |
| §1 五条已确立事实 | 一条未被推翻，与执行线自述一致。 |

### 11.2 一处证据订正（第三次同类问题）

执行线称"raw 命令名全 `src/` 搜为空"。**该搜索结果为假**：raw 名就在 `src/lib/tauri.ts:1903`，位于导出包装函数 `submitSupervisorResidentAnswer`（`:1899`）内。

- **结论不受影响**：指导线复核确认该包装函数在 `src/` 内无任何调用者，故"前端不可达"成立。
- **但图景要改**：这不是"设计上就死的路"，而是**一个离接上只差一行 UI 代码的孤儿包装函数**；其上方注释写明"S1：用户消息统一注入常驻主管 thread"。更合理的读法是——界面已迁到 transport 路，旧路的后端与 typed client 都还挂着未拆（迁移残留）。这对"07-23 那轮走的哪条路"是有利线索，应写进下一包。

### 11.3 指导线补的一条线索与一条反例

- **可用线索**：`append_resident_delivery_diagnostic` 经核**只属 resident-answer 路**（定义与唯一生产调用点均在 `supervisor_resident_oneshot_session.rs`，调用点 `:2937`，位于 `submit_supervisor_resident_answer_with_parts`）。07-23 acceptance evidence 中该计数**跑前 `1`、跑后 `1`** → 那一轮**没有**追加投递诊断。可作下一包的实证抓手。
- **反例（指导线自查后撤回）**：不可用 `recorded +1` 反推路径。`commands.rs:115-116` 将 `supervisor_resident_user_message_recorded` / `..._supervisor_message_recorded` 同时用作**共享 transport 路**的事件名（`SHARED_CONVERSATION_USER_EVENT` / `SHARED_CONVERSATION_ASSISTANT_EVENT`），两路共用同一套事件词汇。指导线原拟据此定案，复查后撤回，记录在此以免下一包重蹈。

### 11.4 记账

本轮记两条 catch（`docs/harness-catch-log.md` 07-26 末两行）：① 执行线自查的"机械统计剔除边界错了，比不统计更糟"——本账本第一条"统计方法本身坏掉"；② 指导线复核的"第三次『我搜遍了、没有』经不起复搜"，含 11.3 的线索与反例。

### 11.5 仍开口

D1 的 A 层剩余 5 文件 / B 层 / C 层 / 598 warning 判定继续挂账；`a13599e` 的欠账**未还清**。07-23 走哪条路仍未实证收敛——下一步见 11.3 线索。
