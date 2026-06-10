# 任务包 Markdown 预览 v1 总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-29-desktop-shell-task-markdown-preview-v1.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-29-desktop-shell-task-markdown-preview-v1.md`
- Handoff：`product-line/handoffs/2026-05-29-desktop-shell-task-markdown-preview-v1-result.md`
- 被回收产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“任务包 Markdown 预览 v1 已实现”。

不接受为“真实任务包文件生成完成”，不接受为“真实 Codex 会话派发完成”，不接受为“自动编排执行完成”，不接受为“真实 Tauri 窗口预览链路已验证”。

依据：

- 后端新增 `render_task_package_preview`，按索引内项目和现有 `work_item_id` 渲染预览。
- 后端拒绝非索引项目、缺状态文件、缺 workflow、缺 work item、缺 `task_package` artifact。
- 渲染函数不写状态文件，不追加 audit event。
- 前端项目详情页新增任务包 Markdown 预览区，并标注“预览，不是已派发任务包”。
- 复制预览文本复用确认弹层，并说明不写真实任务文件、不派发真实 Codex 会话。
- 总指导线复跑验证通过。

## 先说薄弱点

- 本轮按用户要求没有做真实 Tauri 窗口验证，所以不能说真实窗口里的布局和点击链路已验证。
- 前端离线测试只覆盖入口文案和复制确认弹层，不是完整浏览器布局测试。
- 当前任务草稿字段太少，预览里很多栏目只能显示“待补充”或“未登记”。
- 新增 `copy_task_package_preview` 会写剪贴板；这不是写任务文件，也不是写状态文件，但不能笼统说全流程完全只读。
- 预览模板目前是最小模板，还不能直接当高质量任务包派发。

## 接受内容

接受后端能力：

- `render_task_package_preview` 命令。
- `copy_task_package_preview` 命令。
- 非索引项目拒绝。
- 缺状态文件拒绝。
- 缺 workflow 拒绝。
- 缺 work item 拒绝。
- 缺 `task_package` artifact 拒绝。
- 有任务草稿时生成 Markdown 预览。
- 缺字段时输出“待补充”或“未登记”，不补编业务。

接受前端能力：

- 项目 workflow 区显示任务包 Markdown 预览入口。
- 有任务草稿时可以点“预览 Markdown”。
- 无预览时提示选择任务包草稿。
- 预览区显示 warnings 和 Markdown 文本。
- 复制预览文本前走确认弹层。
- 取消确认不会执行复制。

## 预览字段

当前 Markdown 预览包含：

- 任务名
- 所属开发线
- 背景
- 目标
- 允许读取
- 允许写入
- 禁止事项
- 验收标准
- 必须回传
- 总指导回收重点

缺字段处理：

- 任务名缺失时显示“待补充”。
- 所属开发线缺失时显示“未登记”。
- 目标说明缺失时显示“待补充”。
- 其他未登记栏目用占位说明，不写成确定事实。

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

- 22 个 Rust 单测通过。

任务文件复核：

```bash
ls -1 /Users/yoyi/workspace/product-line/tasks
```

结果：

- 只看到原有任务包文件和本任务包。
- 没有生成额外真实任务包文件。

## 安全和范围判断

接受当前安全边界。

依据：

- 没有生成真实 `product-line/tasks/*.md` 任务包文件。
- 没有写真实工作台状态文件。
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

- 桌面壳能把已有任务包草稿渲染成 Markdown 预览。
- 预览能显示标准任务包主要栏目。
- 缺字段不会被补编成事实。
- 复制预览文本需要用户确认。

仍不能说：

- 真实任务包 markdown 文件生成完成。
- 真实 Tauri 窗口预览链路已验证。
- 真实 Codex 会话派发完成。
- 自动编排执行完成。

## 下一步建议

下一步建议不要立刻生成真实任务包文件。

理由：

- 当前草稿字段太少，直接生成文件会得到很多“待补充”栏目。
- 先补“任务包字段编辑 v1”，让用户能在工作台里补齐背景、目标、允许读取、允许写入、禁止事项、验收标准、必须回传和回收重点。

建议下一条任务：

- `任务包字段编辑 v1`
- 目标是把当前标题、目标说明扩展成标准任务包字段编辑表单。
- 仍不派发 Codex，不运行 harness，不做真实窗口验证。
