# 验证证据：binding 路 failure_family 落地（B1–B7 + A9b + A8*）v1

- 日期：2026-07-27（派发日期 2026-07-26）
- 任务包：`tasks/2026-07-26-shared-supervisor-binding-path-failure-family-package-v1.md`
- 上游：`evidence/2026-07-26-shared-supervisor-failure-family-observability-retrofit-verification-v1.md`（A 路 `ACCEPTED_PARTIAL_FAMILY_RETROFIT`）
- 结论：**`PASS_BINDING_FAMILY`** + 整包 **`NEEDS_GUIDANCE_REVIEW`**
- **未进真机。** 本包不声称 bug 已修，也不声称可以进真机。

---

## 1. 开工实核 + 基线 manifest

### 1.1 派发表笔误订正（开工前）

§3 冻结表里 `src-tauri/src/mcp/supervisor_orchestrator.rs` 的派发 hash 为
`7238b2f0c229483b6a8bc8f4…`（`6a8b`），与实测 `…c229483b8a6bc8f4…`（`8a6b`）差两个字符换位。
该文件 `git status` 为空、仍是 `a13599e` 版本，**文件没动、是表写错了**。已按实测值订正表中面值并在表下留订正说明。
照面值走会误报 `BLOCKED_..._BASELINE_DRIFT` 停在门口。

### 1.2 九项冻结件实核（改动前）

9/9 与派发表（订正后）逐字相符：

| 文件 | 实测 SHA-256 | 权限 |
| --- | --- | --- |
| `src-tauri/src/mcp/supervisor_conversation_binding.rs` | `d9f066d2…90a57f9` | 窄写 B1 |
| `src-tauri/src/commands.rs` | `e9f98ea7…78005126` | 窄写 B2/B3/B5 |
| `src-tauri/src/mcp/supervisor_orchestrator.rs` | `7238b2f0…5fc7c0bf` | 窄写 B4 |
| `src/lib/tauri.ts` | `95587bdd…5d8dc14e` | 窄写 B6 |
| `src/lib/conversationTransport.ts` | `7f0a7cd8…7b81c15e` | 窄写 B6 |
| `src-tauri/src/supervisor_resident_oneshot_session.rs` | `97c9c36f…30774fa0` | 窄写 A8* |
| `src-tauri/src/supervisor_resident_oneshot_tests.rs` | `4402dd66…76c5d711` | 窄写 A9b |
| `src/views/projects/jiaoban/useJiaobanConversationState.ts` | `b86a1dff…33ca071` | **冻结只读** |
| `tests/shared-conversation-transport.test.tsx` | `0debae5b…8acde654` | 窄写（类型变更确需） |

### 1.3 基线副本

`evidence/raw/2026-07-26-shared-supervisor-binding-path-failure-family/baseline/**` 逐字节副本 9/9，
manifest：`.../baseline-manifest.sha256`（9 行，与上表逐条相同）。**先有副本才动第一个字节。**

### 1.4 改前基准

| 项 | 改前 |
| --- | --- |
| `cargo check --lib` | exit 0；0 error；`generated 598 warnings` |
| `cargo test --lib` | **1203 passed / 0 failed / 45 ignored**（exit 0） |

> 1203 而非 1200：1200 是 A 路落地**之前**的数；A 路改动在工作树未提交状态，本包基准含它。

---

## 2. 红证据（§4：区分运行期红与编译期红）

### 2.1 运行期红（§4 硬要求，至少一条）✅

先给 `ConversationTurnBindingError::family()` 落一个**返回空串的占位实现**，再写 B7 断言，
让断言**编译通过、真正跑起来、以 `assertion failed` 失败**：

```
test mcp::supervisor_conversation_binding::tests::binding_error_families_are_fixed_constants_and_pairwise_distinct ... FAILED

thread '...binding_error_families_are_fixed_constants_and_pairwise_distinct' panicked at
src/mcp/supervisor_conversation_binding.rs:847:13:
family 不得为空串：index 0

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 1248 filtered out
```

原文：`raw/gates/red-runtime-b7.txt`。这是 panic 于断言，不是编译错误。

**同批次如实说明**：同轮的 `binding_error_family_takes_variant_identity_without_payload`
在占位实现下**通过**（12 个 family 全是空串时，「同变体不同负载 family 相同」与
ASCII 校验都被空串真空满足）。它只有在实现落地后才有区分力，**不计入运行期红**。

### 2.2 编译期红（单独标注，不与运行期红混报）

`commands.rs` 侧断言引用尚不存在的字段/函数/枚举变体，**必然编译不过**：

```
9 × error[E0433]: cannot find module or crate `binding_failure_family` in this scope
3 × error[E0599]: no variant or associated item named `BindingStoreConflict` found for enum
                 SupervisorConversationBindingEstablishmentError
3 × error[E0425]: cannot find function `supervisor_binding_failure_family_for_establishment_error`
1 × error[E0425]: cannot find function `supervisor_start_failure_receipt_with_family`
1 × error[E0425]: cannot find function `binding_error_variants_for_test`
→ could not compile (lib test) due to 17 previous errors
```

原文：`raw/gates/red-compile-b7-commands.txt`。**强度低于运行期红，故单列。**

### 2.3 A9b 无红（如实说明，不伪造）

A9b 是**纯覆盖补齐**，不改任何实现。新断言首跑即绿：

```
test ...resident_consult_families_including_diagnostic_append_suffix_are_pairwise_distinct ... ok
test result: ok. 1 passed; 0 failed; ... 1250 filtered out
```

原文：`raw/gates/a9b-first-run.txt`。要让它变红只能去改动源码常量——**那是造红，不做**。
它仍有区分力（锁住后缀非空；后缀一旦为空，带/不带后缀两组立即撞名）。

### 2.4 B6 剪线反证（事后，非红先行，单独标注）

B6 的「透传」若失效则整条前端链路静默失效，故做一次剪线复验：
临时删掉 `safeConversationTransportReceipt` 里的 family 投影行后跑该测试——

```
Error: binding_failure_family 必须原样穿过 receipt 投影
    at assert (tests/helpers/offlineInteractionTestUtils.tsx:81:11)
```

原文：`raw/gates/b6-cutwire-proof.txt`。随即还原，`conversationTransport.ts` hash 复核一致。
**这是事后反证，不冒充红先行。**

---

## 3. 逐项落点

### 3.1 B 路

| # | 落点（文件:行） | 内容 |
| --- | --- | --- |
| B1 | `mcp/supervisor_conversation_binding.rs:199` `fn family()` | 12 变体各一固定常量，前缀 `binding_rejected_`：`…_missing_field` / `…_invalid_project_root` / `…_project_id_mismatch` / `…_invalid_run_identity` / `…_invalid_timestamp` / `…_invalid_runtime_limit` / `…_runtime_expired` / `…_invalid_lifecycle_transition` / `…_inactive_lifecycle` / `…_thread_unbound` / `…_context_mismatch` / `…_capability`。5 个带负载变体一律 `(_)` / `{ .. }` **只取变体身份**，负载仍只走既有 `Display` |
| B1 | 同文件 `:223` `binding_error_variants_for_test()` | `#[cfg(test)]` 窥视口，供两处断言全覆盖 |
| B2 | `commands.rs:499-503` | 原 `Err(_)` 改为 `Err(error)`，`error.family()` 进回执。**12 变体不再被整体丢弃** |
| B3 | `commands.rs:437/445/459/473` | **四处**各给自己的 family（详见 §7 catch：派发表写的是三处）：`PRECHECK_CONTEXT_UNRESOLVED` / `PRECHECK_PROJECT_ROOT_MISMATCH` / `PRECHECK_EXISTING_THREAD_REJECTED` / `PRECHECK_RUN_IDENTITY_UNAVAILABLE`。**既有 `binding_stage` 取值一个不动** |
| B4 | `mcp/supervisor_orchestrator.rs:1094` 新增变体 `BindingStoreConflict`；`:1115` `Update(_)` 改映射到它 | `Store(_)` 维持 `BindingStorePrepare` 未动 |
| B4 | `commands.rs:653` stage 映射 / `:678` family 映射 | **stage 仍映射回既有 `BindingConstruct`**（对外契约零变化），只有 family 是 `binding_establish_store_conflict`。见 §5 对 B4 的口径说明 |
| B5 | `commands.rs:198` | `binding_failure_family: Option<String>` + `#[serde(skip_serializing_if = "Option::is_none")]`；`binding_stage` 保留原样 |
| B5 | `commands.rs:204` `mod binding_failure_family` | 13 个固定常量集中定义 |
| B5 | `commands.rs:766` `supervisor_start_failure_receipt_with_family()` | 新增；**既有 `supervisor_start_failure_receipt()`(`:759`) 签名不变**，改为委派并按 stage 取默认 family。既有测试与调用点零改动 |
| B6 | `src/lib/conversationTransport.ts:90` | 类型加 `binding_failure_family?: string \| null` |
| B6 | 同文件 `:404` | 在 `safeConversationTransportReceipt` 投影中透传 |
| B6 | 同文件 `:432` `safeBindingFailureFamily()` | 与既有 `safeSupervisorBindingStage` 同姿态的投影闸：只放行 `/^binding_[a-z0-9_]{1,56}$/`，raw 后端错误串与 payload 一律丢弃 |
| B6 | `src/lib/tauri.ts` | **零字节改动**——它 `import type { ConversationTransportReceipt }`（`:14`），类型集中在 `conversationTransport.ts`，无需也不应在此重复声明。hash 与派发值相同 |
| B7 | `supervisor_conversation_binding.rs` 两个 test / `commands.rs` 三个 test / `tests/shared-conversation-transport.test.tsx` 第 5 段 | 见 §4 |

### 3.2 A 路补口

| # | 落点 | 结论 |
| --- | --- | --- |
| A9b | `supervisor_resident_oneshot_tests.rs:3615` | 13 相位 × 带/不带 `__diagnostic_append_failed` 后缀 = **26 个取值两两不等**，另断言后缀本身非空。**新增测试，不改既有 A9 那条**（避免既有断言被动） |
| A8* | `supervisor_resident_oneshot_session.rs:2989 / 3002 / 3038` | **做了**，见下 |

**A8* 评估与结论：做了。**

先评估：三处返回的是 `Err(String)` 而非 outcome，拿不到 `failure_family` 通道。
问题是能否在**不改错误类型、不改返回形状、不改控制流**的前提下给出子 family。

实核：全仓 grep `delivery_unknown`（`.rs`/`.ts`/`.tsx`/`.json`，排除 `node_modules`/`target`）
**只有这三处产生点，零消费者**——无测试断言其取值，前端也没有任何引用
（`submit_supervisor_resident_answer` 的前端包装 `tauri.ts:1899` 至今仍是孤儿，无调用点）。

因此把三个取值改为**以既有取值为前缀**的三个子 family：
`…_delivery_unknown__replay_lookup` / `…__replay_reply_outcome` / `…__record_recheck`。
错误类型仍是 `String`、返回形状仍是 `Result<Outcome, String>`、`?` 早退位置一字未动，
按 `starts_with` 消费的旧逻辑仍然命中。断言见 `supervisor_resident_oneshot_tests.rs:3662`。

**未为落地它改任何形状。**

---

## 4. §5 硬合同逐条证明

### ① 行为零变化（三数对照）

| | passed | failed | ignored |
| --- | --- | --- | --- |
| 改前 | 1203 | 0 | 45 |
| 改后 | **1210** | **0** | **45** |

增量 `+7`，**逐条对应本包新增断言**（测试名集合 diff，`comm` 逐行比对）：

1. `mcp::supervisor_conversation_binding::tests::binding_error_families_are_fixed_constants_and_pairwise_distinct`（B7）
2. `mcp::supervisor_conversation_binding::tests::binding_error_family_takes_variant_identity_without_payload`（B7）
3. `conversation_transport_command_tests::binding_failure_families_are_pairwise_distinct_across_the_whole_path`（B7）
4. `conversation_transport_command_tests::binding_store_conflict_keeps_its_stage_but_no_longer_shares_construct_family`（B4/B7）
5. `conversation_transport_command_tests::binding_failure_family_rides_the_receipt_and_is_omitted_when_absent`（B5/B7）
6. `supervisor_session_launcher::resident_session_tests::resident_consult_families_including_diagnostic_append_suffix_are_pairwise_distinct`（A9b）
7. `supervisor_session_launcher::resident_session_tests::resident_delivery_unknown_subfamilies_are_distinct_and_keep_the_legacy_prefix`（A8*）

**消失的测试：0 条**（`comm -23` 为空）。**failed 恒为 0**，无既有断言被动。

### ② family 全部固定常量、两两不等、带负载不拼负载

- 12 binding 变体：`binding_error_families_are_fixed_constants_and_pairwise_distinct` 逐个锁死取值 + 两两不等 + 非空。
- 全路径 24 个取值（4 precheck + 12 变体 + 5 establishment + 3 建立后阶段）两两不等：`binding_failure_families_are_pairwise_distinct_across_the_whole_path`。
- 带负载不拼负载：`binding_error_family_takes_variant_identity_without_payload` —— 同变体给**不同**负载，断言 family 逐字相同；并断言负载仍能从既有 `Display` 拿到（`MissingField("project_id").to_string()` 含 `project_id`）；并断言 family 全为 ASCII snake_case 机器令牌。
- A 路 26 个 consult 取值两两不等（A9b）；A8* 三个子 family 两两不等且保留既有前缀。

### ③ 新字段可选、缺失可反序列化

`binding_failure_family_rides_the_receipt_and_is_omitted_when_absent`：
`None` 时 `serde_json` 输出**整键省略**；存量回执（无该键）仍合法。
前端侧：`tests/shared-conversation-transport.test.tsx` 第 5 段断言无该键的存量回执仍合法且不凭空造值，
且不合规取值（raw 后端串、中文文案、数字、对象）一律被投影为 `null`。

### ④ UI 文案逐字不变

- 全部**删除行共 11 行，零中文**（`grep -P '[\x{4e00}-\x{9fff}]'` 对删除行为空）。
- 生产文案函数 `supervisor_start_failure_human_message` 与基线**逐字节相同**（`diff` 为空）。
- 唯一新增的含中文 `human_message` 行位于**新测试夹具**内（`commands.rs` 测试里的 `json!` 存量回执），非生产文案。
- 前端断言 `result.operation_error === "主管对话绑定准备未完成；运输没有启动。"` 逐字锁死。

### ⑤ 三条红线

| 红线 | 结果 |
| --- | --- |
| 既有断言挂掉即停 | **零挂**：failed 0、消失测试 0 |
| 不改既有 `binding_stage` 取值/命名 | Rust `enum SupervisorConversationBindingStage`、TS `SupervisorConversationBindingStage` 联合、`safeSupervisorBindingStage()` 三者与基线 **`diff` 全为空** |
| 不删死码 / 不接孤儿命令 | `reconcileResidentMessageSubmission` 仍在 `useJiaobanConversationState.ts:45`（该文件 hash 零漂移）；`submitSupervisorResidentAnswer` 全 `src/` 仍**只有定义处** `tauri.ts:1899`，零调用点 |
| 不为 A8* 改错误类型/返回形状/控制流 | 见 §3.2 |

---

## 5. B4 的口径说明（重要，请指导线裁）

包 §2.1 B4 写「`Update(_)` 拆出 `binding_conflict` 语义，**不再报成 `BindingConstruct`**」，
而 §5.5 红线写「**不得改既有 `binding_stage` 取值/命名**」。二者在字面上会撞车：
若让 conflict 走一个新的 `binding_stage` 取值，就等于给对外契约新增了枚举值（前端联合类型、
投影闸、既有断言都要跟着动），直接踩红线。

采用的口径：**在 `SupervisorConversationBindingEstablishmentError` 上拆出 `BindingStoreConflict` 变体**
（内部枚举，不是对外 stage），`Update(_)` 映射到它；对外
**`binding_stage` 仍是既有的 `BindingConstruct`，取值域一个不加**，
**分辨完全交给新的 family `binding_establish_store_conflict`**。

即：「不再报成 `BindingConstruct`」按**语义/family 维度**落实，而非按 stage 取值落实。
锁死这条口径的断言：`binding_store_conflict_keeps_its_stage_but_no_longer_shares_construct_family`
（同时断言 stage **等于** `BindingConstruct`、family **不等于** construct 的 family）。

若指导线认为 B4 本意就是要新增一个对外 stage 取值，请明示——那需要放宽 §5.5 红线，本包不自行放宽。

---

## 6. §6 门禁

| # | 项 | 结果 |
| --- | --- | --- |
| 1 | `cargo check --lib` | 改前 exit 0 / 改后 **exit 0**；0 error；warning **598 → 598** |
| 2 | `cargo test --lib` | **1203/0/45 → 1210/0/45**（+7 逐条对应，见 §4①） |
| 3 | `npm run typecheck` | exit 0 |
| 4 | `npm run test:offline-interaction` | exit 0，**37 入口**全过 |
| 5 | shape gate `--mode baseline` | exit 0 |
| 5 | shape gate `--mode check` | exit 1（既有欠账），**finding 集合与改前逐条相同、零新增**，见下 |
| 6 | selftest ×5 | dedup `8/8`、hardcoded-hex `13/13`、machine-face `18/18`、retired-style-family `13/13`、checkpoint-audit `45/45`，全 exit 0 |
| 7 | `git diff --check` | 干净（exit 0） |
| 7 | staged | **空** |
| 8 | 冻结 hash 回算 + 基线 diff 摘要 | 见 §6.3 / §6.4 |

### 6.1 warning 计数口径（§6 提醒 + 本轮真事故）

以 cargo 汇总行为准：改前 `generated 598 warnings` → 改后 `generated 598 warnings`。
进一步**逐条 diff 了 warning 全集**（`grep '^warning: ' | sort` 后 `diff`）：**IDENTICAL，零差异**。

> 中途曾真出现 598 → **599**：`RESIDENT_DELIVERY_UNKNOWN_BASE` 只被 `#[cfg(test)]` 引用，
> 非测试构建判它 `never used`。已加 `#[cfg(test)]` 消除，回到 598。详见 §7 catch。

### 6.2 shape gate 两模式 finding 集合对照

改前一侧不是估的：把 `baseline/` 的 7 个窄写件**临时换回工作树**跑了一次 gate，
再从快照换回、`shasum -c` 校验 7/7 OK。

- 归一化（把 `lines`/`delta`/`line`/`current` 数值置换为 `N`）后 **`diff` 为空 —— 27 条 finding 逐条相同、零新增类别**。
- 唯一变化是既有 `file_over_limit_not_in_ratchet` 里的行数数值：

| 文件 | 改前 | 改后 | Δ |
| --- | --- | --- | --- |
| `commands.rs` | 6794 | 7046 | +252 |
| `supervisor_orchestrator.rs` | 3715 | 3719 | +4 |
| `supervisor_resident_oneshot_session.rs` | 3496 | 3525 | +29 |
| `supervisor_resident_oneshot_tests.rs` | 3638 | 3721 | +83 |
| `src/lib/tauri.ts` | 2008 | 2008 | 0 |

四者本就在 `file_over_limit_not_in_ratchet` 之列，属**同一条既有 finding**。**未为压行数做任何无关重构。**

### 6.3 冻结 hash 回算

| 文件 | 改后 SHA-256 | 状态 |
| --- | --- | --- |
| `src/views/projects/jiaoban/useJiaobanConversationState.ts` | `b86a1dff…33ca071` | **冻结只读，零漂移** ✅ |
| `src/lib/tauri.ts` | `95587bdd…5d8dc14e` | **未改动，零漂移**（B6 无需动它） |
| `mcp/supervisor_conversation_binding.rs` | `c9dc3ee2…85318035` | 已改（B1） |
| `commands.rs` | `dfcf3c75…1f30eeeb` | 已改（B2/B3/B5/B7） |
| `mcp/supervisor_orchestrator.rs` | `3bf3c118…1e782a76da` | 已改（B4） |
| `src/lib/conversationTransport.ts` | `15dbfee0…901fc1f0` | 已改（B6） |
| `supervisor_resident_oneshot_session.rs` | `745b8b0d…d21fabc1bf` | 已改（A8*） |
| `supervisor_resident_oneshot_tests.rs` | `41fd1924…63faf6f12c` | 已改（A9b/A8* 断言） |
| `tests/shared-conversation-transport.test.tsx` | `87b97cbd…32d503a3d1` | 已改（B6 断言） |

### 6.4 基线 diff 摘要

全量 patch：`raw/baseline-narrow-write-diff.patch`

| 文件 | +新增 | −删除 |
| --- | --- | --- |
| `supervisor_conversation_binding.rs` | 135 | 0 |
| `commands.rs` | 239 | 7 |
| `supervisor_orchestrator.rs` | 5 | 1 |
| `conversationTransport.ts` | 12 | 0 |
| `supervisor_resident_oneshot_session.rs` | 30 | 3 |
| `supervisor_resident_oneshot_tests.rs` | 74 | 0 |
| `shared-conversation-transport.test.tsx` | 92 | 0 |

删除共 11 行，逐行列出且零中文（见 §4④）。

---

## 7. 新 catch（2 条，均已入账）

两条均已追加到 `docs/harness-catch-log.md`，行见该文件 2026-07-27 段。

1. **派发表 B3「三处」实为四处，且行号与括注不自洽。**
   §1.2/§2.1 写 `commands.rs:400/419/433`「3 种不同原因共用一 stage」，括注为
   「context 解析失败 / 项目根不匹配 / run_id 生成失败」。实核基线：
   `:400` = context 解析失败、`:407` = **项目根不匹配**、`:419` = **既有 thread 校验失败**、`:433` = run_id 生成失败
   —— 共用 `BindingConstruct` 的是**四处**，包里把 `:419` 错标成「项目根不匹配」，并把 `:407` 整个漏掉。
   照面值只做三处，会让「既有 thread 校验失败」继续顶着一个分辨不出的 stage
   —— 正是本包要消灭的那种缺陷。已按四处落地。

2. **同一个数字 599，上一轮是假象、这一轮是真回归；「上轮结论」不能当判据复用。**
   上一轮 warning 598→599 被指导线判为「把汇总行也数进去的假象」。本轮中途同样出现 599，
   但这次是**真的**：`RESIDENT_DELIVERY_UNKNOWN_BASE` 只被 `#[cfg(test)]` 引用，
   非测试构建报 `never used`。若沿用上一轮结论直接判「599 是假象」，就会放过一条真回归。
   **新规矩：warning 数变化一律以「warning 全集逐条 diff」定性，不得靠记住某个数字或复用上一轮的结论。**
   本包已按此执行（改前/改后 warning 全集 `diff` = IDENTICAL）。

---

## 8. 未做 / 边界

- **未进真机**，未构建或启动 App，未跑 dev server。本包全部证据来自离线测试与静态核对。
- **不修 bug**：为什么首句失败**仍然未知**。本包只让「失败对外自报家门」这一维变得可分辨。
- 未接孤儿命令、未删死码、未改既有 `binding_stage` 取值、未动冻结只读件。
- **零 stage / commit / push。**
- D1 的 A 层剩余 5 文件、B/C 层、598 warning 判定**继续挂账**，本包未动。

---

## 9. 结论

**`PASS_BINDING_FAMILY`**：B1–B7 全部落地，A9b 落地，A8* 评估后**做了**（未改形状）；
运行期红 + 编译期红均已留证并分开标注；行为零变化已证（1203/0/45 → 1210/0/45，+7 逐条对应、零挂零删）。

整包 **`NEEDS_GUIDANCE_REVIEW`**。请指导线特别裁两点：
① §5 的 B4 口径（stage 不动、语义走 family）；② §7 catch 1 的「四处而非三处」是否照收。

**不声称 bug 已修，不声称可以进真机。** 真机首句重验是下一包，需用户在场并单独授权。

## 指导线复核与裁决（2026-07-27 · guidance）

裁决：**`ACCEPTED_BINDING_FAMILY`**。两处待拍板项均已裁定（见 G.3）。

### G.1 指导线独立复核

| 项 | 结果 |
| --- | --- |
| 写入面 | 7 个文件被改，全部在白名单内；`src/lib/tauri.ts` 与 `useJiaobanConversationState.ts` 零字节改动（hash 与派发值相同），与自述一致 |
| 基线副本 | `shasum -c` **9/9 OK**（指导线在 `baseline/` 内复算） |
| 派发表笔误 | **属实，是指导线的错**：真值 `…c229483b8a6b…`，包里写成 `…6a8b…`。基线 manifest 记录的即 `8a6b` 版；文件自 `a13599e` 未动。执行线的处置（先分辨"文件动了 vs 面值抄错"，再订正留痕）正确 |
| B3「四处不是三处」 | **属实，也是指导线的错**：指导线读基线逐处确认——`:400` context 解析、`:407` 项目根不匹配、`:419` `verify_supervisor_existing_thread(...).is_err()`、`:433` run_id 生成，四处全部返回 `BindingConstruct`。包里既漏了 `:407`，又把 `:419` 错标成"项目根不匹配"。执行线按四处落地正确（`PRECHECK_CONTEXT_UNRESOLVED` / `PRECHECK_PROJECT_ROOT_MISMATCH` / `PRECHECK_EXISTING_THREAD_REJECTED` / `PRECHECK_RUN_IDENTITY_UNAVAILABLE`） |
| B4 落地 | 指导线复核：内部新增 `SupervisorConversationBindingEstablishmentError::BindingStoreConflict`；**对外 `SupervisorConversationBindingStage` 枚举 diff 为空**——取值域确未变 |
| B6 前端 | 加法且带**防御性投影**：`safeBindingFailureFamily` 只放行 `^binding_[a-z0-9_]{1,56}$`，后端原始错误串无法经此新字段进入 controller state；新增测试正是断言这一点。指导线认为这比包里要求的"原样透传"更稳，接受 |
| UI 文案 | 后端五处文案计数与内容逐字未变 |
| warning | 指导线自跑：`generated 598 warnings`，与改前一致 |
| shape gate | 指导线自跑 `--mode check`：`17 / 5 / 5`，**finding 集合与上一轮逐条相同**（零新增） |
| typecheck / 37 入口 / staged | 指导线自跑：干净 / exit 0 / 空 |

### G.2 一处指导线自己的证据事故（如实记）

指导线独立复跑 `cargo test --lib`，**第一次得 `1209 passed / 1 failed`**，与执行线所报 `1210/0/45` 不符。但该次命令只保留 `tail -3`，**失败测试名被自己截断丢失**。随后连跑 4 次均为 `1210 / 0 / 45`。

判定：**一次未复现的抖动**，不影响本包裁决（"行为零变化"看的是增量：`1203 → 1210`，`+7` 逐条对应新增断言；4/5 次运行为该值）。但**那条失败具体是哪个测试已永久不可知**——这是复核方自己的证据事故，已入账（07-27 第四行），并立新规矩：批量运行一律全量落盘再摘要。

### G.3 两处待拍板项的裁定

**① B4 口径 —— 维持执行线的做法（内部拆变体、对外 stage 不变）。**

包 §2.1 的 B4 与 §5.5 的红线**确实自相矛盾**，这是指导线的措辞事故。裁定按红线走，理由：

- 本包目的是"让失败可分辨"，family 已经做到；新增对外 stage 取值**不是必要条件**；
- `binding_stage` 在前端是被**枚举化消费**的（`safeSupervisorBindingStage`），新增取值会让既有/其他消费者见到未知成员——这正是红线要防的爆炸半径；
- 若日后确需扩充 stage 词表，那是一次**独立的对外契约变更**，应单独立包、单独评估消费者。

执行线**没有自行放宽红线而是回交请示**，处置正确。

**② B3 计数 —— 采纳四处，包内表述由指导线订正。**

### G.4 记账

本轮 4 条 catch（`docs/harness-catch-log.md` 07-27 四行）：执行线记 2 条（B3 计数错、"上轮结论不能当判据复用"），指导线记 2 条（派发表 hash 笔误的处置、指导线自己截断输出丢失失败名）。

其中"**同一个数字 599：上一轮是假象、这一轮是真回归**"值得单独标注：上一轮指导线判定 `599` 是把汇总行数进去的计数假象；本轮执行线中途**真的**引入了一条新 warning（`RESIDENT_DELIVERY_UNKNOWN_BASE` 仅被 `#[cfg(test)]` 引用），若它套用上轮结论一挥手，就会放过一条真回归。**结论只对当次证据成立，不能当判据复用**——这条对指导线同样成立。

### G.5 现状与下一步

两条路的 family 至此全部落地并经独立复核。**下一包才是真机首句重验**（读 `binding_failure_family` / `failure_family` 一次归因 + 核 `append_resident_delivery_diagnostic` 计数），**需用户在场并单独授权**。代码仍留在工作树未提交。
