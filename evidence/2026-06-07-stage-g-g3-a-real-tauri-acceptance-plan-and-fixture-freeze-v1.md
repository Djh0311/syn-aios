# Evidence: Stage G / G3-A Real Tauri Acceptance Plan And Fixture Freeze v1

日期：2026-06-07

## 结论

G3-A 已完成，接受为：

```text
real_tauri_acceptance_plan_and_fixture_freeze_completed
```

只接受为：

- G3 真实 Tauri 验收拆为 G3-A / G3-B / G3-C。
- G3-B 截图路径、文件命名、覆盖清单和 fixture 边界已冻结。
- 降级规则已冻结：普通浏览器 smoke 不能替代真实 Tauri。
- 权限边界已冻结：启动 Tauri、截图、端口检查 / 清理需在 G3-B 执行前确认。

不接受为：

- 真实 Tauri 已启动。
- 截图已采集。
- G3 整体完成。
- G4 回放完成。
- G5 最终冻结或阶段 G 完成。

## 改动文件

- `tasks/2026-06-07-stage-g-g3-a-real-tauri-acceptance-plan-and-fixture-freeze-v1.md`
- `evidence/2026-06-07-stage-g-g3-a-real-tauri-acceptance-plan-and-fixture-freeze-v1.md`
- `handoffs/2026-06-07-stage-g-g3-a-real-tauri-acceptance-plan-and-fixture-freeze-v1-result.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`

## 冻结内容

截图目录：

```text
evidence/tauri-verification/2026-06-07-stage-g-g3/
```

最小覆盖：

- 权限确认弹层。
- 项目页。
- 项目工作流画布。
- 节点详情。
- 智能体会话中心。
- send / resume 边界。
- 记忆中心。
- 知识库。
- 任务记忆包预览。
- 通知、待办、运行中。
- 管理 runtime log + diagnostics。

## 验证

本轮是文档 / 计划任务，未改产品代码。

已完成：

- G3-A 任务包 / evidence / handoff 创建。
- G3-B / G3 / G4 / G5 已完成冒领禁止项已写入任务包。

已补齐：

- 权威入口已同步到 G3-A 已完成，G3-B 待开始。

## 过程说明

G3-A worker 已先创建三份 G3-A 文档。主管主线程在等待超时后误判 worker 卡住，随后接管同名文件创建；经后续核对，当前三份文件内容完整，后续只做入口同步，不再覆盖 G3-A 主体内容。

未执行：

- 未启动 Tauri。
- 未截图。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未读取完整 transcript / rollout、auth、token、`.env`、secret、keychain、OAuth、provider credential。

## 当前结论

G3-A 已完成。下一步是 G3-B Real Tauri Manual Screenshot Acceptance 待开始；执行前需要确认启动真实 Tauri、截图和必要端口检查 / 清理授权。
