# 任务包内容修正与派发准备 v1 总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-29-desktop-shell-task-package-dispatch-readiness-v1.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-29-desktop-shell-task-package-dispatch-readiness-v1.md`
- Handoff：`product-line/handoffs/2026-05-29-desktop-shell-task-package-dispatch-readiness-v1-result.md`
- 被检查真实任务包：`product-line/tasks/2026-05-29-generated-task-draft-smoke.md`
- 被回收产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“任务包派发准备检查与拦截 v1 已实现”。

不接受为“当前已有可派发任务包”，不接受为“任务包内容已修正完成”，不接受为“新版 ready 任务包已生成”，不接受为“真实 Codex 会话派发完成”，不接受为“自动编排执行完成”。

依据：

- 后端新增只读命令 `inspect_task_package_dispatch_readiness`。
- 后端返回 `status`、`blocking_reasons`、`warnings`、`artifact_path`、`can_generate_next_version`。
- 当前真实任务包 `/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md` 被判定为 `not_ready`。
- 判定依据包括测试草稿标题、输入法污染、占位内容和历史冲突禁令。
- 前端新增派发准备面板，能显示状态、原因、warning 和 artifact path。
- `not_ready` 时“生成可派发版本”按钮禁用。
- 本轮没有生成新版真实任务包文件。
- 总指导线复跑验证通过。

## 先说薄弱点

- 这轮没有把当前任务变成可派发任务，只是防止它被误派发。
- readiness 规则是保守字符串规则，不是完整语义审核。
- 前端还没有独立的“修正派发字段”专用表单，只能依赖现有字段编辑能力和 readiness 面板配合。
- 没有用户提供真实业务内容，所以没有生成新版 ready 文件。
- 当前真实任务包仍然不适合派发 Codex。

## 接受内容

接受后端能力：

- 能检查索引项目、workflow、work item 和 `task_package` artifact。
- 能识别缺失 artifact path。
- 能识别测试草稿标题。
- 能识别目标、允许读写、验收标准中的空值、`待补充`、`未登记`。
- 能识别输入法污染。
- 能识别历史冲突禁令，例如“不生成真实任务包文件”。
- 能识别必须回传字段是否缺少关键项。
- 能在字段修正并生成文件后判定为 `ready`。
- 检查命令本身只读，不写真实 workflow state。

接受前端能力：

- 项目任务草稿区域新增派发准备面板。
- 面板能触发 readiness 检查。
- 面板能显示 `not_ready` / `ready` / `blocked`。
- 面板能显示 blocking reasons、warnings 和 artifact path。
- `not_ready` 前禁用“生成可派发版本”。
- 后续生成路径仍走确认边界，并说明不派发 Codex、不启动 Codex CLI、不运行 harness。

## 当前真实任务包状态

文件：

```text
/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md
```

状态：

```text
not_ready
```

依据：

- 标题仍像测试草稿。
- 内容存在输入法污染。
- 仍有 `待补充` / `未登记`。
- 禁止事项里仍有“不生成真实任务包文件”这类历史冲突禁令。

当前真实 generated 文件复核：

- `find product-line/tasks -maxdepth 1 -type f -name '2026-05-29-generated-*.md'` 只返回 `product-line/tasks/2026-05-29-generated-task-draft-smoke.md`。
- 该文件大小仍为 1610 bytes，修改时间仍是 `May 29 16:19:39 2026`。
- 本轮没有生成新版真实任务包文件。

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

- 41 个 Rust 单测通过。
- 1 个真实确认测试保持 ignored。

## 安全和范围判断

接受当前安全边界。

依据：

- 没有生成新版真实任务包文件。
- 没有派发真实 Codex 会话。
- 没有启动 Codex CLI。
- 没有运行 harness。
- 没有写 `/Users/yoyi/.codex`。
- 没有改 Codex 状态库。
- 没有写项目业务目录。
- 没有打印完整真实 `workflow-state.v0.json` 正文。
- 没有读取或展示 auth、env、密钥、令牌、授权文件内容。
- 没有读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 没有接入非 Codex agent。
- 没有知识库、向量搜索、模型调度、release 打包或网络依赖拉取。

## 当前状态

这条任务从“进行中”改为“已回收”。

当前可以说：

- 工作台能识别当前真实任务包不适合派发。
- 工作台能显示不适合派发的原因。
- 工作台不会把 `not_ready` 任务包包装成 ready。
- 后续生成可派发版本前已有拦截点。

仍不能说：

- 当前已有可派发任务包。
- 当前任务内容已经修正。
- 新版 ready 任务包已生成。
- 真实 Codex 会话派发完成。
- 自动编排执行完成。

## 下一步建议

下一步不要直接做 Codex 派发。

建议先做“任务字段修正输入 v1”：

- 让用户在工作台里输入或粘贴真实任务内容。
- 保存前显示字段级预览。
- readiness 通过后再生成新版任务包文件。
- 仍不派发 Codex、不运行 harness，直到有一个 ready 任务包。
