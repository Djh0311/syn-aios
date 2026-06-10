# 任务包字段编辑 v1 总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-29-desktop-shell-task-fields-edit-v1.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-29-desktop-shell-task-fields-edit-v1.md`
- Handoff：`product-line/handoffs/2026-05-29-desktop-shell-task-fields-edit-v1-result.md`
- 被回收产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“任务包字段编辑 v1 已实现”。

不接受为“真实任务包文件生成完成”，不接受为“真实 Codex 会话派发完成”，不接受为“自动编排执行完成”，不接受为“真实 Tauri 窗口字段编辑链路已验证”。

依据：

- 后端新增 `update_task_package_draft_fields`，按索引内项目和现有 `work_item_id` 更新结构化字段。
- 后端拒绝非索引项目、缺状态文件、缺 workflow、缺 work item、缺 `task_package` artifact。
- 写入前会备份旧状态文件。
- 写入使用已有原子 JSON 写入能力。
- 写入追加 `task_package_fields_updated` audit event。
- `artifacts[].path` 继续保持 `null`，表示没有生成真实任务文件。
- Markdown 预览改为优先读取结构化字段。
- 前端新增“编辑字段”表单，保存前走确认弹层。
- 总指导线复跑验证通过。

## 先说薄弱点

- 本轮按任务包边界没有做真实 Tauri 窗口验证，所以不能说真实窗口里的表单、保存和预览刷新链路已验证。
- 前端离线测试不是完整浏览器布局测试，也不是原生 Tauri 保存测试。
- 当前多任务草稿时，字段编辑表单绑定第一个草稿；后续需要把“选择草稿”和“编辑字段”合并成明确选择态。
- 当前只是编辑工作台状态里的结构化字段，还没有生成真实 `product-line/tasks/*.md`。
- 当前没有派发真实 Codex 会话，没有运行 harness，也没有登记真实 handoff / evidence / review。

## 接受内容

接受后端能力：

- `update_task_package_draft_fields` 命令。
- 非索引项目拒绝。
- 缺状态文件拒绝。
- 缺 workflow 拒绝。
- 缺 work item 拒绝。
- 缺 `task_package` artifact 拒绝。
- 有任务草稿时更新结构化字段。
- 写入前备份。
- 原子写入。
- 追加 audit event。
- 空字段保存为空值，并写 missing warning。

接受前端能力：

- 任务包预览区新增“编辑字段”表单。
- 表单覆盖标准任务包主要栏目。
- 列表字段用多行文本表达。
- 保存前走确认弹层。
- 确认文案说明写入工作台状态文件，不生成真实任务文件，不派发真实 Codex 会话。

## 结构化字段

存储在 `artifacts[]` 对应 `artifact_type=task_package` 的记录里：

- `task_name`
- `assigned_line`
- `background`
- `goals`
- `allowed_read`
- `allowed_write`
- `forbidden_actions`
- `acceptance_criteria`
- `required_return`
- `review_focus`
- `template_version=task_package_v1`
- `brief`
- `title`
- `warnings`
- `updated_at`
- `path=null`

同步到 `work_items[]`：

- `title`
- `assigned_role_id`
- `updated_at`

空字段处理：

- 空字符串保持空字符串。
- 空列表保持空数组。
- `artifacts[].warnings` 记录 `missing_*`。
- Markdown 预览显示“待补充”或“未登记”。
- 不补编业务内容。

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
find /Users/yoyi/workspace/product-line/tasks -maxdepth 1 -name '*task-fields-edit-v1*' -type f
```

结果：

- 只返回原始任务包 `product-line/tasks/2026-05-29-desktop-shell-task-fields-edit-v1.md`。

## 安全和范围判断

接受当前安全边界。

依据：

- 没有生成真实 `product-line/tasks/*.md` 任务包文件。
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

- 桌面壳能编辑任务包草稿的标准结构化字段。
- Markdown 预览能优先使用编辑后的结构化字段。
- 空字段不会被补编成事实。
- 字段保存需要用户确认，并只写工作台自己的状态文件。

仍不能说：

- 真实任务包 markdown 文件生成完成。
- 真实 Tauri 窗口字段编辑链路已验证。
- 真实 Codex 会话派发完成。
- 自动编排执行完成。

## 下一步建议

下一步建议先修“任务草稿选择态 v1”，再做真实任务包文件生成。

理由：

- 多任务草稿时字段编辑表单现在绑定第一个草稿。
- 如果直接进入真实任务包文件生成，用户可能误生成非目标草稿的任务文件。

建议下一条任务：

- `任务草稿选择态 v1`
- 目标是让任务列表、Markdown 预览、字段编辑表单共享同一个明确选中的 task draft。
- 仍不派发 Codex，不运行 harness，不做真实窗口验证。
