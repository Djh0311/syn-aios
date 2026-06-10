# Handoff：Stage E / E7 Session Adapter Model Boundary Acceptance v1

日期：2026-06-06

## 1. Stage E 结论

阶段 E 总复核完成，结论为：

```text
accepted_with_deferred_items
```

接受范围：

- E1 adapter descriptor / model credential readonly foundation。
- E2 session operation boundary contract / readonly UI。
- E3 provider / model / credential availability readonly boundary。
- E4 session continuation protocol / permission preview / guard。
- E5 Level A `codex-local` controlled continuation code path / stub / sidecar / readback unavailable boundary。
- E6 runtime session attention / readback failed and unavailable boundary。
- E-to-F handoff 已完成，F1 可以开始。

不接受范围：

- 真实 `codex exec` / `codex exec resume` 已执行。
- 真实 prompt 已发送。
- 真实 readback 已完成。
- planned adapters 真实接入。
- provider credential store / model verification 完成。
- stop / restart / delete / export / favorite 完成。
- 自动重试、完整 runtime log、诊断中心、真实 Tauri 全面验收或中间版本最终验收完成。

## 2. Deferred 项

- E5 Level B 真实 send / resume：必须另行获得用户对具体 session、cwd、prompt、读写范围、回滚和证据的明确授权。
- Planned adapters 真实接入：留给后置 adapter 专题或最终蓝图，不进入 F1。
- Provider credential store / model verification / provider probe：留给后置 provider / credential 专题。
- stop / restart / delete / export / favorite：留给独立 operation 任务。
- runtime log：进入 G1。
- diagnostics / health / degraded state：进入 G2。
- real Tauri acceptance and screenshots：进入 G3。
- middle-version end-to-end replay：进入 G4。
- final authoritative acceptance / deferred freeze：进入 G5。

## 3. 是否允许进入 F1

允许进入 F1，但 F1 需要单独任务包。

F1 允许做：

- 项目工作流画布读模型收敛。
- 从 workflow state、authorization、task package、memory packet、permission、readback、audit 等既有来源派生统一画布节点 / 边 / 状态 / badge / attention。
- 把 React Flow 继续限定为渲染和交互层，不作为事实源。

F1 不能做：

- 不执行 Level B 真实 send / resume。
- 不把 planned adapters 改成可执行。
- 不实现 provider credential / model verification。
- 不启动 MCP canvas run。
- 不改 `workflow-state.v0.json` 顶层结构。
- 不做 runtime log / diagnostics / real Tauri 验收。
- 不把 readback unavailable / failed 显示成 0 条真实读回。

## 4. F1 开始前必须读

- `CURRENT.md`
- `AUTHORITY.md`
- `tasks/README.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `decisions/2026-06-01-project-workflow-canvas-authority-v1.md`
- `evidence/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`

建议同时查阅 E1-E6 evidence / handoff，尤其是 E5 Level A 和 E6 readback boundary。

## 5. 当前权威入口

- 当前事实：`CURRENT.md`
- 权威索引：`AUTHORITY.md`
- 任务队列：`tasks/README.md`
- 阶段计划：`STAGE_PLAN.md`
- 产品线入口：`README.md`
- E/F/G 细化计划：`docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- E7 evidence：`evidence/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`

## 6. 边界声明

本轮只做文档、证据、扫描和权威入口同步。没有改产品代码，没有运行真实 `codex exec` / `codex exec resume`，没有发送真实 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 auth/token/`.env`/keychain/OAuth/provider credential/完整 transcript，没有迁移数据库，没有改 `workflow-state.v0.json`。
