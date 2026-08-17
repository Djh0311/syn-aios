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

## 2026-08-18 用户修订（交接：`handoffs/2026-08-18-syn-new-bus-director-m1-prerequisite-and-reduced-m5r07-standard-v1.md`）

用户 2026-08-18 明确三点，覆盖本阶段与之冲突的既有口径，其余不变：

1. **M1 纳入 M5 验收前置。** 复核发现 REC-00 前置矩阵的 M1 项判定错误：非测试代码中登记 M1 精确别名的只有 env 门控的验收 fixture 与 `#[cfg(test)]` 调用，普通启动路径无真实项目登记入口，老项目身份仍由 `lib.rs` 的 `project_id()` 从路径字符串派生。按 M5/M6 事实重整计划，矩阵出现 GAP 必须走 M5R00，因此 **M5R00 按真实缺口重开为唯一 current leaf，M5R07 挂起**。M5R07 已有 scoped PASS 全部保留，不得反向写成 FAIL。

2. **M5R07 验收要求降级：只砍界面类证据，保留组合类证据。** 取消两个 scratch 场景的 11 项交互矩阵、逐场景窗口截图、真桌面 computer use（改可选，不卡验收）与旧壳界面外观/交互证据；理由是 2026-08-17 已定 lightcode fork 为长期壳载体、旧 Tauri 壳降为存续期载体，其像素与交互证据将随旧壳过期。**真桌面像素证据挪到新壳 F5 一次性完成，属明确记账欠项，不是取消。** 必须真实通过的仍有六项：真实启动路径取得项目身份、真进程与普通产品组合、用户拒绝零副作用、强杀重启后持久状态、端口给出精确对象引用、以旧壳为真实非测试客户端把完整执行链走通一次。已有后端定向矩阵照跑。

3. **`.claude/harness-lite/*` 与 `AGENTS.md`/`CLAUDE.md` 的改动归属用户本人**，已提交为 `0db02ef`，不再作为来源不明 WIP 审查。

节点机制：开发主管到节点必须停下，把 authorization 打回精确 closed，并在仓库外 `/home/synadmin/workspace/.syn-gates/open/` 写节点请求文件等独立验收。节点只有两个——M5R00 前置完成、M5R07 收口前。

## 叶子

- [x] REC-00 事实恢复门：R0 恢复载体、分层归责、Harness 重建、前置矩阵与路由
- [ ] M5R00 前置实现与 adapter 修正（2026-08-18 按真实 M1 缺口重开，唯一 current leaf；原 `NOT_NEEDED` 判定作废，见下节修订）
- [x] M5R01 执行合同矫正与旧数据映射
- [x] M5R02 持久编排核心与 ExecutionGrant
- [x] M5R03 WorkerReport、独立审查与事实提升
- [x] M5R04 普通项目的持久 Project Supervisor
- [x] M5R05 受控执行、恢复与 runtime conformance
- [x] M5R06 ProjectSummary 正式投影
- [ ] M5R07 项目 UI、隔离 App 与阶段候选（2026-08-18 挂起，等 M5R00 前置完成后拉回 current；已有 scoped PASS 全部保留，非 FAIL、非撤销）
- [ ] M5 独立验收与 closeout
