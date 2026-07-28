# Task Package: {{TASK_ID}} — {{TASK_TITLE}}

authority-schema: harness-active/v3
authority-id: {{TASK_ID}}
authority-status: DRAFT
outcome: PENDING
mode: {{MODE}}
owner: {{OWNER}}
acceptance-owner: {{ACCEPTANCE_OWNER}}
accepted-by: PENDING
updated-at: {{UPDATED_AT}}
goal: {{GOAL}}
next-action: {{NEXT_ACTION}}
git-disposition: PENDING

## 负责哪块

- Observable outcome:
- Success criteria:
- 本任务只承接这一块，不复述父任务的产品背景、方案或完整计划。

## 边界（允许读写、禁止）

### 允许读写

- May write:
- Base / branch / worktree:
- Allowed staging scope:
- Local add / commit authority:

### 禁止

- Must not change:
- Push / merge / release authority:
- 不得把未声明的 Git carrier、验证结论或共享面改动写成既成事实。

## 交付什么

- Changed files:
- Git commits/disposition:
- Remaining issue / carryover owner:
- Next owner/action:

## 怎么验证

- Direct checks:
- Verification result:
- Proof level and limits:

## 遇到什么必须停

- Must stop and request fresh approval when scope, authority, or Git reality conflicts.
- 身份、base、branch、worktree、完整 commit OID、验证输出引用或写面无法唯一核对时停止。
- integrate、push、发布和物理清理仍需分别确认；事实记录不替代验收或退场。

上方 legacy reader header placeholders 仅为旧读取器兼容保留；它们不授予
写入、生命周期或退出权限，也不描述 Harness v2 的状态转换。

A proposal while `authority-status` is `DRAFT` is only a request description:
it grants no authority to write, to transition lifecycle, or to exit.

For the legacy header semantics: `complete` records an explicit Git disposition and exits the Harness context;
by itself it does not run Git, integrate, push, publish, or physically clean
resources.

Harness v2 的实际入口如下：

1. `task.js propose ... --verification <JSON数组|文件>` 只读取并组装冻结 proposal；
   每条验证合同必须先声明，不能携带已执行的 run。
2. `task.js start ... --write` 只在明确授权和完整 proposal digest 下创建开工现场；
   branch、worktree、task package 与 opening commit 都有各自的现实核对。
3. ACTIVE 期间，先执行 `task.js record ... --inspect` 取得 receipt，再以同一
   payload 的 `--write --receipt` 写入 canonical product/WIP carrier、验证 run 或
   disposition。事实记录不等于验收、集成、发布或退出。
4. 终态 closeout / exit 另有自己的 inspect、receipt、权限与 Git 现实门；不能由
   本模板或 record 代替。

`QUICK` is intentionally not a task-package mode. Pure question-and-answer work
does not create control documents; use `PLAN`, `GUIDANCE`, or `DEVELOPMENT`.

Ordinary diagnosis, revision, and retry stay in this package. Create a new
package only when the outcome, scope, authority, or risk boundary changes.

The header is bounded for routing. The body has no validity line or size cap;
over 32 KiB is a soft context-cost advisory only.

完整 commit OID、验证输出引用和 disposition 都只是可复核的事实字段；它们
本身不证明 merge、deployment、release 或 production acceptance。无法唯一核对时
应停在 inspect 结果，不应补猜或把后续文档提交当作前一版本的证据。
