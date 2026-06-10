# Evidence: Project Blackboard Task D Receive And Next v1

日期：2026-06-01

## 做了什么

- 复核 Task D 交付：
  - `evidence/2026-06-01-project-blackboard-minimal-read-model-d-v1.md`
  - `handoffs/2026-06-01-project-blackboard-minimal-read-model-d-v1-result.md`
- 抽查模型和读模型入口：
  - `src-tauri/src/types.rs` 中新增 `ProjectBlackboard`、`BlackboardEntry`、`BlackboardEntryKind`、`BlackboardSourceRef`、`BlackboardPromotionDecision`。
  - `src/lib/types.ts` 中新增对应前端类型。
  - `src-tauri/src/lib.rs` 派生 `project_blackboards`。
  - `ProjectsView.tsx` 新增只读“项目黑板”面板。
- 同步当前入口：
  - `AUTHORITY.md`
  - `README.md`
  - `tasks/README.md`
  - `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`
- 补充纠偏：
  - `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md` 的 Task D 小节曾残留“最小写入命令”表述；已改为“模型和只读读模型”，并明确黑板写入必须放到 D-followup 或 Task E 之后，经控制核心确认边界和迁移计划约束。

## 接收判断

Task D 可以接收为完成最小只读切片。

接受为：

- 项目黑板模型已建立。
- 项目黑板只读读模型已建立。
- 项目页能只读展示子智能体汇报、风险、权限请求、工具摘要、记忆候选、知识引用。
- 黑板条目的升级状态默认是 `candidate_pending_control_core`。

不接受为：

- 黑板写入命令已完成。
- 黑板候选已可升级为正式事实。
- 黑板候选已可升级为正式记忆。
- 控制核心确认命令已完成。
- workflow state JSON 结构已迁移。

## 验证依据

来自 Task D handoff：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --lib project_blackboard_read_model_derives_candidates_without_state_promotion` 通过。
- `cargo test --lib` 通过，82 passed、1 ignored。
- `cargo fmt --check` 未通过，原因是既有 `src/lib.rs` 和 `src/mcp/**` 格式差异；本轮未做全仓库格式化。

## 边界

- 本轮没有改 `prototypes/**` 代码。
- 本轮没有运行测试。
- 本轮没有读取 `/Users/yoyi/.codex`。
- 本轮没有执行 `codex exec` 或 `codex exec resume`。
- 本轮没有启动 MCP canvas run。
- 本轮没有写 workflow state JSON。
- 本轮没有迁移数据库。
- 本轮只修正文档口径，没有新增能力实现。

## 下一步

建议进入 Task E：控制核心命令收敛。

如果要继续补黑板写入能力，必须先单开 D-followup：

- 定义黑板候选如何经控制核心确认。
- 定义确认后如何升级为正式事实、正式记忆、审计事件或状态变化。
- 不要直接补一个写 workflow state JSON 的黑板接口。
