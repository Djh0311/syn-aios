# 真实任务包文件生成 v1 总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-29-desktop-shell-real-task-file-generation-v1.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-29-desktop-shell-real-task-file-generation-v1.md`
- Handoff：`product-line/handoffs/2026-05-29-desktop-shell-real-task-file-generation-v1-result.md`
- 被回收产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“真实任务包文件生成能力 v1 已实现”。

不接受为“真实 `product-line/tasks/*.md` 已由本轮生成”，不接受为“真实 Codex 会话派发完成”，不接受为“自动编排执行完成”，不接受为“真实 Tauri 窗口生成链路已验证”。

依据：

- 后端新增 `generate_task_package_file`。
- 后端真实输出目录固定为 `/Users/yoyi/workspace/product-line/tasks`，前端不能传任意目录。
- 后端生成前校验索引项目、状态文件、workflow、work item 和 `task_package` artifact。
- 后端复用结构化字段渲染逻辑。
- 后端支持不冲突文件名，不静默覆盖已有文件。
- 后端生成后更新 `artifacts[].path`、`artifacts[].updated_at`、`artifacts[].warnings`。
- 后端追加 `task_package_file_generated` audit event。
- 前端新增“生成任务包文件”入口和确认弹层。
- 总指导线复跑验证通过。
- 本轮明确没有在真实 `/Users/yoyi/workspace/product-line/tasks/` 目录下生成任务包文件。

## 先说薄弱点

- 真实 `product-line/tasks/*.md` 文件没有在本轮生成，只在 Rust 临时目录夹具里验证了写入能力。
- 没有真实 Tauri 窗口验证。
- 当前文件名前缀固定为 `2026-05-29`，适合本任务日期，但长期产品化应改成真实日期来源。
- 生成文件后仍不派发 Codex、不运行 harness，也没有进入回收链路。
- 任务队列 `tasks/README.md` 没有被生成命令更新；这是任务包边界要求，本轮只生成任务包文件能力。

## 接受内容

接受后端能力：

- `generate_task_package_file` 命令。
- 非索引项目拒绝。
- 缺状态文件拒绝。
- 缺 workflow 拒绝。
- 缺 work item 拒绝。
- 缺 `task_package` artifact 拒绝。
- 临时 tasks 目录生成 Markdown 文件。
- 冲突时生成后缀文件名，不覆盖已有文件。
- 更新 artifact path。
- 追加 audit event。
- 缺字段保留“待补充”或“未登记”，不补编业务。

接受前端能力：

- 选中草稿后显示“生成任务包文件”入口。
- 生成前确认弹层显示写入目录。
- 确认弹层说明不派发 Codex、不启动 Codex CLI、不运行 harness、不写 `.codex` 或 Codex 状态库。
- 取消确认不会调用生成动作。
- 已有 `artifact_path` 时显示已生成状态并禁用按钮。

## 文件名和冲突策略

文件名格式：

- `2026-05-29-generated-<slug>.md`

slug 规则：

- 只保留小写 ASCII 字母、数字和连字符。
- 空 slug 回退为 `task-package-<work-item-short-id>`。
- 过长 slug 会截断。

冲突策略：

- 不覆盖已有文件。
- 冲突时选择后缀，例如 `-2.md`。

## 状态更新

更新 `artifacts[]`：

- `path`
- `updated_at`
- `warnings`

warning 策略：

- 移除 `draft_only_no_markdown_file`。
- 保留 missing-field warnings。
- 不把缺失字段伪装成已填写。

追加 `audit_events[]`：

- `event_type=task_package_file_generated`
- `target_ref=<selected work_item_id>`
- `actor_ref=user_confirmed_desktop_shell`
- `permission_level=user_confirmed_write`
- `before_state=draft`
- `after_state=draft`

`work_items[].state`：

- 保持 `draft`。

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

- 37 个 Rust 单测通过。

真实任务文件复核：

```bash
find /Users/yoyi/workspace/product-line/tasks -maxdepth 1 -type f -name '2026-05-29-generated-*.md'
```

结果：

- 无输出。
- 判断为本轮没有在真实 tasks 目录生成任务包文件。

## 安全和范围判断

接受当前安全边界。

依据：

- 没有派发真实 Codex 会话。
- 没有启动 Codex CLI。
- 没有运行 harness。
- 没有写 `/Users/yoyi/.codex`。
- 没有改 Codex 状态库。
- 没有写项目业务目录。
- 没有写 `product-line/tasks/README.md`。
- 没有覆盖已有任务包文件。
- 没有读取或展示 auth、env、密钥、令牌、授权文件内容。
- 没有读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 没有接入非 Codex agent。
- 没有知识库、向量搜索、模型调度、release 打包或网络依赖拉取。

## 当前状态

这条任务从“待派发”改为“已回收”。

当前可以说：

- 桌面壳已经具备用户确认后生成真实任务包文件的后端和前端能力。
- 生成能力已通过临时目录写文件测试验证。
- 生成后能更新 artifact path 和 audit event。

仍不能说：

- 本轮已经在真实 `product-line/tasks/` 生成了任务包文件。
- 真实 Tauri 窗口生成链路已验证。
- 真实 Codex 会话派发完成。
- 自动编排执行完成。

## 下一步建议

下一步建议做“真实任务包文件生成确认 v1”。

理由：

- 当前只是能力实现，没有真实落盘。
- 下一步需要在用户确认下生成一个真实任务包文件，作为后续 Codex 派发入口。

建议下一条任务：

- `真实任务包文件生成确认 v1`
- 目标是在真实工作台状态和真实 `product-line/tasks/` 下生成一个任务包文件，并回填 artifact path。
- 仍不派发 Codex，不运行 harness。
