# SYN M2 高危动作整体预授权 · v1

> **状态校正（2026-08-09）：EXPIRED AUTHORIZATION / 历史授权记录。** M2 已于 2026-08-08 关闭，本文按自身效力条款同时失效；不得用于 M3、整体文档清理或任何后续工程动作。当前权限只看当前用户指令与当前轻量开发护栏授权。

日期：2026-08-03
拍板人：用户（原话："高危动作不要一事一批，现在工作台根本没真实用过，随便改"）
记录人：总指导线
效力范围：M2 阶段全程，自本拍板起生效；M2 关闭或用户撤回时失效。

## 拍板

M2 阶段计划 §8 列的独立授权项**整体预授权**，执行时不再逐项请示：

- 生产 schema migration（additive 为先，destructive 也放——数据可弃）
- 只读 live-manifest preflight、真实 store shadow / parity
- 每个 domain 的主读 / 主写切换、DB/JSON reconcile
- 真实 App 强退 / 崩溃注入验收
- fake / 真实 adapter 接入（仍不接真实外部 provider，见下）
- 旧写路关闭
- 工作台**自有数据**（真实 HOME `Application Support/CodexGovernanceWorkbench/**`、
  各 worktree 内 store/fixture）的任意修改与删除

理由（用户陈述）：工作台从未投入使用，其自有数据无业务价值、可丢弃。

## 兜底纪律（工程谨慎，不是审批仪式）

- 涉及真实 HOME 工作台自有 store 的迁移/切换/删除前，**先留一次完整副本**
  （`cp -R` 到仓外 temp），副本位置写进任务 evidence。
- 每次提交仍按 AGENTS.md 走：commit 问一次、`catch:` 标记、CURRENT.md 回写。

## 不覆盖（仍是硬线，仍需逐次明确授权）

1. `git push` / 对外发布 / 合并到共享分支（本地 merge 到 integration main 已含在本授权内）
2. 写 `/Users/yoyi/.codex` 或读取其凭据（auth/token/secret）
3. 让 codex 在**工作台自身以外的真实项目目录**真执行（固定测试项目
   `/Users/yoyi/codex-workflow-mario-test` 按 2026-06-22/23 两项拍板仍为轻档）
4. 删除工作台自有数据**以外**的不可恢复数据
5. 接真实外部 provider / 凭据（邮件、日历、OpenConnector）——M2 计划本身也不做

## 备注

本拍板不改变 M2 计划的阶段边界（不接外部 provider、不切真实业务真源——因为不存在
真实业务），只是把"每件都要问一次"换成"一次问完"。出任何事按 mistake-ledger /
catch-log 既有规矩记账。
