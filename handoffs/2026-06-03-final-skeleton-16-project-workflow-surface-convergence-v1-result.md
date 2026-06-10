# Handoff：final-skeleton-16 项目工作流页最终收敛 v1

日期：2026-06-03

## 结论

`final-skeleton-16-project-workflow-surface-convergence-v1` 已完成。

接受为：

- 秘书摘要已从通知、待办、审计、项目运行右侧详情中移出。
- 右侧竖栏已有独立 `秘书只读摘要` 入口。
- 秘书入口仍是只读摘要，不是写入、确认、执行或权限批准入口。
- 项目工作流页主区域保留项目工作流画布。
- 候选治理从画布下方独立主 strip 降为项目画布侧栏详情卡。
- 离线测试覆盖右侧入口分离和项目页候选治理降权。

不接受为：

- 秘书聊天完成。
- 秘书自动执行完成。
- 秘书能直接派发任务。
- 秘书能直接批准权限。
- 秘书能写正式记忆。
- 项目画布变成通用节点自动化平台。
- 候选治理升级为正式事实写入。
- 真实 Tauri 窗口截图验收完成。

## 改动文件

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `CURRENT.md`
- `tasks/README.md`

新增：

- `evidence/2026-06-03-final-skeleton-16-project-workflow-surface-convergence-v1.md`
- `handoffs/2026-06-03-final-skeleton-16-project-workflow-surface-convergence-v1-result.md`

## 手动测试清单

在应用里测试：

1. 打开应用，观察右侧竖栏。
2. 点击右侧 `通知中心`，右侧详情应显示通知 / feed，不应出现“秘书只读摘要”。
3. 点击右侧 `待办中心`，右侧详情应显示待办 / feed，不应出现“秘书只读摘要”。
4. 点击右侧 `审计中心`，右侧详情应显示审计 / feed，不应出现“秘书只读摘要”。
5. 点击右侧 `项目运行`，右侧详情应显示项目运行信息，不应出现“秘书只读摘要”。
6. 点击右侧 `秘书只读摘要` / `秘` 入口，右侧详情应只显示秘书摘要和只读边界。
7. 在秘书详情里确认没有“确认执行”、派发、批准权限、写记忆等操作按钮；关闭按钮除外。
8. 进入 `项目`，选择一个项目，打开 `工作流`。
9. 主区域应优先看到项目工作流画布和右侧节点详情，不应在画布下方看到独立的候选治理大面板。
10. 在项目画布右侧详情栏向下查看，应能看到候选治理详情卡，文案仍说明只写候选 sidecar，不写正式事实、不写正式长期记忆、不推进 workflow state。
11. 如果点击候选治理按钮，只应弹出本机动作确认；不要在本轮手动点“确认执行”，除非另开写入验证任务。
12. 不应看到独立 `CanvasView` 被当作项目 workflow state 事实源。

文件层手动核对：

1. `workflow-state.v0.json` 结构不应因本轮改变。
2. 不应新增正式 `MemoryRecord`。
3. 不应新增数据库迁移。
4. `/Users/yoyi/.codex` 不应因本轮操作产生读写。

## 验证

已通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

记录：

- `npm run test:offline-interaction`：`offline interaction tests passed: 8`
- `npm run build`：通过；仍有 Vite chunk size warning。

未运行：

- Rust 测试：本轮未改 Rust。
- 真实浏览器 / Tauri 截图：当前对话未暴露可用浏览器或 Tauri 截图工具。

## 边界确认

本轮未执行真实 Codex，未执行 `codex exec` / `codex exec resume`，未运行 MCP canvas run，未运行 harness，未读写 `/Users/yoyi/.codex`，未读取 auth / token / `.env` / 完整 transcript，未写 workflow state JSON，未写正式事实，未写正式记忆，未迁移数据库。

注意：收尾核对时有一次 shell 引号错误，双引号内的反引号文本触发 zsh 对 `/Users/yoyi/.codex` 的命令替换尝试并返回 permission denied；没有读写 `.codex`，随后已用单引号重跑核对。后续搜索含反引号文本继续使用单引号或 `rg -F`。
