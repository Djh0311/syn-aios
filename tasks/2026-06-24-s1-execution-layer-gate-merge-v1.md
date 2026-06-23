# 实现任务包：S1 执行层合一「B 画布派发过 A 银行级强闸」· 主导线 → 执行线 v1

日期：2026-06-24

出自：主导线（Claude）。性质：**高危#3（改安全闸路径）实现包**。计划：`docs/plans/2026-06-23-next-stage-unify-and-requirements-plan-v1.md` 的 **S1**。上游审查：`docs/plans/2026-06-23-next-stage-review-*`（两线 / 两套闸 findings）。

## 0. 接手须知

- 你是**执行线**。流水线：**你实现 + 测试（cargo 单测用假/stub runner，绝不在自动测试里真跑 codex）→ 独立复核 → 主导线审实物（核 diff + 重跑闸 + 扫高危）**。真跑进测试项目的端到端验证 = 单独一步（§6，轻档·测试项目），不混进本包自动测试。
- 先读：本文 + `CURRENT.md` 首条 + `AGENTS.md`（**高危#1/#3/#4**）+ 这几处实物：
  - B 派发：`commands.rs:1916` `execute_project_workflow_node` / `:1937` `execute_project_workflow_node_at`
  - B 弱闸：`commands.rs:1677` `workflow_engine_test_project_unsealed`（path-lock，现抄 5 处：`:1688/:1757/:1920/:2254` + `workflow_chain_controller.rs:304`）
  - A 强闸：`real_execution_command.rs:185` `decide_real_execution_command` + 入参 `:118` `RealExecutionCommandGateInput`
  - A 判据 helper：`has_active_attempt_in_h4_scope`、`inspect_codex_local_execution_guard`、`inspect_authorization_matrix`（在 `session_continuation_store.rs` / 相关文件）
  - 沙箱底座：`codex_local_runner.rs:1429` `command_plan_for`
- **全程中文。子线不 `git add` / `git commit`。**
- **关键安全（本包死线）**：见 §3。一句话：**合一 = 只加强、不放松**——给 B 加上 A 的强判据，但**测试项目 path-lock 仍是放行的必要条件**，沙箱底座一字节不动，绝不顺手放开非测试项目或自动连环。

## 1. 拍板摘要

- **要做的事**：现在两条真执行路径各有一套闸（B `execute_project_workflow_node_at` 走单行 path-lock；A H5 走 7 判据强闸 `decide_real_execution_command`）。本包把 **B 的派发改成过 A 的强闸**：B 在真起 runner 前，从自己的上下文算出 `RealExecutionCommandGateInput` 的各判据、调 `decide_real_execution_command`，**只有判决 `authorized_for_real_runner` 才真跑**；**path-lock 命中并入 `authorization_complete` 的必要项**（从抄 5 处收敛为进闸前算一次）。
- **代价**：一轮实现 + 测试；做完后 B 派发获得 A 的在飞去重 / guard / readback / 授权完整 等判据，安全面从"单行 path-lock"升级到"银行级强闸"，且两套闸收敛为一套判决核。
- **不做的后果**：两套闸继续并存、安全面分叉（path-lock 抄 5 处、B 无在飞/guard/readback 检查），统一无从谈起。
- **关键澄清**：本包**不放开非测试真实项目**（高危#1 不动）、**不动沙箱底座 `command_plan_for`**（高危#3 边界）、**不开/不放宽自动连环**（高危#4 不动）、**不改 A 线自己的路径**、**不改 `decide_real_execution_command` 本体判决逻辑**（只是给它喂正确的输入）。

## 一句话判据

判某改动在不在本包内——问：**「是不是在让 B 的 `execute_project_workflow_node_at` 真起 runner 前过 `decide_real_execution_command`、path-lock 命中作 `authorization_complete` 必要项、沙箱 `command_plan_for` 字节未动、默认仍只放行固定测试项目、没放开连环、没改强闸本体逻辑、没真跑 codex 进非测试？」** 是 → 做；否（尤其要放开非测试项目 / 改沙箱 / 开连环 / 改 `decide_real_execution_command` 判决体 / 动 A 线路径）→ **停、回主导线**。

## 2. 建什么（B 派发过 A 强闸）

在 `execute_project_workflow_node_at`（`commands.rs:1937`）真起 `RealWorkflowNodeCodexRunner` 之前，插入一道**统一判决**：从 B 的上下文（project_root / node_id / work_item_id / workflow_id / 节点 canvas_payload / 会话绑定）算出 `RealExecutionCommandGateInput`，调 `decide_real_execution_command`，非 `authorized_for_real_runner` 即返回对应 blocked 错误、不起 runner。

各判据映射（执行线据此实现 + 验证；拿不准的判据先取**保守值=更拦**，并在 evidence 说明）：

| `RealExecutionCommandGateInput` 字段 | B 怎么算 |
|---|---|
| `authorization_complete` | **必须包含 path-lock 命中**（`workflow_engine_test_project_unsealed(project_root)==true`）。这是放行的**必要条件**——不命中即 authorization 不完整、判决拦截。 |
| `duplicate_blocked` | 查该 node 是否已有在飞派发（端口/复用 A 的 `has_active_attempt_in_h4_scope` 同款思路；B 现在**无此检查**，同节点可重复派发，要补上）。 |
| `guard_blocked` | 调 `inspect_codex_local_execution_guard`（A 已有）。 |
| `diagnostics_blocked` | 接 A 同一诊断降级源。 |
| `readback_required` | 映射 B 的回读计划；无回读计划即视为未就绪、拦截。 |
| `stale_memory_blocked` | B 不走任务记忆包 → 可取 `false`（在 evidence 说明为何安全）。 |
| `user_rejected` | 无每次审批时取 `false`（本包不引入逐次审批 UI，那是 S2 方案授权制）。 |
| `command_name`/`command_family`/`operation_id`/`h5_unified_product_command` | 据实填，使判决能正常进入授权分支；以测试钉死期望判决。 |

**path-lock 收敛**：现 5 处散落的 `workflow_engine_test_project_unsealed` 调用，收敛为"进 `decide_real_execution_command` 前算一次、并入 `authorization_complete` 必要项"。**保留 `workflow_engine_test_project_unsealed` 函数本体不删**（它是 path-lock 真值来源）。

**沙箱底座不动**：`command_plan_for`（`codex_local_runner.rs:1429`，拼 `--sandbox` / `-C` / `--add-dir` / argv-only）**一字节不改**——本包只在"起 runner 前的判决"这层加闸，不碰"怎么跑"。

## 3. 安全硬约束（本包死线，必须成立）

- **铁律·path-lock 必含**：任何路径下 `authorized_for_real_runner` ⟹ `workflow_engine_test_project_unsealed(project_root)==true`。**漏掉 = 真跑逃逸到非测试真实仓 = 不可逆（高危#1）**。用测试正反钉死。
- **沙箱底座零改动**：`command_plan_for` 及其拼的 `--sandbox/-C/--add-dir/argv-only` diff 必须空（高危#3 边界）。
- **不放开非测试真实项目**：非固定测试项目 `project_root` 一律判决拦截（高危#1）。
- **不开 / 不放宽自动连环**：`workflow_chain_controller` 的 4 护栏（runaway 上限 / 可中断 / 审计 / 失败即停）不削；不借本包把连环放开到非测试 / 多项目 / auto-approve（高危#4）。
- **不改强闸判决体**：`decide_real_execution_command` 的判决逻辑（7 拦顺序）不动，只给它喂输入。
- **不动 A 线自己的路径**：`controlled_session_continuation` / H5 现有调用零变化。
- **不真跑 codex 进非测试**：自动测试用 stub/fake runner（不起真 codex），真 codex 仅 `#[ignore]` + 测试项目（§6）。
- **碰线就停**：要放开非测试 / 改沙箱 / 开连环 / 改判决体 / 动 A 线 → **停、回主导线**。

## 4. TDD 验收门（测试钉死）

- **path-lock 必含（正反）**：`project_root` 非测试项目 → 判决拦截、不起 runner；是测试项目但其他判据不满足（如 guard_blocked）→ 仍拦截；测试项目 + 各判据满足 → `authorized_for_real_runner`。
- **新判据真拦**：构造 duplicate（同节点在飞）/ guard_blocked / readback 缺失 各自 → 判决拦截。
- **stale_memory/user_rejected** 取值有测试覆盖 + evidence 说明安全。
- **runner 只在 authorized 才被调**：用 stub runner 计数，authorized 才调一次、否则零调用。
- **A 线 regression**：`controlled_session_continuation` / H5 既有测试全绿、行为零变化。
- **沙箱 regression**：`command_plan_for` diff 空、其相关测试全绿。
- **全量**：`cargo test --lib`（报通过数，基线 575/0）/ `cargo fmt -- --check` / `npm run typecheck` / `npm run test:offline-interaction` / `npm run build` / `node scripts/harness/workbench-shape-gate.js --mode check` / `git diff --check`。

## 5. 本包不做（deferred）

- 不放开**非测试真实项目**真跑（高危#1，单独授权）。
- 不开 / 不放宽**自动连环**（高危#4）。
- 不动沙箱 `command_plan_for`、不改 `decide_real_execution_command` 判决体、不动 A 线路径。
- 不做**方案授权制 UI / 逐次审批**（那是 S2；本包 `user_rejected` 取 false）。
- 不做 A 角色循环接前端 / 记忆全救 / A 旧面板整理（S2）。
- 不真跑 codex 进非测试项目。

## 6. 真跑验证（单独步、本包之后、测试项目·轻档）

本包实现 + stub 测试验通 + 复核 + 主导线审实物**之后**，单独一步：在**固定测试项目** `/Users/yoyi/codex-workflow-mario-test`（轻档·path-lock 锁死）真跑一个画布节点，确认：① 走了 A 强闸、② path-lock 命中才放行、③ 改个非测试 project_root → 判决拦截、④ 沙箱只动测试目录。**这一步不在本包自动测试里。**

## 7. 验证 + 回交

- 跑 §4 各门；回交：实现 diff + evidence（path-lock 必含正反证据 + 沙箱 `command_plan_for` diff 空证明 + 新判据真拦证据 + A 线/沙箱 regression 绿 + 全量闸输出 + 没真跑 codex 进非测试的证明）→ 独立复核 → 主导线审实物。
- 子线不 commit；回交 diff + evidence 给主导线，commit 由主导线带 CURRENT 回写做。

## 8. 不接受为

- 不接受为：放开了非测试真实项目 / 改了沙箱 `command_plan_for` / 开或放宽了自动连环 / 改了 `decide_real_execution_command` 判决体 / 动了 A 线路径 / `authorized` 漏了 path-lock 必含 / 自动测试里真跑了 codex / S2 的方案授权制或角色接前端被提前做。
- 不接受为 S1 全部完成（§6 测试项目真跑验证未做前，只算"逻辑接通+锁死验过"，不算端到端通）。
