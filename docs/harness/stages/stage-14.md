# 阶段14 M5 项目主管与执行闭环（事实重整与产品闭环）

总计划：product-line 唯一基线与 Harness Lite 切换

目标：按 `docs/plans/2026-08-16-syn-m5-m6-fact-reconciliation-and-product-closure-plan-v1.md` 完成 M5 项目主管与执行闭环的事实重整与产品闭环。先完成 REC-00 事实恢复门，再按前置矩阵判定 M5R00（仅 GAP 时），随后顺序完成 M5R01–M5R07，形成只含 M5 投影的 candidate commit series 后停止等待独立验收。本阶段不激活 M6/M7。

当前用户边界（2026-08-16 用户明确“按计划开始 M5”）：

- 以 5600X WSL `/home/synadmin/workspace/syn` 为权威仓库；不 reset、stash、clean、覆盖或丢失既有 WIP；
- 不接真实个人资料、真实用户项目写入、真实模型/provider、真实消息、账号、凭据、connector 或外部网络业务动作；产品层证据只用隔离 app-data、scratch projects、fake roles/provider/runtime 与白名单合成动作；
- 不 push、merge、rebase、部署、发布；M6 保持未激活。

干完的标准：

- REC-00 完成：R0 恢复载体冻结并校验、分层归责、真实 closed/active 控制状态、真实 stage-14 与唯一 REC-00 current leaf、前置载体、前置矩阵、`M5M6-REC00-fact-freeze.md` 与 `M5M6-REC00-provenance.json`、明确下一路由；
- M5R00 仅在前置矩阵出现 GAP 时执行并转 PASS；全部 PASS 时记 `NOT_NEEDED`；
- M5R01–M5R06 各自独立内容提交与定向证据，逐项进入 done；任一完成的实现不得冒充整阶段完成；
- M5R07 在 disposable checkout 形成绑定 candidate SHA 的原始 receipts 与候选报告，保持 `AWAITING_INDEPENDENT_ACCEPTANCE`、authorization 回 closed；
- 独立验收通过后 closeout：归档 M5R07 与 stage-14、同步 current-state / master / M5 计划 / 计划索引 / Harness plan、形成 M6 输入 handoff，单独 lifecycle commit；
- `git diff --check` 通过；M5/M6 写面零未知 delta；stage-12、D0C04/D0C05、M1–M4 冻结合同全程只读保全。

允许动：

- docs/harness/authorization.json
- docs/harness/plan.md
- docs/current-state.md
- docs/harness/stages/stage-14.md [新增]
- docs/harness/leaves/REC-00-m5-fact-freeze-git-and-harness-rebuild.md [新增]
- docs/harness/unfinished/REC-00-m5-fact-freeze-git-and-harness-rebuild.md [退场时新增]
- docs/harness/reports/M5M6-REC00-fact-freeze.md [新增]
- docs/harness/reports/M5M6-REC00-provenance.json [新增]
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn
- docs/plans/2026-08-01-syn-stage-5-project-supervisor-and-execution-loop-plan-v1.md
- docs/plans/2026-08-01-syn-master-development-plan-v1.md（如存在）
- prototypes/productized-desktop-shell/src-tauri/src/m5_*.rs、m6_*.rs、worker_report.rs、lib.rs（仅对应 M5R 包的 KEEP/REWRITE 写域）
- prototypes/productized-desktop-shell/src-tauri/Cargo.toml、Cargo.lock（仅 M5R 包所需最小依赖，M6 依赖禁止）

不许动：

- stage-12、D0C04、D0C05 与 unfinished/D0C04、D0C05（只读保全，不恢复、不关闭、不归入 M5/M6）
- M1–M4 冻结合同正文；如需补充只能新建不改旧 hash 的增补合同
- 真实资料/项目写入、真实模型/provider/message/connector、凭据、外部网络业务写、push/merge/rebase/deploy/release
- reset、stash、clean、覆盖或丢弃既有 WIP；Git add -A 吞入混合 WIP
- 伪造 Hook receipt、authorization、stage/leaf、测试或 App 证据
- M6、M7–M11、Headless Core、Primary/epoch 激活或实现
- 物理删除旧执行入口、旧 review、Agent Center 或 compatibility 数据

停止与回滚：

- R0 前后并发变化、共享字节无法归入 L1–L5/LX、secret/credential/真实运行数据/未知 symlink/special file 或受保护大文件、要求伪造证据、M1–M4 或 stage-12/D0C04/D0C05 意外修改、硬退出门未成立却要启动 M6、候选 commit 与新鲜证据 SHA 不一致时立即停止交总线。
- bootstrap transaction 任一步失败整体恢复 R0 preimage，不留下半套 current chain；authorization 保持精确 closed 两字段，不手填 executionReceipt/session/turn/expiresAt。
- WSL Hook 尚未 trusted/observed 时始终保持 closed，只执行当前用户明确回合；每次 leaf 切换先 closed 再按真实 receipt 重新签发，禁止旧 active JSON 跨 leaf 续用。

## 叶子

- [x] REC-00 事实恢复门：R0 恢复载体、分层归责、Harness 重建、前置矩阵与路由
- [x] M5R00 前置实现与 adapter 修正（NOT_NEEDED，前置矩阵无 GAP）
- [x] M5R01 执行合同矫正与旧数据映射
- [x] M5R02 持久编排核心与 ExecutionGrant
- [x] M5R03 WorkerReport、独立审查与事实提升
- [x] M5R04 普通项目的持久 Project Supervisor
- [x] M5R05 受控执行、恢复与 runtime conformance
- [x] M5R06 ProjectSummary 正式投影
- [ ] M5R07 项目 UI、隔离 App 与阶段候选（AWAITING_INDEPENDENT_ACCEPTANCE）
- [ ] M5 独立验收与 closeout
