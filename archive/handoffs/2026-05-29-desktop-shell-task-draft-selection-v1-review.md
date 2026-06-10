# 任务草稿选择态 v1 总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-29-desktop-shell-task-draft-selection-v1.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-29-desktop-shell-task-draft-selection-v1.md`
- Handoff：`product-line/handoffs/2026-05-29-desktop-shell-task-draft-selection-v1-result.md`
- 被回收产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“任务草稿选择态 v1 已实现”。

不接受为“真实任务包文件生成完成”，不接受为“真实 Codex 会话派发完成”，不接受为“自动编排执行完成”，不接受为“真实 Tauri 窗口选择态链路已验证”。

依据：

- 前端新增 `selectedWorkItemId` 选择态。
- 多任务草稿时，任务列表显示“当前选中 / 选择”。
- Markdown 预览使用当前选中的 `work_item_id`。
- 复制预览使用当前选中的预览对象。
- 字段编辑表单绑定当前选中的任务草稿。
- 保存字段使用当前选中的 `work_item_id`。
- 没有新增后端命令。
- 没有新增持久化状态字段。
- 总指导线复跑验证通过。

## 先说薄弱点

- 本轮按任务包边界没有做真实 Tauri 窗口验证，所以不能说真实窗口里的选择、预览、编辑和保存链路已验证。
- 当前离线测试不是完整 React renderer，hook 选择态通过纯函数和静态组件检查间接覆盖。
- 选择态没有持久化，刷新页面后仍会回到默认选择；这是本轮有意不做，不作为退回项。
- 仍没有生成真实 `product-line/tasks/*.md`。
- 仍没有派发真实 Codex 会话、运行 harness 或登记真实 handoff / evidence / review。

## 接受内容

接受前端能力：

- 有草稿且无当前选择时默认选择第一个草稿。
- 当前选择仍存在时保留。
- 当前选择消失时回到第一个可用草稿。
- 没有草稿时清空选择并显示下一步提示。
- 任务列表展示选中状态。
- 预览、复制、字段编辑和保存字段共用同一个选中 `work_item_id`。

接受测试覆盖：

- 两个任务草稿夹具。
- 缺失旧选择时回到第一个草稿。
- 切换到第二个草稿后保留第二个。
- 选择解析返回第二个草稿。
- 保存字段和复制预览使用第二个 `work_item_id`，不是第一个。

## 总指导线复跑验证

在 `product-line/prototypes/productized-desktop-shell/` 复跑：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，输出 `offline interaction tests passed: 3`。
- `npm run build` 通过。

在 `product-line/prototypes/productized-desktop-shell/src-tauri/` 复跑：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home \
CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target \
cargo test --offline
```

结果：

- 29 个 Rust 单测通过。

任务文件复核：

```bash
find /Users/yoyi/workspace/product-line/tasks -maxdepth 1 -name '*task-draft-selection-v1*' -type f
```

结果：

- 只返回原始任务包 `product-line/tasks/2026-05-29-desktop-shell-task-draft-selection-v1.md`。

## 安全和范围判断

接受当前安全边界。

依据：

- 没有生成真实 `product-line/tasks/*.md` 任务包文件。
- 没有新增后端命令。
- 没有新增持久化状态字段。
- 没有启动 Codex CLI。
- 没有派发真实 Codex 会话。
- 没有运行 harness。
- 没有写 `/Users/yoyi/.codex`。
- 没有改 Codex 状态库。
- 没有写项目业务目录。
- 没有读取或展示 auth、env、密钥、令牌、授权文件内容。
- 没有读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 没有接入非 Codex agent。
- 没有知识库、向量搜索、模型调度、release 打包或网络依赖拉取。

## 当前状态

这条任务从“待派发”改为“已回收”。

当前可以说：

- 任务列表、Markdown 预览、复制预览和字段编辑表单已经共享同一个选中任务草稿。
- 多任务草稿时不再默认把字段编辑固定绑到第一个草稿。
- 后续真实任务文件生成可以依赖选中的 `work_item_id`。

仍不能说：

- 真实任务包 markdown 文件生成完成。
- 真实 Tauri 窗口选择态链路已验证。
- 真实 Codex 会话派发完成。
- 自动编排执行完成。

## 下一步建议

下一步建议做“真实任务包文件生成 v1”。

理由：

- 任务草稿已能登记。
- 任务草稿已能预览 Markdown。
- 标准任务包字段已能编辑。
- 选择态已经统一，能避免多草稿时选错目标。

建议下一条任务：

- `真实任务包文件生成 v1`
- 目标是在用户确认后，从选中任务草稿的结构化字段生成真实 `product-line/tasks/*.md` 文件。
- 仍不派发 Codex，不运行 harness，不做真实窗口验证。
