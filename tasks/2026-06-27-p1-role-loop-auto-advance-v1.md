# 实现任务包：P1 · 角色循环「授权后自动推进」编排（件 B + 件 C-1）· 主导线 → 执行线 v1

日期：2026-06-27　性质：**高危#4-轻档**（圈固定测试项目·授权范围内自动推进 = 拆+派+链跑；**方案授权人闸不省**；真跑用户在场）。

## 0. 接手须知（冷启即读，本包自包含）

- 你是**执行线**（新开、干净上下文）。**子线不 `git add`/`commit`。** 全程中文。
- **上游已拍板**：决策 `decisions/2026-06-27-role-loop-semi-auto-orchestration-light-tier-v1.md`（agent 层半自动编排·圈测试项目轻档·方案授权人闸不省·偏离中间版§1.3）+ 设计 `docs/plans/2026-06-27-role-loop-auto-orchestration-spec-v1.md`（§件 B/C 看这）。本包做**件 B + 件 C-1**。
- **零件全现成**（核过·committed）：
  - `preview_project_director_task_plan_for_index_at`（`commands.rs:748` 命令的内层·主管 LM 拆任务）
  - `prepare_authorized_auto_dispatch_for_index_at`（`c4_c6_workflow_governance_entrypoints.rs:51`·→ prepared dispatches）
  - `run_director_task_chain`（`director_agent.rs:231`·worker 链·四护栏·入口 `require_test_project_path_lock`）
  - 它们现是**三个独立 Tauri 命令**（手动点 3 次）。**本包只新增一个把它们串起来的编排命令**，三者**本体 0-diff**。
- **方案授权前提**：自动推进**只在 active 授权已存在时**跑（用户已确认方案 + 边界复核过）。**本命令不创建、不跳过授权**——查不到 active 授权 → 直接拒（principles §4 / 中间版 §0.3：LM 不得绕权限确认执行）。
- **别碰**：挂起的 S2 手动前端（C1 按钮 + 3 bug 修，工作树未提交·手动挡 override）——本包是后端编排、不动那些。
- **先读**：① 上面 3 个内层 fn ② `lib.rs` 的 `start_project_director_chain`（async+spawn_blocking 范本）+ §5 `s3_director_chain_real_run`（真跑范本）③ `c4_c6...:annotate`（看 needs_binding/blocked/prepared 状态怎么来）④ 决策 + 设计方案 ⑤ 记忆 `real-codex-run-flaky-verify-by-artifact`。
- **一句话**：加一个 async 编排命令——**前提 active 授权** → 主管 LM 拆任务 → prepare → **没绑会话/越界就停（件 C-1）**、prepared 出来就跑 worker 链；圈测试项目、四护栏、超范围/失败必停、复用现成命令本体 0-diff。

## 1. 拍板摘要

- **要做的事**：把「主管拆任务 → prepare → 链跑」从**手点 3 次**塌成**授权后一键自动推进**。补全「方案授权 → 自动执行范围内推进」（中间版 §156：已确认方案授权范围内可自动派发）。
- **代价**：一轮——一个 async 编排命令 + needs_binding 停（C-1）+ 测试。**复用现成命令本体、不改逻辑。**
- **关键边界**：方案授权人闸**在命令之前**（命令查 active 授权才跑、不碰授权）；圈测试项目；超范围/失败/没绑 → 停 + 可见 + 等用户。

## 一句话判据

判改动在不在本包——问：**「是不是只加一个『查 active 授权 → 自动 拆+prepare+(没绑停)+链跑』的编排命令、复用现成三命令本体、圈测试项目+四护栏、没创建/跳过授权、没改闸/沙箱/plan/prepare/chain 本体?」** 是 → 做；否（创建/跳过授权、改本体、放开非测试、绕 S1）→ **停、回主导线。**

## 2. 建什么

**async 编排命令**（`auto_advance_authorized_role_loop` 或近名·照 `start_project_director_chain` 范本 async+spawn_blocking）：
1. **入参**：`project_root` / `workflow_id`（+ 解析当前 active 授权）。
2. **查 active 授权**：无 → `Err("无 active 授权：请先确认方案 + 全局边界复核")`。**死线·不创建不跳过。**
3. **主管拆任务**：`preview_project_director_task_plan_for_index_at` → `planned_tasks`（主管 LM·授权范围内）。
4. **prepare**：`prepare_authorized_auto_dispatch_for_index_at(planned_tasks)` → prepared 结果。
5. **件 C-1·分流**（按 prepared 结果的 status 计数）：
   - `prepared_count == 0` 且 `needs_binding > 0` → **停**，返回 `stage="needs_binding"` + 人话「需先给 codex-dev 节点绑一条 Codex 会话再自动推进」。**不自动绑**。
   - `blocked > 0`（越界）→ **停**，`stage="blocked"`（超授权范围·中间版 §156 必停）。
   - 其它 0 prepared → 停，`stage="no_dispatchable"`。
   - `prepared_count > 0` → 进 6。
6. **链跑**：`run_director_task_chain(prepared 的 planned_tasks)` → chain outcome（四护栏在·失败即停）。
7. **返回**：`{ stage（planned/needs_binding/blocked/ran）, plan 摘要, prepared_count, chain_outcome 或 stop_reason }`——**每阶段可见**（中间版 §161 失败可见 + 审计）。
- **每步审计**：编排起 / 拆任务 / prepare / 停因 / 链跑 进 `audit_events`（复用现成审计帮手）。

## 3. 安全死线

- **方案授权人闸不省**：命令前提 = active 授权已存在；**查不到即拒、不创建不跳过授权**（principles §4 / §0.3）。
- **圈固定测试项目**：链跑入口 `require_test_project_path_lock` 现成兜；preview/prepare 限授权项目。非测试仍锁。
- **四护栏**（链现成）+ **超范围/失败/没绑 → 停 + 可见 + 等用户**（§156/§161/§3·不自动重试）。
- **0-diff**：`decide_real_execution_command` / `command_plan_for` 沙箱 / `execute` / `preview_..._for_index_at` / `prepare_..._for_index_at` / `run_director_task_chain` **本体不改逻辑**——只新增编排命令 + 注册。若要改本体 → 停、回主导线。
- **不自动绑会话**（件 C-1 = 停提示·C-2 自动绑是后续别做）；不放开非测试/多项目/auto-approve；**不碰挂起的 S2 手动前端**。
- 自动测试不真起 codex（stub）；真链跑仅 `#[ignore]` + 用户在场。

## 4. TDD 验收门

- **全链 stub**（active 授权 fixture · stub runner）：编排 → 拆任务 → prepare → 链跑 → outcome `ran` + 链 completed；每阶段审计在。
- **件 C-1 没绑则停**：fixture 不绑会话 → prepared_count==0/needs_binding → **停在 `needs_binding`、不跑链**（断言 chain 没起、无 proof）。
- **无授权则拒**：没 active 授权 → `Err`（证人闸不省）。
- **path-lock 正反**：非测试 root → 拦（链入口）；测试项目+授权 → 跑。
- **regression**：闸/沙箱/preview/prepare/chain **本体 0-diff**（扫 diff 自证）；既有测试全绿、`cargo test --lib` 计数不降。
- **全量**：`cargo test --lib` / `cargo fmt -- --check`（只本包改的文件）/ `git diff --check`。

## 5. 真跑验证（单独步·`#[ignore]`/用户在场·测试项目·四护栏）

stub 验通 + 主导线核实物**后**：测试项目造「active 授权方案」（proof-goal）+ 绑一条真 Codex 会话 → **经编排命令一下** → 主管 LM 真拆 → prepare → worker 链真 codex 接连跑 → proof 建出 + 回读。验：**一个命令把"授权后自动推进"跑通**、proof 在测试项目、每节点过 S1 闸 path-lock 命中、沙箱只动测试目录 `.codex`/auth 没碰、四护栏在、授权前提真拦（无授权 → 拒）。flake → retry。

## 6. 本包不做（deferred·显式）

- **件 A 接咨询 LM**（出方案自动化）= P2，另包。
- **件 D 触发 + 方案授权卡 UI** = P3（真机），另包。本包只到**后端编排命令**。
- **件 C-2 自动绑会话**（本包用 C-1 停提示）；全局主管 agent；真·NL 主对话启动；非测试真实项目（高危#1）；多项目；auto-approve 把方案授权也自动（**永不做**）。

## 7. 回交

- 跑 §4；回交：实现 diff（确认闸/沙箱/preview/prepare/chain 本体 0-diff、只 +编排命令/注册/测试）+ 全链 stub 证据 + 件 C-1 没绑则停证据 + 无授权则拒证据 + path-lock 正反 + 计数 + 真跑证据（一命令跑通授权后推进·proof·confinement·授权前提拦）→ 主导线核实物（重跑 + 扫 diff + 真跑用户在场 + 核 confinement）。**子线不 commit。**

## 8. 不接受为

- 不接受为：命令**创建或跳过方案授权**（人闸不省）/ 改了闸·沙箱·preview·prepare·chain 本体没经批 / 没绑会话却硬跑（该停）/ 越界任务硬跑（该停）/ worker 跑非测试 / 放开多项目或 auto-approve / 绕 S1 / 自动测试真起 codex。
- **不接受为 S3 agent 层整体完成**（本包只到「授权后自动推进的后端编排」；件 A 咨询/件 D UI/真 NL/别角色 另算）。
