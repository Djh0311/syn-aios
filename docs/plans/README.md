# Plans Directory

Purpose: store PRDs, design documents, implementation plans, migration plans, rollout plans, and review plans.

This template describes an installed project's `docs/plans/**` directory. It does not define where the standard rule source package stores its own development plans; source-package plans belong under repo-root `plans/**`.

## Required Plan Links

Current active product-line plan:

- `2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md`：Stage J 后续“日常可用 Codex 工作台产品化”计划。K0、K1、K2、K2.5、K3-Level-A 和 K3-Level-B 字段冻结已完成；K2 结论为 `accepted_with_deferred_items`，K2.5 结论为 `accepted`，K3-Level-A 结论为 `accepted`，K3-Level-B 字段冻结结论为 `accepted_with_pre_execution_blocker`。本轮未执行新的真实 `codex exec` / `codex exec resume`，未读写 `/Users/yoyi/.codex`。下一步进入 K3-B0 专用 bridge / harness。

Completed / historical product-line plans:

- `2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`：G5 后续 H-I 开发计划；阶段 H 先产品化 `codex-local` 真实自动化工作流，阶段 I 再抽象多 agent / 多模型中立协作协议。H0 已完成文档冻结并已通过全局主管复核；H1 已完成并已通过全局主管复核；H2 通用真实 resume 产品化任务包已创建并已完成 Phase B `mario test` 真实 resume 产品化探针；H2.0 real resume preflight authorization guard 已完成，只接受为执行前授权预检 / blocked attempt / audit 底座；H2.1 real resume authorization matrix and execution decision freeze 已完成，只接受为执行前授权矩阵和主管决策材料完成；H2.2 real resume authorization readiness read model and readonly UI 已完成，只接受为执行前授权准备读模型和只读 UI 完成；H2.3 real resume request builder and CodexLocal guard bridge 已完成，只接受为 request builder / guard bridge 完成；H2.4 real resume execution authorization and fixture freeze 已完成，只接受为执行前授权包和 fixture freeze 完成；H2.5 real resume runner execution path and authorized fixture run Phase A 已完成；H2.6 Phase B readiness / fixture session binding / runtime log hardening 已完成；H2.7 Phase B authorization / fixture / target session confirmation 已完成为当时的授权准备复核和阻断状态冻结；H2.8 real execution permission dialog / audit summary / readiness decision surface 已完成并回收；后续 H2 Phase B 已在 2026-06-08 对 `mario test` 授权并完成一次真实探针；H3-A new session authorization / fixture / boundary freeze 已完成；H3.1 new session request / guard / permission envelope / no-op runner 已完成并已通过全局主管复核；H3-B final approval / real new session fixture run 已执行一次隔离 fixture 真实 probe 但失败分类完成，产品路径已补 `--skip-git-repo-check`，等待新的 retry 授权；H4 readback / failure / timeout / duplicate guard Level A 非真实产品化已完成并通过全局主管复核；不授权直接执行新的 H2/H3/H4 真实 Codex。

- `2026-06-01-workbench-architecture-implementation-plan-v1.md`：从最终蓝图倒推当前 app 的架构落地执行计划；Task A 架构只读审计、Task B 保守拆模块切片、Task C 项目工作流画布权威收敛、Task D 项目黑板最小只读切片和控制核心切片均已完成。
- `2026-06-06-stage-e-f-g-refinement-plan-v1.md`：阶段 E/F/G 细化计划；E3-E7、F1-F5、G1/G2/G3-A/G3-C/G4/G5 已完成，G3-B 只接受为 10 / 13 真实 Tauri 部分截图证据。

Every implementation plan should link to:

- `docs/requirements-matrix.md` requirement IDs.
- `docs/task-queue.md` task IDs.
- `docs/decisions.md` decision IDs that constrain implementation.
- `docs/open-questions.md` unresolved questions and conservative defaults.

## Recommended Plan Structure

```markdown
# <Feature Or Phase> Implementation Plan

Goal:

Architecture:

Relevant Requirements:
- R-001

Relevant Decisions:
- D-001

Open Questions:
- Q-001, or None

Tasks:
- TASK-001

Verification:

Risks:
```

## Rules

- Do not use plans as the only source of current truth. Update `docs/current-state.md` and `docs/requirements-matrix.md` as work progresses.
- If a plan changes behavior or scope, add or update a decision in `docs/decisions.md`.
- If a plan depends on unresolved information, add the question to `docs/open-questions.md`.
