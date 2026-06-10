# 任务包：product-line 当前权威清理与归档 v1

## 所属开发线

总指导线。

## 背景

当前 `product-line` 已经积累了大量阶段性任务包、evidence、handoff 和决策文档。粗略盘点显示，`tasks/`、`evidence/`、`handoffs/`、`decisions/` 下已有约 194 个 Markdown 文件。

当前主线已经多次纠偏：

- 不是任务包管理器。
- 当前主线是 Codex 会话管理和 Codex 工作流编排。
- 任务包保留为内部协议、审计、导出和交接物，不作为主界面中心。
- 会话开发方案保留，但优先把工作流跑起来。
- 多 agent、个人知识库、向量搜索、模型调度、复杂 UI 参考源都后置。

如果不清理，后续任何 agent 进入项目都会被旧任务、旧验证、旧方向误导。

## 薄弱点

- 不能凭文件名直接删。依据：很多旧任务包仍是历史依据、回收依据或安全边界证据。
- 不能只新增一份“当前说明”就结束。依据：旧任务包、旧 README、旧阶段计划仍会继续误导后续 agent。
- 不能把历史 evidence 全删。依据：部分 evidence 是路线变更、安全边界和能力验证的依据。
- 清理本身有破坏风险。依据：移动或删除任务包后，`tasks/README.md`、handoff 链接和决策引用可能断。

## 目标

把 `product-line` 打扫到“后续 agent 扫一眼能看懂”的状态：

1. 明确当前权威入口。
2. 明确当前主线。
3. 明确哪些能力已完成、哪些只是探针、哪些后置、哪些暂停、哪些废弃。
4. 旧文件能删就删，不能删就归档。
5. 所有保留下来的文件都有理由。
6. 所有归档文件不再污染当前入口。
7. 让 `README.md`、`STAGE_PLAN.md`、`tasks/README.md`、`PROTOTYPE_WORK_LINES.md` 与最新口径一致。

大白话目标：

让下一个 Codex、Claude、OpenCode 或其他 agent 一进来，不用翻 200 个文件，也能知道现在该做什么、不该做什么、哪些东西只是历史。

## 非目标

- 不改产品功能代码。
- 不改 Tauri / React / Rust 实现。
- 不运行 Codex CLI。
- 不写 `/Users/yoyi/.codex`。
- 不读取业务会话正文。
- 不运行 harness。
- 不做真实 UI 验证。
- 不重新评审所有历史技术正确性。
- 不把旧 evidence 全部压缩成一个文件后删除原件。

## 当前权威口径

必须落实到对应文档里：

- 当前阶段：Codex 会话管理 + Codex 工作流编排。
- 当前工作流顺序：
  1. 会话全文读取。
  2. 会话控制探针。
  3. 工作流状态流转。
  4. 工作流节点绑定会话。
  5. 工作流节点派发 Codex 指令。
  6. 执行结果读回。
  7. 总指导回收。
- 任务包能力：内部协议、审计、导出、交接，不是主界面中心。
- 会话线：保留并继续推进。
- 桌面应用线：负责把工作流能力接进工作台。
- 验证线：按需另派，不作为每个实现包的共同执行线。

## 明确后置

必须落实到对应文档里：

- 多 agent 接入。
- OpenClaw / OpenCode / Claude Code / VS Code 真接入。
- 个人知识库。
- 向量搜索和向量库选型。
- 模型调度。
- Skill 自动安装和仓库化。
- Harness 自动运行。
- 复杂画布编辑器。
- Codex++ 式删除、移动、归档、CDP 注入。
- AionUi / Multica / Langflow / Dify / n8n 等参考源的功能复刻。

## 明确不做

必须落实到对应文档里：

- 不把工作台做成任务包管理器。
- 不直接写 Codex 内部状态库。
- 不读取 `auth.json`、`.env`、密钥、授权文件。
- 不默认全量展开所有会话正文。
- 不把索引推断当成用户确认事实。
- 不把 safe probe 包装成真实业务自动执行。
- 不绕过用户确认写 `.codex`。

## 建议目录策略

新增归档目录：

```text
product-line/archive/
product-line/archive/tasks/
product-line/archive/evidence/
product-line/archive/handoffs/
product-line/archive/decisions/
product-line/archive/README.md
```

建议新增当前权威入口：

```text
product-line/CURRENT.md
```

`CURRENT.md` 只写当前事实，不写流水账：

- 当前目标。
- 当前技术栈。
- 当前权威文档。
- 当前进行中任务。
- 下一步建议。
- 暂停 / 后置 / 不做。
- 安全边界。

## 文件处理规则

### 保留在当前目录

保留条件：

- 当前仍是权威。
- 当前任务正在做或下一步要做。
- 被 `README.md`、`STAGE_PLAN.md`、`tasks/README.md` 直接引用。
- 是当前方向的关键决策。
- 是当前能力边界的关键证据。

### 归档

归档条件：

- 已完成但只是历史阶段。
- 已被新决策 supersede。
- 仍有依据价值，但不应出现在当前入口。
- 旧 UI、旧静态壳、旧验证线结果。
- 旧任务包管理器方向下的中间能力。

归档要求：

- 归档后在 `archive/README.md` 里记录原路径、新路径、归档原因。
- 如果当前文档引用了原路径，必须改成新路径或删除引用。

### 删除

删除条件：

- 明确是重复文件。
- 明确是临时 smoke 文件。
- 明确被更完整文件取代，且没有独立证据价值。
- 明确是错误生成、污染内容或已废弃的草稿。

删除要求：

- 删除前必须在清理报告列出理由。
- 不确定就不要删，先归档或列入待确认。

## 必须盘点的文件

至少盘点：

- `product-line/README.md`
- `product-line/STAGE_PLAN.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/principles.md`
- `product-line/backlog.md`
- `product-line/tasks/README.md`
- `product-line/decisions/*.md`
- `product-line/tasks/*.md`
- `product-line/evidence/*.md`
- `product-line/handoffs/*.md`
- `product-line/prototypes/**/README.md`

## 允许读取

允许读取：

- `product-line/**/*.md`
- `product-line/prototypes/**/README.md`
- `product-line/prototypes/productized-desktop-shell/package.json`
- `product-line/prototypes/productized-desktop-shell/src-tauri/tauri.conf.json`
- `product-line/prototypes/index-kernel/codex-index.json` 的统计和结构字段

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 业务会话正文

## 允许写入

允许写入：

- `product-line/CURRENT.md`
- `product-line/README.md`
- `product-line/STAGE_PLAN.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/principles.md`
- `product-line/backlog.md`
- `product-line/tasks/README.md`
- `product-line/archive/**`
- `product-line/tasks/**`
- `product-line/evidence/**`
- `product-line/handoffs/**`
- `product-line/decisions/**`
- `product-line/evidence/2026-05-29-product-line-cleanup-current-authority-v1.md`
- `product-line/handoffs/2026-05-29-product-line-cleanup-current-authority-v1-result.md`

禁止写入：

- `product-line/prototypes/**`，除非只是 README 中的路径指针修正。
- `/Users/yoyi/.codex`
- Codex 状态库。
- 工作台真实 workflow state。
- 项目业务目录。

## 建议执行步骤

1. 统计根目录、任务包、evidence、handoff、decision 数量。
2. 建立清理清单，给每个文件打标签：
   - current
   - keep-reference
   - archive
   - delete-candidate
   - needs-user-confirmation
3. 先更新当前权威入口：
   - `CURRENT.md`
   - `README.md`
   - `STAGE_PLAN.md`
   - `tasks/README.md`
   - `PROTOTYPE_WORK_LINES.md`
4. 再归档旧文件。
5. 最后删除明确无价值文件。
6. 检查链接是否断。
7. 写 evidence / handoff。

## 当前建议保留为权威的决策

至少保留：

- `decisions/2026-05-27-technical-stack-and-expansion-architecture.md`
- `decisions/2026-05-28-extensible-first-development-rule.md`
- `decisions/2026-05-28-workflow-state-storage-v0.md`
- `decisions/2026-05-28-codex-workflow-min-model.md`
- `decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `decisions/2026-05-29-codex-session-plan-retained-workflow-first.md`
- `decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `decisions/2026-05-29-ui-reference-sources.md`

可以考虑归档但不能直接删：

- 早期静态网页壳方向。
- 早期旧 UI 骨架方向。
- 早期任务包管理器方向的任务包。
- 已被新口径 supersede 的验证任务。

## 清理后必须形成的当前入口

执行后，后续 agent 的阅读顺序应变成：

1. `product-line/CURRENT.md`
2. `product-line/README.md`
3. `product-line/STAGE_PLAN.md`
4. `product-line/tasks/README.md`
5. 当前待执行任务包
6. 相关 decision

不应该要求后续 agent 先翻所有 evidence 和 handoff。

## 验收标准

必须满足：

- 有 `CURRENT.md`。
- `README.md` 能说明当前项目是什么、当前阶段做什么、下一步做什么。
- `STAGE_PLAN.md` 与当前阶段一致。
- `tasks/README.md` 不再停留在旧“下一步建议 Codex 工作流编排运行模型 v1”的过期状态。
- 当前待执行任务只保留最新任务。
- 旧任务包要么归档，要么标清历史。
- 旧 evidence / handoff 不再污染当前入口。
- 明确列出“后置”和“不做”。
- 明确任务包能力仍有用，但不是主界面方向。
- 链接检查没有明显断链。
- 没有读取或写入 `.codex`。

## 建议检查命令

```bash
find product-line -maxdepth 2 -type f -name '*.md' | sort
rg -n "任务包管理器|下一步建议|暂停|后置|supersede|当前权威|Codex 会话|工作流" product-line/*.md product-line/tasks/README.md product-line/decisions
rg -n "今天|昨天|刚刚|最近|上周|today|yesterday|recently" product-line/*.md product-line/tasks product-line/decisions product-line/evidence product-line/handoffs
```

说明：

- 相对时间不一定全删，历史记录里可以保留；当前权威文档里不能保留。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 本轮做了什么。
3. 改了哪些当前权威文件。
4. 新增了哪些 archive 目录或索引。
5. 归档了哪些文件，为什么。
6. 删除了哪些文件，为什么。
7. 哪些文件保留为当前权威。
8. 哪些文件列入待确认。
9. 新增 evidence / handoff。
10. 是否读取或写入 `.codex`，答案应为没有。
11. 后续 agent 应该从哪里开始读。

## 总指导回收重点

回收时重点看：

- 是否真的减少了后续 agent 的理解成本。
- 是否把旧方向从当前入口里移走。
- 是否没有误删关键 evidence。
- 是否把当前主线、后置、不做写清楚。
- 是否把任务包能力放回“内部协议 / 审计 / 导出”的位置。
- 是否能让新 agent 五分钟内知道下一步做什么。

