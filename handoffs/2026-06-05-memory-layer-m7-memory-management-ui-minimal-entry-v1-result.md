# Handoff：Memory Layer M7 Memory Management UI Minimal Entry v1 Result

日期：2026-06-05

## 回收结论

接受为 M7 记忆管理 UI 最小入口完成。

不接受为中间版本记忆系统完成；不接受为正式记忆生命周期操作完成；不接受为知识库 / Obsidian 接口完成；不接受为关系治理、维护任务、成熟模式或跨项目记忆完成；不接受为真实 worker / Codex 执行完成；不接受为真实窗口 / 截图验收完成。

## 当前可操作状态

- 全局 `记忆` 一级入口已复用原导航并替换为 `MemoryCenterView`。
- `src/lib/memoryCenter.ts` 负责从现有 stores 和 workflow task package summary 派生前端只读 `MemoryManagementSummary`。
- 正式记忆条目显示来源、版本、审计、scope、权限策略、模型外发策略、lint / conflict 摘要和任务包入选资格。
- 候选条目显示状态、风险、确认要求、采纳回链和“不是正式记忆”边界。
- 观察只显示为观察来源，不进入正式记忆列表。
- 任务包冻结快照只显示可证明的 included / excluded / review materials 摘要；没有逐条 snapshot 明细时不伪造入选原因。
- 项目相关记忆只做轻量摘要。

## 重要非目标

- M7 不新增正式记忆编辑、删除、废弃、冻结、归档、合并、拆分、上升全局、下沉项目等生命周期按钮。
- M7 不新增 UI 写正式记忆动作。
- M7 不新增后端写命令，不写任何 memory / observation / lint / workflow sidecar。
- M7 不接知识库 / Obsidian / 向量库 / 图数据库。
- M7 不把 `candidate_confirmed`、observation、knowledge hit、LLM summary 或 task package content 说成正式记忆。
- M7 不执行真实 worker、真实 Codex、`codex exec` 或 `codex exec resume`。

## 验证记录

- 红灯：`npm run test:offline-interaction` 初次失败，缺 `../src/lib/memoryCenter` 和 `../src/views/MemoryCenterView`。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 9`。
- `npm run typecheck`：通过。
- `npm run build`：通过；保留既有 Vite chunk size warning。
- 禁止文案扫描新增文件：通过，无 M7 禁止误导文案命中。
- `curl -sS -I http://127.0.0.1:4173/`：通过，返回 `HTTP/1.1 200 OK`。

## UI 验收缺口

真实窗口 / 截图验收未完成。

原因：

- 当前线程工具发现没有 in-app Browser 导航 / 截图工具。
- 项目未安装 `playwright` 或 `@playwright`。
- 未下载新依赖。

已做：

- 沙箱内 Vite 启动被端口绑定 EPERM 拦截后，按权限流程在沙箱外启动 dev server。
- HTTP smoke 通过。
- 结束前已关闭 dev server。

## 下一步建议

下一步进入 M8 时，重点是知识库 / Obsidian-compatible 接口占位和边界，不要把 M7 记忆中心 UI 当作生命周期后台。M9 才能讨论正式记忆生命周期动作；M10-M13 继续补关系治理、维护任务、成熟模式、跨项目记忆和最终验收。
