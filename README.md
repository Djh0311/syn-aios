# Syn

Syn is a local-first personal AI workbench for durable work across long-running projects. It coordinates role identity, project scope, persistent conversations, handoffs, permissions, audit evidence, recovery, and daily attention without treating a model transcript or frontend cache as product truth.

Syn is built as a Tauri 2 desktop application with a Rust backend, React and TypeScript frontend, and SQLite persistence.

Public repository: https://github.com/Djh0311/syn-aios

License: [Apache-2.0](LICENSE). Copyright 2026 呆头鹅.

## Current maintainer preview

The repository contains implemented foundations for server-owned role sessions, persistent turns and provider handles, explicit handoffs, source-first inbox and open-loop coordination, deterministic daily windows, audit receipts, and recovery-oriented state machines.

The project is still pre-1.0 and in active development. External connectors, a packaged public release, broad adoption, and long-term real-world operation are not claimed. See [`docs/current-state.md`](docs/current-state.md) for the exact implementation and evidence boundary.

## Why Syn exists

Most agent tools optimize a single prompt or coding session. Syn explores the harder maintenance problem: how a person and multiple long-running agents can keep identity, scope, knowledge, state, evidence, and recoverability coherent over time.

Syn is not a Codex shell, and it is not only a chat app, project manager, knowledge base, or coding workflow tool.

## Explore the current tree

There is no public installer yet. The desktop prototype lives in [`prototypes/productized-desktop-shell`](prototypes/productized-desktop-shell). Build and run notes there are development notes, not a release claim.

```text
git clone https://github.com/Djh0311/syn-aios.git
```

After clone, start from this README, then [`docs/current-state.md`](docs/current-state.md) and the product documents below. Do not treat isolated test fixtures, handoffs, or old evidence screenshots as a shipped product.

## 产品入口

- Syn 是什么、长期必须满足什么：`docs/product/syn-product-canon-v1.md`
- 哪些文件现在有效、各自能决定什么：`docs/product/authority-register-v1.md`
- 尚未拍板的问题：`docs/product/candidate-register-v1.md`
- 所有智能体怎样使用资料和技能说明：`docs/product/knowledge-infrastructure-canon-v1.md`
- 系统怎样分层和协作：`docs/workbench-system-architecture-v1.md`
- 普通界面、专业界面和开发界面分别显示什么：`docs/workbench-frontend-display-boundary-v1.md`
- 当前代码和开发事实：`docs/current-state.md`
- 当前总开发计划：`docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`

产品正本、架构正本、当前实现、候选设想、验收证据和施工权限是不同的东西。验收报告只证明点名版本和场景；交接只说明当时做到哪里；开发护栏只约束本轮怎么施工。三者都不单独定义 Syn 最终是什么。

## 开发入口

先读 `AGENTS.md`，再按轻量开发护栏的当前链进入：

```text
docs/harness/plan.md → 当前阶段 → 唯一当前任务包（leaf，当前叶）→ docs/harness/authorization.json
```

没有活动阶段和当前任务包时，不从旧计划、旧授权、交接或验收报告推导新的施工权限。远端、部署、发布、真实服务提供方、真实账号和真实消息仍需对应的明确授权。

贡献与安全报告见 [`CONTRIBUTING.md`](CONTRIBUTING.md) 和 [`SECURITY.md`](SECURITY.md)。
