# Evidence: Workflow C6 Global Final Result Review, User Result View And Stage C Acceptance v1

日期：2026-06-05

## 结论

C6 已完成：工作台新增全局主管最终结果复核、用户结果决定和阶段 C 验收摘要链路。阶段 C 的 C1-C6 受控闭环可接受为阶段完成；中间版本整体、完整记忆系统、真实 worker / Codex 执行、完整自动重试和运维日志仍未完成。

接受为：

- 全局主管可以基于 C1-C5 前置证据记录最终结果复核。
- 全局最终复核记录写入现有 `workflow-state.v0.json` 的 `reviews[]` 和 `audit_events[]`。
- 用户可以单独记录本次结果决定，且不能由秘书、项目主管、worker 或 system 代替。
- 用户结果决定记录写入现有 `reviews[]` 和 `audit_events[]`。
- 阶段 C 验收摘要写入现有 `artifacts[]` 和 `audit_events[]`。
- `WorkflowResultSummaryReadModel` 会派生 final review status、user decision status、stage C gate 摘要、open blockers、deferred items 和 warnings。
- 项目工作流侧栏新增“C6 结果 / 阶段验收”摘要和三个确认动作。
- 确认弹层明确最终复核不代表用户接受、用户决定只适用于本次结果、阶段 C 摘要不执行真实 worker 且不写正式记忆。
- C6 不写 observation，不生成 MemoryCandidate，不写正式记忆。

不接受为：

- 中间版本整体完成。
- 完整记忆系统完成。
- M7-M13 完成。
- 真实 worker 已执行。
- 真实 Codex 已执行。
- `codex exec` 或 `codex exec resume` 已执行。
- C5 worker report 已成为正式事实。
- C5 `process_fact` observation 已成为正式记忆。
- 用户已接受所有未来任务结果。
- 完整自动重试系统、完整运行日志体系、真实 Tauri 全面验收或运维诊断完成。

## 关键实现

- 后端类型 / 命令：
  - `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 前端类型 / Tauri wrapper：
  - `prototypes/productized-desktop-shell/src/lib/types.ts`
  - `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- 前端确认和 UI：
  - `prototypes/productized-desktop-shell/src/App.tsx`
  - `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
  - `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- 测试：
  - `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

后端实现要点：

- 新增 `GlobalFinalResultReviewInput`、`UserResultDecisionInput`、`GenerateStageCAcceptanceSummaryInput`、`StageCAcceptanceGate`、`StageCAcceptanceSummary`、`WorkflowResultSummaryReadModel`。
- 新增命令：`record_global_final_result_review`、`record_user_result_decision`、`generate_stage_c_acceptance_summary`。
- 全局最终复核校验 C2 confirmed proposal、C3 active authorization、C4 prepared dispatch / task package artifact、C5 worker report 和 process fact decision。
- `accepted` final review 必须引用已确认过程事实；未处理的 rework / block process fact 会阻断 accepted。
- 用户 `accept_result` 必须引用当前最新且 accepted 的全局最终复核。
- C4 task package artifact 归属兼容既有 schema：允许通过 artifact `workflow_id` 或 `source_ref` work item 回链判断。
- 未新增 `workflow-state.v0.json` 顶层数组，未修改 workflow / work item / node / dispatch 状态枚举。

前端实现要点：

- `Workflow.result_summary` 接入 TS 类型和读模型展示。
- `ProjectsView` 在工作项编排卡中新增 C6 结果摘要面板。
- 面板显示 final review、user decision、stage gate 计数、process facts、open items 和 deferred items。
- 面板提供 `记录最终复核通过 / 记录需要修改 / 记录阻断`、`记录用户接受 / 记录用户要求修改 / 记录用户拒绝`、`生成验收摘要` 确认动作。
- `PermissionDialog` 新增 C6 三类确认摘要。

## 验收结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - 输出：`offline interaction tests passed: 9`
- `npm run build`
  - Vite build 通过；保留 chunk size warning。
- `cargo test --lib global_final_result_review`
  - 3 passed。
- `cargo test --lib user_result_decision`
  - 1 passed。
- `cargo test --lib stage_c_acceptance`
  - 1 passed。
- `cargo test --lib process_fact`
  - 3 passed。
- `cargo test --lib dispatch_readback_stats`
  - 6 passed。
- `cargo test --lib workflow_authorization`
  - 1 passed。
- `cargo test --lib plan_authorization`
  - 8 passed。
- `cargo test --lib`
  - 202 passed, 1 ignored。
- `rustfmt --check src/control_core.rs src/commands.rs src/types.rs src/observation_store.rs src/codex_transcript.rs src/codex_db.rs src/lib.rs`

说明：

- `cargo test --lib` 保留既有 warning：`JsonRpcError::invalid_params` 未使用。
- `npm run build` 保留 Vite chunk size warning，不影响构建通过。

## UI / Smoke

已完成：

- Vite dev server 已启动：`http://127.0.0.1:5176/`
- HTTP smoke：
  - `curl -sS -I http://127.0.0.1:5176/` 返回 `HTTP/1.1 200 OK`。

未完成：

- 真实窗口 / 截图验收未完成。
- 本轮工具搜索没有暴露可用 in-app browser 导航 / 截图工具；Node 环境也没有 Playwright 包。
- 未联网安装依赖，未读取 `/Users/yoyi/.codex` 下插件或技能文件。
- HTTP smoke 只能证明普通浏览器静态壳可服务，不能证明 Tauri 数据桥或真实项目数据验收完成。

## 禁止文案搜索

在 `prototypes/productized-desktop-shell/src` 中搜索以下文案均无命中：

- `中间版本已完成`
- `完整记忆系统已完成`
- `worker 汇报已成为正式事实`
- `系统已记住`
- `真实 worker 已执行`

## 边界确认

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec`。
- 未执行 `codex exec resume`。
- 未创建新的 Codex session。
- 未读写 `/Users/yoyi/.codex`。
- 未迁移数据库。
- 未新增 `workflow-state.v0.json` 顶层数组。
- 未修改 workflow / work item / node / dispatch 既有状态枚举。
- 未把 worker report 写成正式事实。
- 未把 process fact observation 写成正式记忆。
- 未自动生成 MemoryCandidate。
- 未写正式记忆。
- 已完成全局主管最终结果复核链路。
- 已完成用户结果决定链路。
- 接受为阶段 C 的 C1-C6 受控闭环完成。
- 不接受为中间版本整体完成。
- 未完成真实窗口 / 截图验收。

## 后续

下一步建议进入阶段 D / M7-M13：补齐正式记忆生命周期、关系治理、维护任务、成熟模式、跨项目记忆和最终验收。阶段 G 仍需单独补真实 Tauri 全面验收、运行日志、自动重试和运维诊断。
