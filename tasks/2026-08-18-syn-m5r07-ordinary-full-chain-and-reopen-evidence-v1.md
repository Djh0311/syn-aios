# SYN-M5R07 ordinary 完整闭环与重启证据窄包 v1

日期：2026-08-18
阶段：stage-14 / current leaf M5R07
基线：`bf64a8e3e02982c1403567294d4ff76680b08e2e` 加当前未提交 D1/D2 successor overlay
写者：Grok `grok-4.6 --reasoning-effort high`
状态：IMPLEMENTATION TASK / NOT ACCEPTANCE / NOT CLOSEOUT

## 目标

只补当前 M5R07 ordinary disposable positive Tauri 验收的三个直接证据缺口：

1. 让现成旧壳这个真实非测试客户端在成功 runtime 后继续走 `ExecutedReport -> independent Review -> ResultUserDecision`；
2. 后端 receipt 输出可回到持久权威表的精确对象引用与 exact join，不只输出布尔 present；
3. 第二真实进程证明 canonical ProjectId、主管 binding/RoleSession、M3 worker/reviewer RoleSession、闭环状态和 M1 registry revision/bytes 全部保持，且来源重放不重复登记。

本包不改产品业务语义、DTO、页面布局、execution kernel 或 M1 authority；只扩充已存在的 ordinary acceptance driver/receipt/runner。

## 写域

只许修改：

- `prototypes/productized-desktop-shell/src/main.tsx`（仅 ordinary acceptance driver；shared-isolated driver 不动）
- `prototypes/productized-desktop-shell/src-tauri/src/m5_ordinary_control_acceptance.rs`（仅 ordinary backend receipt/read-only acceptance assertions）
- `prototypes/productized-desktop-shell/scripts/run-m5-ordinary-control-acceptance.mjs`（仅 ordinary launcher、receipt 聚合和断言）

不许改其他文件，不许 stage/commit，不许 reset/stash/clean，不许覆盖或归责现有混合 WIP。

## A. 真实旧壳客户端完整链

- 保留 ordinary 现有 `open -> reject -> approve -> seed FAILED no-effect -> retry -> runtime -> duplicate runtime` 路径；拒绝仍须零 Grant、零 durable operation，retry 必须形成新 Attempt/Grant/Dispatch lineage，重复 runtime 必须零第二 effect。
- 在首次成功 runtime 与 duplicate-runtime 断言之后，ordinary driver 必须依次点击已有按钮并等待已有 UI log：
  - `[data-m5-action="report"]`
  - `[data-m5-action="review"]`
  - `[data-m5-action="result"]`
- 每一步调用 `m5r07OrdinaryWrite` 写独立 DOM + backend receipt，phase 分别为 `report`、`review`、`result`。不得借 shared-isolated driver 的 receipt 冒充 ordinary positive。
- 不新增/重画 UI，不绕过已有 Tauri command，不从 renderer 伪造任何 authority ID。

## B. 后端精确引用与 exact join receipt

扩充 ordinary backend receipt（可升级 schema 到明确的新版本），所有权威字段都必须由 SQLite/M3 authority 只读加载，不接受 DOM 输入。至少输出：

- 主管：`project_id`、`binding_id`、supervisor actor/role-session；
- M3：worker actor/role-session、independent reviewer actor/role-session，且 reviewer actor/session 与 worker 均不同；
- 正式闭环精确对象：Proposal、AuthorizationDecision、Authorization、WorkflowRun、WorkItem、PreparedAttempt、Grant、Dispatch、RuntimeReceipt、ExecutionAttemptReadback、Executed claim/report、Review、ResultUserDecision、ProjectFact 的 ID；
- 必要 join carrier：project/orchestration/workflow/work-item/node/attempt/grant/dispatch/worker-session/receipt/claim/review/result/fact 能从持久表机械核对为同一条链；terminal readback 与 EXECUTED claim 已存在，review outcome 为 `VERIFIED`，result decision 为 `ACCEPTED_RESULT`，fact 绑定同一 claim/result/project；
- 计数：至少涵盖 grants、attempts、dispatches、durable operations、execution readbacks、claims、reviews、result decisions、project facts；
- receipt 需显式给出 `exact_chain_complete` 与 `independent_reviewer`，只能由上述后端核对结果计算，不得由 phase 名或 DOM 推断。

查询必须按当前 project 和当前 formal progress 精确定位；不能用数据库中任意最新一行拼接。若链不完整，字段可为空且布尔为 false，但不得吞掉数据库/schema 错误后伪造完整。

## C. 第二进程持久化与 M1 重放

- Node runner 在首次进程启动后读取 `${APP_DATA_RELATIVE_PATH}/m1/project-index-v1.json`，记录 SHA-256 与 `registry_revision`；第二进程完成 reopen receipt 后再次读取同一文件。
- 最终 launcher receipt 显式输出并断言：
  - `same_project`、`same_binding`、`same_supervisor_role_session`；
  - `same_worker_role_session`、`same_reviewer_role_session`；
  - 第二进程仍可读到完整 exact chain，权威闭环计数与精确对象 ID 不变；
  - M1 registry 第一次/第二次 revision 相同且 bytes SHA-256 相同，canonical ProjectId 不变；
  - `no_second_effect` 仍成立。
- runner 必须等待并收集 `report`、`review`、`result` backend receipts；最终 PASS 必须要求 `result.exact_chain_complete === true`、`result.independent_reviewer === true` 以及 reopen 的相同事实。
- 保持 truthful 边界：`ORDINARY_TAURI_CONSTRUCTOR_SYNTHETIC_ISOLATED_INPUTS`、`SYNTHETIC_INPUTS`、`NO_REAL_USER_DATA`、`NOT_DEPLOYED`、`NO_WINDOW_CAPTURE`、非日常运行、非发布、非 stage closeout。

## D. 直接测试与自检

- 在 `m5_ordinary_control_acceptance.rs` 增加/调整直接测试，至少证明 backend receipt 的完整链精确引用来自 store，reviewer 与 worker 独立；不完整链不会得到 `exact_chain_complete=true`。
- 不降低现有 D1/D2、ordinary 4/4、task-memory、M5 backend 测试标准。
- 完成后运行（仓外 target）：

```bash
cd /home/synadmin/workspace/syn/prototypes/productized-desktop-shell/src-tauri
CARGO_TARGET_DIR=/tmp/syn-m5r07-full-chain-target cargo test --lib m5_ordinary_control --offline
CARGO_TARGET_DIR=/tmp/syn-m5r07-full-chain-target cargo check --lib --offline
cd /home/synadmin/workspace/syn/prototypes/productized-desktop-shell
npm run typecheck
npm run build
```

若本机 Xvfb/Vite 可用，再运行 `node scripts/run-m5-ordinary-control-acceptance.mjs`；它必须自行启动真实普通 Tauri binary，且最终 exit 0。所有结果如实报告，不能用 `0 tests` 或旧 receipt 冒充。

## 必须保全

- 不反写 U01a/U01b/U01c/U02 scoped PASS；本包是 successor evidence correction。
- 不改 `worker_report.rs`、M1-M4 冻结合同、M1 实现、M5 product command 语义、shared-isolated、M6、stage-12、D0C04/D0C05。
- 不接真实资料、凭据、provider、账号或外部网络业务写；不 push/merge/rebase/deploy/release。
- 不 close M5R07/stage-14；完成后仍只是工作树候选，等待主管形成不可变 candidate 与新鲜 evidence-binding。
