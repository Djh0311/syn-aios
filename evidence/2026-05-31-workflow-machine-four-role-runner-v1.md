# 四角色工作流机器 Evidence

## 薄弱点

- 本轮实现的是工作流机器代码路径和 UI 启动入口，还没有真实执行 mario test 四会话闭环。
- 新增 runner 目前是串行执行，不是并发调度。
- 失败后会停止并落账，不会自动改 prompt 重试；多轮继续依赖总指导最后回复没有 `WORKFLOW_MACHINE_FINAL_ACCEPTED`。
- 真实运行仍会执行 `codex exec resume`，会写 `/Users/yoyi/.codex`，会写真实 workflow state，开发线会修改项目目录，需要用户单独确认。

## 做了什么

新增“工作流机器”最小闭环：

1. 一个确认入口启动 run。
2. 后端按固定顺序调用绑定会话：
   - 总指导
   - 开发线
   - 验证线
   - 回收线
   - 总指导结论
3. 总指导最后回复包含 `WORKFLOW_MACHINE_FINAL_ACCEPTED` 时，run 收口为 `accepted`。
4. 未接受且达到最大轮次时，run 收口为 `needs_changes`。
5. 某一步失败时，run 收口为 `failed`。
6. 每一步复用现有 `codex exec resume` 派发路径，继续写 `workflow_node_dispatches[]`、`workflow_execution_controls[]`、`execution_attempts[]`、`audit_events[]`。
7. 新增顶层 `workflow_machine_runs[]` 记录 run 摘要和 steps。

## 改了哪些文件

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/index.html`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/assets/index-BEmSbvji.js`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/assets/index-DDJziFyU.css`

## 新增能力

后端新增：

- `run_workflow_machine`
- `WorkflowMachineRunRequest`
- `WorkflowMachineRunResult`
- `workflow_machine_runs[]`
- 四角色顺序 runner
- stub 测试：`workflow_machine_runs_four_role_loop_to_acceptance`

前端新增：

- “工作流机器 / 总指导循环闭环”区
- “启动闭环”按钮
- 确认弹层展示：
  - 工作项
  - 目标
  - 最大轮次
  - 单步超时
  - 会执行 `codex exec resume`、写 `/Users/yoyi/.codex`、写真实 workflow state、允许开发线改项目目录

## 验证结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，4 个测试。
- `cargo fmt`：通过。
- 默认 `cargo test --offline`：仍因默认 Cargo 缓存只有 `serde_json 1.0.149` 而失败，和本轮代码无关。
- 指定既有 Cargo 缓存后 `cargo test --offline`：通过，68 passed，1 ignored。
- 新增定向测试 `workflow_machine_runs_four_role_loop_to_acceptance`：通过。
- `npm run build`：通过。
- `build_index.py --check codex-index.json`：`validation_ok`。

## 边界

- 是否执行真实 `codex exec` / `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：否。
- 是否修改 `/Users/yoyi/Documents/mario test`：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否运行 harness：否。

## 下一步

下一步才是全流程真实验收：

- 用 `workflow:users-yoyi-documents-mario-test:default:create-mario-demo-v1`
- 启动四角色工作流机器
- 目标：完成 `/Users/yoyi/Documents/mario test` 的马里奥 demo
- 真实运行会写 `/Users/yoyi/.codex`、真实 workflow state 和项目目录
