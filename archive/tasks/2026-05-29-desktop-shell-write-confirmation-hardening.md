# 任务包：桌面壳写入确认防误触加固

## 任务名

加固产品化桌面壳的写入型本机动作确认弹层，降低误点 `确认执行` 的风险。

## 所属开发线

桌面应用线。

当前派给桌面应用线，因为任务涉及前端确认交互和可能的 Tauri 命令调用边界，不是验证线任务。

## 背景

工作流事实层 v0 真实窗口验证不通过。

依据：

- `product-line/handoffs/2026-05-29-desktop-shell-workflow-state-v0-validation-review.md`
- `product-line/evidence/2026-05-28-desktop-shell-workflow-state-v0-validation.md`
- `product-line/handoffs/2026-05-28-desktop-shell-workflow-state-v0-validation-result.md`

失败点不是 v0 面板不可见，也不是确认弹层不可见，而是写入型动作的 `确认执行` 被误点，真实 `workflow-state.v0.json` 曾被创建。

当前 `PermissionDialog.tsx` 里 `确认执行` 是普通按钮，和 `取消` 并排，没有写入型动作的额外确认条件。这个设计对打开目录、复制路径这类低风险动作可以接受，但对初始化状态文件这种真实写入动作不够稳。

## 目标

只做写入确认防误触加固。

必须做到：

- 区分普通本机动作和写入型动作。
- `initialize-workflow-state` 视为写入型动作。
- 写入型动作不能只靠单击 `确认执行` 触发。
- 写入型动作确认弹层必须让用户明确看到：
  - 目标路径
  - 路径来源
  - 写入边界
  - 不写 `.codex`
  - 不写 Codex 状态库
  - 不写项目业务目录
  - 会追加 audit event
  - 会使用临时文件和原子替换
- 写入型动作必须增加至少一种防误触机制，例如：
  - 输入固定确认文本后按钮才可用；或
  - 勾选“我确认写入应用数据目录”后按钮才可用，并让确认按钮默认不可聚焦；或
  - 两步确认，第一步展开风险，第二步才允许执行。
- 优先选择实现简单、可测试、不会污染 UI 的方案。
- 保留普通动作的现有确认体验，不把所有低风险动作都变重。
- 补充离线交互测试，覆盖写入型动作默认不能确认、满足确认条件后才能确认、取消不会触发确认。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/tasks/README.md`
- `product-line/handoffs/2026-05-29-desktop-shell-workflow-state-v0-validation-review.md`
- `product-line/evidence/2026-05-28-desktop-shell-workflow-state-v0-validation.md`
- `product-line/handoffs/2026-05-28-desktop-shell-workflow-state-v0-validation-result.md`
- `product-line/prototypes/productized-desktop-shell/`

允许做存在性检查，不读取内容：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

## 允许写入

- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/prototypes/productized-desktop-shell/package.json`
- `product-line/prototypes/productized-desktop-shell/tsconfig.json`
- `product-line/evidence/`
- `product-line/handoffs/`

如确实需要修改 Rust 类型或命令边界，允许写入：

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/`

## 禁止事项

- 不创建真实 `workflow-state.v0.json`。
- 不点击或调用真实初始化写入。
- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不写项目业务目录。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不把索引候选自动升级成本地事实。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、LM 调度。
- 不做 release 打包。
- 不拉取外网依赖。
- 不扩大到节点编辑、边编辑、review 登记或工作项状态转换。

## 验收标准

- 写入型动作不能通过一次普通点击直接执行。
- `initialize-workflow-state` 弹层在默认状态下确认按钮不可执行，或需要明确的第二步确认。
- 弹层仍展示目标路径、路径来源和写入边界。
- 普通路径动作仍能打开确认弹层，不被写入型确认流程拖重。
- 离线交互测试覆盖写入型确认防误触。
- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- 如改 Rust，`cargo test --offline` 通过。
- 真实状态文件在任务前后都不存在。
- 5173 无监听残留。

## 必须回传

1. 薄弱点
2. 做了什么
3. 改了哪些文件
4. 新增或修改了哪些测试
5. 写入型动作现在如何防误触
6. 普通本机动作是否受影响
7. 验证命令和结果
8. 真实状态文件任务前后是否存在
9. 是否触碰禁止事项
10. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否只做了写入确认防误触，没有扩大范围。
- 是否没有创建真实状态文件。
- 是否没有写 `.codex`、Codex 状态库或项目业务目录。
- 是否离线测试真的覆盖写入型动作默认不可执行。
- 是否适合重新派发真实 Tauri 窗口验证任务。
