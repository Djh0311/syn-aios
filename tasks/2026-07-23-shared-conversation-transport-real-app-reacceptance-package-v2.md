# 任务包：共享 Conversation Transport 真实 App 三句重验 v2

- 日期：2026-07-23
- 状态：**HOLD / 知识 relay 前置已满足；等待用户新的真实运行授权与无并行 writer 窗口**
- 负责人：独立 Codex 对话线（`gpt-5.6-terra`，reasoning=`ultra`）
- 指导/验收：当前总指导对话
- 前置审计：`evidence/2026-07-23-shared-conversation-transport-parallel-restart-audit-v1.md`
- 工具面决策：`decisions/2026-07-23-supervisor-read-only-exact-five-capability-surface-v1.md`
- 取代未来执行入口：`tasks/2026-07-23-shared-conversation-transport-real-app-substitution-acceptance-package-v1.md`

## 对齐块

- `authority_chain`：`AGENTS.md` → `CURRENT.md` → `AUTHORITY.md` → 双线并行决策 → 五工具面决策 → 本任务包。
- `plan_anchor`：`docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md` 的共享对话底座真实替代性验收。
- `existing_before_new`：复用共享 transport、七阶段 binding 失败闭锁、精确 capability registry、Pending 卡与既有真实 App 停点；本包不改实现。
- `capabilities_touched`：只验收 `submit_proposal + knowledge_search/read/open/cite` 的可见性；三句中只允许实际调用一次 `submit_proposal`。
- `forbidden_alternatives`：旧 resident/private-home 主路线、现场修码、单工具 profile、放宽 read-only/空写根、重发、补卡、点卡、chain/worker。

## 0. Kickoff

- 任务：以一枚当前源码冻结 binary，在固定测试项目交办页执行一次严格三句真实 App 重验。
- 负责人：独立 `gpt-5.6-terra / ultra` 对话线。
- 交付物：新的脱敏真实 App evidence、原始截图/日志 manifest、最小 CURRENT/AUTHORITY 结果同步。
- 完成标准：同一新 thread 完成自然回复、durable Active binding、精确五工具发现、一次 `submit_proposal`、一张未批准 Pending 卡和第三句无重复动作；退出后进程/holder/registry/lock 清零。

本包当前仅冻结合同，**尚未授权执行**。

## 1. 开跑前两道硬前置

两项都满足前不得启动任何 App、构建或读取真实 store：

1. **已满足**：`tasks/2026-07-23-l3-knowledge-open-relay-secret-sink-closure-repair-package-v1.md` 已完成，并经指导线独立核 diff、secret/hash/cleanup 测试和 staged 状态后验收通过。真正开跑前仍须在无并行 writer 窗口重新冻结共享承重文件 hash。
2. **未满足**：用户对本 v2 包明确给出一次新的真实运行授权。历史 v1 的“现在就可以开”已经在上一轮使用，不自动续期。

此外，`tasks/2026-07-25-l3-syn-n2r-r1-single-shell-convergence-package-v1.md` 施工期间，本包只能做只读准备；不得构建或启动，以免冻结 binary 混入尚未验收的知识 UI WIP。

## 2. 精确工具面

首句首次成功 `tools/list` 的工具名称集合必须精确等于：

```text
submit_proposal
knowledge_search
knowledge_read
knowledge_open
knowledge_cite
```

比较集合，不以返回顺序作为失败条件。空集、子集、额外项、重复项、大小写/空白变体或身份无法对账均停止。

本三句只允许第二句调用一次 `submit_proposal`。`knowledge_search/read/open/cite` 调用数必须全部为 0；它们的真实功能由知识库 N6 十二项验收单独结算。

## 3. Gate 0：新鲜现场

1. 冻结 HEAD、staged、porcelain、共享承重源码 hash、固定测试项目 HEAD/porcelain/manifest。
2. 确认 scoped Workbench/Tauri/dev/Vite/Codex/MCP process、registry、lock、workflow state、DB/WAL/SHM holder 全空，registry entries=0；任一不满足即不启动。
3. 只读记录 SQLite integrity、DB/JSON 安全投影、storage mode，以及新的基线：
   - workflow revision/audit；
   - recorded/injected/reply/diagnostic；
   - supervisor session/audit/binding，按 lifecycle 分开；
   - proposal/Pending/decision/proposal audit；
   - chain/execution attempt/node dispatch；
   - registry revision/entries。
4. 不复用历史数字，不读取或记录用户正文、完整 identity、argv、grant、endpoint、stderr 或私有路径。

## 4. Gate 1：冻结 binary

- 只构建一枚当前源码 debug binary；记录命令、exit、SHA-256、size、mtime。
- 构建前后共享承重源码 hash 必须一致；若知识线或其他线程仍在写，停止。
- 只启动这一枚 binary。不得同时启动知识库真实验收。

## 5. Message-scoped 时序

同一脱敏 `turn_id/run_id` 必须实际关联：

`首句记录 → binding Starting 双端持久化 → thread.started 被宿主观察 → binding Active 双端持久化 → 首次 tools/list 五项精确集合 → 第二句 tools/call submit_proposal`

任何一步缺失、顺序无法关联或 DB/JSON 不一致，按最早可证事实停止；不得猜成某个内部子因。

特别关注：如果真实 client 在宿主观察 `thread.started` 并激活 binding 前发出首次 `tools/list`，导致空集合或错集合，本包判失败并停止，不现场修改时序。

## 6. 三句停止合同

### 第一句

只发送一次：

`我想给这个游戏里的标题改成小马里奥`

必须同时满足：

- 自然回复可见；
- `recorded +1`；
- 同一 run 的 binding 在 JSON/SQLite 中从本轮新增并最终为同一 `Active + thread_id`；
- `thread.started`、Active、首次 `tools/list` 可按第 5 节对账；
- 工具集合精确五项；
- proposal/Pending/chain/worker 均 `+0`。

任一不满足，立即停止，不发第二句。

### 第二句

仅第一句全绿后只发送一次：

`按这个出方案`

必须同时满足：

- 同一 thread，自然回复保留；
- 仅此句出现一次 `tools/call submit_proposal`，handler 与 outcome 成功；
- proposal/Pending 严格 `+1/+1`；
- chain/worker 严格 `+0/+0`；
- 唯一新卡匹配“小马里奥”并保持 `PendingUserConfirmation`。

任一不满足，立即停止，不发第三句；不重发、不补卡、不点卡。

### 第三句

仅第二句全绿且卡未触碰后只发送一次：

`先别执行，告诉我这个方案准备改哪些地方。`

必须同时满足：

- 同一 thread，自然回复可见；
- proposal/Pending/chain/worker 全部 `+0`；
- `submit_proposal` 和四项知识能力调用数都不再增长。

任一不满足立即停止。

## 7. 关闭与对账

- 成功候选最多 refresh 一次，所有本轮计数不得重复增长。
- 正常 Quit 后确认 scoped process、holder、registry、lock 均为 0。
- 只有零 holder 后才能把 SQLite 复制到临时目录做 integrity/query。
- 固定测试项目 manifest 必须不变。
- 若残留进程，不自行 kill；只有本包届时的现场授权明确覆盖且 PID/二进制身份精确时才能处理。

## 8. 写入白名单

本包执行时只允许：

- `evidence/2026-07-23-shared-conversation-transport-real-app-reacceptance-v2.md`
- `evidence/raw/2026-07-23-shared-conversation-transport-real-app-reacceptance-v2/`
- `CURRENT.md`
- `AUTHORITY.md`
- 本任务包的实际状态
- `docs/harness-catch-log.md`：仅出现新实际 catch 时追加

不得改代码、测试、配置、schema、依赖、固定测试项目或知识 vault；不得 stage/commit/push/reset/clean/stash。

## 9. 回交

逐项报告 Gate 0/1、三句每句前后计数、实际 message-scoped 时间线、完整工具名称集合、所有工具调用、Pending 卡、进程/store 清理、实际写入文件、staged 状态和未决问题。只有全部完成才可写“真实 App 替代性验收通过”。
