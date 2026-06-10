# Handoff：final-skeleton-15 秘书核心只读模型 v1

日期：2026-06-03

## 结论

`final-skeleton-15-secretary-core-readonly-model-v1` 已完成。

接受为：

- 秘书只读模型第一版完成。
- 可从现有 snapshot、workflow state、黑板候选 sidecar、记忆候选 sidecar 和 adapter descriptor 派生秘书上下文。
- 能展示风险、建议、候选和下一步查看提案。
- UI 文案明确“建议，不是事实变更”。
- UI 文案明确“候选，不是正式记忆”。
- 测试证明 secretary proposal 不会自动执行。
- 没有写 workflow state。
- 没有写正式事实。
- 没有写正式记忆。

不接受为：

- 秘书聊天完成。
- 秘书自动执行完成。
- 秘书能直接派发任务。
- 秘书能直接批准权限。
- 秘书能直接写正式记忆。
- 记忆管理界面完成。
- Obsidian / 知识库集成完成。
- Claude / OpenClaw / OpenCode 接入完成。

## 改动文件

- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/components/SecretaryBrief.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `CURRENT.md`
- `tasks/README.md`

新增：

- `evidence/2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1.md`
- `handoffs/2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1-result.md`

## 可操作状态

前端：

- `deriveSecretaryContext()` 可复用，输入为 `WorkbenchSnapshot`、`WorkflowStateSnapshot`、黑板候选 store、记忆候选 store 和 `workflowStateError`。
- `SecretaryBrief` 已接入全局右侧详情区域。
- 右侧详情打开通知、待办、审计或项目运行时，顶部可看到“秘书只读摘要”。

边界：

- `SecretarySuggestion` 只提示查看或确认。
- `SecretaryActionProposal` 全部 `executable_now: false`。
- `SecretaryMemoryCandidate` 全部 `is_formal_memory: false`。

## 手动测试清单

在应用里测试：

1. 打开应用。
2. 点击右侧竖向入口里的“通知中心”“待办中心”“审计中心”或“项目运行”任意一个。
3. 在右侧详情顶部找到“秘书只读摘要”。
4. 检查摘要里显示：
   - “需要你确认”
   - “建议，不是事实变更”
   - “候选，不是正式记忆”
5. 如果当前 workflow state 有 pending 权限请求，摘要的“权限”计数应大于 0，并在建议中出现权限查看方向。
6. 如果当前 workflow state 有 failed / timed_out attempt，摘要风险中应出现失败或超时提示。
7. 如果已有 `blackboard-candidates.v1.json` pending 记录，摘要的“黑板候选”计数应反映待处理候选。
8. 如果已有 `memory-candidates.v1.json` pending 记录，摘要的“记忆候选”计数应反映待审候选。
9. 检查摘要区域没有“秘书已执行”“秘书已处理”“已记住”“正式事实已写入”等文案。
10. 点击摘要区域不会触发确认弹层、Codex 派发、workflow state 写入或记忆写入。

文件层手动核对：

1. `workflow-state.v0.json` 不应新增秘书字段。
2. 不应出现正式 `MemoryRecord` 文件或数据库迁移。
3. `/Users/yoyi/.codex` 不应因本轮操作产生读写。

## 验证

已通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

记录：

- `npm run test:offline-interaction`：`offline interaction tests passed: 7`
- `npm run build`：通过；仍有 Vite chunk size warning。

未运行：

- Rust 测试：本轮未改 Rust。
- 真实浏览器 / Tauri 截图：当前线程未暴露浏览器控制工具，本轮按任务包要求完成离线验证。

## 下一步

可以进入 `final-skeleton-16` 项目工作流页最终收敛。

下一步仍不能做：

- 秘书直接改事实。
- 秘书直接派发任务。
- 秘书批准权限。
- 秘书写正式记忆。
- 把候选当作正式任务包上下文或正式记忆。
