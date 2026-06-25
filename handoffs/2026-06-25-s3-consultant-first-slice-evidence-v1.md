# 回交：S3 咨询第一刀 v0（契约 + CLI 只读 impl + 静态档案 + ProjectContext）· 执行线 → 主导线 v1

日期：2026-06-25　性质：**建是轻档**（Rust + stub 测试）；首次真咨询=单独步（只读上真项目·用户在场）　任务包：`tasks/2026-06-25-s3-consultant-first-slice-v0-build-v1.md`　spec：`docs/plans/2026-06-25-s3-agent-layer-consultant-first-slice-spec-v1.md`　上游：S2-3 已入库 `e53895d`/`dc2bcb4`

## 0. 一句话结论

咨询第一刀建好、stub 全链验过：**契约 trait + ConsultationProposal + ProjectContext 装配 + v0 静态档案 + CliConsultantAgent（结构性只读）+ 喂 C1**。咨询 codex **结构性硬钉只读**（read-only 沙箱·写盘根空·写死在构造里），`command_plan_for`/沙箱/判决体/A 线 **0-diff**。**执行线未 commit。** 真咨询（§6·只读上猫猫点菜·用户在场）是单独下一步。

## 1. 建了什么

**新文件 `consultant_agent.rs`**（include! 进 crate root，同 commands.rs）：
- `trait ConsultantAgent { fn consult(&self, ctx, question) -> Result<ConsultationProposal> }`（**稳定契约缝**，tier-2 换 impl 同契约后）。
- `ConsultationProposal`（目标/范围/为什么/风险与不确定/必停点/下一步）。
- `ProjectContext` + `load_project_context`：找入口文档(README/CURRENT/index)全文注入、递归文档地图、**git 优先→无 git 用 mtime 降级**、黑板/记忆有才注空就跳（**防御式降级·有啥塞啥**）。
- `CONSULTANT_V0_PROFILE`（spec §3C 静态档案，写死先不 derive）。
- `CliConsultantAgent`（tier-1）：拼 prompt(档案+ctx+question+输出格式) → `readonly_codex_consult` → 抠 json 块成 ConsultationProposal。
- `map_consultation_to_c1_input`：ConsultationProposal → `CreateProjectConsultationProposalInput`（喂 C1，下游一字不动）。

**`codex_local_runner.rs` +2 函数（86 行纯新增，command_plan_for/沙箱本体 0-diff）**：
- `build_readonly_consult_request`：**写死 `sandbox="read-only"` + `allowed_write_roots=[]`、零权限参数** → 结构性只读、可单测。
- `readonly_codex_consult`：builder → 现成 `command_plan_for` → `RealCodexLocalPhaseBProcessRunner.run_phase_b` → 读 last_message。

## 2. 安全死线（§3）

- **咨询 codex 结构性只读** ✅：`sandbox="read-only"`+`allowed_write_roots=[]` 写死在 `build_readonly_consult_request`、不收权限参数 → 调用方拿不到改可写/可执行的机会；codex 只读被咨询项目(cwd)、写不了、跑不了命令。单测 `s3_readonly_consult_request_is_structurally_readonly` 钉死。
- **command_plan_for/沙箱 0-diff** ✅：只**调**它不**改**它（diff 无删除行）。
- **不走 worker 执行闸** ✅：只读 ≠ 执行，不经 `decide_real_execution_command`；但结构性只读自带 confinement。
- **0-diff** ✅：`decide_real_execution_command`(判决体)/`session_continuation_store`(A 线)/`workflow_chain_controller`(连环) 全 0 行；既有封堵(commands.rs)未碰。
- **限项目不碰凭据**：cwd=被咨询项目根、`allowed_write_roots=[]`（无 --add-dir 越界）、请求不带授权 scope/凭据 ref。
- **自动测试不真起 codex** ✅：stub `ConsultantAgent`；真 codex 仅 `s3_real_consult_mao_mao_dian_cai`(`#[ignore]`)。

## 3. 「无漏网」表更新——新增 read-only-confined 类

把只读咨询 codex 路登记进入口×守卫表（spec §3 要求）：

| 真 codex 入口 | 守卫类 | 能写/能执行? |
|---|---|---|
| automation b1/b2/k3_b · H5 · S1 worker/dispatch/experiment/chain | (a) **path-lock** | 测试项目内可写（轻档）|
| PCR phase_b 族 | (b) **授权矩阵** | 授权后任意项目（重档）|
| `*_phase_a` ×3 | (c) prepare-only | 不执行 |
| **咨询 codex（CliConsultantAgent→readonly_codex_consult）** | (d) **read-only-confined**〔本包新增〕 | **永不**（read-only 沙箱·写盘根空·结构写死）|

**结论**：新增的咨询路是 (d) 类——既非 path-lock 也非授权矩阵，但**结构性只读、永远写不了/执行不了**，不是「能写又无闸」的漏网。审计可见、知道它只读。

## 4. 验收门

| 门 | 结果 |
|---|---|
| 契约+装配(stub) | ✅ `s3_consultant_full_chain_stub_feeds_c1`（ProjectContext 装配→stub 咨询→映射→喂 C1 成功）|
| ProjectContext 装配+降级 | ✅ `s3_project_context_assembles_entry_doc_map_and_signal`（入口全文/地图/mtime 降级/空黑板）+ `_degrades_when_no_entry_doc`（无入口优雅退化）|
| 只读 confinement 单测 | ✅ `s3_readonly_consult_request_is_structurally_readonly`（read-only/写盘根空/cwd=项目/无授权 scope）|
| 解析 | ✅ `s3_parse_consultation_proposal_extracts_json_block` + `_rejects_empty` |
| regression | ✅ `cargo test --lib` = **590 passed / 0 failed / 28 ignored**（S2-3 基线 584/27 + 6 S3 stub；+S3 真咨询 #[ignore]）|
| 0-diff | ✅ command_plan_for/沙箱/判决体/A线/连环/既有封堵 |
| fmt / git diff --check | ✅ 我的新代码 fmt 干净；git diff --check 干净（剩余 fmt 是 codex_db/mcp-storage/codex_local_runner 旧区 pre-existing 债·非我）|
| 改动范围 | `consultant_agent.rs`(新) + `codex_local_runner.rs`(+86·2 只读函数) + `lib.rs`(include! + 6 stub + 1 #[ignore] 测试) |

## 5. 主导线核实物 + §6 真咨询

- **核 diff**：command_plan_for/沙箱无删除行(0-diff)、juddecide/A线/chain 0-diff、咨询请求结构写死 read-only。
- **重跑**：`cargo test --lib`（590/28）。
- **§6 真咨询（单独步·你在场·只读）**：`cargo test --lib s3_real_consult_mao_mao_dian_cai -- --ignored --nocapture`——上猫猫点菜问防幻觉真题（红队 19 条 vs 开发计划 M0 交叉）。验：答案落地引用真读到的文档、给能进循环的结构化方向、喂得进 C1；**confinement 实物**：codex 只读没写文件、`~/.codex/auth.json` mtime 没变、没越出项目。注意真 codex 偶发 flake → retry（记忆 `real-codex-run-flaky-verify-by-artifact`）。
- **收口**：执行线不 commit；你核实物 + commit + CURRENT 回写「S3 咨询第一刀 stub 验过·真咨询待你在场跑」。

## 6. 本包没做（deferred·spec §6）

tier-2（6 只读工具 harness loop·等 API）/ 接 API / 档案行为层 derive / 别的角色 / NL 拆解 / 一句话启动。**第一刀只证「LM+harness 能就着真项目落地答」**——真咨询跑过才算证到。
