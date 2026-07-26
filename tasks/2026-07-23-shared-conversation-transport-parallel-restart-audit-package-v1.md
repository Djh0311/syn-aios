# 任务包：共享 Conversation Transport 并行恢复只读审计 v1

- 日期：2026-07-23
- 状态：**DONE / 指导线验收通过**
- 负责人：独立 Codex 对话线（`gpt-5.6-terra`，reasoning=`ultra`）
- 指导/验收：当前总指导对话
- 并行决策：`decisions/2026-07-23-knowledge-and-conversation-parallel-workstreams-v1.md`

## 对齐块

- `authority_chain`：`AGENTS.md` → `CURRENT.md` → `AUTHORITY.md` → 双线并行决策 → 本任务包。
- `plan_anchor`：`docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md` 的对话底座与真实替代性验收。
- `existing_before_new`：复用已完成的共享 transport、主管只读 profile、turn binding、七阶段失败语义与既有真实 App 停点；不另造 transport。
- `capabilities_touched`：本轮只读核对 conversation transport/binding/MCP 时序和真实验收合同，不改能力。
- `forbidden_alternatives`：旧 resident/private-home 路线、边审边修、启动真实 App、读取真实 store、与知识库线并写共享承重文件。

## Kickoff

- 任务：恢复对话底座方向的准确当前状态，形成下一包可直接执行的真实 App 重验合同。
- 负责人：独立 `gpt-5.6-terra / ultra` 对话线。
- 交付物：`evidence/2026-07-23-shared-conversation-transport-parallel-restart-audit-v1.md`。
- 完成标准：明确区分已完成离线能力、真实 App 已证失败、仍缺的最早 message-scoped 事实、与知识库线的共享写面/运行资源，以及下一包的三句停止合同和精确计数。

实际交付：`evidence/2026-07-23-shared-conversation-transport-parallel-restart-audit-v1.md`。指导线已核对正文、边界和静态依据；审计通过，但不等于真实 App 通过。

## 只读范围

必须阅读：

- `tasks/2026-07-23-shared-conversation-transport-real-app-substitution-acceptance-package-v1.md`
- `evidence/2026-07-23-shared-conversation-transport-real-app-substitution-acceptance-v1.md`
- `tasks/2026-07-23-shared-supervisor-conversation-binding-establishment-repair-package-v1.md`
- `evidence/2026-07-23-shared-supervisor-conversation-binding-establishment-offline-verification-v1.md`
- `tasks/2026-07-23-shared-supervisor-conversation-binding-phase-semantics-and-failure-closure-rework-package-v1.md`
- `evidence/2026-07-23-shared-supervisor-conversation-binding-phase-semantics-and-failure-closure-rework-verification-v1.md`
- 当前相关源码与测试。

只允许新增上述 evidence。不得改代码、测试、CURRENT、AUTHORITY、既有任务/evidence 或 catch-log。

## 必须回答

1. 当前哪些结论是源码事实、离线测试事实、真实 App 事实或仍属推断。
2. 上次真实 App 首句停在哪里；七阶段修复解决了什么，又没有证明什么。
3. 下一次重验必须捕获哪些 message-scoped 事实，尤其是 `thread.started`、binding Active、`tools/list`/`tools/call` 的顺序。
4. 三句验收每句的输入、预期、停止条件、canonical/binding/tool/card/chain-worker 前后计数。
5. 与知识库线重叠的文件、Rust build lock、真实 store、进程和运行时资源。
6. 哪些工作现在可安全并行；哪些必须等待共享 relay 安全返工稳定。
7. 下一包的精确写面、真实运行授权和回交格式建议。

## 禁止

- 不运行 Cargo/npm/shape 等会抢构建资源或写输出的命令。
- 不启动 Syn、Codex CLI/MCP、Obsidian 或真实 App。
- 不读取、复制或修改真实 store/vault。
- 不修改任何产品代码或现有文档。
- 不 stage、commit、push、reset、clean、stash。

若只读时发现承重文件正在变化，记录审计时点和 hash，把结论标成 snapshot-scoped；不得等待或干预知识库线程。
