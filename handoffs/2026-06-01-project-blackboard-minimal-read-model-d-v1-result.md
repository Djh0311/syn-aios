# Project Blackboard Minimal Read Model Task D Result

日期：2026-06-01

## 结果

Task D 已完成最小只读切片。

本轮建立了项目黑板模型和读模型：

- `ProjectBlackboard`
- `BlackboardEntry`
- `BlackboardEntryKind`
- `BlackboardSourceRef`
- `BlackboardPromotionDecision`

黑板条目能承载：

- 子智能体汇报
- 风险
- 权限请求
- 工具摘要
- 记忆候选
- 知识引用

所有条目默认只是候选，升级状态是 `candidate_pending_control_core`。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `CURRENT.md`
- `evidence/2026-06-01-project-blackboard-minimal-read-model-d-v1.md`
- `handoffs/2026-06-01-project-blackboard-minimal-read-model-d-v1-result.md`

## 没做的事

- 没有新增黑板写入命令。
- 没有改 workflow state JSON 结构。
- 没有迁移数据库。
- 没有执行真实 Codex。
- 没有读取 `/Users/yoyi/.codex`、auth、token、`.env` 或完整 transcript。
- 没有让黑板推进 workflow 状态。
- 没有让黑板写正式记忆。
- 没有把知识引用当成记忆。

## 验证结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过。
- `npm run build`：通过，有既有 Vite chunk 大小 warning。
- `cargo test --lib project_blackboard_read_model_derives_candidates_without_state_promotion`：通过。
- `cargo test --lib`：通过，82 passed、1 ignored。
- `cargo fmt --check`：失败，输出覆盖大量既有 `src/lib.rs` 和 `src/mcp/**` 格式差异；本轮未做全仓库格式化。

## 手动测试清单

1. 打开应用，进入“项目”。
2. 选择一个有项目 workflow 的项目，打开“项目工作流”。
3. 在“运行前检查”附近检查是否出现“项目黑板”面板。
4. 如果该项目已有派发、汇报、权限请求、工具摘要或显式知识/记忆引用，黑板里应显示对应条目，状态应是 `candidate`。
5. 检查每条黑板条目的“升级”状态，应是 `candidate_pending_control_core`。
6. 检查黑板面板不应出现“批准”“推进状态”“写正式记忆”之类按钮。
7. 检查知识引用和记忆候选是两个不同条目：`knowledge_ref` 不应被显示成正式记忆。
8. 本轮手动测试不要点击“派发指令”“审核后派发”“启动四角色工作流机器”或“启动实验运行”，这些不属于 Task D 验收。

## 下一步建议

不要直接补一个黑板写 JSON 的接口。

如果继续做写入能力，建议先进入 Task E 或单开 D-followup：由控制核心定义“黑板候选如何被确认、拒绝、升级为审计事件、正式事实或正式记忆”，再接写入命令。
