# 当前工作排期 v1（咨询线维护）

> **2026-06-16 已过期 / superseded**：本文是 2026-06-14 的调度快照。其后 Stage R 已收口（2026-06-16，见 `CURRENT.md`）——A 线（harness）、C 线（研究落盘 / 冲突报告 / agentmemory 退役 / checkpoint-audit）、B 线（R3 Level B → R5 → Stage R 收口）均已完成；记忆层研究归档见 `docs/research/memory-layer-research-and-conflict-digest-v1.md`。**当前事实以 `CURRENT.md` 为正本**，本文仅留作历史调度记录，不再维护。

日期：2026-06-14（更新于 harness 合并 + agentmemory 退役后）
出自：咨询线（Claude）。性质：跨两条线的当前工作排期；不替代 `CURRENT.md` / stage-r 执行正本，只做调度视图。
**一句话判据**：想知道"接下来干啥"，看 A / B / C 三线各自的"下一步"。

## A 线 · harness 改进 —— ✅ 已完成并合并 main（`9c17bb8`）
- [x] A1 P2 修复（catalog 计数 79→80）
- [x] A2 复核结论归档（`evidence/2026-06-14-harness-hg-batch-review-claude-v1.md`）
- [x] A3 咨询线合并前最后一扫（全绿：边界/catalog/shape-gate）
- [x] A4 合并 `harness-hg` → main（fast-forward；worktree + 分支已清）

## B 线 · 产品主线（Codex；需用户在场）
- [ ] **B1 R3 Level B —— 进行中**：B0 preflight 已发 Codex（只读侦察 state root，用户已预授权读、免每步回问；禁读路径硬底线不变），等 B0 报告。
  之后 B1 production apply 起需**逐窗用户在场、另拍**；研究 SQLite schema 为输入、**R3 契约为正本**。
- [ ] B2 R5 文档与蓝图对齐 → Stage R 收口。

## C 线 · 尾巴（与 B 并行安全）
- [x] **C1** 研究报告落盘 ✅ → `syn-research/2026-06-14-memory-agent-research-v2.md`（266 行/24KB，含完整 DDL/核实/来源；研究线已存，2026-06-15 咨询线核实）。注：syn-research 非 git 跟踪，durability 待定（见下）
- [x] **C2** 冲突报告落盘 → `docs/research/2026-06-14-three-projects-vs-canon-conflict-report-v1.md`
- [x] **C3** agentmemory 9 件退役（catalog 已改；AgentMemory 设计点已入 `memory-layer-design §3.5`，退役的只是没接线的脚本）
- [ ] **C4** checkpoint-audit 工具（kickoff 已写；排在 C1 收口后发，**不依赖 R3**）

## 依赖与并行
- A / B / C 三线互不挡：不同区；B0 只读、C 线动文档，均非"真实执行"，不踩"R3 收口前不开多 agent 并行"闸。
- C4 不依赖 R3，只等 C1 收口；B2 依赖 B1。
