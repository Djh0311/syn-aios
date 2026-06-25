# 实现任务包：S2-3 补丁「封死 j2_b_b2 旁路（漏封补全）」· 主导线 → 执行线 v1

日期：2026-06-25

出自：主导线（Claude）。性质：**高危#3**（改安全闸/封旁路）。上游：S2-3（`tasks/2026-06-24-s2-3-role-loop-real-run-end-to-end-v1.md`）回交后，主导线跑对抗复核（4 只读 agent）+ 亲手核真码，发现 S2-3 的旁路封堵**漏了 `j2_b_b2`**：执行线封了 `j2_b_b1` + `k3_b`，**同族的 `j2_b_b2` 没封**。本包只补这一个洞 + 证明 automation 族再无漏网。

## 0. 接手须知

- 你是**执行线**。流水线：你实现 + stub 测试（自动测试绝不真起 codex）→ 主导线核实物。子线不 `git add` / `git commit`。
- 先读：本文 + **b1 封堵实物**（`commands.rs:856-872` 的 `run_project_workflow_automation_j2_b_b1` —— 照它一模一样镜像）+ `require_test_project_path_lock`（`commands.rs:1702`）+ b1 的封堵测试（`lib.rs:4458` 起）+ `AGENTS.md` 高危#3。
- **全程中文。子线不 commit。** 一句话：**把 `j2_b_b2` 的 tauri wrapper 按 `j2_b_b1` 的样照搬一道 path-lock gate，封死它真跑——不碰 `_at` / runner / 沙箱 / 既有 b1·k3_b·H5 封堵 / PCR 授权路。**

## 1. 拍板摘要

- **要做的事**：给 `run_project_workflow_automation_j2_b_b2`（`commands.rs:875`）的 wrapper 开头加一道 `require_test_project_path_lock(J2_B_B2_PROJECT_ROOT, ...)?;`，与 `j2_b_b1` 完全同构。`J2_B_B2_PROJECT_ROOT` = `/Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project`（**非测试项目**、`workspace-write` 能写）≠ 固定测试项目 → 此 gate **永远拦、永久封死该入口真跑**。
- **代价**：约 3 处微改（1 词可见性 + 1 个 gate + 1 个测试）。做完后 automation 三入口（b1/b2/k3_b）**全封死**，S2-3 的「封全旁路」命根落地。
- **不做的后果**：`j2_b_b2` 仍是「无 path-lock 能 `workspace-write` 真跑 codex」的入口（虽写死到固定 tmp 夹、实际风险低，但任务包命根是封全、不留同族漏网）。

## 一句话判据

判某改动在不在本包内——问：**「是不是只在 `j2_b_b2` wrapper 照搬 b1 的 path-lock gate + 配套 1 词可见性 + 1 个拦截测试，且没碰 `_at`/runner/沙箱/既有封堵/PCR 授权路/别的命令？」** 是 → 做；否（尤其要碰 `_at` 逻辑 / runner / 沙箱 / 去 gate PCR 那条授权路 / 改别的命令）→ **停、回主导线。**

## 2. 建什么（3 处微改·照 b1 镜像）

**A · 1 词可见性**（`project_workflow_automation.rs:45`）
- `const J2_B_B2_PROJECT_ROOT` → `pub(crate) const J2_B_B2_PROJECT_ROOT`（**只改可见性、值不动**，与当初给 `J2_B_B1_PROJECT_ROOT` 做的同样 1 词改一致，好让 `commands.rs` 引用它）。这是本文件**唯一**改动。

**B · wrapper 加 gate**（`commands.rs:875-885`，照 `j2_b_b1:861-865` 镜像）
- 在 `run_project_workflow_automation_j2_b_b2` 体首、调 `_at` 之前插入：
  ```rust
  // 旁路封堵：j2_b_b2 真跑写死 J2_B_B2_PROJECT_ROOT（非测试 product-line/tmp 隔离项目、workspace-write）→ 此 gate 永远拦、封死真跑。
  require_test_project_path_lock(
      project_workflow_automation::J2_B_B2_PROJECT_ROOT,
      "run_project_workflow_automation_j2_b_b2",
  )?;
  ```
- **gate 加在 wrapper（tauri 命令），不加在 `_at`** —— 与 b1 一致。已确认无测试直接调 wrapper（唯一调用者 `..._j2_b_b2_at` 在 `project_workflow_automation.rs:4930` 的 in-crate 测试，走 `_at`、不经 wrapper gate），故**不撞任何既有测试**。

**C · 拦截测试**（`lib.rs`，照 b1 的 `4458-4467` 镜像）
- 加一条：`require_test_project_path_lock(project_workflow_automation::J2_B_B2_PROJECT_ROOT, "x")` 应返 `Err`（非测试 root `product-line/tmp/...` 被拦），与 b1 那条并列。

## 3. 安全死线（本包死线·必须成立）

- **永久封死**：`J2_B_B2_PROJECT_ROOT`（`product-line/tmp/...`）≠ `WORKFLOW_ENGINE_TEST_PROJECT_ROOT`（`/Users/yoyi/codex-workflow-mario-test`）→ gate 必恒拦。测试钉死「此入口非测试 root 被拦」。
- **不碰判决/沙箱/runner**：`_at` 体、`RealCodexLocalPhaseBProcessRunner`、`command_plan_for`、`decide_real_execution_command` **0-diff**。
- **不动既有封堵**：`j2_b_b1` / `k3_b` / H5-continuation 那 3 道已封 gate **0-diff**（别顺手重排）。
- **PCR 授权路明确不碰**（关键）：`run_real_execution_product_command_phase_b` / `..._new_session_phase_b`（`commands.rs:814-841`）是 **sanctioned 高危#1 重档路径**（靠 prepare→confirm→授权矩阵守，不靠 path-lock），**不是旁路**。**不许给它加 path-lock gate**——加了会拦死合法的「任意项目·明确授权」真跑。本包只碰 `j2_b_b2`。
- **自动测试不真起 codex**。**碰线就停**：要改 `_at`/runner/沙箱/别的命令/去 gate PCR → 停、回主导线。

## 4. TDD 验收门（测试钉死）

- **新拦截测试**：B 的 gate 对 `J2_B_B2_PROJECT_ROOT` 返 `Err`（非测试被拦）；正向冗余可加一条「固定测试项目 root → `Ok`」复用 helper 既有断言。
- **完整性证明**（命根）：grep 列出 `commands.rs` 里**所有**碰真 codex runner 的 tauri 入口，逐个标注其守卫类型——(a) path-lock（b1/b2/k3_b + S1 worker 路）/ (b) 授权矩阵（PCR phase_b 族）/ (c) 仅 prepare 不执行（phase_a 族）——证明加上 b2 后**再无「既非 path-lock 又非授权矩阵又真能写」的入口漏网**。
- **regression**：既有 b2 的 `_at` 测试（`project_workflow_automation.rs:4930`）仍绿；`j2_b_b1`/`k3_b`/H5 封堵测试仍绿；`cargo test --lib` 计数**不低于 584 passed / 27 ignored**。
- **全量**：`cargo test --lib` / `cargo fmt -- --check` / `git diff --check`（本包不碰前端，不需 typecheck/offline/build）。

## 5. 本包不做（deferred）

- **不碰 PCR phase_b 族**（它是授权路、非旁路，见 §3）。
- 不改 `_at` 逻辑 / runner / 沙箱 / 判决体 / 既有 3 道封堵。
- 不碰前端、不碰别的命令、不做 S2-3 之外的真跑。
- phase_a 族（仅 prepare 不执行）不需 gate，不动。

## 6. 回交

- 跑 §4 各门；回交：3 处微改的 diff（确认 `_at`/runner/沙箱/既有封堵/PCR 全 0-diff）+ 新拦截测试输出 + **完整性证明那张「入口×守卫类型」表** + `cargo test --lib` 计数 → 主导线核实物（重跑计数 + 扫 diff 确认只动这 3 处 + 确认 PCR 没被误 gate）。子线不 commit。

## 7. 不接受为

- 不接受为：碰了 `_at`/runner/沙箱/判决体 / 动了既有 b1·k3_b·H5 封堵 / **给 PCR 授权路加了 path-lock（误封合法重档路）** / 改了别的命令 / 自动测试真起了 codex / 没给完整性证明表（只闷头封 b2 不证明无漏网 = 没解决命根）。
- 本包做完 = automation 三入口全封 + 证明无漏网；与 S2-3 §6 真跑一起，S2-3 的旁路封堵才算真完整、可收口 commit。
