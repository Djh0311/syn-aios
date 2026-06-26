# 实现任务包：S3 项目主管 agent 第一刀（LM 拆解已授权方案 → planned_tasks → 喂 prepare）· 主导线 → 执行线 v1

日期：2026-06-25　性质：**轻档**（主管 agent 只读·只产「拆解计划」结构化文本；真派发在下游走现成 S1 闸 + 用户审，本包不碰）。
上游：S3 agent 层；**咨询第一刀已收口（`df64e10`，tier-1 注入+只读 harness 已证）**——本包**原样复用那套 harness**；插入点已 ground：`prepare_authorized_auto_dispatch` 收 `planned_tasks`（`commands.rs` 确定性版只是「没传就兜底」），LM 主管产出 planned_tasks 顶掉它。

## 0. 接手须知

- 你是**执行线**。流水线：实现 + stub 测试（自动测试不真起 codex）→ 主导线核实物 → 真跑（`#[ignore]`/用户在场·固定测试项目·只读）。子线不 `git add`/`commit`。
- 先读：① **咨询 harness**（`consultant_agent.rs`：`CliConsultantAgent`/`load_project_context`/注入正文/`parse_consultation_proposal`/`map_consultation_to_c1_input`）+ `codex_local_runner::readonly_codex_consult`（**只读 codex·原样复用**）② 产出契约 `types.rs:2444 ProjectDirectorPlannedTask`（要填 `title/objective/scope/depends_on/acceptance_criteria/report_format`）+ `2483 PreviewProjectDirectorTaskPlanInput` + `2494 PrepareAuthorizedAutoDispatchInput.planned_tasks` ③ 已授权方案怎么读（`proposal_id`+`authorization_id` → 方案内容）④ 记忆 `tier1-codex-exec-no-ondemand-read-inject`（**tier-1 必须注入·别靠 codex 按需读**）。
- **全程中文。子线不 commit。** 一句话：**复用咨询 harness 建 DirectorAgent——读已授权方案+项目上下文注入 → LM 只读拆解 → 产出 `planned_tasks`（带目标/依赖/验收）→ 喂 `prepare_authorized_auto_dispatch.planned_tasks`；只读不执行、不碰闸/派发机器、不自动派发。**

## 1. 拍板摘要

- **要做的事**：给角色循环的**主管拆解步**装上 LM——把「已授权的咨询方案」**智能拆成 worker 任务**（title/objective/scope/依赖排序/验收标准/汇报格式），顶掉现在的确定性笨拆。补上"AI 编排/拆解"那半，"对话→AI 编排→审→跑"才闭环。
- **代价**：一轮后端，**基本是咨询 harness 换档案/输入/产出/解析**。做完后角色循环**第一次有智能编排脑**。
- **关键澄清**：主管 agent **只读、只产计划**（结构化文本）；**不自动派发**（planned_tasks 给你审）；真派发走**现成 prepare→S1 闸→worker codex**（本包 0-diff、不碰）。

## 一句话判据

判改动在不在本包——问：**「是不是复用咨询 harness 建 DirectorAgent、只读产 planned_tasks 喂 prepare，且没碰 S1 闸/派发机器/确定性兜底逻辑、没自动派发、没改沙箱?」** 是 → 做；否（尤其要碰派发/闸、自动触发派发、改沙箱让它可写/可执行）→ **停、回主导线。**

## 2. 建什么（复用咨询 harness）

| | 复用 | 换成 |
|---|---|---|
| harness 基座 | `readonly_codex_consult`（只读 codex）+ tier-1 注入 | **原样** |
| 上下文 | `load_project_context` 注入法 | 注入**已授权方案正文**（objective/scope/proposed_steps）+ 项目基本态 |
| 档案 | 咨询档案的写法 | **主管档案**：你是项目主管，把已授权方案拆成可派发 worker 任务；定清每个任务的目标/范围/依赖顺序/验收标准/汇报格式；只读只规划、不执行、不自己派发 |
| 解析 | `parse_consultation_proposal`（抠 JSON） | `parse_director_plan` → `Vec<ProjectDirectorPlannedTask>` |
| 喂点 | →C1 | → `prepare_authorized_auto_dispatch.planned_tasks` |

- 产出每个 task 填 `title/objective/scope/depends_on/acceptance_criteria/report_format`；下游字段（`work_item_id/guard_result/prepared_dispatch_id` 等）**留空、由 prepare/派发机器填**。
- **第一刀对象** = 固定测试项目（角色循环+派发真在那跑）。

## 3. 安全死线（本包死线·必须成立）

- **主管 agent 只读**：原样复用咨询的只读 confinement（`sandbox=read-only`+写盘根空），项目只读不可写。**不让它可写/可执行。**
- **不碰派发/闸**：`prepare_authorized_auto_dispatch` 本体、确定性兜底 `deterministic_project_director_planned_tasks`、S1 闸 `decide_real_execution_command`、`command_plan_for` 沙箱、worker 派发路 **0-diff**。主管只**产 planned_tasks**喂 prepare 的现成入参槽。
- **不自动派发**：主管产出 planned_tasks **给用户审**，不自己触发 prepare/dispatch（真派发是用户审过的单独步、走 S1 闸）。
- **不漏敏感**：注入/产出**不含 prompt body / `.codex` / 凭据 / 全文 transcript**（沿用咨询脱敏）。
- **碰线就停**：要碰闸/派发机器 / 自动派发 / 改沙箱可写可执行 / 漏敏感 → 停、回主导线。

## 4. TDD 验收门

- **stub**：DirectorAgent（stub LM）→ 产出合法 `planned_tasks`（字段齐、`depends_on` 自洽）→ **喂得进 `prepare_authorized_auto_dispatch`**（映射对）。
- **解析**：`parse_director_plan` 从 codex JSON 抠出 planned_tasks、空/坏报错。
- **只读 confinement**：复用咨询的只读断言（构造的 codex 请求只读·写盘根空·限项目）。
- **regression**：确定性兜底仍在（没传 planned_tasks 时还走它）、既有 director/dispatch 测试全绿、`cargo test --lib` 计数不降。
- **全量**：`cargo test --lib` / `cargo fmt -- --check` / `git diff --check`。

## 5. 真跑验证（单独步·`#[ignore]`/用户在场·固定测试项目·只读）

stub 验通 + 主导线核实物**后**：在固定测试项目造一个真「已授权方案」，DirectorAgent 真 LM 只读拆解 → 产出 planned_tasks。验：**拆得合理**（任务有目标/依赖/验收、对得上方案 objective、非瞎编）、喂得进 prepare、**只读 confinement 守住**（测试项目没被写、auth 没碰）。真 codex 偶发 flake → retry（记忆 `real-codex-run-flaky-verify-by-artifact`）。`#[ignore]` 默认不跑。

## 6. 本包不做（deferred）

- 不碰派发/S1 闸/worker 路/确定性兜底（主管只产 planned_tasks）。
- 不自动派发（给用户审）。
- tier-2 / API / 别的角色（worker/秘书/全局主管）/ 沿层级反馈边。
- 前端（主管计划的 UI 归布局重做）。

## 7. 回交

- 跑 §4；回交：实现 diff（确认闸/派发/沙箱/确定性兜底 0-diff）+ stub 喂 prepare 证据 + 只读断言 + 真跑 planned_tasks（看拆解质量）+ confinement 实物 + 计数 → 主导线核实物（重跑计数 + 扫 diff 确认没碰闸/派发、只读、没自动派发 + 真跑你在场看拆解）。子线不 commit。

## 8. 不接受为

- 不接受为：碰了 S1 闸/派发机器/确定性兜底 / 自动触发了派发 / 主管 agent 能写能执行 / 漏敏感 / 产出喂不进 prepare / 自动测试真起了 codex。
- 不接受为 S3 主管整体完成（本包只到「主管 agent 能就着真方案智能拆出 planned_tasks 喂 prepare」；真派发联调/UI/别角色另说）。
