# 派发结果读回 UI 与总指导回收记录 v1 evidence

## 范围

- 任务包：`/Users/yoyi/workspace/product-line/tasks/2026-05-30-dispatch-result-readback-ui-and-director-review-v1.md`
- 开发线：桌面应用线 / 总指导线
- 本轮只实现工作台读回展示和总指导 review 记录入口。

## 薄弱点

- 真实 workflow state 当前是否应写入“接受 / 需要修改 / 暂停 / 废弃”需要用户明确结论；本轮用户没有给出具体结论，所以没有写真实 review。
- 后端 review 写入只记录 `reviews[]` 和 audit event，不自动把 work item 状态推进到 `accepted`、`needs_changes`、`paused` 或 `discarded`。依据：任务包验收要求是“总指导 review 记录引用 dispatch id 和 work item id”以及“写入 audit event”，没有要求同步改 work item state。
- UI 只能证明“入口和确认边界存在”；真实点击确认后的写入由后端单元测试覆盖，未在真实 state 上执行。

## 做了什么

- 工作台 UI 增加“总指导回收”区域。
- UI 显示 completed dispatch 的最终回复摘要、dispatch id、transcript events、hits、warnings。
- UI 在 work item 为 `ready_for_review` 时提供四个总指导结论按钮：接受、需要修改、暂停、废弃。
- 总指导结论按钮只创建待确认动作，确认弹层明确边界：只写 `reviews[]` 和 audit event；不启动 Codex、不 resume、不发送消息、不写 `/Users/yoyi/.codex`、不读取 transcript。
- 修正派发结果查找逻辑：按 workflow id + work item id 查找，不再依赖当前节点 id。依据：进入 `ready_for_review` 后当前节点可能是 review node，而派发记录属于执行节点。
- 后端增加总指导 review 写入命令和测试覆盖。

## 真实状态写入

- 是否写真实 workflow state：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否执行 `codex exec resume`：否。
- 是否发送 safe probe：否。
- 是否读取完整 transcript：否。
- 是否读取 auth、密钥、token、`.env`：否。
- 是否触碰真实业务会话：否。

## 已实现的 workflow state 字段类型

未在本轮写真实状态。代码路径在用户确认 UI 动作后会写：

- `reviews[]`：`review_id`、`project_id`、`workflow_id`、`work_item_id`、`dispatch_id`、`reviewer_role`、`decision`、`summary`、`evidence_refs`、`handoff_refs`、`created_at`、`updated_at`、`warnings`
- `audit_events[]`：`event_id`、`event_type=workflow_dispatch_director_review_recorded`、`target_ref`、`actor_ref`、`source_kind`、`permission_level`、`before_state`、`after_state`、`created_at`、`reason`
- 顶层 `updated_at`
- 真实执行时会先生成 workflow state backup；本轮没有真实执行，所以没有真实备份路径。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 3`。
- `npm run build`：通过，Vite build 成功。
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`：通过，58 passed，1 ignored。

## 新增测试覆盖

- 前端离线测试覆盖：
  - `ready_for_review` 下显示 completed dispatch 摘要。
  - 显示 events、hits、warnings。
  - 四个总指导结论按钮生成 `record-director-review` 待确认动作。
  - 确认弹层显示 work item、dispatch、decision 和安全边界。
- 后端单元测试覆盖：
  - completed dispatch 可以写入 director review。
  - 写入 `reviews[]` 和 `workflow_dispatch_director_review_recorded` audit event。
  - 非 `ready_for_review` 拒绝。
  - 非 completed dispatch 拒绝。
  - 未知 decision 拒绝。

