# 实现任务包：P2 · 接咨询 LM 出方案（件 A·出方案自动化）· 主导线 → 执行线 v1

日期：2026-06-27　性质：**轻档**（咨询结构性只读·不碰执行闸；真咨询跑真 codex·读only·用户在场）。

## 0. 接手须知（冷启即读，本包自包含）

- 你是**执行线**（新开、干净上下文）。**子线不 `git add`/`commit`。** 全程中文。
- **上游**：决策 `decisions/2026-06-27-role-loop-semi-auto-orchestration-light-tier-v1.md` + 设计 `docs/plans/2026-06-27-role-loop-auto-orchestration-spec-v1.md`（§件 A）。P1（授权后自动推进）已 committed `bd503a3`。本包做**件 A：接咨询 LM**——让「出方案」从**手填模板**变 **AI 真咨询出**。
- **现状缺口**：咨询 agent（`CliConsultantAgent`·价值已证）**没接 Tauri 命令**，现 UI 是手填模板（面板写明"不会调用真实咨询智能体"）。
- **零件全现成**（核过·committed）：
  - `ConsultantAgent::consult(ctx, question) -> ConsultationProposal`（`consultant_agent.rs:28`·trait）+ `CliConsultantAgent`（`:175` 区·tier-1·复用咨询只读 codex `readonly_codex_consult`）
  - `map_consultation_to_c1_input`（`consultant_agent.rs:387`·`ConsultationProposal` → `CreateProjectConsultationProposalInput`）
  - `create_proposal`（`project_consultation_proposal_store.rs:53`·写进方案 store·status=PendingUserConfirmation）
  - `load_project_context`（建 `ProjectContext` 注入）
  - 它们现只测试/手动用。**本包只新增一个把它们串起来的命令。**
- **先读**：① `consultant_agent.rs`（consult trait / CliConsultantAgent / map_consultation_to_c1_input / `readonly_codex_consult` 死线注释）② `project_consultation_proposal_store.rs:53` create_proposal ③ `lib.rs` 咨询 `#[ignore]` 真跑（`s3_consultant_*` 范本）④ S3 咨询 spec `docs/plans/2026-06-25-s3-agent-layer-consultant-first-slice-spec-v1.md`（注入策展文档正文 / timeout 420s / flake retry）⑤ 决策 `decisions/2026-06-25-consultant-readonly-guard-exemption-v1.md`（咨询只读·读真实项目不碰高危#1）⑥ 记忆 `tier1-codex-exec-no-ondemand-read-inject` / `real-codex-run-flaky-verify-by-artifact`。
- **一句话**：加一个 async 命令——收目标 → `load_project_context` → `CliConsultantAgent.consult(目标)` → `map_consultation_to_c1_input` → `create_proposal` → 返回新方案；咨询**结构性只读**（不碰执行闸）、consult/map/create 本体 0-diff。

## 1. 拍板摘要

- **要做的事**：让用户**说个目标 → AI 真咨询出方案**（结构化·目标/范围/为什么/风险/必停点），写进方案 store，喂现有方案授权流（你确认 → 边界复核 → P1 自动推进）。**取代手填模板。**
- **代价**：一轮·**后端命令**·复用现成零件、串起来。
- **关键边界**：咨询**全只读**（`readonly_codex_consult`：read-only 沙箱 + 写盘根空·**不碰执行闸**）；**读真实/非测试项目不碰高危#1**（#1 是写/执行·不是读·决策 `2026-06-25-consultant-readonly-guard-exemption`）。出的方案仍要**用户确认 + 边界复核**才生效（人闸不省）；下游执行仍 path-lock 圈测试项目（P1/链入口）。

## 一句话判据

判改动在不在本包——问：**「是不是只加一个『目标 → 咨询 LM 出方案 → 写进方案 store』的只读命令、复用现成 consult/map/create 本体、咨询不碰执行闸、出的方案仍走人确认?」** 是 → 做；否（咨询碰写/执行、改 consult/create 本体、跳过用户确认、自动让方案生效）→ **停、回主导线。**

## 2. 建什么

**async 命令** `run_project_consultation`（或近名·照 P1/C1 范本 async + spawn_blocking·咨询真 codex 长耗时）：
1. **入参**：`project_root` / `workflow_id` / `goal`（用户目标）/ actor。
2. `load_project_context(project_root)` → `ProjectContext`（注入策展文档正文等·咨询 spec §3）。
3. **咨询 LM**：`director`/`consultant` 注入式——`consultant.consult(&ctx, goal)` → `ConsultationProposal`（生产注 `CliConsultantAgent`·真 codex 只读；stub 测试注假咨询不起 codex）。timeout 沿用咨询 spec（420s）。
4. `map_consultation_to_c1_input(proposal, project_root, workflow_id, ...)` → `CreateProjectConsultationProposalInput`。
5. `create_proposal(path, &input, ts, write_id)` → 写进方案 store（status=PendingUserConfirmation·**不自动确认**）。
6. **返回**：新建的方案（`proposal_id` / 目标 / 范围摘要 / status）——前端拿去展示给用户审（喂现有方案授权 UI）。
- **不自动确认、不自动边界复核、不自动推进**——出方案就停，等用户走方案授权（人闸）。

## 3. 安全死线

- **咨询结构性只读**：`readonly_codex_consult`（read-only 沙箱 + 写盘根空·非豁免 guard reason 仍拦·`command_plan_for` 0-diff）——**不碰执行闸、不写、不起 worker**。
- **不跳过人确认**：命令只**出方案（PendingUserConfirmation）**，**不确认、不边界复核、不让授权生效**（principles §4 / §0.3·人闸不省）。
- **0-diff**：`consultant_agent.rs`（consult/CliConsultantAgent/map）/ `project_consultation_proposal_store.rs`（create）/ `readonly_codex_consult` / `command_plan_for` 沙箱 **本体不改逻辑**——只新增命令 + 注册。
- 读真实/非测试项目 = 只读·**不碰高危#1**（决策 `2026-06-25-consultant-readonly-guard-exemption`）；下游执行仍 path-lock 圈测试项目（P1/链·本包不碰）。
- 自动测试不真起 codex（stub 咨询）；真咨询仅 `#[ignore]` + 用户在场。

## 4. TDD 验收门

- **stub 全链**：注入假咨询（不起 codex）→ 命令 → 方案写进 store（status=PendingUserConfirmation）→ 返回方案；断言**没自动确认**（store 里无 user_confirmed、无 active 授权）。
- **只读不碰闸**：断言命令路径不触发执行闸 / 不写 worker（结构性：咨询无执行工具；可断言无 dispatch / 无 chain run）。
- **map 无损**：`ConsultationProposal` → create input → store 的目标/范围/风险字段无损。
- **regression**：consult/map/create/`readonly_codex_consult` 本体 0-diff（扫 diff 自证）；既有测试全绿、计数不降。
- **全量**：`cargo test --lib` / `cargo fmt -- --check`（只本包改的文件·别 `--write` 碰预存偏差文件）/ `git diff --check`。

## 5. 真跑验证（单独步·`#[ignore]`/用户在场·真咨询）

stub 验通 + 主导线核实物**后**：真项目（测试项目或 spec 的猫猫点菜实例）+ 目标 → **经命令** → `CliConsultantAgent` 真 codex 只读咨询 → 出**落地非幻觉**方案（引用真读到的文档）→ 写进 store。验：**一个命令把"目标→AI 出方案"跑通**、方案 grounded、status=PendingUserConfirmation（没自动确认）、咨询只读 confinement 守住（没写、`.codex`/auth 没碰）。真 codex flake → retry（咨询偶发·核方案实物）。

## 6. 本包不做（deferred·显式）

- **前端"AI 出方案"按钮 + 触发 UI** = P3（件 D·真机），本包只到后端命令。
- **件 B 授权后自动推进** = 已 done（P1·`bd503a3`）。
- tier-2 / API 咨询 impl（同契约·后续）；档案行为层 derive；别角色；自动确认方案（**违 §0.3·永不做**）。

## 7. 回交

- 跑 §4；回交：实现 diff（确认 consult/map/create/`readonly_codex_consult` 本体 0-diff、只 +命令/注册/测试）+ stub 全链证据（方案写进 store·没自动确认）+ 只读不碰闸证据 + map 无损 + 计数 + 真跑证据（目标→AI 出方案·grounded·status=PendingUserConfirmation·confinement）→ 主导线核实物（重跑 + 扫 diff + 真咨询用户在场 + 核只读 confinement）。**子线不 commit。**

## 8. 不接受为

- 不接受为：咨询**碰写/执行/起 worker**（结构性只读破了）/ 改了 consult·create·`readonly_codex_consult` 本体没经批 / 命令**自动确认方案或让授权生效**（跳人闸）/ 自动测试真起 codex / 出的方案幻觉（不基于真读文档）。
- **不接受为 S3 agent 层整体完成**（本包只到「目标→AI 出方案的后端命令」；前端触发/件 D UI / 别角色 / tier-2 另算）。
