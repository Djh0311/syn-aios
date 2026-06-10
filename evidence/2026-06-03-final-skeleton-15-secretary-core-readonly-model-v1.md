# final-skeleton-15 秘书核心只读模型 evidence v1

日期：2026-06-03

## 先说薄弱点

- 本轮没有做真实浏览器或 Tauri 窗口截图验收：当前线程没有可用浏览器控制工具，且任务包强制验证命令是离线测试、typecheck、build。
- 秘书摘要只是读模型展示，不是聊天入口、调度入口或确认入口。
- `SecretaryActionProposal` 仍只是“可查看方向”的 proposal；没有接 `PendingAction`，没有可执行按钮。
- 记忆候选展示覆盖 sidecar 和黑板派生候选，但不写 `MemoryRecord`，不代表工作台已经长期记住。

## 本轮目标

按 `tasks/2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1.md` 完成秘书核心只读模型第一版：

1. 定义 `SecretaryContext`、`SecretarySuggestion`、`SecretaryRiskSignal`、`SecretaryMemoryCandidate`、`SecretaryActionProposal`。
2. 从现有 snapshot、workflow state、黑板候选 sidecar、记忆候选 sidecar、adapter descriptor 派生状态。
3. 在 UI 中给出小型只读摘要，明确建议不是事实变更，候选不是正式记忆。

## 已实现

新增：

- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/components/SecretaryBrief.tsx`

修改：

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `CURRENT.md`
- `tasks/README.md`

实现内容：

- `deriveSecretaryContext()` 是前端纯函数，不新增 Tauri command。
- `source_kind` 固定为 `derived_read_model`。
- `warnings` 固定包含 `secretary_context_is_read_only`。
- 风险信号覆盖：
  - workflow state error。
  - diagnostics warning。
  - pending permission requests。
  - failed / timed_out execution attempts。
  - pending blackboard candidates。
  - pending memory candidates。
  - adapter descriptor warnings。
- 建议最多 5 条，且全部：
  - `requires_user_confirmation: true`
  - `is_fact_change: false`
- action proposal 全部：
  - `requires_user_confirmation: true`
  - `executable_now: false`
  - 带 `blocked_reason`
- `SecretaryMemoryCandidate` 显示固定边界：`候选不等于工作台已经长期记住。`
- `SecretaryBrief` 接入全局右侧详情区域，不新增固定秘书页面，不塞进项目画布右侧栏。

## 红灯测试

先写离线测试后实现：

- 初次运行 `npm run test:offline-interaction` 失败，原因是缺少：
  - `../src/components/SecretaryBrief`
  - `../src/lib/secretaryReadModel`
- 补实现后，新增 secretary 场景通过。

## 验证结果

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell` 通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 7`。
- `npm run build`：通过；仍有 Vite chunk size warning，构建产物生成成功。

未运行 Rust 验证：

- 本轮未修改 Rust。

## 边界自检

新增秘书生产文件固定文本搜索无命中：

- `codex exec`
- `/Users/yoyi/.codex`
- `createMemoryCandidate`
- `已记住`
- `秘书已执行`
- `正式事实已写入`

说明：

- `tests/offline-permission-dialog.test.tsx` 内有历史确认弹层文案包含 `codex exec resume` 和 `/Users/yoyi/.codex`，它们是既有离线测试边界文案，不是本轮新增秘书生产路径。

## 没有做

- 没有做秘书聊天。
- 没有调用 LLM。
- 没有新增后端命令。
- 没有执行真实 Codex。
- 没有执行 `codex exec` / `codex exec resume`。
- 没有读写 `/Users/yoyi/.codex`。
- 没有读取 auth、token、`.env`、完整 transcript。
- 没有写 workflow state JSON。
- 没有改 `workflow-state.v0.json` 结构。
- 没有写正式事实。
- 没有写正式 `MemoryRecord`。
- 没有写正式长期记忆。
- 没有接 Obsidian、向量库、图数据库。
- 没有运行 MCP canvas run。
- 没有运行 harness。

## 下一步判断

可以进入 `final-skeleton-16` 项目工作流页最终收敛。

进入下一步时仍需保持：

- 秘书摘要是只读协作层，不是执行器。
- 候选治理不能升级为正式事实或正式记忆。
- 项目工作流画布仍是项目 workflow 主入口，但不能变成通用节点执行器。
