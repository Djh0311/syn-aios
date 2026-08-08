# M2C02 main 集成与完整验收报告

日期：2026-08-08

结论：PASS。M2 干净候选已进入 `main`，并在干净主线树上完成直接验收。该结论支持进入项目级 M2 收口；不代表 M3 激活、live Workbench 迁移、provider 接入或远端发布。

## 主线集成

- M2 实现提交：`d6bf4e464e32bd5310dfdfb2e46dfd0a47fd787f`
- Harness 生命周期提交：`3bbed9f7fbbc91b3792fd81a1cba45d29dd2cd80`
- Harness 主线验证记录提交：`b2f9b3ace49d8f076681a18bf570ddcfc17e5305`
- `main` 从 Stage 3 激活提交 `43578fd845f43e87154a04c0791bb25babea31e5` fast-forward 到候选提交，再记录验证门结果；未产生合并提交。
- 干净 R4 验收基线：HEAD `b2f9b3ace49d8f076681a18bf570ddcfc17e5305`，tree `dd237e2718a8e4fd9b2c2db5aa6da3eba167aeab`。

## 验证结果

- `cargo check --lib --quiet`：exit 0。
- M2 reference slice：11 passed / 0 failed。
- `worker_report`：31 passed / 0 failed。
- M2 SQLite schema：7 passed / 0 failed。
- M2 execution-report ingress：2 passed / 0 failed。
- 完整 Rust 库测：1385 passed / 0 failed / 45 ignored，exit 0；总计 1430。
- 前端 `npm run typecheck`：exit 0。
- Harness Lite `hl check quick`：1/1 PASS。
- Harness Lite task check：3/3 PASS（CLI syntax、Rust offline check、frontend typecheck）。
- Code Map 六个 domain JSON 的 source path 检查：PASS。
- `git diff --check`：PASS。

完整库测首次在受限沙箱中仅因 host PID `lstart` 读取返回 `Operation not permitted` 而失败；相同代码在主机权限环境复跑为全绿。R4 首次受限沙箱启动返回 spawn `EPERM`；主机权限环境复跑通过。这两项属于运行环境差异，不计作产品通过证据；本报告只采用主机权限环境的最终结果。

## 干净主线 R4 回执

- 回执：`/private/var/folders/nj/y6s1fvl936xgfwg20w08sk6r0000gn/T/syn-r4-acceptance-2wolW1/m2-reference-slice-suite-receipt.json`
- SHA-256：`fbd799a347934225f5e2eb652d286b690d8137c69c7baa55b4835fbebfc3ac13`
- `scenario_count=7`，七项均为 `PASS`：S1 冷启动与重放、S2 commit 前 SIGKILL、S3 commit 后 SIGKILL 与 DB-primary 投影恢复、S4 投影失败与重放、S5 重复命令、S6 JSON-leading 启动 fail-closed、DAT-004/008 同一 `update_work_item_state` effect/result/recovery。
- before/after 均为 HEAD `b2f9b3a...`、tree `dd237e2...`；worktree diff、index diff、untracked paths 均为空 SHA-256，`untracked_count=0`，`stable=true`。
- 七个绑定源文件的 SHA-256 在回执前后保持一致；`environment_unchanged=true`。

## 边界与保全

- 唯一记入 M2 完成积分的具名 persistence port 是 `workflow-state-sidecar.repository.m2.v1` 的 concrete SQLite implementation；泛型 `m2_*` 模块保持私有候选，不作为权威生产 port。
- R4 证据只覆盖 isolated scratch 的 bounded reference slice，不覆盖真实 Workbench 数据、DAT-007 live cutover、provider、真实账号或生产发布。
- grant-bearing report 仍停在 claim/readback 边界；review、decision 和 source-owner apply-result 仍是后续独立 owner command，不在 M2 内提前实现。
- M3 未激活；未 push、未部署、未发布。
- 混合开发工作树 `/Users/yoyi/workspace/product-line-syn-fnd-002` 保持只读：分支头、index、64 tracked + 14 untracked、13 项战略 WIP 指纹均未被本收口流程改写。
