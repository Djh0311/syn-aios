# 任务字段修正输入 v1 总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-29-desktop-shell-task-field-correction-input-v1.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-29-desktop-shell-task-field-correction-input-v1.md`
- Handoff：`product-line/handoffs/2026-05-29-desktop-shell-task-field-correction-input-v1-result.md`
- 被回收产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“任务字段修正输入 v1 已实现”。

不接受为“真实任务字段已修正完成”，不接受为“当前已有 ready 任务包”，不接受为“新版真实任务包已生成”，不接受为“真实 Codex 会话派发完成”，不接受为“自动编排执行完成”。

依据：

- 前端新增明确的“修正任务字段”面板。
- 字段覆盖任务名、所属开发线、背景、目标、允许读取、允许写入、禁止事项、验收标准、必须回传、总指导回收重点。
- 前端保存前显示字段级预览和缺字段提示。
- 保存前走确认弹层，确认文案包含不生成真实任务包、不派发 Codex、不启动 Codex CLI、不运行 harness、不写 `.codex` 或 Codex 状态库。
- 后端新增 `correct_task_package_dispatch_fields`。
- 后端写入路径会备份旧状态、写 audit，并保留已有 `artifact.path`。
- 字段级真实状态检查显示本轮没有保存真实修正字段，`dispatch_correction_events = 0`，`audit_events = 4`。
- 总指导线复跑验证通过。

## 先说薄弱点

- 这轮没有把真实任务修成 ready。
- 这轮没有保存真实业务字段到真实 workflow state。
- 当前真实 generated 文件仍然只有 `2026-05-29-generated-task-draft-smoke.md`。
- 当前预览是字段级预览，不是完整 Markdown 预览。
- 保存后 live UI 是否自动刷新 readiness 还没有真实窗口验证；回传里也建议后续补清楚。

## 接受内容

接受前端能力：

- 有清楚的“修正任务字段”入口。
- 有字段级预览。
- 有缺字段提示，例如 `目标缺失`、`允许写入缺失`。
- 有“不自动补编”的口径。
- 保存动作走确认弹层。

接受后端能力：

- `correct_task_package_dispatch_fields` 专用于派发字段修正。
- 非索引项目拒绝。
- 缺状态文件拒绝。
- 缺 workflow / work item / task_package artifact 拒绝。
- 写入前备份旧状态。
- 写入 audit event：`task_package_fields_corrected_for_dispatch`。
- 保留已有 `artifact.path`，不清掉已生成任务包路径。
- 空字段不会自动补编。
- 保存后可以复检 readiness。

## 真实状态复核

没有打印完整真实 `workflow-state.v0.json` 正文，只做字段级读取。

字段级结果：

```json
{
  "artifacts": 1,
  "audit_events": 4,
  "matching_artifact_paths": 1,
  "dispatch_correction_events": 0,
  "task_names": [null]
}
```

判断：

- 本轮没有写真实修正字段。
- 本轮没有新增真实 correction audit。
- 原 artifact path 仍保留，指向 `/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md`。

真实任务文件复核：

- `find product-line/tasks -maxdepth 1 -type f -name '2026-05-29-generated-*.md'` 只返回 `product-line/tasks/2026-05-29-generated-task-draft-smoke.md`。
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

- 46 个 Rust 单测通过。
- 1 个真实确认测试保持 ignored。

## 安全和范围判断

接受当前安全边界。

依据：

- 没有生成新版真实任务包文件。
- 没有保存真实修正业务字段。
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

- 工作台已经有明确的任务字段修正入口。
- 用户可以在 UI 里输入真实任务字段。
- 保存前可以看到字段级预览和缺字段提示。
- 保存动作有确认边界。
- 后端具备保存修正字段并复检 readiness 的能力。

仍不能说：

- 当前真实任务已经修正。
- 当前已有 ready 任务包。
- 新版真实任务包已生成。
- 真实 Codex 会话派发完成。
- 自动编排执行完成。

## 下一步建议

下一步建议做“真实任务字段填写与 ready 文件生成确认 v1”。

理由：

- 输入能力已经实现。
- 下一步需要由用户提供真实任务内容，确认保存到真实 workflow state。
- readiness 通过后，再生成新版 ready 任务包文件。

建议边界：

- 仍不派发 Codex。
- 仍不运行 harness。
- 仍不写 `/Users/yoyi/.codex` 或 Codex 状态库。
