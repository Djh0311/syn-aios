# 任务包：Codex 项目级工作流最小数据模型

## 任务名

定义阶段 3 的 Codex 项目级可视化编排最小数据模型。

## 所属开发线

信息架构线。

这是现有信息架构线任务，不新增常设开发线。

## 背景

新版 UI 骨架已经完成，并通过真实 Tauri 窗口 smoke 验证。下一步不能直接上复杂画布或自动化调度，需要先定义最小工作流数据模型。

用户已确认新的开发规则：一切以最终版本状态去开发，保持高扩展性。当前任务必须按“架构和模型高扩展、功能交付仍 Codex-only”的口径执行。

已知依据：

- `product-line/STAGE_PLAN.md` 阶段 3 要求项目级可视化编排。
- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-redesign-review.md` 接受新版 UI 骨架，但明确 Director、Review、边关系、状态机仍缺数据模型。
- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-tauri-smoke-validation-review.md` 接受真实 Tauri 窗口 smoke 验证，但明确还不能说可编辑自动化工作流完成。
- `product-line/decisions/2026-05-28-extensible-first-development-rule.md` 确认后续开发必须按最终形态预留扩展，但当前阶段仍只交付 Codex 治理。
- 用户补充：当前制作的 harness 基本是针对 Codex 的，并且以文件夹形式存在；当前 harness 索引模式可能有问题。

## 目标

- 定义阶段 3 最小本地工作流模型。
- 模型命名和字段必须保持 agent-agnostic，不能写死成 Codex-only schema。
- 明确项目、角色、会话、任务包、handoff、evidence、review、harness 在工作流里的关系。
- 明确哪些字段来自当前 `codex-index.json`，哪些字段需要新增本地工作台状态文件。
- 明确哪些字段不能来自 Codex 内部状态库。
- 明确工作流状态枚举和状态转换边界。
- 明确 UI 工作流画布第一版应该展示哪些节点和边。
- 明确“Codex 专用、文件夹形式 harness”的正确建模方式。
- 明确当前 Codex-only 实例如何映射到未来可扩展的 agent / adapter / LM Director 模型。
- 输出后续任务拆分建议：
  - 索引内核线是否需要补文件夹式 harness 扫描。
  - 桌面应用线是否需要接入最小模型。
  - 验证线需要覆盖哪些模型和 UI 场景。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/tasks/README.md`
- `product-line/decisions/2026-05-28-codex-workbench-ui-ia-direction.md`
- `product-line/decisions/2026-05-28-extensible-first-development-rule.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-ia-redesign-review.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-redesign-review.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-tauri-smoke-validation-review.md`
- `product-line/evidence/2026-05-28-codex-workbench-ui-shell-redesign.md`
- `product-line/evidence/2026-05-28-codex-workbench-ui-shell-tauri-smoke-validation.md`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/HarnessBoardView.tsx`

## 允许写入

- `product-line/evidence/`
- `product-line/handoffs/`
- `product-line/decisions/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不改前端、Tauri、Rust 或索引内核代码。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不接入非 Codex agent。
- 不把 OpenClaw / VS Code / OpenCode / Claude Code 写成已可用能力。
- 不做个人知识库、向量搜索、模型调度。
- 不把 LM 建议当作事实状态；LM 只能作为未来智能核心预留，事实状态必须落到本地工作流模型、evidence、handoff、review 或审计记录。
- 不做 release 打包。
- 不把“文件夹式 Codex harness”写成已被当前索引完整支持；必须标为待验证或待补扫描能力。
- 不把模型字段写死为 `codex_task`、`codex_session_node`、`codex_harness_only` 这类未来难迁移结构。

## 必须回答的问题

1. 阶段 3 最小工作流对象有哪些？
2. 每个对象的最小字段是什么？
3. 哪些对象是只读索引派生，哪些对象需要工作台本地状态？
4. 工作流节点有哪些类型？
5. 工作流边有哪些类型？
6. 状态枚举是什么？
7. 哪些状态转换需要用户确认？
8. Director、Coder、Reviewer、Tester 在当前 Codex-only 版本里如何表达？
9. 任务包、handoff、evidence、review 的关系如何表达？
10. Codex 专用 harness 是资源节点、验证节点，还是项目能力节点？
11. 文件夹形式 harness 应该如何建模？
12. 当前 `codex-index.json` 的 harness 候选字段缺什么？
13. 后续索引内核线要补哪些扫描规则？
14. 桌面应用线接入模型时，第一版要展示什么，不能展示什么？
15. 验证线后续要怎么验证模型没有读正文、没有写 Codex 状态库？
16. 哪些字段用于未来扩展：`agent_type`、`agent_id`、`adapter_id`、`artifact_type`、`source_kind`、`permission_level`、`workflow_version`、`node_type`、`edge_type`、`model_policy`？
17. 当前 Codex-only 功能如何填充这些通用字段？
18. 哪些长期能力只是预留，不进入当前实现？

## 验收标准

- 有 evidence 和 handoff。
- 如形成正式决策，必须写入 `product-line/decisions/`。
- 明确区分事实、假设、缺口和后续任务。
- 明确说明当前 harness 索引模式的风险，尤其是 Codex 专用文件夹式 harness。
- 不扩大到多 agent、知识库、向量搜索、模型调度。
- 模型必须按最终形态预留扩展字段，但不得把未实现能力写成已完成。
- 必须明确 `source_kind`、`permission_level`、`workflow_version` 或等价字段。
- 必须明确 LM 是未来智能核心，不是事实核心；事实状态必须有本地记录。
- 不要求本轮实现代码。
- 不把当前 UI 骨架包装成可编辑工作流完成。
- 不把当前 harness 索引包装成已完整支持文件夹式 Codex harness。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增了哪些 evidence / handoff / decision
4. 最小工作流模型是什么
5. harness 文件夹问题如何处理
6. 哪些字段来自当前索引
7. 哪些字段需要新增本地状态
8. 哪些结论仍不确定
9. 建议派给索引内核线、桌面应用线、验证线的后续任务
10. 风险是什么
11. 如何满足“最终形态预留扩展”的开发规则

## 总指导回收重点

回收时必须判断：

- 是否接受为阶段 3 最小工作流模型。
- 是否正确处理 Codex-only 范围。
- 是否满足高扩展开发规则，避免写死 Codex-only schema。
- 是否把文件夹式 Codex harness 风险讲清楚。
- 是否没有把缺失能力写成已完成。
- 是否能据此派发下一轮实现或索引修正任务。
