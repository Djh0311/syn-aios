# 全 views · 内部字段渐进披露收口方案 v1

> 日期：2026-06-20 · 作者：主导线 · 状态：待执行线落地
> 一句话判据：任何视图里，凡"用户完成任务 / 信任结果"用不到的内部字段，默认不显示；失败时给人话 + 下一步；原始诊断收进渐进披露（开发者详情 / 折叠）。

## 0. 缘起 / 与既有方案的关系

- **本方案 = CURRENT.md「下一步」那条**：*"UI 信息架构问题（现有前端全平铺、没层级、没渐进披露）——独立一摊，待排"*。现在正式开出来。
- **不是第二份（ui-prototype-landing 结构重构）的一部分**：第二份管「拆瘦 / 分区 / 导航 / 落位」，已基本完成（证据见 `docs/evidence/2026-06-19-ui-prototype-landing-*`，约 20 份；巨石已拆瘦 App 1104→695 / 记忆 1340→676 / 侧栏 953→340）。本方案管「每个面板**显什么字段**」。两件事，故新开、不并入。
- **复用已验模板**：2026-06-20 智能体聊天框收口已落地并验过——`AgentManualRelayDeveloperDetails`（原始字段进开发者详情折叠）+ `userFacingAgentError`（失败转人话），offline 测试两头焊死（composer 藏 / 详情留）。**本方案 = 把同一套规则推到其余所有面板。**

## 1. 一条规则（全局）

对每个面板的每个字段，三选一：

1. **产品内容** → 留在正常态，用人话标签。
2. **管道 / 诊断**（布尔标志、规范化路径、hash、reason 码、`process_kind` / receipt / `*_status` / `source_kind` 这类）→ 默认收进该区域的「开发者详情 / 折叠」，正常态不显示。
3. **失败信息** → 顶层一句「人话 + 下一步」+「查看开发者详情」入口；原始码进折叠。

**死线（继承聊天框那条，不可破）**：任何报错 / 阻断，原始 reason 必须在折叠里**可达** + 顶层有人话。**藏 ≠ 删 ≠ 静默。**

## 2. 目标面（2026-06-20 扫的，当指针，执行线落地前重核行号）

按泄漏密度，从 `grep` 实测：

- **项目页（大头）**：`ProjectWorkflowGovernancePanels`(11)、`ProjectWorkflowMemoryPanels`(10)、`ProjectWorkflowRunCheckPanel`、`ProjectWorkflowDerivedPanels`、`ProjectTaskDraftPanels`、`ProjectWorkspaceShell`。
- **智能体页**：`AgentExecutionPanels`（执行控制 raw 字段）、`AgentAdapterBoundaryPanels`（能力 / 供应方 / 会话操作三块边界）、`AgentContinuationBoundaryPanels`。
- **其余**：`MemoryListPanels`、`KnowledgeBaseView`、`HarnessBoardView`、`RunningWorkflowsView`。
- **已合规（本次模板，不重做）**：智能体聊天框 composer + `AgentManualRelayDeveloperDetails`。
- **注**：原始错误码平铺已基本清干净（全 views 只剩聊天框 1 处，且是恒为 null 的死分支，顺手删）。剩下的几乎都是「raw 字段平铺」，不是「raw 错误码」。

## 3. 一句冷水（关键 · 别无脑删）

项目页那些治理面板，**有些现在可能就是这个 app 的主内容**（甲阶段产品本来就是 codex 治理 / 工作流编排），不全是管道。**不能一把 find-replace 删光。** 每个面板要先做一次轻判断：这字段是"没人想看的水管"还是"真产品信息"。**这一步判断是本方案的主要工作量，不是机械替换。** 聊天框好办因为它纯是水管；项目页要逐面板过。

## 4. 分批（叶子 → 重头）

- **批 1（低风险，先走）**：纯诊断面板，字段明显是管道——`AgentAdapterBoundaryPanels`、`AgentContinuationBoundaryPanels`、`MemoryListPanels`、`HarnessBoardView`、`RunningWorkflowsView`。直接套模板（raw 进折叠 / 失败转人话）。
- **批 2（要判断，量最大）**：项目页治理面板群——`ProjectWorkflow*`、`ProjectTask*`。逐面板分"留 / 藏"，最需要产品判断。
- **批 3（最敏感，放最后）**：`AgentExecutionPanels`——跨 H1/H2 真执行那条线，字段敏感（`real_codex_executed` / `writes_codex_home` 等），单独过，确保藏的是显示、不是审计可达性。

## 5. 边界 / 高危

- **轻档**，纯前端呈现层。**不碰任何后端 guard / sandbox / runner**；不碰执行逻辑，只改"字段怎么显示"。
- 死线见 §1。
- **与 codex-layout 那轮**：本方案改字段呈现、那轮改布局结构，落点会在 agent 面板重叠；**排在 codex-layout 稳定后**做，或与执行线分文件，避免冲突。

## 6. 验证（每批都要）

- typecheck + offline 绿。
- 每个改过的面板补 / 沿用断言（照搬聊天框 offline 那套两头断言）：
  - 正常态 markup **不含**该面板的 raw 字段；
  - 折叠里**仍含**原始字段 + 失败有人话。
- 扫 diff：确认**没碰后端**；确认没有报错 / 阻断分支被改成"什么都不显示"。

## 7. 落地前确认

- 各面板的实际挂载点 / 默认是否可见，执行线落地前逐个核（有些可能已经在某层折叠里，那就只补人话、不重做）。
