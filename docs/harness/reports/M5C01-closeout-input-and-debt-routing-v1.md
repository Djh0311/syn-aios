# M5C01 closeout 输入与欠账路由 v1

日期：2026-08-18

状态：`CLOSEOUT_CONTENT_CANDIDATE / STAGE_14_STILL_OPEN / M6_NOT_ACTIVE`

## Harness

- `M5R09-20260818-1836.verdict.md` 已独立放行内容候选 `c91d8fc72bcbf80186736caff841cb7a9b0660d1` / tree `fe2d982267d474631ca4ea7b3f90ed846f72a89d`、记账 `8e6f59f48d2d90891d3c02396378921e4a2f5d6e` / tree `2043660c9547c6c102ae24414674918ca8215eb0` 与 M5R09 自身生命周期收口。
- M5R09 已归档；当前唯一 leaf 是 M5C01 closeout-only。它不改产品源码，不开第四个加固叶。stage-14 在最终 lifecycle 记账前仍保持 open；stage-15/M6/F2/F3/F5 仍未激活。
- M5R00–M5R09 的 scoped PASS 与证据边界不重写。M5 最终产品内容锚仍是 `c91d8fc`；M5C01 只生成 closeout 文档候选和后续 lifecycle 记账。

## 产品与证据上限

- M5 已接受范围覆盖持久 Project Supervisor、Proposal → AuthorizationDecision → Authorization → Run/WorkItem + worker RoleSession binding → PreparedAttempt → attempt-scoped Grant → Dispatch → runtime → RuntimeReceipt/ExecutedReport → independent Review → ResultUserDecision、持久恢复/duplicate-effect、ProjectSummary/QueryPort、普通 Tauri command graph 和 M1 enrollment 生产入口。
- M5R09 独立验收在 detached Linux x86_64 WSL checkout 复跑：`cargo check` 0；`m5r09_` 23/23；memory/mature 各 14/14；ordinary identity source 4/4；完整 `m5_` 188/188；前端 typecheck/default build 0；默认 bundle 无 M5R07 acceptance marker，且包含 enrollment command；候选与记账 `git diff --check` 0。
- 证据不包括真实个人资料/项目、真实 provider/账号/凭据、外部业务写、发布、macOS/BSD 实机、真窗口像素或新壳运行。旧 Tauri 只承担普通非测试客户端的组合证明，不是长期壳视觉验收。

## 8 项欠账的 18:40 路由

| verdict 欠账 | closeout 判定 | 载体 |
|---:|---|---|
| 1. 验收期间新增 OSS 门面载体 | 用户本人已于 `c1025ba81b6c7885a16529b8f66c919655db48e4` 精确 7 路径独立提交；不属 M5R09/M5C01，不改、不吸收、不阻塞 | 既有 `OSS-01-public-push-and-codex-oss-application.md` 保持 unfinished；closeout WIP 报告单列用户载体 |
| 2. canonical ProjectId 尚未扩到 workflow/执行链 | M6 前置，但当前 M5 ordinary product 不因此整体不可用；不返修、不阻塞 closeout | `unfinished/M6P00-canonical-project-id-consumption-and-relation-owner-typing.md` |
| 3. `UNENROLLED` 首启缺主动提示 | 已有人工登记入口但引导不足；按判不准即记录处理 | `unfinished/F3-m1-unenrolled-guidance-and-status-projection.md` |
| 4. relation source foreign project owner 不可判别 | 需要 source kind 类型化，属后续 M6 域层前置；不在 closeout 改治理代码 | `unfinished/M6P00-canonical-project-id-consumption-and-relation-owner-typing.md` |
| 5. 两个测试 helper 可能 `0 == 0` 空转 | 断言收紧；非产品可用性阻断 | `unfinished/ENG-01-post-m5-nonblocking-hardening-and-worktree-hygiene.md` |
| 6. path-derived `validate_preview_input` 死代码 | 后续删除/并入 canonical 校验；本轮不改产品源码 | 同上 |
| 7. 883 条 warning 需分类 | 工程债分类；不把 warning 冒充运行失败 | 同上 |
| 8. 13 个历史 worktree 注册项 | 需逐个 owner/占用确认后处理；不在 closeout prune | 同上 |

结论：按用户 18:40 优先纪律，没有一项被提升为新的 closeout 前产品修复 current leaf；全部被精确归入用户载体或 unfinished，且不丢失后续责任。

## M6 输入

- ProjectSummary 权威合同：`docs/contracts/m5-project-summary-projection-v1.md`；实现入口：`m5_project_summary.rs::{ProjectSummary, SummaryConsumer, ProjectSummaryQueryPort, PersistentProjectSummaryPort}`。必须保留 version、watermark、source refs、summary hash、consumer RoleSession/scope/expiry/policy gate、stale/foreign 拒绝与只读不可反写。
- execution identity envelope 权威合同：`docs/contracts/m5-execution-identity-and-worker-report-v1.md`、`m5-persistent-orchestration-and-execution-grant-v1.md`、`m5-controlled-execution-and-runtime-conformance-v1.md`。M6 TemporaryAgent/Advisory 只能引用完整 `project_id + orchestration_id + workflow_run_id + work_item_id + node_id + dispatch_id + attempt_id + grant_id + worker_role_session_id + authoritative receipt + trusted actor + hashes`，不得从 report 自报或缺字段拼接。
- compatibility/rollback：`m5_runner_entry_registry` 已按 `new-grant / guarded-legacy / blocked` 分类入口，旧路不物理删除；回切只能保留权限、Grant、receipt/audit 和 quarantine，不得把 guarded legacy 升格或重放 effect。M6 不得采纳当前 6 个未跟踪 `m6_*.rs` 为基线。
- 未完成前置：M6 域层施工前先处理 `M6P00` 的 canonical ProjectId 消费扩面与 relation owner 类型化；该记录不激活 stage-15，也不授予产品写入。

## 载体

- M5 产品内容锚：`c91d8fc72bcbf80186736caff841cb7a9b0660d1` / tree `fe2d982267d474631ca4ea7b3f90ed846f72a89d`。
- M5R09 接受记账：`8e6f59f48d2d90891d3c02396378921e4a2f5d6e` / tree `2043660c9547c6c102ae24414674918ca8215eb0`。
- 用户 OSS 门面：`c1025ba81b6c7885a16529b8f66c919655db48e4` / tree `f60a315ff743ebb24eea192378c388ea277bda75`，精确 7 路径，独立于本叶候选。
- M5R09 → M5C01 lifecycle opening：`b2429f6`；只归档 M5R09、建立 closeout current 并记账，不是 stage close。
- M5C01 内容候选与最终 lifecycle SHA/tree 将由精确提交和节点请求绑定；本文件不预填未来值。
