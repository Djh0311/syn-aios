# 验证任务包：S1-③ 测试项目真跑端到端「验合并闸 runtime 真生效」· 主导线 → 执行线 v1

日期：2026-06-24

出自：主导线（Claude）。性质：**真跑 codex 进固定测试项目 = 高危#1 轻档**（path-lock 锁死 + 沙箱守住）；同时是 **S1（高危#3 改闸）的 runtime 端到端验证**——第一次真跑过那道新合并的强闸。上游：S1 实现包已审过入库（commit `d0e1e03`）；实现/回交见 `tasks/2026-06-24-s1-execution-layer-gate-merge-v1.md` §6 + `handoffs/2026-06-24-s1-execution-layer-gate-merge-handoff-v1.md`。

## 0. 接手须知

- 你是**执行线**。本包**纯验证、不改任何实现代码**（S1 实现已入库、审过）。流水线：**你真跑取证据 → 回交 → 主导线核证据**。子线不 `git add` / `git commit`。
- 真跑只打**固定测试项目** `/Users/yoyi/codex-workflow-mario-test`（轻档·git 可回滚）。
- 先读：S1 实现（`commands.rs` 的 S1 闸 block `:2175` 起 + 3 helper `:2249` 起）+ 上面两份 S1 文档 + `CURRENT.md` 首条 + `AGENTS.md`（高危#1/#3）。
- **关键安全（本包死线）**：见 §3。一句话：**只在测试项目真跑；非测试拦截用例只验"被拦、没起 codex"，绝不真跑进非测试项目；不改任何闸/实现代码。**

## 1. 要做的事

在测试项目真跑**一个画布工作流节点**（经 `execute_project_workflow_node` 产品路径，不是单测桩），取 **4 条 runtime 证据**，证明 S1 合并闸在真实执行里成立：① 真走了 A 强闸、② path-lock 命中才放行、③ 非测试 root 被运行时拦截（铁律 runtime 真生效）、④ 沙箱只动测试目录。

## 2. 怎么跑（步骤）

**准备**：测试项目里备好一个画布工作流 + 一个 worker 节点 + 绑一条真 codex 会话（可复用此前 chain 真机验 / mario-test 既有 fixture；缺就按现有产品流程建：bootstrap 工作流 → 建节点 → 绑会话）。节点 prompt 让 codex 写一个证明文件（如 `s1-step3-proof.txt`，内容含本次时间戳）。

**跑①·测试项目真跑（正向）**：对该节点走 `execute_project_workflow_node`（project_root = 测试项目）。期望：判决 `authorized_for_real_runner` → 真起 codex → codex 建出 proof 文件（内容对）→ dispatch `state=completed`、exit 0。**取证**：proof 文件内容 + dispatch 记录 + runtime/audit 里能看到走了 S1 闸（authorized）。

**跑②·非测试拦截（关键·只验被拦，不真跑）**：把同一请求的 `project_root` 改成一个**非测试路径**（如 `/tmp/s1-step3-nontest` 或当前仓 `/Users/yoyi/workspace/product-line`）再调 `execute_project_workflow_node`。期望：返回 `real_execution_gate_blocked:...`（path-lock 不命中 → `authorization_complete=false` → 判决拦截）、**不起任何 codex 进程**、非测试路径下**无任何写入**。**取证**：返回的 blocked 错误串 + 证明没有 codex 子进程被拉起（ps / 无新进程）+ 非测试路径无 proof/无改动。**⚠️ 这一步是验"它会拦"，绝不是真往非测试项目跑。**

**跑③·沙箱隔离**：①的真跑后，确认 codex 写入**只在测试目录**：`s1-step3-proof.txt` 在测试项目内；`$HOME`、`~/Documents`、`~/Desktop`、`/tmp`、`/Users/yoyi/.codex` **无任何本次新增/外溢**。**取证**：测试目录内有、外面 grep 无。

## 3. 安全硬约束（本包死线）

- **只真跑固定测试项目**；①③ 的真跑 project_root 恒 = `/Users/yoyi/codex-workflow-mario-test`。
- **②非测试用例只验"被拦"**：必须返回 `real_execution_gate_blocked` 且**没起 codex**；**绝不真跑 codex 进非测试项目**（真跑进非测试 = 高危#1 违规）。
- **不改任何代码**：本包是验证，S1 实现/闸/沙箱一字不动；要改 = 停、回主导线。
- **不开连环 / 不放开授权**：单节点真跑，不起链、不碰自动连环。
- **沙箱不外溢**：发现写到测试目录外（尤其 `.codex` / home）→ **立即停、回主导线**（说明 S1 或沙箱有洞）。
- **碰线就停**：要改代码 / 要真跑非测试 / 沙箱外溢 / 要读 `.codex` 凭据 → 停、回主导线。

## 4. 验收门（要交的证据）

- **①正向**：proof 文件路径+内容（在测试目录）、dispatch `state=completed`/exit0、runtime 或 audit 显示判决 authorized（走了 S1 闸）。
- **②非测试拦截**：blocked 错误串原文、**无 codex 进程被拉起**的证明、非测试路径零写入。
- **③沙箱隔离**：proof 只在测试目录；`$HOME`/`~/Documents`/`~/Desktop`/`/tmp`/`.codex` 本次无外溢（给 grep/ls 证据）。
- **回归**：`cargo test --lib`（应仍 580/0，本包不该改动测试数）。

## 5. 本包不做

- 不改 S1 实现 / 闸 / 沙箱任何代码。
- **不真跑 codex 进非测试项目**（只验它被拦）。
- 不起自动连环 / 不放开授权 / 不接 S2。
- 不删旧 5 处 path-lock（纵深防御，留着）。

## 6. 回交

- 交：§4 四类证据 + 一句话结论（4 条是否全成立）→ 主导线核证据。**子线不 commit。**
- ③ 全过 → 主导线回写 CURRENT「S1 整体完成（实现审过 + 测试项目真跑验过）」+ 收口；然后接 S2。

## 7. 不接受为

- 不接受为：①没真跑过 codex / ②非测试拦截没真触发（或更糟：真往非测试跑了）/ ③沙箱外溢到测试目录外 / 改了任何实现代码 / 读了 `.codex` 凭据 / 起了自动连环。
- 不接受为 S2 任何内容被提前做。
