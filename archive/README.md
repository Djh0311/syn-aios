# Product Line Archive

## 归档口径

这个目录保存历史任务包、历史 evidence、历史 handoff 和不再作为当前入口的决策。

归档文件只提供历史依据，不提供当前行动顺序。当前入口仍然是：

- `../CURRENT.md`
- `../README.md`
- `../STAGE_PLAN.md`
- `../tasks/README.md`

## 归档数量

本轮归档：

- `archive/tasks/`：48 个任务包。
- `archive/evidence/`：50 个 evidence。
- `archive/handoffs/`：89 个 handoff / review。
- `archive/decisions/`：2 个决策。

当前目录保留：

- `../tasks/README.md`
- `../decisions/` 下 8 个当前权威决策。

## 路径规则

原路径到新路径的规则：

- `tasks/<file>.md` -> `archive/tasks/<file>.md`
- `evidence/<file>.md` -> `archive/evidence/<file>.md`
- `handoffs/<file>.md` -> `archive/handoffs/<file>.md`
- `decisions/<file>.md` -> `archive/decisions/<file>.md`

## 归档原因

### tasks/

归档原因：

- 已完成但只是历史阶段。
- 已被后续决策 supersede。
- 属于旧任务包管理器方向的中间能力。
- 属于暂停项或清理任务自身。
- 仍有审计价值，但不应留在当前任务入口。

特别说明：

- `archive/tasks/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1.md` 已回收为代码路径实现，不再是待执行任务。
- `archive/tasks/2026-05-29-desktop-shell-write-confirmation-hardening.md` 是暂停任务。
- `archive/tasks/2026-05-29-generated-task-draft-smoke.md` 是历史生成任务包，不作为当前产品方向依据。
- `archive/tasks/2026-05-29-product-line-cleanup-current-authority-v1.md` 是本轮清理任务包，清理完成后归档。

### evidence/

归档原因：

- evidence 是历史依据和验收材料。
- 当前入口不应通过 50 个 evidence 推导下一步。
- 需要查某项能力来源时按文件名定位。

特别说明：

- 会话读取、会话控制、绑定派发、桌面派发代码路径等 evidence 仍保留在归档里作为依据。
- 归档不等于删除或否定能力。

### handoffs/

归档原因：

- handoff / review 是历史回收材料。
- 当前入口只保留状态摘要，具体依据移入归档。

特别说明：

- `archive/handoffs/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1-review.md` 是当前判断“代码路径已实现、真实 safe probe 未执行”的主要依据。
- `archive/handoffs/2026-05-29-codex-bound-session-dispatch-probe-v1-result.md` 是判断 `codex exec resume` 无业务绑定派发通过的主要依据。

### decisions/

归档决策：

- `archive/decisions/2026-05-27-desktop-container-route.md`
- `archive/decisions/2026-05-29-ui-reference-sources.md`

归档原因：

- 桌面容器路线已经被当前技术栈决策吸收。
- UI 参考源只作为后置参考，不作为当前路线入口。

## 当前保留的权威决策

- `../decisions/2026-05-27-technical-stack-and-expansion-architecture.md`
- `../decisions/2026-05-28-extensible-first-development-rule.md`
- `../decisions/2026-05-28-codex-workflow-min-model.md`
- `../decisions/2026-05-28-workflow-state-storage-v0.md`
- `../decisions/2026-05-28-codex-workbench-ui-ia-direction.md`
- `../decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `../decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `../decisions/2026-05-29-codex-session-plan-retained-workflow-first.md`

## 删除情况

本轮没有删除 Markdown 文件。

原因：

- 历史 evidence 和 handoff 仍有审计价值。
- 没有足够依据断言某个历史 Markdown 没有独立价值。
- 不确定的文件按任务要求优先归档。
