# REC-00 M5/M6 事实恢复门：事实、Git 与 Harness 重建

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

目标：在不 reset、stash、clean、覆盖或丢失既有 WIP 的前提下，建立可恢复、可归责、可逐层提交的 WSL 事实基线；恢复 Harness Lite 0.8 的真实 fail-closed 控制状态，从当前时刻建立合法的 stage/leaf 生命周期；产出前置载体与前置矩阵并明确下一路由。本叶只冻结和判定，不修产品代码。

来源收据：用户 2026-08-16 明确“链接 5600X WSL，找到 syn 项目，按计划开始 M5”；执行 `docs/plans/2026-08-16-syn-m5-m6-fact-reconciliation-and-product-closure-plan-v1.md` REC-00 包。

产品：无（REC-00 只冻结与判定）

证据：docs/harness/reports/M5M6-REC00-fact-freeze.md、docs/harness/reports/M5M6-REC00-provenance.json、r0 恢复载体（/home/synadmin/workspace/.m5-rec00-work）

载体：working-copy-only + 独立归档载体，无产品 commit

允许动：

- docs/harness/authorization.json
- docs/harness/plan.md
- docs/current-state.md
- docs/harness/stages/stage-14.md [新增]
- docs/harness/leaves/REC-00-m5-fact-freeze-git-and-harness-rebuild.md [新增]
- docs/harness/reports/M5M6-REC00-fact-freeze.md [新增]
- docs/harness/reports/M5M6-REC00-provenance.json [新增]
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn
- /home/synadmin/workspace/.m5-rec00-work [新增，独立临时工作根]

不许动：

- stage-12、D0C04、D0C05 及 unfinished/D0C04、D0C05（只读保全，不移动、不改写、不恢复）
- prototypes/productized-desktop-shell 下全部产品源码、测试、脚本、Cargo.toml/lock 与前端源码（M5/M6 候选 WIP 只冻结与裁决，不施工）
- M1–M4 冻结合同正文与 docs/contracts
- 真实资料/项目写入、真实模型/provider/message/connector、凭据、外部网络业务写
- 权威 worktree 上的 git add/commit/push/merge/rebase/reset/stash/clean
- 读取、复制或输出 secret/credential/私钥/口令/token 内容
- .git、target、node_modules、.env、活动 DB、旧 rollback carrier、symlink、special file 的读写（只记录元数据）
- 伪造 Hook receipt、authorization、stage/leaf、测试或 App 证据
- M5 产品代码施工、M6/M7–M11 激活、Headless Core/Primary/epoch、部署或发布

## 执行记录

- R0a 元数据预检完成（2026-08-16，branch=main、HEAD=9103c3b26b060e854be119a8cedaa856a2a900ce、index 相对 HEAD 无 staged delta、tracked 3581 / untracked 54 / ignored 7458、HOLD 项仅 .git/target/rollback carrier，无 secret/DB/node_modules/symlink/special file）；
- R0b 受控归档完成并校验（3609 项常规文件、210MB、tar + SHA-256 manifest + 独立目录全量恢复校验一致）；
- bootstrap transaction：authorization 恢复精确 closed 两字段，plan.md/current-state 修正，创建真实 stage-14 与唯一 REC-00 current leaf（closed auth）；
- 分层归责 provenance 完成（`docs/harness/reports/M5M6-REC00-provenance.json`，L2=76/L3=14/L4=6/L5=6/LX=15）；
- 前置矩阵完成：M1/M2/M3 全部 PASS，无前置 GAP → M5R00=NOT_NEEDED；
- fact-freeze 报告完成（`docs/harness/reports/M5M6-REC00-fact-freeze.md`），路由 M5R01；
- 待办（REC-00 未完项）：L1/L2 reviewed local carrier commits 须在 disposable worktree 按来源形成并进入 M5 candidate ancestry（plan §3.4），之后 REC-00 才整体退场。
