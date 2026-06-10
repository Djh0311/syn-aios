# 任务包：桌面壳项目工作流最小编排闭环 v1

## 所属开发线

桌面应用线。

关联口径来源：

- 信息架构线
- Codex 会话线

后续验证：

- 本任务完成后另派验证线；验证线不是本任务的共同执行线。

## 背景

当前已确认：会话开发方案保留，但优先做工作流。

依据：

- `product-line/decisions/2026-05-29-codex-session-plan-retained-workflow-first.md`
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`

当前已有：

- 工作流事实层 v0 读写底座。
- 项目默认工作流草稿初始化。
- 项目页工作流骨架。
- 项目页 Agent 会话入口。
- 任务包内部协议能力。

但当前还缺：

- 工作流状态机。
- 工作项在工作流节点之间的清晰流转。
- 总指导、执行线、回收之间的最小闭环表达。
- 每次流转的审计记录。

## 薄弱点

- 本任务不做真实 Codex 自动执行。依据：用户当前只确认优先工作流，没有给出启动或 resume 真实 Codex 会话的精确授权。
- 本任务不补 Agent 会话中心 UI v2。依据：当前优先级是工作流，不是会话页体验精修。
- 本任务做出的“跑起来”是工作台状态流转跑起来，不是 Codex 真正开始写代码。依据：`codex resume` 多轮控制仍未验证。
- 如果状态机设计过重，会拖慢当前阶段；所以 v1 只做最小闭环。

## 已知、未知和假设

已知：

- 工作台可以写自己的 `workflow-state.v0.json`。
- 默认项目 workflow 已能生成节点和边。
- 任务包能力可作为内部消息和导出物，但不是主界面中心。
- Agent 会话能力后续会被工作流节点复用。

未知：

- 后续真实执行时，一个工作流节点应新建会话、复用会话，还是绑定已有会话。
- 工作流节点并发运行和失败重试规则还没定。
- 人工审核阶段的最细权限边界还没定。

假设：

- v1 只做单项目、单工作流、单工作项的最小闭环。
- v1 只写工作台自己的状态文件，不写 Codex 状态库。
- v1 不读业务会话正文，除非用户在已有会话入口里明确打开。
- v1 不生成真实任务包文件，除非沿用已有明确生成入口且用户确认。

## 目标

在项目详情页实现一个最小可用的工作流编排闭环：

1. 用户能在项目里看到当前默认工作流。
2. 用户能创建一个工作项。
3. 用户能把工作项绑定到工作流节点或角色。
4. 用户能推进工作项状态：
   - 草稿
   - 待派发
   - 执行中
   - 待回收
   - 已接受
   - 需修改
   - 暂停
5. 页面能清楚显示当前工作项在哪个节点、由哪个角色负责、下一步是什么。
6. 每次状态变化写入 `audit_events[]`。
7. 工作流页以“编排和流转”为中心，不以任务包文件为中心。

大白话目标：

让项目里的工作流先真的能被你安排和推进，而不是只看到一堆索引信息或任务包字段。

## 非目标

- 不启动 Codex CLI。
- 不执行 `codex resume`。
- 不新建真实 Codex 业务会话。
- 不自动读取业务会话正文。
- 不自动运行 harness。
- 不做多 agent 接入。
- 不做个人知识库。
- 不做复杂画布编辑器。
- 不做节点拖拽连线。
- 不做并发调度。
- 不做 release 打包。
- 不把任务包文件作为主入口。

## 最小工作流模型

建议 v1 使用已有默认节点，并补齐工作项流转：

- `director`：总指导，负责规划和回收。
- `developer`：开发线，负责执行。
- `review`：回收评审，负责接受或要求修改。

如果现有默认节点更多，可以保留展示，但 v1 的状态流只要求覆盖上面三类角色。

建议最小状态流：

```text
draft -> ready_to_dispatch -> running -> ready_for_review -> accepted
draft -> ready_to_dispatch -> running -> ready_for_review -> needs_changes -> ready_to_dispatch
任意非终态 -> paused
paused -> ready_to_dispatch
```

界面显示可以用中文：

- 草稿
- 待派发
- 执行中
- 待回收
- 已接受
- 需修改
- 暂停

## 建议实现范围

### 后端

在 Tauri Rust 后端新增或扩展命令：

- 创建工作项。
- 更新工作项状态。
- 绑定工作项到节点或角色。
- 读取项目工作流详情。
- 追加审计事件。

写入规则：

- 只写工作台自己的 `workflow-state.v0.json`。
- 写入前备份。
- 临时文件写入后原子替换。
- 写入后重新读取校验。
- 非索引项目拒绝。
- 缺 workflow 时拒绝，或提示先创建项目默认 workflow。
- 非法状态跳转拒绝。

### 前端

项目详情页的 `工作流` 视图需要调整为：

- 左侧保留项目窄功能列表。
- 中间显示工作流主区域。
- 右侧显示当前工作项或节点详情。

最小 UI：

- 工作流状态条。
- 节点列表或轻量画布。
- 工作项列表。
- 当前工作项详情。
- 下一步动作按钮：
  - 标记待派发
  - 标记执行中
  - 标记待回收
  - 接受
  - 要求修改
  - 暂停
- 审计事件摘要。

要求：

- 不再把任务包字段编辑作为工作流主视觉。
- 按钮文案要说明写入的是工作台状态，不是 Codex 状态库。
- 长标题、长路径、长摘要不能把布局撑乱。
- 系统缺数据时显示缺口，不补编业务。

### 与会话线关系

本任务只保留会话绑定入口，不做真实聊天。

可以展示：

- 当前工作项是否绑定 Codex 会话。
- 绑定来源是索引推断、用户绑定、工作流绑定还是未知。
- 从节点跳到项目 Agent 会话页。

不做：

- 发送消息。
- resume 会话。
- 创建业务会话。
- 读取业务会话正文。

## 允许读取

允许读取：

- `product-line/decisions/2026-05-29-codex-session-plan-retained-workflow-first.md`
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/src-tauri/`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/prototypes/index-kernel/codex-index.json`

允许读取真实工作台状态文件的元数据和必要结构：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 与本任务无关的业务会话正文

## 允许写入

允许写入项目内：

- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/src-tauri/`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/evidence/2026-05-29-desktop-shell-workflow-orchestration-min-loop-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-workflow-orchestration-min-loop-v1-result.md`

允许在用户通过 UI 确认时写入工作台自己的状态文件：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

测试中优先使用临时目录或夹具，不默认改真实状态文件。

## 禁止事项

- 禁止写 `/Users/yoyi/.codex`。
- 禁止改 Codex 状态库。
- 禁止运行 `codex resume`。
- 禁止运行真实业务 Codex 会话。
- 禁止自动读取业务会话正文。
- 禁止运行 harness。
- 禁止生成真实任务包文件作为本任务验收。
- 禁止把任务包字段编辑放回主流程中心。
- 禁止把索引推断的项目归属写成用户确认事实。
- 禁止把缺字段补编成业务内容。

## 验收标准

必须满足：

- 项目工作流页能看到工作流节点、工作项、状态和下一步动作。
- 能创建工作项，并写入工作台状态。
- 能合法推进状态，并拒绝非法状态跳转。
- 每次状态变化写入审计事件。
- UI 能显示当前状态、负责角色、下一步动作和最近审计事件。
- 没有 workflow 的项目显示明确缺口或引导创建默认 workflow。
- 非索引项目被后端拒绝。
- 任务包相关能力不再占据工作流主界面。
- 会话入口只作为绑定或跳转，不新增发送和 resume。

验证命令：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --offline
```

如果改了 Rust，必须跑 Rust 测试。  
如果没有改 Rust，说明为什么没跑。

## 必须回传

回传时必须说明：

- 做了什么。
- 改了哪些文件。
- 新增了哪些 evidence / handoff。
- 是否写了真实 workflow state。
- 如果写了，写入了哪些类型的字段，不要打印完整状态正文。
- 是否写了 `/Users/yoyi/.codex`，答案应为没有。
- 是否运行 Codex CLI，答案应为没有。
- 是否读取业务会话正文，答案应为没有，除非任务中另有明确授权。
- 测试命令和结果。
- 当前仍不能自动执行 Codex 的缺口。

## 总指导回收重点

回收时重点看：

- 这轮是否真的把工作流状态流转做起来。
- UI 是否回到工作流中心，而不是任务包字段中心。
- 是否保持会话线作为后续执行底座。
- 是否没有越权写 `.codex` 或启动真实 Codex。
- 状态流转是否有审计，非法跳转是否被拒绝。
