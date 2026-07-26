# 任务包：首句失败——从症状反查来源（D2-R1 返工包）v1

- 日期：2026-07-26
- 状态：**待用户授权派发（DRAFT_AWAITING_DISPATCH）**
- 负责人：独立对话底座线执行线
- 指导/验收：当前总指导对话
- 上游：`tasks/2026-07-26-shared-supervisor-line-content-review-and-first-turn-binding-diagnosis-package-v1.md` 及其 evidence（含指导线 §8 裁决：D1 `PARTIAL` 接受、D2 头号结论 `REJECTED`）
- 目标 evidence：`evidence/2026-07-26-shared-supervisor-first-turn-symptom-traceback-v1.md`

## 0. Kickoff

上一包的 D2 结论错了，错法值得写清楚，因为返工的方法直接由它决定：

- 它断言「那句真机 UI 文案在当前代码里没有生产来源」——**只搜了前端**。前端那个 helper 确实是死码，但同一句话在**后端** `supervisor_resident_oneshot_session.rs` 的 **5 处生产区**产出，且由已注册命令 `submit_supervisor_resident_answer` 直达（指导线逐层核过，见上游 evidence §8.2）。
- 由此它推出"07-23 刻画不能当现状、下次按全新观测重跑"——**前提为假，推论作废**。
- 而它漏掉的那条路，恰恰就是本该找的折叠点：那 5 个分支的 `status` 与 `message` **完全相同**，其中两处前置是 `.is_err()`、错误值当场丢弃。

**指导线同时自认一半责任**：上一包 §6.1 预先把链路画成 binding 那条路，执行线照图施工才漏了另一条。**本包因此换方法：从症状反查来源，不预画链路。**

本包仍是**只读**：零产品写入、零真机、零 Codex CLI/MCP、零真实 store/vault，**不修 bug**。

## 1. 已确立的事实（不得重做、不得推翻，除非给出更强实证）

1. 那句文案的生产来源：`supervisor_resident_oneshot_session.rs:2773`（`recorded_resident_reply_outcome`，:2727）、`:2889 / :2950 / :2968 / :2990`（`submit_supervisor_resident_answer_with_parts`，:2847）；可达链 `command_registry.rs:166` → `_with`(:2829) → `_with_parts`(:2838)。该文件内 `#[cfg(test)]` 为单项属性，非模块边界。
2. binding 那条路**已经端到端可归因**：回执带 `binding_stage` + 每相人话文案；前端 `conversationTransport.ts:87 / 244-250 / 400` 已接。
3. 前端 `reconcileResidentMessageSubmission`（`useJiaobanConversationState.ts:45`）是死码：`src/` 内零调用点。该事实成立，但**不支持**"全仓无来源"的推论。
4. 上一包 D1 的 3 条发现成立：`commands.rs:455`、`commands.rs:400/419/433`、`supervisor_orchestrator.rs:1110`。
5. 07-22 R3 两个根因假设、07-22 R4a 字段对账、07-23 七相语义——**均未被推翻**，继续有效。

## 2. 方法（本包的核心，必须照此走）

**从症状反查，不沿假想链路走。** 具体：

1. 把 07-23 真机可观测到的**每一条症状**列成清单（至少：那句 UI 文案、`conversation_turn_binding=0`、`recorded +1`、`registry entries=0`、无 `thread.started`、无 `tools/list`）；
2. 对每条症状，**全仓反查其产出点**——搜索必须同时覆盖 `src/`、`src-tauri/`、`tests/`，并在 evidence 里贴出实际命令与命中行；
3. 每个产出点标注：所属函数、是否生产区（须核 `#[cfg(test)]` 是单项属性还是模块边界）、可达链是否通到某个已注册命令；
4. 只有走完 1-3，才允许谈"哪一相失败"。

**禁止**：先假定链路再找证据；把"某一半代码里搜不到"写成"全仓不存在"；在没有可达链的情况下断言某分支不可能触发。

## 3. D2-R1 合同

### 3.1 resident-answer 路的完整折叠点普查（必做）

对 `supervisor_resident_oneshot_session.rs`、`supervisor_session_launcher.rs` 及该路上其余生产文件，逐处标出折叠点，并对上一包的 `raw/folding-point-census.txt` **补全**（不是重做 8 个文件，是把漏掉的这条路加进去，并说明新旧表如何合并）。

重点回答：那 5 个返回同一 `status` + 同一 `message` 的分支，**外部到底能不能区分**：

| 分支 | 触发条件 | `reply_injected` | `thread_id` | 外部可区分？ |
| --- | --- | --- | --- | --- |
| `:2773` | | | | |
| `:2889` | | | | |
| `:2950` | | | | |
| `:2968` | | | | |
| `:2990` | | | | |

"可区分"必须以**调用方实际能读到的字段**为准（不是源码里能看出来就算）。

### 3.2 两条路的关系（必做）

说清楚：交办页发一句话时，**走的是 binding 那条路、resident-answer 那条路、还是两条都走**？给出可达链证据。若两条都可能，说明分流条件在哪。

这一条直接决定 07-23 那轮"binding=0 + 通用文案"是不是自洽——如果首句根本不走 binding 路，那 `binding=0` 就不是"binding 失败"，而是"binding 压根没被调用"，这两者的修法完全不同。

### 3.3 可观测性方案改写（必做）

把上一包 §4 的 8 项方案**按两条路重写**：

- resident-answer 路：那 5 个分支各自需要什么 `failure_family` 才能区分；
- binding 路：沿用已有 `binding_stage`，只补 §1.4 那 3 条发现造成的并流（12 变体压成一个、3 原因共用一 stage、`Update(_)` 报成构造失败）；
- 明确标注每项会不会碰到既有断言、会不会改变行为（本包只出方案，不动手）。

### 3.4 离线复现（尽力而为，允许判不出来）

在 §3.2 结论明确之后再做：沿**真正的**那条路用离线夹具走一遍，看能否复现 `status=message_recorded_supervisor_incomplete`。判不出来必须明写"判不出来 + 需要什么才能判"，**不得补猜**。

## 4. 不在本包内

- D1 的 A 层剩余 5 文件、B 层、C 层、598 warning 逐个判定——**继续挂账**，上一包的 evidence 不等于 `a13599e` 欠账已还清；
- 任何代码修复；
- 真机验证。

## 5. 边界与写入白名单

- 冻结：HEAD `a13599e`；**零产品写入**（`src` / `src-tauri` / `tests` 一个字节不许动）。
- 允许写：`evidence/raw/2026-07-26-shared-supervisor-first-turn-symptom-traceback/**`、`evidence/2026-07-26-shared-supervisor-first-turn-symptom-traceback-v1.md`、本包 §8 回填、`docs/harness-catch-log.md`（有真 catch 才追加）。
- 禁止 stage / commit / push；禁止启动真实 App / Codex CLI / MCP；禁止读写真实 store/vault/`~/.codex`。

## 6. 必须回传

1. 开工实核（HEAD / `git status` / 进程 / 端口）；
2. 症状清单 → 产出点反查表（含实际搜索命令与命中行，覆盖 `src/` + `src-tauri/` + `tests/`）；
3. §3.1 的 5 分支可区分性表（填满）；
4. §3.2 两条路的关系结论 + 可达链证据；
5. §3.3 改写后的可观测性方案；
6. §3.4 的实际结果（判不出来须明写）；
7. 与 §1 已确立事实的对照：有没有推翻其中任何一条（要推翻必须给更强实证）；
8. `git status` 收口证明；
9. 新 catch；没有则明写零新 catch；
10. 结论。

## 7. 结论枚举

- `PASS_D2R1_TRACEBACK`（症状反查完成、5 分支可区分性判明、两路关系说清）
- `PARTIAL_D2R1`（说明哪部分未完成及原因）
- `BLOCKED_D2R1_<原因>`
- 整包 `NEEDS_GUIDANCE_REVIEW`；不得自行开修、不得声称 binding 已修或对话线可重验。

## 8. 实际执行回填

- 状态：**已施工并回交指导线复核**（2026-07-26，用户明确"执行"）。evidence：`evidence/2026-07-26-shared-supervisor-first-turn-symptom-traceback-v1.md`。
- 结论：**`PASS_D2R1_TRACEBACK`**；整包 `NEEDS_GUIDANCE_REVIEW`。零代码改动（`src`/`src-tauri`/`tests` 一个字节未动）。
- 开工实核：HEAD `a13599e` ✅、无残留进程 ✅、`5173` 空闲 ✅；`git status` 另有 ` M CURRENT.md` 与 ` M docs/harness-catch-log.md`，**两条均非本线所改**。
- **认错**：上一包头号结论错在——自己证据里已写明"(c) 另有来源，本轮只 grep 了 src 与 tests"，却仍把强结论顶到标题。不是方法没给到，是知道搜索不完整还下断言。
- **本轮自查又抓到上一包第二个错，更严重**：折叠点普查用 `awk` 在首个 `#[cfg(test)]` 处整体截断，而该属性多数是单项属性。8 个文件里 6 个被静默丢弃大片生产代码（`commands.rs` 丢 85%、`supervisor_orchestrator.rs` 丢 69%）。已用 `raw/folding-point-census-v2.py` 重做：**`commands.rs` 真实折叠点 93 处（上次报 22）**，`supervisor_orchestrator.rs` 38（上次 18）。**上一包 evidence §3.2 那张表作废。**
- §3.1 五分支可区分性**表已填满**：调用方只能读到 `status`/`reply_injected`/`thread_id`/`supervisor_reply`/`message`，其中 status 与 message 五处逐字相同、supervisor_reply 五处全 `None`。5 个分支对外只有 **3 种签名**：`:2889` 与 `:2950` 同形、`:2773` 与 `:2968` 同形，只有 `:2990`（`reply_injected=true`）唯一可分。**最要命的是 `:2950`（首次发送、主管没接上）与 `:2889`（幂等重放找不到记录）完全不可分**。另逐处列出该路 4 个错误丢弃点，含 `:2937` `let _ =` 把**诊断写入自身的失败**整个丢掉。
- **§3.2 两条路关系（本轮最有价值）**：交办页首句走 **binding/transport 路**（`start_supervisor_conversation_transport`，`useJiaobanConversationState.ts` 的 `transportController` 在用）；**resident-answer 路的命令 `submit_supervisor_resident_answer` 在前端零调用点**。搜索为闭合式，专门补上了上次漏掉的两个口子：raw 命令名全 `src/` 搜、动态 `invoke` 搜，均为空。→ 上一包"07-23 刻画不能直接当现状"的**方向**仍成立，但**理由必须换掉**：不是"全仓无来源"（错），而是"产出该文案的那条路当前前端不可达"（可达链事实）。**不据此断言那轮就是旧构建**，仍需 receipt 原件或逐字抓取。
- §3.3 方案按两条路重写：resident-answer 路新增 9 项（5 分支各自 family + 诊断失败可见 + 三处 `delivery_unknown` 拆子 family + 断言任意两分支 family 不相等）；binding 路沿用上一包 6 项（复核后仍成立）；死码处置 1 项。**只出方案未动手。**
- §3.4 离线复现：**未做**，如实写明"判不出来 + 需要什么才能判"——那条路当前前端不可达，离线即使触发 `:2950` 也回答不了"交办页首句为什么失败"。不补猜。
- 与 §1 五条已确立事实对照：**一条都没推翻**，其中第 1 条已独立复核确认（含 `#[cfg(test)]` 单项属性判定）。新增一条 §1 未涵盖的事实：该命令前端零调用点（与 §1.1 的"Rust 侧可达链成立"不冲突）。
- 新 catch：**一条（本线自查）**——"机械统计的剔除边界错了，比没统计更糟，因为它看起来像证据"。
