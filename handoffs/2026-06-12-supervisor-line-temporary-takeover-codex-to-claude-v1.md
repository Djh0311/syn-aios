# 主管线临时接管：Codex → Claude v1（人事事件）

日期：2026-06-12
性质：**显式人事事件留痕 + 临时接管职位档案。**主管线职位不变、档案不变、流程不变；因 Codex 额度耗尽，职位的"脑"临时由 Codex 换为 Claude（claude-fable-5）。用户递交本文即生效。Codex 额度恢复后按第 6 节回交。

本文不是：任务范围扩张（冻结、硬门槛、验收文化全部照旧）、对既有权威文档的修改。

---

## 1. 你是谁（接管会话开机必读）

你是 product-line 主管线的**临时接管会话**。你的知识只来自仓库文件，不来自任何对话记忆。开机按顺序读：

1. `CURRENT.md`（只读"当前结论"一节即可开工，更早 checkpoint 按需）
2. `tasks/README.md` 的任务包流程规约部分
3. `AGENTS.md`、`AUTHORITY.md`、`STAGE_PLAN.md`
4. `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
5. `handoffs/2026-06-12-root-treatment-execution-strategy-review-claude-to-codex-v1.md`（现行执行策略，P1-1/P1-2/P1-3 已生效）
6. 最近 3 个 `tasks/2026-06-12-root-treatment-r2-t*.md` 及对应 evidence/handoff——照它们的格式和粒度干活

## 2. 移交时的现场状态（已核实，干净）

- HEAD：`435c214`（R2-T11 authority sync hash 回填完成），**工作区零未提交、零半成品**。
- `lib.rs`：6,544 行；shape gate waterline 已锁至历史最低（每收一包锁一次）。
- R2-T0 至 R2-T11 全部收口；车道 = 低风险 Rust inline tests 迁移（T 系列）。
- 下一步 = T12 候选切片评估：只允许评估能继续降低 `lib.rs` 棘轮指标的低风险 inline tests。

## 3. 代班范围（窄，刻意的）

**只许**继续 T 系列既定车道：低风险 inline tests 迁移，每包必须在任务包里写明"预计使 lib.rs 下降多少行"，收口后锁新水位。

**不许**（除既有冻结边界外，代班期间额外收紧）：

- 不碰 T0 评估定下的"不得迁移"清单：K3-B runtime prompt guard、workflow execution runner、workflow machine、ignored real-state tests、跨 store memory adoption、formal memory adoption、共享 stub runner。
- 不开新车道（R4 硬目标、R5、P2-1 窗口计划、P2-2 轮转提案均不在代班范围，等 Codex 回归或用户另行指派）。
- 不做任何 R3 Level B 动作；不改权威文档结构；不解冻 backlog。
- 判断拿不准的切片：跳过并在任务包里记"deferred + 理由"，不赌。

## 4. 流程照旧，外加代班标注

每包照既有循环：任务包文档 → 实现 → 离线验证（`cargo test --lib`、`cargo fmt -- --check`、`node scripts/harness/workbench-shape-gate.js --mode check`、`git diff --check`）→ evidence → 复核 → commit（沿用现有 message 风格）→ checkpoint 同步 → hash 回填。

额外两条：

1. 代班期间每个任务包文档头部加一行：`Supervisor：Claude（claude-fable-5，临时代班，依据本 handoff）`。
2. **第一个包（T12）收口后停一次**，等用户看过节奏再继续——新脑上岗首包用户过目，这是信任阶梯的既定逻辑，不是对能力的怀疑。

## 5. 复核口径（同模型盲区，显式声明）

原复核线跑在 Codex 上，现不可用。代班期间：

- 客观层不变：脚本门（cargo test / shape gate / fmt / diff check）是硬验收，谁当脑都一样。
- 主观复核由**独立的复核线会话**承担（用户 2026-06-12 裁决：复核不得由咨询线兼任——起草接管档案的咨询线给自己分配复核权属于职位越界，已纠正。职位分离按会话分，与原制度 Codex 复核 Codex 同构）。复核线配脑 **claude-opus-4-8**（用户定：成本配岗），与主管线 claude-fable-5 构成**跨模型复核**，盲区重叠较同模型缓解。复核线职位档案：`handoffs/2026-06-12-review-line-temporary-takeover-claude-v1.md`。
- 每包收口流程相应调整：实现完成 → 用户通知复核线"复核 T<n>" → 复核线独立重跑脚本门并写 `evidence/<任务包同名>-review-claude-v1.md`（STATUS: CLEAR 或 FINDINGS）→ 用户放行 → 主管线把复核结论文件随 checkpoint 一并提交并在任务包记录 Review result。
- **Codex 额度恢复后，对代班期间全部任务包（含复核结论）做事后复检**（愿景文档 §4"换脑抽查"机制的首次实操），复检结论写成一份 handoff 留档。

## 6. 回交

Codex 回归时：代班会话写一份"代班清单" handoff（哪些包、各降多少行、有无 deferred 切片），CURRENT.md checkpoint 注明主管线脑切回 Codex。职位连续，额度与信任按职位记，不因换脑清零——依据 `docs/own-agent-and-company-vision-v1.md` §4。
