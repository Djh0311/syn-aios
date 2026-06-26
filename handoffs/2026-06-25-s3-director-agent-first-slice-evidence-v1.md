# 回交：S3 项目主管 agent 第一刀（LM 拆解已授权方案 → planned_tasks → 喂 prepare）· 执行线 → 主导线 v1

日期：2026-06-25　性质：**轻档**（主管 agent 只读·只产 planned_tasks；真派发下游走 S1 闸+用户审，本包不碰）　任务包：`tasks/2026-06-25-s3-director-agent-first-slice-v1.md`　上游：咨询第一刀已收口 `df64e10`（tier-1 注入 harness 已证）

## 0. 一句话结论

复用咨询 harness 建 `DirectorAgent`：读已授权方案 + 项目上下文注入 → LM 只读拆解 → `planned_tasks`（目标/依赖/验收/汇报）→ 喂 `prepare_authorized_auto_dispatch.planned_tasks`。**主管只读、不碰闸/派发/确定性兜底、不自动派发；每个任务 scope 取自已授权 scope_draft（LM 不得扩范围）**。**执行线未 commit。**

## 1. 建了什么（复用咨询 harness·新文件 `director_agent.rs`）

| | 复用咨询的 | 换成 |
|---|---|---|
| 只读 codex | `codex_local_runner::readonly_codex_consult`（read-only·写盘根空·项目只读）| **原样** |
| 上下文 | `ProjectContext` + `load_project_context`（含 tier-1 注入文档正文）| **原样** + 注入已授权方案正文（user_goal/goal_summary/proposed_steps/scope/验收）|
| JSON 抠取 | `consultant_extract_json_block` | **原样复用** |
| 档案 | 咨询档案写法 | **主管档案**：拆已授权方案成 worker 任务·定目标/依赖/验收/汇报·只读只规划不派发 |
| 解析 | `parse_consultation_proposal` | `parse_director_plan` → `Vec<ProjectDirectorPlannedTask>` |
| 喂点 | →C1 | → `prepare_authorized_auto_dispatch.planned_tasks` |

- LM 只定**任务拆解**（title/objective/target_role/depends_on/acceptance_criteria/report_format）；**scope 由 `director_task_scope_from_proposal` 从已授权 `scope_draft` 派生**（read/write/tools/checks/stop 全取授权值）——**LM 不能扩范围**。
- 下游字段（work_item_id/guard_result/prepared_dispatch_id/...）**留空**，由 prepare/派发机器填。

## 2. 安全死线（§3·全守住）

- **主管只读** ✅：复用 `readonly_codex_consult`（sandbox=read-only·写盘根空），项目只读不可写。
- **0-diff** ✅：确定性兜底 `deterministic_project_director_planned_tasks`(c4_c6) / `prepare_authorized_auto_dispatch`(c4_c6) / S1 闸 `decide_real_execution_command` / `command_plan_for` 沙箱(codex_local_runner) / worker 派发(commands) 全 **0 行 diff**——主管只产 planned_tasks 喂 prepare 现成入参槽。
- **不自动派发** ✅：DirectorAgent 只 `plan()` 返回 planned_tasks，不触发 prepare/dispatch（真派发是用户审过的单独步）。
- **不漏敏感** ✅：沿用咨询脱敏（注入/产出经只读 harness；scope 从授权 draft、不回显敏感）。
- **scope 不扩** ✅：每任务 scope 硬取自授权 scope_draft（测试钉死 `plan[].scope.allowed_write_scope == proposal.scope_draft.allowed_write_roots`）。

## 3. 验收门

| 门 | 结果 |
|---|---|
| stub→合法 planned_tasks·喂 prepare | ✅ `s3_director_stub_plans_valid_tasks_feed_prepare`（字段齐·depends_on 自洽·scope 取自授权·下游留空·喂进 `fixture_project_director_prepare_input.planned_tasks`）|
| 解析 | ✅ `s3_director_parse_plan_extracts_tasks`（从 codex JSON 数组抠 tasks·scope 取自授权）+ `_rejects_empty_or_bad`（无 json/空数组报错）|
| 只读 confinement | ✅ 复用咨询只读路（`readonly_codex_consult`，已由 `s3_readonly_consult_request_is_structurally_readonly` 钉死）|
| regression | ✅ `cargo test --lib` = **600 passed / 0 failed / 30 ignored**（+3 director stub +1 ignored）；确定性兜底/既有 director/dispatch 测试全绿 |
| 0-diff | ✅ c4_c6(兜底+prepare)/commands/real_execution_command/codex_local_runner |
| fmt / git diff --check | ✅ 干净 |
| 范围 | `director_agent.rs`(新) + `lib.rs`(include! + 4 测试) |

## 4. §5 真跑（单独步·用户在场·固定测试项目·只读）

写了 `#[ignore]` `s3_director_real_plan`：真 LM 拆解一个已授权方案 → planned_tasks。**显式跑**：`cargo test --lib s3_director_real_plan -- --ignored --nocapture`。验：拆得合理（任务有目标/依赖/验收·对得上方案 objective·非瞎编）、每任务 scope 仍取自授权 draft、confinement（测试项目没被写、auth 没碰）。真 codex 偶发 flake → retry。
> 注：fixture 方案是 `create_active_project_director_authorization_fixture` 的通用方案（证机制）；要看丰富拆解质量，跑时可换个有真目标的方案。

## 5. 主导线核实物 + 收口
- 重跑 `cargo test --lib`（600/30）；扫 diff 确认 director_agent 只产 planned_tasks、**c4_c6 兜底/prepare/S1 闸/沙箱 0-diff**、没自动派发。
- §5 真跑你在场：跑 `s3_director_real_plan -- --ignored` 看拆解质量 + confinement（测试项目没被写）。
- 执行线不 commit；你核实物 + commit + CURRENT 回写。

## 6. 本包没做（deferred）
真派发联调（prepare→S1 闸→worker 真跑用 director 的 planned_tasks）/ tier-2 / 别的角色（worker/秘书/全局主管）/ 前端（主管计划 UI 归布局重做）/ 沿层级反馈边。本包只到「主管 agent 能就着真方案智能拆 planned_tasks 喂 prepare」。
