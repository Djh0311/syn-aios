# 实现任务包：咨询 agent 的方案要**自己提执行范围（写范围/工具/检查）**·按任务包结构出方案 · 主导线 → 执行线 v1

日期：2026-07-01　性质：**轻档**（改的是「方案里装什么」的数据装配 + 咨询 prompt/档案；**不碰**咨询只读 confinement、不碰任何执行闸/沙箱/path-lock）。

## 0. 接手须知（冷启即读，本包自包含）

- 你是**执行线**（后端·新开干净上下文）。**子线不 `git add`/`commit`。** 全程中文。
- **背景（用户实测踩到）**：用户在工作台「说目标 → AI 出方案 → 确认 → 一键自动推进」，一键自动推进**停在 blocked**，停因「授权写入范围为空」。根因：咨询 agent 出的方案**写范围恒为空**——`map_consultation_to_c1_input`（[consultant_agent.rs:432-443](prototypes/productized-desktop-shell/src-tauri/src/consultant_agent.rs:432)）把 `allowed_write_roots` **硬编码成 `vec![]`**、`allowed_tools` 硬编码 `["read_file"]`。于是每份 AI 方案都是只读的，凡是"要改东西"的目标，下游一定卡停。
- **用户拍板（原话，硬约束）**：「**不修，写范围也是方案要写的内容，没写就不对，缺少写范围终止了就给我反馈**」。=> **绝不在后端把缺失的写范围默认成某个值**；写范围是**咨询 agent 该在方案里写清的内容**。它没写就是方案有缺，下游停、给反馈（停因文案已建，见下）。
- **用户设计指令（原话）**：「你在给咨询 agent 做设计的时候就**按照我们现在的方式来**就可以啊，你想想你出任务包出方案是什么结构」。=> 咨询的方案要**长得像开发任务包**：目标 + **写范围（哪些目录可写）+ 目标文件 + 工具 + 验收检查** + 风险 + 必停点。现在方案缺的正是这块「执行范围」。
- **关键机制（已核实物·别凭记忆）**：下游主管（`CliDirectorAgent`）拆的**每个 task 的 scope 不是自己编的、是从方案的 `scope_draft` 原样 clone 的**（[director_agent.rs:102-127](prototypes/productized-desktop-shell/src-tauri/src/director_agent.rs:102)）：`allowed_write_scope = scope_draft.allowed_write_roots`、`callable_tool_capabilities = scope_draft.allowed_tools`、`required_checks = scope_draft.allowed_checks`；`target_role` 用主管 LM 的、**空则默认 `codex-dev`**（:110-113）。所以**方案的 scope_draft 是唯一真源**——它空/缺，全链跟着空/缺。
- **两道会连着炸的闸（已核，按命中顺序）**：
  1. **C4 预检**（[c4_c6_workflow_governance_entrypoints.rs:2058](prototypes/productized-desktop-shell/src-tauri/src/c4_c6_workflow_governance_entrypoints.rs:2058)）：`allowed_write_roots` 空 → 「授权写入范围为空，不能生成可派发任务包」→ 阻断。**这是用户当前看到的那条。**
  2. **control_core 授权闸**（[control_core.rs:278-307](prototypes/productized-desktop-shell/src-tauri/src/control_core.rs:278)）：dispatch 的 `target_role_id`(=`codex-dev`) 必须 ∈ `allowed_role_ids`。现在 `allowed_role_ids=["project_consultant"]` → **改完写范围后这条会接着炸**「目标角色不在授权范围内」。=> **写范围和角色授权要一起补，只补写范围仍会卡在角色。**
- **一句话**：把 `ConsultationProposal` + 咨询 prompt/档案 + `map_consultation_to_c1_input` 改成——**咨询 agent 自己在方案里提出「执行范围」（写范围目录 + 目标文件 + 工具 + 检查）**；map 忠实透传咨询提的写范围/工具/检查（**不造默认**），并把「有写范围就结构上必需」的**执行角色授权（codex-dev）接上**。咨询**自身仍结构性只读**（`readonly_codex_consult` 本体 0-diff）；下游所有闸/沙箱/path-lock 本体 0-diff。

## 1. 拍板摘要

- **要做的事**：让咨询 agent 出的方案**像任务包一样带执行范围**——写范围（可写目录）、目标文件、工具、验收检查——由**咨询自己判断并写进方案**；后端只忠实装配、不默认。这样"要改东西"的目标，方案本身就完整，一键自动推进不再因"写范围为空"卡停。
- **为什么**：现在 map 硬编码写范围为空 → 每份 AI 方案只读 → 自动推进必卡。写范围是方案内容（用户拍板），得让咨询产出它。
- **代价**：一轮·后端。改 `ConsultationProposal`（加执行范围字段）+ 咨询 prompt/档案（要它产出）+ JSON 解析 + `map`（透传而非硬编码空 + 接执行角色授权）。**咨询只读 confinement、执行闸、沙箱、path-lock、方案授权人闸——全不碰。**

## 一句话判据

判改动在不在本包——问：**「是不是只让**咨询 agent 在方案里提执行范围**、后端忠实透传（不默认写范围）、并接上写范围结构上必需的执行角色授权，而咨询自身只读 confinement / C4·control_core·path-lock·沙箱本体 / 方案授权人闸全 0-diff？」** 是 → 做；否（后端默认/兜底写范围、动了 `readonly_codex_consult`、碰了任何执行闸/沙箱/path-lock、让咨询自己能写或触发执行、跳方案授权人闸）→ **停、回主导线。**

## 2. 建什么

### 2.1 `ConsultationProposal` 加「执行范围」（[consultant_agent.rs:9-25](prototypes/productized-desktop-shell/src-tauri/src/consultant_agent.rs:9)）
加一个**可空**的结构（可空 = 纯问答咨询本就不需要写范围，见 §3）：
```rust
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub(crate) struct ConsultationExecutionScope {
    pub(crate) write_roots: Vec<String>,   // 写范围：下游执行可写的目录（须在被咨询项目根内）
    pub(crate) target_files: Vec<String>,  // 目标文件：预期改动的具体文件·相对项目根（细粒度·可空）
    pub(crate) tools: Vec<String>,         // 工具：下游 worker 需要的能力（要写就得含写能力，如 write_file/apply_patch）
    pub(crate) checks: Vec<String>,        // 验收检查：怎么验（如 cargo test / npm test）
}
// ConsultationProposal 里加：
pub(crate) execution_scope: Option<ConsultationExecutionScope>, // None = 纯咨询/只读·不需要下游改任何东西
```

### 2.2 咨询档案 + prompt 要它产出（[consultant_agent.rs:229-234 档案 / 266-281 输出格式](prototypes/productized-desktop-shell/src-tauri/src/consultant_agent.rs:229)）
- **档案 `CONSULTANT_V0_PROFILE`**：
  - 产出那行补上执行范围，明说"像开发任务包那样"：`产出:结构化咨询方案——目标/范围/【要下游真改东西时·像开发任务包那样圈"写范围(可写目录)+目标文件+工具+验收检查"】/为什么这么判/风险/必停点/建议下一步。`
  - 边界那行**必须澄清（护 honesty·别让人以为放开了咨询写权）**：`边界:只读、不写、不执行。你永不触发执行、也永不自己写。但你可以(该有时必须)在方案里【提名下游执行需要的写范围/工具】——那是交用户授权、交主管派活的**方案内容**，不是你动手。`
- **prompt 输出格式 JSON**：在结构里加（并给"像任务包"的指引）：
```jsonc
"execution_scope": {
  "write_roots": ["下游执行可写的目录(必须在被咨询项目根内·别写整盘·别碰 .codex)·写范围要窄:只圈目标真需要的目录"],
  "target_files": ["预期改动的具体文件·相对项目根(可空)"],
  "tools": ["下游 worker 需要的工具,如 read_file, write_file"],
  "checks": ["怎么验收,如 cargo test / npm test"]
}
```
  指引原则（写进 prompt·模仿任务包）：`如果这个目标需要下游真改代码/文件,就像开发任务包那样把执行范围写清:哪些目录可写、预期改哪些文件、要什么工具、怎么验收——这是你方案的一部分,你不写下游没法在授权范围内派活、会卡停。若只是回答问题、不需要改任何东西,execution_scope 给 null,并在 scope_note 注明"纯咨询/只读"。写范围要窄:只圈真需要的目录,别图省事圈整个项目根。`

### 2.3 JSON 解析（[consultant_agent.rs:285-359](prototypes/productized-desktop-shell/src-tauri/src/consultant_agent.rs:285)）
- `ConsultProposalJson` 加 `#[serde(default)] execution_scope: Option<ConsultExecutionScopeJson>`（子结构 4 字段各 `#[serde(default)]`）。
- `parse_consultation_proposal` 里映射进 `ConsultationProposal.execution_scope`：codex 给了 execution_scope 且 write_roots 非空 → Some；给了 null / 整块缺 / write_roots 全空 → **None**（别硬造 Some 空壳）。

### 2.4 `map_consultation_to_c1_input`（[consultant_agent.rs:387-450](prototypes/productized-desktop-shell/src-tauri/src/consultant_agent.rs:387)）——核心
把硬编码的 scope_draft 改成**按 execution_scope 分流**：
```rust
let scope_draft = match &proposal.execution_scope {
    // 有执行范围：忠实透传咨询提的写范围/工具/检查 + 接上执行角色授权（结构·见 §3 辨析）
    Some(es) => ProjectConsultationProposalScopeDraft {
        allowed_role_ids: vec!["codex-dev".to_string(), "project_director".to_string()], // 下游执行角色·非"写范围默认"
        allowed_agent_ids: vec![],                              // 空=不约束具体 agent（control_core 对空放行·permissive）
        allowed_read_roots: vec![project_root.to_string()],
        allowed_write_roots: es.write_roots.clone(),            // ← 咨询提的·不再硬编码空
        allowed_tools: es.tools.clone(),                        // ← 咨询提的（要写就含写能力）
        allowed_checks: es.checks.clone(),                      // ← 咨询提的
        allowed_task_package_kinds: vec!["task_package".to_string()],
        stop_conditions,
        max_worker_dispatches: None,
        max_runtime_minutes: None,
    },
    // 纯咨询/只读：保持现状（只读·空写范围）——这**不是**"默认兜底缺失的写范围",是忠实映射"咨询判定不需要改东西"
    None => ProjectConsultationProposalScopeDraft {
        allowed_role_ids: vec!["project_consultant".to_string()],
        allowed_agent_ids: vec![],
        allowed_read_roots: vec![project_root.to_string()],
        allowed_write_roots: vec![],
        allowed_tools: vec!["read_file".to_string()],
        allowed_checks: vec![],
        allowed_task_package_kinds: vec!["task_package".to_string()],
        stop_conditions,
        max_worker_dispatches: None,
        max_runtime_minutes: None,
    },
};
```
- **`target_files` 去哪**：塞进 `proposed_steps` 或 `acceptance_criteria`（让方案卡/主管能看到具体文件），别丢。建议：`proposed_steps` 前面加一条"目标文件：{join}"。实现者定，但**不许丢**。
- **轻量护栏（拒绝·不 fix·对齐"不修"）**：`Some` 分支里，若某 write_root 为空串 / 逃出 `project_root`（canonicalize 后不在 project_root 下）→ **返回 Err**（说清"写范围越出被咨询项目/含空值"），别静默修正、别截断。这只是**更早更清的报错**，不是安全边界（安全边界在下游·见 §3）。

## 3. 安全死线（0-diff / 不碰 / 不绕）

- **咨询自身仍结构性只读·本体 0-diff**：`CliConsultantAgent::consult` 仍走 `codex_local_runner::readonly_codex_consult`（read-only 沙箱·写盘根空·不走执行闸）。**咨询在方案里提写范围 ≠ 咨询能写**——它只往方案数据里填"下游该给谁什么写权"的建议，交方案授权。`readonly_codex_consult` 本体、`consult` 的调用方式 **byte-0-diff**。
- **不造默认写范围（用户硬约束）**：map 只透传咨询提的；execution_scope=None → 写范围空（纯咨询）。**绝不**拿"需要改东西的目标"去默认圈 `project_root` 或任何值兜底。缺了 → 方案有缺 → 下游 C4 预检停 + 停因反馈（已建 [director_agent.rs:778](prototypes/productized-desktop-shell/src-tauri/src/director_agent.rs:778)：「方案缺了它该写的内容(如写范围/工具/检查)」）。
- **`allowed_role_ids` 接 codex-dev 是"结构"不是"写范围默认"·辨析**：用户禁的是"默认**写范围**"。执行角色 `codex-dev` 是角色循环**固定的**执行者（主管 `target_role` 本就默认 codex-dev）。当咨询提出"要写"（execution_scope=Some），"授权那个写的角色"是这句话**结构上蕴含**的必需项、不是对写范围内容的猜测。写范围（写**哪**）仍 100% 来自咨询。这条不违背"不修默认"。
- **下游闸全在·本体 0-diff**：C4 预检（挡空写范围 / 挡 `.codex`·[c4_c6:2058,2073](prototypes/productized-desktop-shell/src-tauri/src/c4_c6_workflow_governance_entrypoints.rs:2058)）/ store 校验（挡读写范围空值·[project_consultation_proposal_store.rs:459](prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs:459)）/ control_core 授权闸（挡越授权·[control_core.rs:278](prototypes/productized-desktop-shell/src-tauri/src/control_core.rs:278)）/ path-lock（圈测试项目·高危#1）/ 方案授权人闸——**一道不碰、一道不绕**。咨询提得不对，下游照样拒。
- **写范围窄·在项目内**：§2.4 护栏保证咨询提的 write_root 在 `project_root` 内；真能不能自动跑另由 **path-lock** 定（只测试项目放行·非测试项目仍 高危#1 挡）——两者正交、都在。
- **不放开**：非测试真实项目真执行（高危#1）/ 多项目接力 / auto-approve / 方案授权自动（**永不**）。

## 4. 验收（**后端线自己机器验 + 端到端两条线合验·不丢给用户**）

- **单测·有执行范围**：注入 stub 咨询（返回 execution_scope=Some，write_roots=[测试项目子目录]、tools=["read_file","write_file"]、checks=["cargo test"]）→ `map` → 断言 `scope_draft.allowed_write_roots` == 咨询提的、`allowed_tools` == 咨询提的、`allowed_checks` == 咨询提的、`allowed_role_ids` 含 `codex-dev`。
- **单测·纯咨询**：注入 stub（execution_scope=None）→ `map` → `allowed_write_roots` 空、`allowed_tools`==["read_file"]、`allowed_role_ids`==["project_consultant"]（**证不默认写范围·只读方案照旧**）。
- **单测·护栏**：注入 write_roots 含空串 / 逃出 project_root → `map` 返回 Err（证拒绝·不 fix）。
- **单测·解析**：带 `execution_scope` 块的 codex 输出样本 → `parse_consultation_proposal` 能解析进 `ConsultationProposal.execution_scope`；不带该块的旧样本 → execution_scope=None（**向后兼容·旧方案不炸**）。
- **真跑**（`#[ignore]`·真 codex·偶发 flake·retry·见记忆 `real-codex-run-flaky-verify-by-artifact`）：真咨询对一个"要改代码"的目标 → 产出非空 write_roots；对一个"纯问答"目标（如红队交叉核对）→ execution_scope=None。**核实物**：读 codex 原始输出 + 解析后的 proposal，别只信管道 exit code。
- **regression**：`readonly_codex_consult` 本体 + `consult` 调用 0-diff（扫 diff 自证）；现有 consultant/map 单测**按新口径调整后全绿**（原来断言 write_roots 恒空的测试要改）；`cargo test --lib` 计数不降；`cargo fmt -- --check`（只本包改的文件·别 fmt 到 codex_db/codex_local_runner/mcp 等既有偏差文件·见记忆 `rustfmt-recurses-mod-children-breaks-0diff`）；`git diff --check`。
- **端到端·北极星**（两条线·真机·**测试项目** `/Users/yoyi/codex-workflow-mario-test`·**不要用户**）：在测试项目工作流上说一个"要改文件"的目标 → AI 出方案（**这次带写范围**）→ 确认 → 全局边界复核 → **一键自动推进**→ 断言**这次不再停在"授权写入范围为空"、也不停在"目标角色不在授权范围内"**、真派 codex-dev worker 写、跑出 proof 文件。这是这整件事通没通的判据。

## 5. 本包不做（deferred）

- 咨询动态收窄到**精确文件级**写范围（tier-1 只读注入文档·未必知确切子目录；v1 允许圈到目录、target_files 尽力即可）。
- tier-2（API loop）里咨询按需读更多再定写范围。
- 用户在方案卡里**手改** write_roots 的编辑 UI —— UI 线（若要，另起）。
- 让主管 LM 提 `target_role` 越出 codex-dev 时的角色白名单治理（本包 `allowed_role_ids` 覆盖默认 codex-dev + director；主管乱编角色是另一条鲁棒性线）。
- 停因在 UI 的呈现/可操作按钮 —— 见姊妹包 `2026-07-01-ui-auto-advance-stop-reason-verify-actionable-v1.md`（且那块显示码**已存在**、多半只差重编）。

## 6. 回交

- 跑 §4；回交列：**改了哪几处**（ConsultationProposal 加字段 / 档案 / prompt / 解析 / map 分流 + 护栏）+ 单测证据（有范围透传·纯咨询不默认·护栏拒绝·解析向后兼容）+ 真跑 proof（要改→非空 write_roots；问答→None）+ **`readonly_codex_consult` 与执行闸/沙箱/path-lock 本体 0-diff 自证**（扫 diff 重点看"没在后端兜底写范围、没碰咨询只读、没碰任何执行闸")+ 计数 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 不接受为：后端**默认/兜底**了缺失的写范围（违用户硬约束）/ 动了 `readonly_codex_consult` 让咨询能写或触发执行 / 碰了 C4·control_core·path-lock·沙箱本体 / 跳了方案授权人闸 / 让 write_root 逃出 project_root 也过 / 只补写范围没接执行角色（会卡在"目标角色不在授权范围内"、等于没通）。
- 不接受为角色循环整体完成（本包只到"咨询方案自带完整执行范围、要改东西的目标能过 C4+角色两闸";端到端真派 worker 由 §4 北极星验、UI 呈现另包）。
