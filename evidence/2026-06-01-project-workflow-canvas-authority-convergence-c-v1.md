# Project Workflow Canvas Authority Convergence Task C v1

日期：2026-06-01

## 本轮目标

执行 Task C：项目工作流画布权威收敛。

目标边界：

- 项目页工作流画布是当前项目 workflow 的主入口。
- 独立 `CanvasView` 暂定为实验、模板或后置能力，不作为项目 workflow state 的事实源。
- 右侧运行入口应回到项目页，不表现为第二个权威画布。
- 本轮只做 UI/入口/文档小步收敛，不合并两套数据模型。

## 禁止项执行情况

| 禁止项 | 本轮结果 | 依据 |
|---|---|---|
| 不执行真实 Codex | 已遵守 | 未运行 `codex exec` / `codex exec resume`。 |
| 不启动 MCP canvas run | 已遵守 | 未点击或调用 `canvasStartRun`、MCP orchestrator 或 MCP run。 |
| 不改 workflow state JSON | 已遵守 | 未写 `workflow-state.v0.json`。 |
| 不把独立 canvas 文件层改成项目事实源 | 已遵守 | 未修改 `src-tauri/src/mcp/**`，未改 CanvasDefinition 存储。 |
| 不把工作台改成通用节点执行器 | 已遵守 | 只改入口和文案，保留项目画布的“不做通用节点执行器”边界。 |
| 不读 `/Users/yoyi/.codex` | 已遵守 | 本轮未打开该目录或其中任何文件。 |
| 不读 auth/token/`.env`/完整 transcript | 已遵守 | 本轮未读取这些内容。 |
| 不迁移数据库 | 已遵守 | 未运行迁移命令。 |

## 依据

- `decisions/2026-06-01-project-workflow-canvas-authority-v1.md` 已决策：项目 workflow state 是项目工作流事实源；独立 `CanvasView` 和 MCP canvas/run 文件层暂定为实验/模板/后置能力。
- `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md` Task C 要求：产品入口和代码边界收窄到“项目工作流画布以项目 workflow state 为权威；独立 CanvasView 暂为实验/模板/后置能力”。
- `evidence/2026-06-01-workbench-architecture-readonly-audit-v1.md` 已指出风险：项目页画布和独立可编辑画布并存，容易形成两个都像权威画布的入口。

## 改动清单

| 文件 | 改动 | 判断 |
|---|---|---|
| `prototypes/productized-desktop-shell/src/App.tsx` | 全局左侧 `workflow` 入口 label 从“工作流”改成“实验画布”；右侧栏 `running` 从“运行中工作流/运行中/run”收敛为“项目运行”；右侧统计从“工作流”改成“项目工作流”。 | 入口收敛，不改 view key 和路由逻辑。 |
| `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx` | 项目工具入口 label 改成“项目工作流”；项目画布 header 改成“项目工作流主入口”；派生画布 header 改成“项目工作流画布”； badge 改成“项目事实”；增加“事实源：项目 workflow state / 派生读模型”。 | 明确项目页是主入口，仍只读 `workflowState` / `derived_workflow`。 |
| `prototypes/productized-desktop-shell/src/views/CanvasView.tsx` | 独立画布 header 改成“实验 / 模板画布”； fallback 节点从 `workflow` 改成 `template`；运行区文案改成“实验运行”。 | 降低独立画布权威感，不改 canvas 存储、保存、启动、停止实现。 |
| `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx` | 测试断言从旧“工作流编排/工作流画布”更新为“项目工作流主入口/项目工作流画布”，并断言“事实源：项目 workflow state / 派生读模型”。 | 跟随 UI 权威文案，测试覆盖没有缩窄。 |
| `CURRENT.md` | 当前任务记录从 Task B 下一步建议更新为 Task C 已完成，并指向本 evidence/handoff。 | 文档状态收敛，不改产品规则。 |

## 未改内容

- 未修改 Rust 后端。
- 未修改 `src-tauri/src/mcp/**`。
- 未修改 workflow state schema 或任何真实 workflow state JSON。
- 未修改状态机、派发、回收、四角色工作流机器、MCP canvas run 逻辑。
- 未把独立 `CanvasView` 合并进项目 workflow state。

## 验证

| 命令 | 结果 | 说明 |
|---|---|---|
| `npm run typecheck` | 通过 | 最终通过。 |
| `npm run test:offline-interaction` | 先失败后通过 | 第一次失败原因是测试仍断言旧文案“工作流编排”；更新断言后通过，输出 `offline interaction tests passed: 2`。 |
| `npm run build` | 通过 | Vite 输出 chunk 大小 warning：`index-*.js` 超过 500 kB；不是本轮新增错误。 |

未运行 `cargo test --lib`：

- 本轮只改前端 UI 文案和前端测试。
- 架构计划测试要求中，前端-only 任务至少跑 `npm run typecheck`、相关前端测试和 `npm run build`。

## 结论

Task C 的保守切片已完成。

当前入口关系已经收敛为：

- 项目页 `项目工作流` 是项目 workflow 主入口。
- 项目页画布标为 `项目工作流主入口` / `项目工作流画布` / `项目事实`，事实源显示为项目 workflow state 和派生读模型。
- 全局独立 `workflow` route 仍保留，但对用户显示为 `实验画布`，独立页面显示为 `实验 / 模板画布`。
- 右侧运行入口显示为 `项目运行`，点击仍回到项目页。

剩余风险：

- 独立 `CanvasView` 仍保留可保存、启动实验运行、停止实验运行能力；本轮只通过入口和文案降权，没有冻结按钮或删除功能。
- `ProjectsView.tsx` 仍包含任务包草稿、派生账本和状态机细节；Task C 没有把这些面板收进节点详情或右侧展开，避免扩大改动。
- 未来如要让独立画布影响项目 workflow，仍必须另开迁移计划和控制核心接入计划。
