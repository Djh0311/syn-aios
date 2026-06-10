# Stage H / H0 Safety Boundary And Task Package Freeze Evidence v1

日期：2026-06-07

## 1. 结论

H0 已完成文档冻结，并已通过全局主管复核。

本轮只接受为：

- 阶段 H 真实执行安全边界冻结。
- H1-H7 任务顺序和前置关系冻结。
- H2 / H3 真实执行前授权条件冻结。
- `.codex` 读写授权矩阵冻结。
- 测试项目选择原则冻结：默认隔离测试项目，不能默认 `mario test` 或真实业务项目。
- UI 显示边界和真实 Tauri 验收要求冻结。

本轮不接受为：

- H1/H2/H3/H4/H5/H6/H7 已完成。
- 通用真实 send / resume 产品化完成。
- 真实 `codex exec` / `codex exec resume` 已执行。
- planned adapters 真实接入完成。
- provider credential / model verification 完成。
- G3 全量真实 Tauri 截图验收补齐。

## 2. 已读取材料

入口和计划：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`

边界文档：

- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

相邻任务证据：

- `tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`
- `evidence/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`
- `handoffs/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1-result.md`
- `tasks/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`
- `evidence/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`
- `handoffs/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1-result.md`
- `tasks/2026-06-06-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md`
- `evidence/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md`
- `handoffs/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1-result.md`
- `tasks/2026-06-06-stage-g-g2-diagnostics-health-and-degraded-state-v1.md`
- `evidence/2026-06-07-stage-g-g2-diagnostics-health-and-degraded-state-v1.md`
- `handoffs/2026-06-07-stage-g-g2-diagnostics-health-and-degraded-state-v1-result.md`
- `tasks/2026-06-06-stage-g-g5-final-authoritative-acceptance-and-deferred-freeze-v1.md`
- `evidence/2026-06-07-stage-g-g5-final-authoritative-acceptance-and-deferred-freeze-v1.md`
- `handoffs/2026-06-07-stage-g-g5-final-authoritative-acceptance-and-deferred-freeze-v1-result.md`

## 3. 关键事实

- G5 已把中间版本冻结为 `accepted_with_deferred_items`。
- G3-B 未完成，只接受为真实 Tauri 启动、区域截图探针成功和 10 / 13 张编号截图。
- E5 Level A 只接受为 preview / guard / stub / continuation record / readback unavailable 边界。
- E5 Level B 只接受为指定 `mario test` 总指导 session 的一次真实 resume 健康探针。
- G1 只接受为 runtime log 最小底座，不接受为真实执行或自动重试。
- G2 只接受为 diagnostics / degraded state 只读底座，不接受为 provider credential / model verification。
- H-I 计划不是执行授权；H0 之前不能直接做 H2/H3。

## 4. 本轮实际操作

已创建：

- `tasks/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md`
- `evidence/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md`
- `handoffs/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1-result.md`

已最小同步：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`

未执行：

- 未改产品代码。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 未读取完整 transcript / rollout。
- 未启动 Tauri、GUI、截图或端口清理。
- 未运行 npm / cargo。

## 5. 冻结范围

### H1-H7 前置关系

- H1 必须在 H0 主管复核后开始，且不允许真实执行或 `.codex` 读写。
- H2 必须在 H1 完成后，以独立任务包授权真实 resume。
- H3 必须在 H1 完成后，建议 H2 最小链路稳定后再做真实 send / 新会话。
- H4 必须基于 H2/H3 的真实路径或明确 stub 路径处理 readback / failure / duplicate guard。
- H5 必须在 H2/H3/H4 最小安全链路完成后，才能接项目工作流真实派发。
- H6 必须基于 H2-H5 的真实事实记录做 UI 产品化和真实 Tauri 验收。
- H7 只做 H 阶段总复核、deferred freeze 和 H-to-I handoff。

### `.codex` 授权

- H0：不允许读写。
- H1：不允许读写。
- H2：允许，但必须由 H2 任务包逐项列明并获授权。
- H3：允许，但必须由 H3 任务包逐项列明并获授权。
- H4：默认不允许；如需复现 readback / failure，必须单独授权。
- H5：只能通过 H2/H3/H4 产品化链路继承最小授权。
- H6：不新增 `.codex` 范围。
- H7：不新增 `.codex` 范围。

### 测试项目

- 默认隔离测试项目。
- 不能默认 `mario test`。
- 不能默认真实业务项目。
- H2/H3 必须再次确认具体项目、路径、备份 / 回滚和 evidence。

### UI / Tauri

- H0 不改 UI，因此不需要截图验收。
- 后续涉及真实执行 UI、权限弹层、项目工作流状态、智能体 send / resume、运行中、通知、待办、管理入口的任务，必须真实 Tauri 截图或明确 incomplete。
- 普通浏览器 smoke 不能替代真实 Tauri。

## 6. 验证命令

本节记录 H0 收尾扫描结果。H0 不改产品代码，因此未运行 npm / cargo。

固定字符串扫描：

- 过度完成声明扫描结果：无越界完成声明。
- 入口方向扫描结果：`CURRENT.md` 和 `tasks/README.md` 均指向 H0 已完成文档冻结并已通过全局主管复核，当前下一步 H1；没有把当前下一步直接写成 H2/H3。
- 产品代码检查结果：H0 标题和 H0 文件 slug 未写入 `prototypes/productized-desktop-shell/src` 或 `prototypes/productized-desktop-shell/src-tauri/src`。

## 7. 风险

- H0 是边界冻结，不提供真实执行证据；主管复核时需要防止后续线程把 H0 误读为 H2/H3 授权。
- E5 Level B 的 `mario test` 健康探针有历史价值，但不能成为 H2/H3 默认测试项目。
- H2/H3 如果没有在任务包里再次冻结 `.codex` 范围、prompt、readback、allowed write roots 和回滚策略，就不应执行。
- H6 之前，UI 仍可能只表达 stub / boundary / partial evidence，不能显示为真实产品化完成。

## 8. 全局主管复核

复核结论：通过。

主管复核确认：

- H0 三份产物已存在。
- 固定过度声明扫描无命中。
- 入口文档没有把当前下一步写成 H2/H3。
- H0 标题和文件 slug 未写入产品代码目录。
- H0 只接受为安全边界和任务包冻结完成，不授权真实 Codex 执行或 `.codex` 读写。

复核后下一步：进入 H1 CodexLocalRunner 架构和数据契约。
