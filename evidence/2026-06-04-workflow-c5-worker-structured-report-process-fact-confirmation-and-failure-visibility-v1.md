# Evidence: Workflow C5 Worker Structured Report, Process Fact Confirmation And Failure Visibility v1

日期：2026-06-04

## 结论

C5 已完成：工作台新增 worker 结构化汇报记录、项目主管过程事实确认、低风险本项目 `process_fact` observation 写入，以及项目工作流侧栏的 readback / permission / failure 最小可见化。

接受为：

- worker 可以通过确认动作提交结构化汇报或 handoff 记录。
- worker report 只写入现有 `workflow-state.v0.json` 的 `audit_events[]`，事件类型为 `worker_structured_report_recorded`。
- worker report 会进入现有 `SubagentReport` 读模型，但仍不是正式事实、不是正式记忆。
- 项目主管可以对 worker report 做 `confirm_process_fact` / `request_rework` / `block_and_escalate` 决定。
- 只有 `project_director` 可以确认过程事实；秘书、worker、system 都不能确认。
- 低风险、本项目、非 sensitive / secret 的 confirmed process fact 会写入 `observations.v1.json`，`observation_type = "process_fact"`，状态为 recorded。
- `request_rework` 和 `block_and_escalate` 只写 `reviews[]` / `audit_events[]`，不写 observation。
- 高风险、secret / sensitive、cross-project process fact 被拒绝，不能由项目主管单独确认。
- 同一 report / process fact 重复确认会被拒绝。
- 项目工作流侧栏新增“C5 worker 汇报 / 过程事实”卡片，显示 report count、pending fact、confirmed fact、readback、permission、failure 和最多 3 条 issue / risk。
- 确认弹层明确“确认后只记录过程事实 observation；不写正式记忆，不完成最终验收”。

不接受为：

- 真实 worker 已执行。
- 真实 Codex 已执行。
- `codex exec` 或 `codex exec resume` 已执行。
- worker 汇报已成为正式事实。
- observation 或 candidate 已成为正式记忆。
- MemoryCandidate 已自动生成。
- 正式记忆已写入。
- 全局主管最终结果复核完成。
- 用户已接受结果。
- 自动化工作流产品化闭环完成。
- 完整自动重试系统完成。
- 真实 Tauri 窗口 / 截图验收完成。

## 关键实现

- 后端类型 / 命令：
  - `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
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

- 新增 `WorkerStructuredReportInput`、`ProcessFactCandidate`、`ProjectDirectorProcessFactDecisionInput`、`ProjectDirectorProcessFactDecisionResult`。
- 新增 `record_worker_structured_report` 和 `record_project_director_process_fact_decision` Tauri commands。
- `ReviewResult` 扩展 `reviewer_role`、`report_id`、`accepted_fact_ids`、`observation_ids`，用于 C5 读模型显示。
- `control_core::validate_observation_type` 增加 `process_fact`，只允许作为 observation 类型，不改变正式记忆语义。
- C5 不新增 `workflow-state.v0.json` 顶层数组，不改 workflow / work item / node / dispatch 状态枚举。

前端实现要点：

- `ProjectsView` 在工作项编排卡中新增 C5 摘要区。
- C5 摘要区显示 readback 成功 / 真实 0 条结果 / 读取失败 / rollout 不可访问 / 解析失败，permission pending / approved / rejected / requires_user_confirmation，以及 failed / timed_out / cancelled / direction risk。
- `PermissionDialog` 新增 worker report 和 process fact decision 摘要，明确 observation 不是正式记忆、不是最终验收。

## 验收结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - 输出：`offline interaction tests passed: 9`
- `npm run build`
  - Vite build 通过；保留 chunk size warning。
- `cargo test --lib worker_structured_report`
  - 2 passed。
- `cargo test --lib process_fact`
  - 3 passed。
- `cargo test --lib dispatch_readback_stats`
  - 6 passed。
- `cargo test --lib workflow_exception`
  - 1 passed。
- `cargo test --lib observation`
  - 13 passed。
- `cargo test --lib workflow_authorization`
  - 1 passed。
- `cargo test --lib plan_authorization`
  - 8 passed。
- `cargo test --lib`
  - 197 passed, 1 ignored。
- `rustfmt --check src/control_core.rs src/commands.rs src/types.rs src/observation_store.rs src/codex_transcript.rs src/codex_db.rs src/lib.rs`

说明：

- `cargo test --lib` 保留既有 warning：`JsonRpcError::invalid_params` 未使用。
- `npm run build` 保留 Vite chunk size warning，不影响构建通过。
- 任务包建议中的 `process_fact_confirmation` / `workflow_failure_visibility` / `workflow_readback_visibility` 按实际测试名分别调整为 `process_fact`、`workflow_exception`、`dispatch_readback_stats`。

## UI / Smoke

已完成：

- 普通 Vite dev server 启动：`http://127.0.0.1:4173/`
- HTTP smoke：
  - `curl -sS -I http://127.0.0.1:4173/` 返回 `HTTP/1.1 200 OK`。
  - `curl -sS http://127.0.0.1:4173/` 返回 Vite HTML 壳和 `/src/main.tsx` 入口。

未完成：

- 真实 Tauri 窗口 / 截图验收未完成。
- 本轮工具搜索没有暴露可用 in-app browser 导航 / 截图工具；本地也没有 Playwright / Puppeteer 依赖。未联网安装依赖，也未读取 `/Users/yoyi/.codex` 下插件或技能文件。
- HTTP smoke 只能证明普通浏览器静态壳可服务，不能证明 Tauri 数据桥或真实项目数据验收完成。

## 禁止文案搜索

在 `prototypes/productized-desktop-shell/src` 中搜索以下文案均无命中：

- `worker 汇报已成为正式事实`
- `自动化工作流已完成`
- `最终结果已通过`
- `系统已记住`

`rg -F 'codex exec' evidence handoffs tasks docs CURRENT.md STAGE_PLAN.md` 有大量历史记录和边界文案命中；本轮未执行 `codex exec` 或 `codex exec resume`。

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
- 已允许项目主管确认后的 `process_fact` 写入 ObservationStore。
- 未自动生成 MemoryCandidate。
- 未写正式记忆。
- 未完成最终结果复核。
- 未完成真实窗口 / 截图验收。

## 后续

下一步建议单独拆 C6：全局主管最终结果复核、用户结果查看和阶段 C 验收。C6 仍必须继续遵守 C1-C5 授权、过程事实、observation、candidate / formal memory 和真实 Codex 执行边界。
