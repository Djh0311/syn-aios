# 中间版本权威口径固化记录

日期：2026-06-03

## 目标

在继续研究记忆层之前，先把用户已经确认的中间版本方案落实到当前权威入口，避免后续对话因为上下文漂移，把旧草案、SQLite 建表、候选 sidecar 或秘书只读模型误当成中间版本目标。

## 本轮改动

更新文件：

- `docs/middleware-version-development-plan-v1.md`
- `CURRENT.md`
- `tasks/README.md`

没有改产品代码，没有改 workflow state JSON，没有迁移数据库，没有写正式事实或正式记忆。

## 已固化的权威口径

`docs/middleware-version-development-plan-v1.md` 顶部状态已从“待修订，不可按原文直接执行”改为：

- 已确认中间版本权威口径。
- 原始阶段草案保留为历史素材。
- 执行必须按本文件第 0 节和后续任务包解释。

第 0 节继续作为当前解释权最高的中间版本口径：

- 中间版本必须完成自动化工作流和记忆层两个核心闭环。
- 自动化工作流采用方案授权制，不采用每一步确认制。
- 用户确认方案和最终结果，不盯每个 worker。
- 项目主管管理过程、派 worker、看汇报、确认项目内过程事实。
- 全局主管复核方案边界和最终结果，中途只在重大异常时介入。
- 秘书只整理、解释、提醒和收纳想法，不确认 worker 汇报，不判断过程事实，不直接派活，不直接写正式记忆。
- worker 汇报不是正式事实；项目主管确认后的过程事实才可以进入后续 worker 的任务包。
- 正式长期记忆必须满足来源、版本、权限、冲突和审计规则。

## 当前入口修正

`CURRENT.md` 已移除旧口径：

- 旧句子：`docs/middleware-version-development-plan-v1.md` 已标为待修订，不可按原文直接执行。

替换为：

- `docs/middleware-version-development-plan-v1.md` 已落实“确认后的中间版本权威口径”。
- 第 0 节是当前解释权最高的中间版本方案。
- 下方原始阶段草案只能作为历史素材。

`tasks/README.md` 已补当前并行准备两条线：

- `tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md` 已写好，可交给其他对话执行。
- 记忆层实现切片必须先复核 `docs/memory-layer-design-v1.md`、`docs/plans/2026-06-01-memory-governance-schema-v1.md` 和现有候选治理实现，再写 `docs/plans/memory-layer-implementation-slice-v1.md`。

## Adapter 任务包状态

已存在：

- `tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`

它的目标是把当前前端只读 adapter 能力声明收敛到后端 typed read model，输出结构化 `agent_adapters[]`，为 Claude Code / OpenClaw / OpenCode 后续接入预留接口。

本任务包明确不允许：

- 接 Claude Code / OpenClaw / OpenCode 真实实现。
- 执行 `codex exec` 或 `codex exec resume`。
- 改真实 Codex 执行语义。
- 读写 `/Users/yoyi/.codex`。
- 改 workflow state JSON 结构。
- 写正式事实或正式记忆。

## 记忆层状态

本轮没有开始写 `docs/plans/memory-layer-implementation-slice-v1.md`。

原因：用户明确要求先彻底研究记忆层设计文档，不相信当前上下文已经理解透彻。当前只做前置权威固化，防止研究时上下文漂移。

下一步研究必须至少复核：

- `docs/memory-layer-design-v1.md`
- `docs/plans/2026-06-01-memory-governance-schema-v1.md`
- `tasks/2026-06-03-final-skeleton-14-memory-governance-minimal-implementation-v1.md`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`

## 安全事故记录

在前序并行规划 / 验证 adapter 任务包的过程中，曾有一次 shell 命令用双引号包住带反引号的文本，例如 Markdown 里的 `codex exec`。zsh 将反引号内容当成命令替换，导致意外尝试执行 `codex exec`。

当时输出显示 Codex CLI 尝试读取 stdin，未收到 prompt，并尝试打开 `/Users/yoyi/.codex/state_5.sqlite`，随后因 readonly / 权限问题失败。

结论：

- 这是安全事故，不能被淡化为普通搜索失败。
- 后续搜索含反引号文本必须使用单引号或 `rg -F '...'`。
- 后续 evidence / handoff 如果涉及该任务包或安全边界，必须继续保留这条风险。

## 边界

本轮固化文档时：

- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未改产品代码。
- 未改 workflow state JSON。
- 未迁移数据库。
- 未写正式事实。
- 未写正式记忆。
- 未运行真实 Tauri 窗口验收。

## 验收方式

只读核对以下内容存在：

- `docs/middleware-version-development-plan-v1.md` 包含“已确认中间版本权威口径”。
- `CURRENT.md` 不再保留“中间版本方案待修订、不可按原文直接执行”的旧入口口径。
- `tasks/README.md` 写明 adapter 任务包已可交给其他对话，记忆层实现切片必须先研究再写。

