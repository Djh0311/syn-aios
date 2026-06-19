# 智能体视图 · 信息呈现收口方案 v1

> 日期：2026-06-20 · 作者：主导线 · 状态：待执行线落地
> 一句话判据（碰不碰本方案）：**只动「智能体」视图里字段怎么显示；不在该视图、不属于呈现层，就不碰。**

## 0. 一句话目标

正常态界面只剩任务本身；内部字段默认不存在；失败时翻译成**一句人话 + 下一步**；原始诊断收进**已有的**「开发者详情」折叠，审计能力不丢。

## 1. 动机

- 用户实机发消息撞到 `Manual relay 失败：manual_relay_guard_blocked:manual_relay_denied_material_requested`——把后端原始错误码直接甩用户脸上。
- 拍板原则（用户定）：**没有人想看 `target_cwd_canonical` / `real_codex_executed` / `path_verified` 这类字段，自用也不想。这些是「出问题时」的诊断提示，不是常驻界面。**
- 当前「智能体」视图把审计/信封/回执仪表盘当成产品界面在摊，受众混了：开发/审计 instrumentation ≠ 用户产品界面。

## 2. 重要重定位（决定方案大小）

**这是「收口 + 补缺」，不是从零造，量和风险都小。** 项目里已有底子：

- 底部「开发者详情」`<details>` 折叠，默认关：`AgentConversationShell.tsx:258`（`developerOpen=false`）、`:785-794`（折叠壳）、`developerDetails` 是外部传入的 `React.ReactNode`（`:192/239`）。
- 三块边界面板已归堆：`AgentDeveloperPanels.tsx:91-99`（能力 / 供应方 / 会话操作，来自 `AgentAdapterBoundaryPanels.tsx`）。
- 人话映射底子充足：`agentLabels.ts` 已有 42 个 Label 函数，含 `codexControlReasonLabel`（reason→人话）、边界摘要人话化（`agentLabels.ts:303`）。
- **目标风格已有现成样板**：`AgentConversationShell.tsx:378` 的超时提示就是「人话 + 下一步」——「Codex 运行超过 10 分钟，自动停止失败：…。你可以重新发送或手动处理。」

> ⚠️ 落点是 2026-06-20 快照，当**指针**用。执行线落地前按当前代码重核行号，不照搬。

## 3. 现状泄漏点（要改的）

| # | 位置（指针） | 问题 |
|---|---|---|
| L1 | `AgentChatComposer.tsx:172` `:173` | 生错误码平铺。`:173` 是 `Manual relay 失败：{manualRelayError}`——**用户撞的这条**。无人话层。 |
| L2 | `AgentExecutionPanels.tsx:345` | `操作失败：{error}` 同病，生串平铺。 |
| L3 | `AgentChatComposer.tsx:174-262` | 撰写区**自带一个常驻「边界」折叠**，与底部「开发者详情」**功能重复**；内里铺 envelope（`target_cwd_canonical` / `sandbox` / `allowed_write_roots` / `path_verified` / `payload_layers`）+ receipt（`process_kind` / `real_codex_executed` / `real_process_killed` / `syn_read_codex_home` / `killed_by_user`）全生字段。 |
| L4 | `AgentChatComposer.tsx:114` `:182` | 状态行/目标条显示**完整项目路径** `selectedProjectRoot`，应显**项目名**。 |
| L5 | `AgentChatComposer.tsx:240` | `中转被阻断：{relayGuard.reasons.join(" / ")}`——生 reason 码拼串；`relayDirectSendBlockedReason` 等同类。 |

## 4. 核心三件事

### 4.1 正常态瘦身
- 删掉 composer 常驻「边界」折叠（L3，`174-262`），其有价值内容**并入已有的**底部「开发者详情」——**不要两个并存**。
- 目标条项目路径 → 项目名（L4）。`projectOptions` 已有 `label`，复用。
- 撰写区正常态只剩：目标（项目名 + 会话名）、一个状态、输入框、发送/Stop/恢复轮询。

### 4.2 失败态转人话
- 新增 relay/guard `reason-code → {人话, 下一步}` 映射，**照搬现成 `codexControlReasonLabel` 模式**放进 `agentLabels.ts`。
- 所有 `error-text` 平铺点（L1/L2/L5）改走该映射。
- 给不出专门文案的兜底：**「没发出去：<一句概括>。展开『开发者详情』看原始原因。」**——绝不直接显原始码当主信息。

### 4.3 诊断收口、审计保留
- receipt / envelope 生字段（L3 内容）迁进「开发者详情」，**不删**。
- 保证每个失败：顶层有人话提示 **且** 详情里能查到原始 reason + receipt。

## 5. 边界与高危分类

- **轻档**。纯前端呈现层。
- **不碰安全闸 / 沙箱 / guard 逻辑本身（高危清单 #3）**：后端 `manual_relay.rs` / `codex_local_runner.rs` **一行不动**；guard 照常算 reason，本方案只改「reason 怎么显示」。
- **一条死线——失败 reason 不许吞**：任何 `blocks_execution` / error 必须**至少**在「开发者详情」里能看到原始码，并在顶层给一句人话。**藏 ≠ 删 ≠ 静默。** 不许出现「失败了但界面什么都不显示」。
- **范围只限「智能体」视图**（composer + 该视图的开发者详情 + `agentLabels` 映射）。其它视图（projects 等）若同病，**本方案不顺手扩**，记为「同样问题，待用户点头再做」。
- `denied_material_policy` 描述字段（`manual_relay.rs:70/334` + `manualRelay.ts:36`）目前无人渲染，本方案**不处理**；前端这轮稳定后单独决定去留。

## 6. 与其它方案的关系 / 执行顺序

- 本方案是「信息呈现」**一件事**，独立于第一份（智能体 codex 布局结构）和第二份（整 syn 重构）。
- 落点压在 `AgentChatComposer` / `AgentConversationShell` 上，与**第一份 codex-layout 那轮同文件**。建议：**排在 codex-layout round 稳定之后**做，或与执行线明确分文件/分段，避免冲突。

## 7. 验证（完成必附真证据）

- typecheck + 现有 offline checks 全绿。
- 手动：
  - 正常发一条 → 撰写区无任何生字段（无 `process_kind` / 路径 / hash）。
  - 故意不绑项目触发失败 → 顶层一句人话 + 下一步；展开「开发者详情」能看到原始 reason + receipt。
- 扫 diff：确认**没动**后端 guard 任何文件；确认没有 error/`blocks_execution` 分支被改成「什么都不显示」。

## 8. 待执行线落地前确认的点

- `AgentDeveloperPanels`（三块边界面板）的**实际挂载处**：在 `src/views/agent/` 内未 grep 到引用，它是经 `developerDetails` prop 喂进折叠、还是从别处（如 `AgentView`）挂载？落地前先核这一处，确定「开发者详情」里到底现有什么、迁入的内容放哪。
