# 项目默认工作流草稿初始化 v1 交接

## 回收对象

- 任务包：`product-line/tasks/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`

## 结论

建议接受为“项目默认工作流草稿初始化 v1 已实现”。

不建议接受为“自动编排执行完成”。依据：本轮没有派发真实 Codex 会话，没有生成任务包文件，没有执行 harness，没有做节点拖拽、边编辑或状态转换。

## 薄弱点

- 默认 workflow、node、edge 都是草稿事实，不是执行事实。
- 状态文件不存在时，本轮选择在用户确认创建项目 workflow 时同时初始化最小 v0 状态；这仍是确认式写入，不是自动创建。
- `stable_id` 当前是简单路径归一化，不是最终 hash。
- 没有做并发写入锁。

## 做了什么

- 新增后端 `bootstrap_project_workflow` 命令。
- 命令从当前索引确认项目存在，只允许索引内项目。
- 写入工作台自己的 `workflow-state.v0.json`。
- 状态文件不存在时，在同一次用户确认写入里创建最小 v0 状态并追加项目 workflow 草稿。
- 已有状态文件写入前会备份。
- 同一项目已有默认 workflow 时不重复创建。
- 前端项目详情页新增项目工作流草稿区。
- 前端显示当前项目是否已有本地 workflow、workflow id、state、node 数、edge 数。
- 前端创建按钮走确认弹层。

## 改了哪些文件

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-result.md`

## 新增或修改了哪些测试

Rust：

- `bootstrap_project_workflow_initializes_missing_state`
- `bootstrap_project_workflow_does_not_duplicate_existing_workflow`
- `bootstrap_project_workflow_rejects_non_index_project`
- `bootstrap_project_workflow_backs_up_existing_state`

前端：

- 项目页可以打开创建默认工作流确认弹层。
- 取消不会调用创建动作。
- 确认动作显示目标路径和写入边界。

## 默认 workflow 写入了哪些对象

`projects[]`：

- 选中项目的本地项目事实记录。

`workflows[]`：

- 一个默认 workflow。
- `state=draft`
- `workflow_version=1`
- `model_policy=none`

`nodes[]`：

- 总指导 / Director
- Codex 开发线
- 验证线
- 任务包
- Handoff
- Evidence
- Review

`edges[]`：

- 总指导派发任务包。
- 任务包交给 Codex 开发线。
- 开发线产出 handoff。
- 开发线产出 evidence。
- 验证线验证 evidence。
- Review 回收 handoff。

`audit_events[]`：

- `project_default_workflow_bootstrapped`
- 记录用户确认创建项目默认工作流草稿。

## 如何避免重复创建

- workflow id 使用项目根路径生成：`workflow:<stable-project-root-id>:default`。
- 写入前检查 `workflows[]` 是否已有同 id。
- 已存在时返回 no-op，不追加 workflow / node / edge / audit。

## 如何保证只对索引内项目生效

- 前端传入的是当前选中项目路径。
- 后端重新读取 `codex-index.json`。
- 后端只在 `projects[].project_root` 中匹配项目。
- 非索引路径直接拒绝。
- 输出路径不来自前端，固定为应用数据目录的 `workflow-state.v0.json`。

## 验证命令和结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`

Rust 当前 10 个测试通过。

真实状态文件：

- 任务前不存在。
- 任务后不存在。
- 本轮没有启动真实 Tauri 窗口点击创建按钮。

端口：

- `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无监听输出。

## 是否触碰禁止事项

未触碰。

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未写项目业务目录。
- 未读取或展示 auth、env、密钥、令牌、授权文件内容。
- 未读取或展示会话正文、工具输出、命令输出、输入历史、记忆正文。
- 未自动运行 harness。
- 未把索引候选自动升级成已验证能力。
- 未接入非 Codex agent。
- 未做知识库、向量搜索、LM 调度。
- 未做 release 打包。
- 未拉取外网依赖。
- 未实现自动编排执行。

## 风险和下一步建议

- 下一步如果做任务包生成，需要新增 work_items/artifacts 写入，并继续用户确认。
- 下一步如果做节点状态流转，需要追加 audit event，不能直接改节点状态。
- 需要补更严格 schema / 引用校验。
- 需要补文件锁或写入串行化，避免多窗口并发写状态文件。
