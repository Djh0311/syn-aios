# 决策 · 下一步 = 解封工作流引擎（限固定测试项目）（v1 · 2026-06-21）

## 拍板
**下一步开发 = 把被硬 block 的工作流执行引擎解封到「只对固定测试项目可跑、真实项目继续挡」**，用来调试「中间·半自动」多节点编排。

## 现状（已亲验）
工作流执行引擎当前被无条件 block：
- `run_workflow_machine` → `legacy_product_command_blocked`，`prototypes/productized-desktop-shell/src-tauri/src/commands.rs:1790-1798`
- `execute_workflow_node_dispatch` → 同上，`commands.rs:1651-1659`
- `read_workflow_node_dispatch_result` → 同上，`commands.rs:1674-1683`
- block 原因：不满足 H5 统一产品命令边界（permission/continuation/audit/readback），`real_execution_command.rs:144-177`。
- 当前唯一真执行 = 甲·手动中转（`manual_relay`，GUI 真发里程碑 `9b7360a`）。

## 理由
- 在**固定测试项目**里调试，真实损害≈0（编排再乱也只动那个测试项目，没有真东西可毁）。按 `AGENTS.md` 这属**轻档**（temp/沙箱真跑一次看效果），不需要每次授权、不需要先建整套 H5 治理。
- 全套 H5 治理（先问/审计/回读）留到将来真把它对准**真实项目**时再加——那才是重档。现在不做。
- 现在卡住的根因不是缺某个开关，而是只有「全自动（危险，该锁）」和「全手动（甲，已开）」两极，缺中间「一步一审、人盯着」那档；而这档的危险度≈甲（已允许）。

## 风险 / 边界（重档那一下）
- **改 block 闸本身 = 高危清单 #3（改安全闸/审批逻辑）= 重档**：需用户**明确授权那一下** + 看 diff 确认「真实项目仍挡死、沙箱 `workspace-write`+`--add-dir` 限定仍在、拒审批绕过仍在」。
- 真跑 codex 进测试项目仍是**受控的一次**，不开自动连环、不碰 `~/.codex`。
- 解封范围**只到固定测试项目**；真实项目执行、自动连环（乙）不在本决策内，仍锁。

## 与并行任务的关系
- 全项目「事实⇄文档」整理（`handoffs/2026-06-21-full-project-fact-reconciliation-task-package-v1.md`）由另一对话并行做，**只读、不碰本引擎、不解封**。本引擎状态以本决策 + 主导线解封进度为准。

## 落地步骤（待执行）
1. 主导线亲读 `workflow_execution_entrypoints.rs` 编排实现 + block 闸，定**最小改动**（放行测试目标、真实项目继续挡）。
2. 定固定测试项目（路径 / 怎么标识为「测试目标」）。
3. 用户授权那一下 + 审 diff → 改闸。
4. 在测试项目里把**单个节点**端到端真跑通一次（中间·半自动第一口）。

---

## 第 1 步 scoping 结论（2026-06-21 · 主导线亲核）

### A. 已亲读核实的安全事实（沙箱强不强制 = 解封安不安全的关键）
- 每个节点执行的指令在 `workflow_execution_entrypoints.rs:1754-1813` 构建：
  - `codex-dev` 角色 → `sandbox_mode="workspace-write"`，写入根**限死** `[execution_root]`；其余角色 → `read-only`、无写入根。
  - `forbidden_actions` 明文禁：读 auth.json/.env/密钥/token、碰执行目录外的其他业务项目、删改 codex 会话、联网装依赖。
- `codex_resume_options_for_context`（:162-189）：真实业务派发**强制要完整 user-reviewed 指令**，缺了直接报错「已阻止真实业务派发」，不会裸跑。
- `command_plan_for`（`codex_local_runner.rs:1317-1326`）：argv **无条件**带 `--sandbox <mode>` + 每个写入根一个 `--add-dir`；**全程不注入任何 approval-bypass / full-auto / dangerously 标**。
- 执行链（亲核）：`execute_workflow_node_dispatch_at`（`workflow_run_dispatch_entrypoints.rs:841`）→ `runner.resume_with_options`（:866）→ spawn codex。
- **结论：沙箱在引擎层强制、写入限死 `execution_root`、不碰凭据、不绕审批。对固定测试项目解封，真实损害被沙箱关在测试项目内 = 安全。**

### B. Explore 两个 ⚠️ 复核后降级
- 「工作流路径不走 manual_relay 防绕过校验（manual_relay.rs:2133-2154）」：属实，但那是**二次断言**；一次构建走的同一个 `command_plan_for` 已无条件加沙箱、从不加 bypass 标。缺的是冗余复检，不是沙箱缺失。→ 建议顺手把该断言补到工作流路径。
- 「不走 7 道 H5 闸」：属实，工作流走较轻的 `plan_authorization`（`workflow_run_dispatch_entrypoints.rs:835`）。对测试项目调试**正合适**（本就不要重治理）；真实项目才需 H5（`preview_h5_project_workflow_dispatch` 已建，`commands.rs:730`）。

### C. 最小改动设计（待 step 2 + 用户授权）
- **改点 = block 命令本身**（`commands.rs:1790 / 1651 / 1674`），集中、好改。
- **加一道「固定测试项目 + env 钥匙」双闸**：
  - `request.project_root` == 指定**固定测试项目路径** **且** env 钥匙已设 → 读 index、构造真 runner、调已存在的 `run_workflow_machine_for_index_at`（它自带 `find_index_project` 再校验）。
  - 其余任何项目 / env 没设 → **维持现状 blocked，行为零变化**。
- 触碰 block 闸 = 高危 #3 → **重档：需用户授权那一下 + 审 diff**（复核点：真实项目仍 `Err(blocked)`、沙箱构建没动、只有测试路径+env 放行）。

### D. 待用户拍板（step 2 前）
1. **固定测试项目指哪条路径？**（须在项目索引内；代码现有候选：`/Users/yoyi/codex-workflow-mario-test`、`/tmp/mario-test`、`/Users/yoyi/Documents/mario test`）
2. env 钥匙：复用 `MANUAL_RELAY_REAL_CODEX_CONFIRM` 还是新设专用（如 `WORKFLOW_ENGINE_TEST_CONFIRM`）？建议专用，免与甲混。
3. 顺手把沙箱二次断言补到工作流路径？建议补。

### E. step 2 实现时再确认（不挡方案）
- `workflow_machine_execution_root()` 写入落在测试项目哪（根 or 子目录）。
- 真 `CodexResumeRunner` impl 构造（复用甲 GUI 那条 → `spawn_codex_like_process_capture_to_files`）。
- `sandbox_mode=None`（safe_probe 探针）→ `request.sandbox` 映射（safe_probe 非真业务）。

---

## ⚠️ 第 1 步深核修正（2026-06-21 · 推翻上面 C「翻闸即可」的假设）

深核发现:**工作流引擎从没真跑过 Codex,只有 stub runner(全在 test)**——所以「解封 = 翻闸 + 调现成真 runner」是错的,那个真 runner **不存在**。

- `CodexResumeRunner` trait(`lib.rs:66`)的实现**只有 4 个 stub**:`lib.rs:5361/5429/5460/5515`,全在 `#[cfg(test)]`。**无任何生产级真实现。**
- `run_workflow_machine_for_index_at` / `execute_workflow_node_dispatch_for_index_at` 的所有调用点都在 `lib.rs` 测试里、传的全是 stub。
- 真 spawn 函数 `spawn_codex_like_process_capture_to_files` **只被 `manual_relay.rs:1068` 调**;工作流 runner 路径**根本不通到它**(Explore 先前的调用链这里报错了)。
- 真能跑 Codex 的是**另一个 trait** `CodexLocalPhaseBProcessRunner`(真实现 `RealCodexLocalPhaseBProcessRunner`,`codex_local_runner.rs:86/131`),走 H5 / Phase B 路径(`session_continuation_store.rs`、`project_workflow_automation.rs`),**不是**工作流机器。
- CLI `__run_workflow_machine_real` 也是 blocked(`lib.rs:98`)。

**修正后的真实工作量**:解封 = ① **建一个真 `CodexResumeRunner`**(把工作流引擎接到真实 codex spawn——复用现成 spawn 机器 / 或经 `RealCodexLocalPhaseBProcessRunner`)+ ② 测试项目闸 + ③ 把测试文件夹弄成可跑(`/Users/yoyi/codex-workflow-mario-test` 存在但**非 git repo**;且工作流走 `resume` 需要先有会话)。**是一块真实现,不是一行闸。**

**安全结论不变**:沙箱机器是真的、已亲核(§A);新 runner 复用它即安全。

**新出现的设计岔路(建库前必须先定)**:被 block 的工作流命令,代码里点名的替代就是 H5 路径(`preview_h5_project_workflow_dispatch` + `controlled_session_continuation`),而 H5 **已有真 runner + 治理**。所以「让测试项目真跑」也许不该给 legacy 工作流机器**新建** runner,而该**走已存在的 H5 真路径**。这是建任何东西前要先核清的岔路——否则可能复活一条代码库已弃用的旧路。
