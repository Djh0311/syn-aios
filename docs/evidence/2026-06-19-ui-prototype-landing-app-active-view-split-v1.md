# UI 原型落地 · App 活动视图分发拆瘦 evidence v1

日期：2026-06-19

关联目标：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 当前 goal：继续按“先拆再翻”推进 UI 原型到真前端落地，为批 B/C 的外壳和结构改动降低风险。

## 本轮目标

继续拆瘦计划点名的巨石文件：

- `src/App.tsx`

本轮只迁出活动视图分发，不改状态读取、不改确认弹层动作链、不改真实 Tauri 写入、不改智能体真实会话链。

## 已改代码

- `src/App.tsx`
  - 删除内联 `renderActiveView(...)` 长函数。
  - 保留 App 主状态、数据加载、候选 store 加载、`confirmAction` 真实动作分发。
  - 改为调用 `renderActiveWorkbenchView({ ... })`。
  - 浏览器预览模式的数据回调在 App 中组装后传入，不改变原预览行为。
- 新增 `src/components/ActiveWorkbenchView.tsx`
  - 承接原活动视图分发：
    - 智能体页
    - 项目页
    - 技能 / Harness
    - 运行中工作流
    - canvas
    - 源稿风格占位页：想法箱 / 建议方案 / 工具 / 模型
    - 知识库
    - 记忆中心
    - 设置
    - 首页 fallback
  - 保留智能体真实会话读取链默认入口：
    - `loadCodexSessionTranscript`
    - `loadCodexSessionTranscriptPage`
  - 保留项目页检查 / 预览入口：
    - `renderTaskPackagePreview`
    - `inspectTaskPackageDispatchReadiness`
    - `inspectWorkflowRunCheck`
    - `inspectAutoDispatchAuthorization`

行数变化：

- `App.tsx`：903 行 -> 695 行
- 新增 `ActiveWorkbenchView.tsx`：271 行
- `WorkbenchShell.tsx`：保持 363 行
- `ProjectWorkspaceShell.tsx`：保持 198 行
- `ProjectTaskDraftPanels.tsx`：保持 786 行
- `styles.css`：保持 8709 行

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过
- `git diff --check`：通过

## 未做 / 暂停

- 未拆 `confirmAction`：它和真实写入 / 候选 store 刷新强耦合，后续要拆也应单独做命令分发表，不和 UI 结构改动混在一起。
- 未拆 `styles.css`。
- 未做批 B 的左栏分组、右栏想法入口、顶栏改造。
- 未碰智能体页、知识库整页方向。

## 风险

- 本轮迁出了所有活动视图分发，因此覆盖点比纯组件搬家稍大；已用 typecheck 和 offline interaction 覆盖入口级回归。
- `ActiveWorkbenchView.tsx` 现在集中承接路由分发，后续如果某个 view 的 props 继续膨胀，应优先在对应 view 内部拆，不把业务逻辑塞进这个分发模块。
