# Handoff：Memory Layer M8 Knowledge Base And Obsidian-compatible Interface Placeholder v1

日期：2026-06-05

## 本轮结果

M8 已完成。`知识库` 入口现在是最小知识库资料界面，不再是 placeholder。

完成内容：

- 新增 `knowledgeBase.ts` 前端读模型。
- 新增 `KnowledgeBaseView.tsx`。
- `App.tsx` 的 `knowledge` view 接入知识库界面。
- 新增 `create-memory-candidate` pending action，复用既有 `createMemoryCandidate`。
- `PermissionDialog` 展示知识库候选写入边界。
- `MemoryCenter` 对 `knowledge_doc` 来源显示“来自知识库资料”。
- 离线测试覆盖知识库边界、候选 action 和弹层文案。

## 接受范围

只接受为：

- 知识库资料最小入口。
- `knowledge_doc` 来源引用。
- 正式记忆、候选和任务包知识引用的反向摘要。
- 从明确知识库资料提出记忆候选。
- Obsidian-compatible 边界占位。

不接受为：

- Obsidian 原生同步。
- vault 自动扫描。
- 知识库文档自动进入长期记忆。
- 知识命中、Markdown 摘要、Canvas / Graph / Bases 结果成为正式记忆。
- 正式记忆生命周期操作。
- 中间版本完整记忆系统完成。

## 验证

通过：

- `npm run test:offline-interaction`
- `npm run typecheck`
- `npm run build`
- `src` 禁用文案扫描无命中。

浏览器 smoke：

- Vite 预览 `http://127.0.0.1:5173/` 已打开并进入 `知识库`。
- 知识库空态、Obsidian-compatible 占位和边界说明可见。
- 因不是 Tauri 窗口，真实 sidecar 数据未加载，页面显示预期降级提示。
- 截图已在会话中捕获展示；PNG 文件保存到 evidence 目录失败，原因是 Browser runtime 写 workspace 报 `EPERM`。

## 当前权威入口

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `evidence/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md`

## 下一步

进入 M9：正式记忆生命周期操作任务包拆分。

建议 M9 先明确：

- 正式记忆可执行哪些生命周期动作。
- 哪些动作必须用户确认，哪些可由项目主管确认。
- 每个动作如何创建新版本和审计。
- 冻结、废弃、归档、合并、拆分、上升全局、下沉项目如何影响任务包召回。

仍需保持：

- 不执行真实 worker / Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不让知识库、候选、observation 或 LLM 摘要绕过正式记忆状态机。
