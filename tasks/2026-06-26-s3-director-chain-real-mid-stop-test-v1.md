# 实现任务包：S3 主管链 · 补「真·中途停」集成测试（C1 §5 缺的那一条）· 主导线 → 执行线 v1

日期：2026-06-26　性质：**高危#4-轻档**（固定测试项目·真 codex 链中途停；`#[ignore]` + 真跑用户在场）。

## 0. 接手须知（冷启即读，本包自包含）

- 你是**执行线**（新开、干净上下文）。**子线不 `git add`/`commit`。** 全程中文。
- **现状**：C1 起链命令 `start_project_director_chain` **已建、在工作树里未提交**（3 文件 M：`director_agent.rs`/`command_registry.rs`/`lib.rs`；cargo 610/0/32 绿）。**别动这些已有改动**——本包**只加一个 `#[ignore]` 测试**到 `lib.rs`，把 C1 包 §5 漏掉的「跑中真停一次」补上，然后和 C1 一起交主导线核+commit。
- **为什么要这条**：C1 §5 要求「跑中用现成 `stop_project_workflow_chain` 真停一次验 app 层可中断真生效」——上一轮执行线没做（嫌真 codex 链里精准 stop 竞态、易误导）。停**机制**已被 B2 stub 确定性证过（停命令找得到主管链记录、边界停、`stopped_reason=user_stop_requested`），但「真 codex 链跑中途真停一次」没端到端演过。本包补这条端到端证据。
- **先读**：① `lib.rs` 的 `s3_director_chain_real_run`（§5 真跑·**照它克隆改造**）② `workflow_chain_controller.rs` 的 `stop_project_workflow_chain_at`（设 flag 的现成命令）+ `chain_node_state`（查某任务链节点状态）③ `director_agent.rs` 的 `run_director_task_chain`（边界查 `stop_requested` 的位置）④ B2 stub `s3_director_chain_interrupts_at_task_boundary_on_stop`（中断断言范本）⑤ 记忆 `real-codex-run-flaky-verify-by-artifact`。
- **一句话**：克隆 §5 真跑 → 另起线程在 **task1 链节点变 `running`** 时调现成 `stop_project_workflow_chain_at` 置 flag → 主链 task1 跑完、task2 边界抓到 flag → 断言 `completed==1` + proof_b **没建**；`#[ignore]`、只动 lib.rs、死线全 0-diff。

## 1. 建什么（一个 `#[ignore]` 测试，加进 lib.rs）

`s3_director_chain_real_run_interrupts_mid_chain`（`#[ignore]`）：

1. **setup 同 `s3_director_chain_real_run`**：真 director LM 拆 2-任务依赖链（建 proof_a → 回读核验后建 proof_b）→ prepare → 拿 `planned_tasks`；跑前删干净 proof_a/proof_b。
2. **另起 `std::thread`（跑链前 spawn）**——置 flag 的「用户点停」模拟：
   - 轮询 `workflow-state` 文件里**本链 chain_run 记录的 task1 链节点状态**（用 `chain_node_state`，按 task1 的 `planned_task_id` 查）。
   - **信号 = task1 节点状态 ∈ {`running`,`completed`}**（=task1 边界已过、worker 在跑/已完）→ 立刻调 `stop_project_workflow_chain_at(path, {project_root, workflow_id})` 置 `stop_requested`。
   - **为什么这个信号**：保证 flag 设在 **task1 边界之后**（task1 必跑、不会 `completed==0`），且会在 **task2 边界**被抓到（task2 必被跳）→ 稳得 `completed==1`。task1 是真 codex（~30s），轮询窗口足够宽。设上限轮询次数防卡死。
3. **主线程跑链**（命令路径内层 `run_director_task_chain`，真 codex）。
4. **join 停链线程**。
5. **断言（实物 + outcome）**：
   - `outcome.stopped_reason == Some("user_stop_requested")`；
   - `outcome.completed == 1`（task1 完、task2 停在边界）、`outcome.dispatched == 1`；
   - **proof_a 存在**（task1 真写了）、**proof_b 不存在**（task2 从没跑）——这条最关键，证「中途」真停在两任务之间；
   - chain_run `state == "stopped"`、审计有 `workflow_chain_run_stopped`。
6. **flaky 兜底注释**（真 codex）：task1 偶发早退 → `completed==0`；或极端时序 task2 抢跑 → `completed==2`。都属真 codex flake，**retry**（按记忆 `real-codex-run-flaky-verify-by-artifact`，读 text + 核 proof 实物分 flake 与真失败）。`#[ignore]` 默认不跑。

## 2. 安全死线

- **圈固定测试项目**：复用 §5 的 `WORKFLOW_ENGINE_TEST_PROJECT_ROOT` + driver 入口 path-lock；不碰非测试 root。
- **0-diff**：**只动 `lib.rs`**（加一个测试）。`run_director_task_chain` / `stop_project_workflow_chain` / gate / 沙箱 / prepare / controller / C1 命令 **本体全不改**——纯加测试。若发现要改本体才能停 → **停、回主导线**（说明哪不够）。
- **不绕 stop**：停一定走现成 `stop_project_workflow_chain_at`（证 app 层那条命令真能中途停主管链），不自己直接改 state 里的 flag。
- **`#[ignore]`**：自动测试不真起 codex；本测真跑仅 `#[ignore]` + 用户在场。

## 3. 验收

- `cargo test --lib`（**不带 --ignored**）= 仍 610/0/32（新测 `#[ignore]` 不进常规跑、计数不变；ignored 32→33）；`cargo fmt -- --check`（只本包改的 lib.rs·别 `--write` 碰预存偏差文件）；`git diff --check`。
- **真跑（用户在场·`#[ignore]`）**：`cargo test --lib s3_director_chain_real_run_interrupts_mid_chain -- --ignored --nocapture` → 看到 task1 完、停在 task2 前；**proof_a 在、proof_b 不在**、`stopped_reason=user_stop_requested`。flake → retry。

## 4. 回交

- 跑 §3 机器侧 + 真跑；回交：测试 diff（确认只加 lib.rs 一个测试、死线 0-diff）+ 真跑证据（outcome `completed=1/stopped`、**proof_a 在 / proof_b 不在** 实物、审计有停链事件）+ flake/retry 几次 → 主导线核实物（重跑常规 + 扫 diff + 真跑用户在场看中途停 + 核 proof_b 确实没建）。**子线不 commit**，连同 C1 改动一起留工作树等主导线核+commit。

## 5. 不接受为

- 不接受为：停在 task1 **之前**（`completed==0`）——那是「起链前停」不是「中途停」，没证两任务之间能停；
- 不接受为：用 stub/直改 state flag 假装真停（B2 已有 stub·本包要的是**真 codex 链 + 现成停命令**端到端）；
- 不接受为：改了 `run_director_task_chain`/停链命令/gate/沙箱 本体（只许加测试）；
- 不接受为：把新测写成常规（非 `#[ignore]`）→ 会在 CI 真起 codex。
