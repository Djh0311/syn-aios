# 任务包：binding 路 failure_family 落地（B1–B6 + A 路两处补口）v1

- 日期：2026-07-26
- 状态：**待用户授权派发（DRAFT_AWAITING_DISPATCH）**
- 负责人：独立对话底座线执行线
- 指导/验收：当前总指导对话
- 上游：`evidence/2026-07-26-shared-supervisor-failure-family-observability-retrofit-verification-v1.md`（`ACCEPTED_PARTIAL_FAMILY_RETROFIT` + 指导线复核段 G.1–G.5）
- 目标 evidence：`evidence/2026-07-26-shared-supervisor-binding-path-failure-family-verification-v1.md`

## 0. Kickoff

上一包把 **resident-answer 路**的五个分支各给了名字并证明行为零变化（指导线自跑 `1203/0/45` 复核通过）。但**交办页首句走的是 binding 路**——那条路的 B1–B6 一项没落。所以现在的状态是：**病灶那条路能分辨了，实际会走的那条路还不能。**

本包补齐 binding 路，外加上一包留下的两处小口。做完之后，才谈真机首句重验。

### 0.1 沿用上一包已拍定的三条默认（不得自选）

1. **行为零变化**：控制流、成功/失败判定、写入面、审计事件一律不动；只改"失败对外怎么自报家门"。
2. **新字段一律可选加法**：`Option<String>` + `serde(default, skip_serializing_if)`，既有消费者与存量记录仍合法。
3. **UI 文案逐字不改**：family 只进回执与可读日志。

### 0.2 明确不做

- 不修 bug（为什么失败仍未知）；
- 不接线孤儿命令 `submitSupervisorResidentAnswer`、不删死码 `reconcileResidentMessageSubmission`；
- 不进真机（下一包，需用户在场）；
- 不为压行数做无关重构。

## 1. 施工依据（已确立，不得重查）

1. `ConversationTurnBindingError` 实测 **12 个变体**（`supervisor_conversation_binding.rs:131-147`）：`MissingField` / `InvalidProjectRoot` / `ProjectIdMismatch` / `InvalidRunIdentity` / `InvalidTimestamp` / `InvalidRuntimeLimit` / `RuntimeExpired` / `InvalidLifecycleTransition` / `InactiveLifecycle` / `ThreadUnbound` / `ContextMismatch` / `Capability`。
2. 三处并流：`commands.rs:455`（`Err(_)` 整体丢弃 12 变体）、`commands.rs:400/419/433`（3 种不同原因共用一 stage）、`mcp/supervisor_orchestrator.rs:1110`（`Update(_)` 报成 `BindingConstruct`）。
3. binding 路已有 `binding_stage` 端到端接线（回执 + `conversationTransport.ts:87/244-250/400`），**在其上补充，不重造、不改既有取值**。
4. A 路已落地的家族名与常量模块可直接参照命名风格（`resident_failure_family`）。

## 2. 写入白名单

### 2.1 B 路（主体）

| # | 位置 | 内容 |
| --- | --- | --- |
| B1 | `mcp/supervisor_conversation_binding.rs` | 给 `ConversationTurnBindingError` 加 `family()`：**12 个变体各一个固定常量**，两两不等；带负载的变体（`MissingField`/`InvalidLifecycleTransition`/`InactiveLifecycle`/`ContextMismatch`/`Capability`）**只取变体身份，不把负载拼进 family**（负载可另走既有 Display） |
| B2 | `commands.rs:455` | 不再 `Err(_)` 丢弃：接住 typed error，把 `family()` 带进回执 |
| B3 | `commands.rs:400 / 407 / 419 / 433` | **四处**各给自己的 family（context 解析失败 / 项目根不匹配 / 既有 thread 校验失败 / run_id 生成失败）；**既有 `binding_stage` 取值不动**〔**2026-07-27 指导线订正**：原文写「三处 `:400/419/433`」，漏了 `:407`，且把 `:419` 错标成「项目根不匹配」（实为 `verify_supervisor_existing_thread(...).is_err()`）。执行线核出并按四处落地，指导线复核确认。〕|
| B4 | `mcp/supervisor_orchestrator.rs:1110` | `DbPrimaryStoreUpdateError::Update(_)` 拆出 `binding_conflict` 语义；`Store(_)` 维持 `BindingStorePrepare`〔**2026-07-27 指导线订正**：原文「不再报成 `BindingConstruct`」与 §5.5「不得改既有 `binding_stage` 取值」自相矛盾——新增对外 stage 取值即改对外契约。**裁定按红线走**：内部拆 `BindingStoreConflict` 变体，**对外 stage 仍为 `BindingConstruct`**，分辨全交给 family。执行线未自行放宽而是回交请示，处置正确。〕|
| B5 | 回执结构（`commands.rs`） | 加 `binding_failure_family: Option<String>`；`binding_stage` 保留原样 |
| B6 | `src/lib/tauri.ts` + `src/lib/conversationTransport.ts` | 类型加可选字段并原样透传到既有诊断出口；**不改任何用户可见文案、不改既有 `binding_stage` 分支** |
| B7 | 合同断言 | 12 变体各断言其 family；**断言任意两者不相等**；断言 B3 三处 family 互不相等且与 B2 的不相等；断言字段缺失时反序列化仍成功 |

### 2.2 A 路两处补口（来自指导线 G.3 与上一包自报）

| # | 内容 |
| --- | --- |
| A9b | 上一包 A9 只断言了 13 个**基础**家族名两两不等，**未覆盖带 `RESIDENT_DIAGNOSTIC_APPEND_FAILED_SUFFIX` 的 13 个**。补齐：26 个取值两两不等 |
| A8* | 三处 `delivery_unknown`（`:2867 / 2880 / 2913` 附近，行号以当前文件为准）**先评估再决定**：它们返回 `Err(String)` 而非 outcome。若能在**不改错误类型、不改返回形状、不改控制流**的前提下给出子 family，则做；**若不能，按 §5 停止条件如实回交"做不了 + 为什么"，不得为落地它而改形状** |

### 2.3 新 evidence

`evidence/raw/2026-07-26-shared-supervisor-binding-path-failure-family/**`（含 baseline）、目标 evidence、本包 §8 回填、`docs/harness-catch-log.md`（有真 catch 才追加）。除此之外一律不写；禁止 stage / commit / push。

## 3. 冻结与基线副本

- HEAD：`a13599e`（工作树含上一包未提交的 A 路改动与文档，属已知状态）。
- **基线副本硬规矩**：改动前把全部窄写目标逐字节复制到 evidence raw 的 `baseline/` 并写 manifest；收口给出逐文件 diff 摘要。**无副本不得开工。**

| 文件 | 派发 SHA-256 | 权限 |
| --- | --- | --- |
| `src-tauri/src/mcp/supervisor_conversation_binding.rs` | `d9f066d2cb99b0707357ff633e3cf73e58eeb9f9498a868e9b3cb232590a57f9` | 窄写（B1） |
| `src-tauri/src/commands.rs` | `e9f98ea7c340c8f871e227505a962905298f345aebb7d5ddbccf904a78005126` | 窄写（B2/B3/B5） |
| `src-tauri/src/mcp/supervisor_orchestrator.rs` | `7238b2f0c229483b8a6bc8f43128319568df19de345a27bd59fcde635fc7c0bf` | 窄写（B4，仅 `:1110` 一带） |
| `src/lib/tauri.ts` | `95587bdd68c7e207e18d6ecdc2c862a260706c9aa7f5c3085b7dcf95d8dc14ee` | 窄写（B6 类型） |
| `src/lib/conversationTransport.ts` | `7f0a7cd82f1d814f13ba3e8d4cff88e6958c1abcc343b0726355fd7b81c15e96` | 窄写（B6 透传） |
| `src-tauri/src/supervisor_resident_oneshot_session.rs` | `97c9c36f64c7b48eefa4107b36421de4edf402a890336e23d016958c30774fa0` | 窄写（**仅 A8\*，且只在评估通过时**） |
| `src-tauri/src/supervisor_resident_oneshot_tests.rs` | `4402dd66b181c2868723f35609fddc1e5759ace7badde845403be876c5d71770` | 窄写（A9b） |
| `src/views/projects/jiaoban/useJiaobanConversationState.ts` | `b86a1dff8b75e8dcb72c746cb3876473ed09a1ad0f551ee472e2a433d33ca071` | **冻结只读** |
| `tests/shared-conversation-transport.test.tsx` | `0debae5bb479e24a3498c0c2265c386914dd01c034943a69cf278e3ec0acde7f` | 窄写（仅类型变更确需时） |

其余一律冻结只读；任一 hash 漂移即停。

> **2026-07-27 派发前订正**：`supervisor_orchestrator.rs` 一行原派发值为 `…c229483b6a8bc8f4…`（`6a8b`），系两字符换位的笔误；实测值为 `…c229483b8a6bc8f4…`（`8a6b`），文件本身自 `a13599e` 起未动（`git status` 对该文件为空）。已按实测值订正表中面值，其余 8 项面值与实测相符。

## 4. Red-first（这次不许再反）

**先写 B7 与 A9b 的断言、跑一次、留下"断言以失败形式跑起来"的输出，再动实现。**

上一包在这一步把顺序做反了，补证只能给出**编译错误**——强度不同。本包因此加一条硬要求：

- 红证据必须包含**至少一条真正跑起来后失败的断言**（例如先加"12 变体 family 两两不等"的断言，但让它读一个尚未实现的空实现/占位，从而以 `assertion failed` 形式失败）；
- 如果某条断言在实现落地前**必然编译不过**（如引用尚不存在的字段），如实说明并单独标注为"编译期红"，**不得与"运行期红"混为一谈**。

## 5. 硬合同与停止条件

1. **行为零变化**：改前/改后 `cargo test --lib` 的 passed/failed/ignored 三数对照，增量必须**逐条**对应本包新增断言；任何既有断言挂掉 → 立即停止上交。
2. family 全部是固定常量，两两不等；带负载变体不把负载拼进 family。
3. 新字段可选，缺失可反序列化。
4. UI 文案逐字不变（给对照）。
5. 不得改既有 `binding_stage` 取值/命名、不得删死码、不得接孤儿命令、不得为落地 A8\* 改错误类型或返回形状。
6. 需要动上述任一红线才能完成 → 停止并回交，**不要自行放宽**。

## 6. 必跑验证

`src-tauri`：`cargo check --lib`；`cargo test --lib` 全量（三数对照）。
`prototypes/productized-desktop-shell`：`npm run typecheck`；`npm run test:offline-interaction`（37 入口）。
仓库根：shape gate 两模式（finding 集合逐条相同、零新增；三个大 `.rs` 行数变化如实报，属既有 finding）；两个 selftest；`git diff --check`；staged 为空；回算冻结 hash + 基线 diff 摘要。

**计数口径提醒**（源自上一轮 G.2）：报 warning 数以 cargo 汇总行 `generated N warnings` 为准，**不要用 `grep -c "^warning:"`**——那会把汇总行本身算进去，凭空造出一个不存在的回归。

## 7. 必须回传

1. 开工实核 + 基线 manifest；
2. **红证据**（区分"运行期红"与"编译期红"，见 §4）；
3. B1–B7、A9b 逐项落点（文件:行 + 常量名）；A8\* 的评估结论（做了 / 做不了 + 为什么）；
4. §5 硬合同逐条证明（含三数对照、UI 文案对照）；
5. §6 全部门禁输出；
6. 新 catch；没有则明写零新 catch——**若声称"已入账"，须同时贴出账本对应行**（源自上一轮 G.4 订正）；
7. 结论。

## 8. 结论枚举

- `PASS_BINDING_FAMILY`（B1–B7 + A9b 落地、红转绿、行为零变化已证；A8\* 做了或已说明做不了）
- `PARTIAL_BINDING_FAMILY`（说明哪几项未落及原因）
- `BLOCKED_BINDING_FAMILY_<原因>`
- 整包 `NEEDS_GUIDANCE_REVIEW`；**不得**声称 bug 已修或可以进真机。

## 9. 下一步（不在本包内）

本包经指导线接受后，才是**一次真机首句重验**：读回执的 `binding_failure_family` / `failure_family` 一次归因，并核 `append_resident_delivery_diagnostic` 计数是否变化。**需用户在场并单独授权。**

## 10. 实际执行回填

**执行日期**：2026-07-27　**结论**：`PASS_BINDING_FAMILY` + 整包 `NEEDS_GUIDANCE_REVIEW`
**Evidence**：`evidence/2026-07-26-shared-supervisor-binding-path-failure-family-verification-v1.md`
**Raw**：`evidence/raw/2026-07-26-shared-supervisor-binding-path-failure-family/`（baseline 9 件 + manifest + gates + 全量 diff patch）

### 10.1 开工前

- §3 表中 `supervisor_orchestrator.rs` 派发 hash 为笔误（`6a8b` ↔ `8a6b` 两字符换位），文件本身未动；已按实测值订正表中面值并在表下留订正说明。
- 九项冻结件实核 **9/9 相符**；基线副本 + manifest 先于任何改动建立。
- 改前基准：`cargo check --lib` 0 error / `generated 598 warnings`；`cargo test --lib` **1203/0/45**。

### 10.2 落地情况

| 项 | 状态 | 备注 |
| --- | --- | --- |
| B1 | ✅ | `supervisor_conversation_binding.rs:199` `family()`，12 变体各一常量（`binding_rejected_*`），带负载只取变体身份 |
| B2 | ✅ | `commands.rs:499` `Err(_)` → `Err(error)`，12 变体不再被整体丢弃 |
| B3 | ✅（**四处**，非三处） | `commands.rs:437/445/459/473`；派发表计数有误，见 §10.4 catch 1 |
| B4 | ✅（口径见 evidence §5） | 新增内部变体 `BindingStoreConflict`；**对外 `binding_stage` 仍为既有 `BindingConstruct`**，拆分只走 family |
| B5 | ✅ | `binding_failure_family: Option<String>` + `skip_serializing_if`；既有 `supervisor_start_failure_receipt()` 签名未变 |
| B6 | ✅ | 全部落在 `conversationTransport.ts`（类型 + 投影 + 新投影闸）；**`tauri.ts` 零字节改动**（它只 import 该类型） |
| B7 | ✅ | Rust 5 条 + 前端 1 段 |
| A9b | ✅ | 26 个取值两两不等；**新增测试**，未改既有 A9 那条 |
| A8* | ✅ **做了** | 三处 `delivery_unknown` 给出以既有取值为前缀的子 family；错误类型/返回形状/控制流一字未动（全仓 grep 确认零消费者） |

### 10.3 红先行（§4）执行情况

- **运行期红**：✅ 先落空串占位实现，B7 断言以 `assertion failed`（`family 不得空串：index 0`）真跑失败。
- **编译期红**：✅ `commands.rs` 侧 17 条 `E0433/E0599/E0425`，**单独标注、未与运行期红混报**。
- **A9b 无红**：如实说明——纯覆盖补齐，首跑即绿；造红需改源码常量，**未造**。
- 另做 B6 剪线反证（事后，已单独标注，不冒充红先行）。

### 10.4 门禁与合同

- `cargo test --lib` **1203/0/45 → 1210/0/45**；`+7` 逐条对应新增断言，**消失测试 0、failed 恒 0**。
- `cargo check --lib` 0 error；warning **598 → 598**，且 **warning 全集逐条 diff = IDENTICAL**。
- `npm run typecheck` / `test:offline-interaction`（37 入口）全过；shape gate 两模式 **finding 集合 27 条逐条相同、零新增**（用基线副本换回工作树跑出真实改前一侧）；5 项 selftest 全过；`git diff --check` 干净；staged 空。
- 冻结只读件 `useJiaobanConversationState.ts` **零漂移**；`tauri.ts` 未改动。
- UI 文案：删除行共 11 行**零中文**，生产文案函数与基线逐字节相同。
- 三条红线全部守住（既有断言零挂 / `binding_stage` 取值命名三处 diff 全空 / 死码与孤儿命令未动）。

### 10.5 新 catch（2 条，已入账 `docs/harness-catch-log.md` 2026-07-27 段）

1. 派发表 B3「三处」实为四处，且行号与括注不自洽（`:419` 被错标、`:407` 被漏）。
2. 同一个数字 599：上一轮是假象、本轮是真回归；**上一轮的结论不得当本轮的判据**，warning 变化一律以全集 diff 定性。

### 10.6 未做 / 待裁

- **未进真机**、零 stage/commit/push、不修 bug。
- 请指导线裁两点：① B4 口径（stage 不动、语义走 family，evidence §5）；② catch 1 的「四处而非三处」是否照收。
