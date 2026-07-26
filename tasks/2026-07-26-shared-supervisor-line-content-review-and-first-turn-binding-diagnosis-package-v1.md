# 任务包：对话/主管线内容复核 + 首句 binding 断点诊断（只读合并包）v1

- 日期：2026-07-26
- 状态：**待用户授权派发（DRAFT_AWAITING_DISPATCH）**
- 负责人：独立对话底座线执行线
- 指导/验收：当前总指导对话
- 目标 evidence：`evidence/2026-07-26-shared-supervisor-line-content-review-and-first-turn-binding-diagnosis-v1.md`

## 0. Kickoff

两件事天生是同一件：`a13599e` 那笔把对话/主管线的 **34 个后端文件**连同知识线一起落库，指导线在提交信息里明写"**只过了编译 + 自身测试 + 风险扫描，没有逐行读**"——这是当前唯一挂账的技术欠账；而产品主线真正卡住的那个 bug（**首句没有 durable binding**）就住在同一批文件里。分开做等于同一批代码读两遍；合起来是带着问题读，更锐利。

- **D1 内容复核**：把这批代码在**内容层面**过一遍，补掉 `a13599e` 标注的欠账，交发现清单（不修）。
- **D2 首句 binding 断点诊断**：把 host → binding 构造 → 持久化 → transport start 这条链读透，标出**每一处错误折叠点**，并在离线夹具上尽可能把 07-23 真机首句的失败收窄到具体相；离线判不出来的，交出"最小可观测性改造方案"，让**下一次真机一次就能归因**。

**本包零产品写入、零真机、零 Codex CLI/MCP、零真实 store/vault。** 只读代码 + 离线夹具 + 写 evidence。

## 1. 已知（读过原件，不得重做）

### 1.1 真机失败的确切形状（`evidence/2026-07-23-shared-conversation-transport-real-app-substitution-acceptance-v1.md`）

- 首句只发一次；页面出现用户消息，随后显示"消息已送到主管，但主管这次没回上来"。
- 最早可证失败面：**主管 conversation turn binding 没有形成可持久化事实**。发送后 / 退出前 / 退出后，JSON 与 SQLite 都保持 `sessions=25`、`conversation_turn_binding=0`。
- 首句只证明 canonical `recorded +1`；`supervisor transport`、`thread.started`、Active binding、首次 MCP `tools/list` 均无证据。
- **该 evidence 自己写明为什么当时停在这里**：「公开命令会把 binding 构造/持久化内部错误折叠，前端又把 start rejection 收敛为通用失败，因此不能诚实地继续归因为某一个 DB、校验、projection 或 `thread.started` 子分支。」→ **这正是 D2 要打开的东西。**

### 1.2 已被排除 / 已固定的（不得当成新发现重报）

- **07-22 R3 injection 诊断**：假设 A（单一代码根因）与 B（首次根因 + 后续 fail-closed）**均已判不成立**——没有共享的稳定 error family、私有 artifact 或 message-scoped terminal record。**不得再把某个源码候选当根因猜**。
- **07-22 R4a DB/JSON 对账**：6 个 serde-default 字段在 JSON 与 DB raw record 中 `74/74` 均存在，`tasks` 两侧一致 → 已排除本轮可安全统计到的缺字段 / 空 `tasks` / 归属差异。
- **07-23 binding 相位语义与失败收口返工**：construct、store prepare、DB-primary persist、JSON projection、activate、transport start、terminate-unconfirmed 各相语义已固定；transport/activate 失败**仅在 JSON 与 SQLite 都确认 `Failed` 后**才这样标记。
- 前序私有副本 `25/0 → 26/1` **只证明 `Starting` 写入**，不证明整链成立。
- 真机那轮 binding 计数是 `0` —— 即**连 `Starting` 都没落**。D2 的第一个问题就是：`Starting` 写入在真机为什么没发生。

## 2. Authority 与边界

- authority_chain：`AGENTS.md` → `CURRENT.md` → 07-19/07-22/07-23 对话线 evidence → `a13599e` 提交信息中的欠账标注 → 本包。
- capabilities_touched：none。**不改任何产品代码、测试、依赖、配置**。
- 本包**不**修 bug：诊断出根因也只写进 evidence，修复归下一包。
- 并发：本包纯读 + 离线夹具，可与任何非写包并行；但不得与真机包同时跑。

## 3. 开工前置

- HEAD：`a13599e`（工作树已清）
- 开工实核：`git status --short` 为空；无残留 Syn / cargo-tauri / Vite；`5173` 空闲。
- 收口时 `git status` 只应多出本包 evidence 与任务包回填——**多出任何代码改动即为违规**。

## 4. 精确写入白名单

1. `evidence/raw/2026-07-26-shared-supervisor-line-review-and-binding-diagnosis/**`（离线夹具脚本、原始输出、折叠点清单等）
2. `evidence/2026-07-26-shared-supervisor-line-content-review-and-first-turn-binding-diagnosis-v1.md`
3. 本任务包 §12 回填
4. `docs/harness-catch-log.md`：仅发现新的真实 catch 时追加，否则明写零新 catch

除此之外一律不写；禁止 stage / commit / push。

## 5. D1 合同：内容复核（分层，不假装全深读）

复核对象 = `a13599e` 落库的对话/主管线 34 个后端文件及其前端/测试改动。**按风险分三层，每个文件必须明确标注属于哪一层**，不得把浏览过说成读过：

- **A 层（逐行读）**：首句链上的承重文件 —— `mcp/supervisor_conversation_binding.rs`（800 行）、`manual_relay/conversation_transport.rs`（1457）、`mcp/supervisor_orchestrator_db_primary.rs`（370）、`mcp/supervisor_orchestrator_submit_proposal.rs`（345）、`mcp/storage.rs`（462）、`mcp/capability_registry.rs`；
- **B 层（结构读 + 定点深入）**：`mcp/supervisor_orchestrator.rs`（3715）等超大文件 —— 读清模块结构与公开面，对 D2 相关路径逐行深入，其余标"结构级"；
- **C 层（风险扫描）**：其余文件 —— 只做安全面与写路扫描（凭据 / `.codex` / 删除 / 沙箱 / 审批 / 外部网络），标"扫描级"。

每层都必须回答同一组问题并逐条给证据（文件:行）：

1. **写路**：这个文件有没有写真实 store/vault/`~/.codex`？写之前有没有边界校验？
2. **失败语义**：错误是被向上传递、被折叠成通用失败、还是被静默吞掉？折叠点逐个记（这与 D2 共用）。
3. **只读声明是否属实**：凡自称只读的路径，是否真的没有写调用（按 `#[cfg(test)]` 内外分别统计）。
4. **死码 / 空承诺**：有没有定义了却没接线的常量/函数/字段（R3E 抓到过 `LAYOUT_PITCH` 定义了不用；`cargo check` 已报 598 个 warning，其中"never used"族要逐个判定是"待接线"还是"该删"）。
5. **测试是否真的锁住语义**：断言是不是只查字符串包含、能不能被无关改动绕过。

输出：**发现清单**，每条含 文件:行 / 属于哪层 / 严重度（阻断 / 应修 / 记账 / 无需动）/ 一句话理由。**不修**。

## 6. D2 合同：首句 binding 断点诊断

### 6.1 折叠点普查（核心产出，必须完整）

沿 **host 收到首句 → binding 构造 → store prepare → DB-primary persist → JSON projection → activate → transport start** 这条链，逐处标出：

- 每一个 `map_err` / `unwrap_or` / `ok()` / `let _ =` / 通配 `Err(_)` / 把 typed error 压成 `String` 或布尔的地方；
- 每一处前端把具体 rejection 收敛成通用文案的地方（`conversationTransport.ts`、`ProjectJiaobanPanel.tsx`、`useJiaobanConversationState.ts`）;
- 对每个折叠点标注：**折叠掉之后，外部还能不能分辨是哪一相失败**（能 / 不能）。

"不能"的那些，就是 07-23 evidence 说"不能诚实归因"的物理原因，必须列全。

### 6.2 离线复现（尽力而为，允许判不出来）

用离线夹具沿同一条链走一遍，重点回答：

1. 真机那轮 `conversation_turn_binding=0` —— **`Starting` 记录本该在哪一步写、由谁调用**？该调用在离线夹具里能否被触发？
2. 如果离线能触发而真机没触发，前面缺的是哪一个前置条件（session 可用性？registry entries=0？可信 binding 前置？）——**注意 registry `entries=0` 是真机那轮的实测值，值得单独查它是不是前置条件**。
3. 若离线**无法**判定，如实写"离线判不出来"，**不得猜**（07-22 R3 已经因此判过两个假设不成立）。

### 6.3 最小可观测性改造方案（若 6.2 判不出来则必交）

给出一份**下一包可直接实施**的最小改造方案，使下一次真机首句**一次就能归因**：

- 每一相失败对外暴露稳定的 `failure_family`（不改行为、不改写路，只让失败可分辨）；
- 前端把该 family 原样呈现或至少落进可读日志，不再收敛成一句通用文案；
- 明确列出：要改哪些文件、加哪些字段、需要哪些新断言、会不会动到已验收合同。

**只出方案，不动手改。**

## 7. 必须回传

1. 开工实核（HEAD / `git status` / 进程 / 端口）；
2. D1 三层清单：每个文件属于哪层 + 发现清单（文件:行 / 严重度 / 理由）；
3. D2 折叠点全表（含"折叠后能否分辨"一列）；
4. D2 离线复现的实际结果，含判不出来的部分——**判不出来必须明写，不得补猜**；
5. 若 6.3 触发：最小可观测性改造方案；
6. 与 §1.2 已排除项的对照：本轮有没有推翻其中任何一条（若推翻，须给证据）；
7. `git status` 收口证明（应只多出 evidence 与本包回填）；
8. 新 catch；没有则明写"零新 catch"；
9. 结论（§8 枚举）。

## 8. 结论枚举

- `PASS_D1_CONTENT_REVIEW`（复核完成，清单已交）/ `PARTIAL_D1`（说明哪些文件未覆盖及原因）
- `PASS_D2_BINDING_ATTRIBUTED`（离线已收窄到具体相，附证据）
- `PASS_D2_OBSERVABILITY_PLAN`（离线判不出来，已交最小改造方案）
- `BLOCKED_D2_<原因>`
- 整包：`NEEDS_GUIDANCE_REVIEW`；**不得**自行开修、不得据此声称 binding 已修或对话线可重验。

## 9. 立即停止条件

- 需要改任何产品代码/测试/配置才能继续；
- 需要启动真实 App、Codex CLI、MCP server，或读写真实 store/vault/`~/.codex`；
- 需要发任何主管消息或触发写命令；
- `git status` 出现代码改动；
- 想把某个候选当根因写死却没有证据 —— 立即停手，改写"判不出来 + 需要什么才能判"。

## 10. 施工后的下一步（不在本包内）

D1 的发现清单与 D2 的结论会分流成：① 值得修的窄修包；② 记账留待以后的项；③ 若走 6.3，则下一包是"最小可观测性改造 + 一次真机首句重验"（真机部分需用户单独授权、且要用户在电脑前）。

## 11. 实际执行回填

- 状态：**已施工并回交指导线复核**（2026-07-26，用户明确"执行这个任务"）。evidence：`evidence/2026-07-26-shared-supervisor-line-content-review-and-first-turn-binding-diagnosis-v1.md`。
- 结论：**`PARTIAL_D1`** + **`PASS_D2_OBSERVABILITY_PLAN`**；整包 `NEEDS_GUIDANCE_REVIEW`。
- 开工实核：HEAD `a13599e` ✅、无 Syn/cargo-tauri 残留 ✅、`5173` 空闲 ✅；`git status --short` 3 行（` M CURRENT.md` **非本线所改**，加 I5 与本包两个任务包文件）。本包零代码写入。
- **D1 只做完一部分，如实标注**：逐行读完 `supervisor_conversation_binding.rs`（800 行）；定点读完 `commands.rs` 首句链段与 `supervisor_orchestrator.rs` binding 段。A 层其余 5 个文件、B 层全部、C 层全部、598 warning 逐个判定**一条未做**，未覆盖清单见 evidence §2.3。不得把本 evidence 当作 `a13599e` 欠账已还清。
- D1 已读部分交出 8 条发现（evidence §2.2），其中 3 条「应修」：①`commands.rs:455` 把 12 变体 typed error 整个丢弃压成单一 `BindingConstruct`；②`commands.rs:400/419/433` 另有 3 种不同原因共用同一 stage（同一 stage 并进 ≥5 种原因）；③`supervisor_orchestrator.rs:1110` 把 `Update(_)`（含"run 已绑定其他 turn"业务冲突）报成 `BindingConstruct`，语义碰撞。
- D2 折叠点普查完成（8 个生产文件的机械计数 + 链上逐处「折叠后能否分辨」判定），原始输出 `raw/folding-point-census.txt`。
- **本轮最硬的一条（部分推翻 §1.2 前提）**：后端**其实已经端到端暴露相位**——`binding_stage` 进了 receipt（`commands.rs:650-663`）、有 7 个相位与各自人话文案，前端也确实接了（`conversationTransport.ts:87/244/249/400`）。而 07-23 真机记录的那句「消息已送到主管，但主管这次没回上来」，其产出函数 `reconcileResidentMessageSubmission` **生产零调用点、只有测试在调**。→ **07-23 的失败刻画不能直接搬到 `a13599e` 当现状**；下一次真机首句应按全新观测对待。三种可能性（旧构建 / 转述而非逐字 / 另有来源）本轮离线无法判定，如实写明未判。
- **D2 §6.2 离线复现：未做**，`registry entries=0` 前置条件：未查。原因与建议的顺序调整见 evidence §3.4。不补猜。
- §6.3 最小可观测性改造方案已交（evidence §4，8 项，含要动哪些文件、加哪些字段、新断言、以及第 4/7 项会碰到既有断言的提示）。**只出方案，未动手。**
- 与 §1.2 已排除项对照：**一条都没推翻**（逐条见 evidence §5）；推翻的只是"当前失败长什么样"这个前提。
- 新 catch：**一条**——legacy 助手生产已不接线，其文案却被当作真机现状证据引用（"测试锁住了一个生产不存在的行为"）。证据已留全，是否入账由指导线定。
