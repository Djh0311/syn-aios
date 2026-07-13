# 对话交接：站 3b 真实只读闭环通过，架构治理暂停 v1

日期：2026-07-13  
状态：`STATION_3B_PASS__WORK_PAUSED__NOT_COMMITTED`

## 0. 一句话

Syn 已在真实项目 `/Users/yoyi/Documents/mario test` 完成一次**单 worker、零写根、只读、可审计**的主管闭环；用户随后明确要求“3b 完成之后先停”。当前不得自行进入站 4、SQLite M0、真实切库或新的项目发射。

## 1. 接手者先读什么

按顺序读取：

1. `CURRENT.md`：当前唯一活正本；
2. 本交接：本轮完成面、未完成面、工作树边界；
3. `tasks/2026-07-12-orchestrator-station3b-readonly-real-project-mario-test-v1.md`：3b 授权、任务与验收正本；
4. `evidence/2026-07-13-orchestrator-station3b-mario-test-readonly-real-run-v1.md`：真实 PASS 总证据；
5. `docs/2026-07-13-architecture-review-v1.md`：架构风险及 M0-M6 建议，但当前处于暂停状态；
6. `evidence/2026-07-13-workflow-state-architecture-risk-remediation-v1.md`：已落 WIP 与 SQLite 阻断实数。

不要从 2026-07-10 的 B2 交接或站 3a 总结推断当前状态；它们仍是历史材料，但本交接更新。

## 2. 已知事实、未知项与接手假设

### 2.1 已知事实

- 站 3b 状态为 `PASS__SINGLE_WORKER__ZERO_WRITE__NOT_COMMITTED`。
- 真实动作顺序为 `dispatch_worker → inspect_worker → finalize(pass) → report_user`。
- 只启动了一个 worker；`allowed_write=[]`；主管与 worker 都是 `--sandbox read-only`。
- worker 回程被解析为 `reported_completed`，主管终标是 advisory `pass`，没有写工作流链态。
- `/Users/yoyi/Documents/mario test` 前后文件清单、`git status --short` 文本和 7 个内容文件 SHA-256 一致。
- worker 逐行读取 README、HTML、CSS、JS，给出了 README 承诺判断、前 5 个问题、精确 `file:line` 和原文，并执行 `node --check game.js`，退出码为 0。
- 调试 App 会话已经停止；`exec-process-registry.v1.json` 当前 `entries=[]`。
- 全部 WIP 未 commit。

### 2.2 尚未证明

- 本次 3b 的 `follow_up_count=0`，所以它证明了普通 inspect 闭环，**没有证明**“派工→追问→读回新报告”的真实追问闭环。
- 最后一处进程登记 `run_id` 可追溯性修复后，只跑了定向测试；没有再跑一遍全库最终回归。
- 站 3b 的授权只覆盖 `/Users/yoyi/Documents/mario test` 的本次只读单，不能外推写单、其它真实项目、多 worker 或自动连环。

### 2.3 接手假设

没有默认续做项。下一步必须由用户重新指定目标；在此之前保持暂停。

### 2.4 站 3b 后新增的路由口径

用户已在 2026-07-13 进一步拍板 `decisions/2026-07-13-orchestration-and-governance-two-axis-routing-v1.md`：

- 站 3b 是低风险真实控制通路验收，不是多 worker 编排价值验收；
- 是否启用主管看协调复杂度，治理强度看后果风险，两根轴不得混为“只读/写入”一条轴；
- 默认单 agent，只有真实拆分、并行、依赖、追问、独立复核或长时接力需要时才升级主管；
- 该判断当前是产品与架构正本，不表示自动路由已经实现，也不构成新发射授权。

## 3. 站 3b 真实运行身份

- supervisor run：`supervisor:workflow-users-yoyi-documents-mario-test-default:1783918485705864000`
- authorization：`plan-auth:project-users-yoyi-documents-mario-test-workflow-users-yoyi-documents-mario-test-default-node-node:1783918484464`
- work item：`work-item:workflow:users-yoyi-documents-mario-test:default:project-director:planned-task-supervisor-pilot-eb33d80132fa15315006376e`
- native worker thread：`019f59d4-1f7a-7a52-88f6-e46308dd9f09`
- dispatch / worker：`dispatch:workflow-users-yoyi-documents-mario-test-default:work-item-workflow-users-yoyi-documents-mario-test-default-project-director-planned-task-supervi:1783918513688`
- worker wrapper/native 进程组：`PGID 94133`，自然结束后登记已注销；
- 本次 debug 构建 SHA-256：`08163d25c5e696f6dfca6d2ff9d5ca1db47d5622d21b3c2cecbf3853869e4fd3`。

五件套均为本次新建，未复用站 2、站 3a 或之前失败尝试。

## 4. 3b 为什么可以判 PASS

### 4.1 链路闭环

控制核心只接受四个动作：

1. `dispatch_worker`：唯一 worker 被预留并启动；
2. `inspect_worker`：合法结构化报告，`evidence_present=true`；
3. `finalize(pass)`：`advisory_only=true`、`workflow_chain_state_written=false`；
4. `report_user`：用户可见报告落账，`user_decision_written=false`。

### 4.2 业务口供

worker 实际读取：

- `README.md:1-20`
- `index.html:1-37`
- `styles.css:1-139`
- `game.js:1-346`

结构化回程位于：

- `evidence/raw/2026-07-12-station3b-mario-test-readonly/attempt-4-worker-report.json`

### 4.3 物理零写

前后证据：

- `evidence/raw/2026-07-12-station3b-mario-test-readonly/attempt-4-pre-launch-baseline.txt`
- `evidence/raw/2026-07-12-station3b-mario-test-readonly/attempt-4-post-run-baseline.txt`

两份文件的排版和采样时间不同，不能直接用整文件 `cmp`；应比较其中的：

- `git status --short` 文本；
- 7 个内容文件的清单；
- 7 个 SHA-256；
- `node --check game.js` 的退出码和输出。

这四组结果均一致。

## 5. 本轮实现上解决了什么

### 5.1 3b 与主管编排

- 给真实项目只读单增加严格的并列小闸：仅允许固定项目根且写根为空，不放宽旧测试项目闸或 legacy 写路径。
- 只读方案也会物化主管任务包，避免主管没有可派工作项。
- `execution_scope`、用户原文、检查命令和验收条件能够进入任务包与 worker objective。
- worker 回程新增通用 `findings`，只读审查不再把核心结论塞进未知字段后被 serde 丢弃。
- inspect、follow-up 报告代际、终标幂等与坏回程停止路径已补齐代码和回归；但真实 3b 没有触发 follow-up。
- 公共 supervisor MCP 保持只读；主管只提动作，授权、绑定、配额、幂等和实际 adapter 调用仍由 Syn 控制核心执行。

### 5.2 进程与运行材料

- Codex worker/manual relay 使用独立进程组登记；异常恢复按已登记且身份匹配的进程组回收，不扩大到未登记进程。
- 每次运行使用唯一 last-message，避免失败运行误读旧结果。
- 新主管运行材料搬到 `runtime-artifacts/`，不再继续向 workflow-state 根写裸 txt；历史 txt 尚未迁移。
- 真跑发现旧请求字段为空时 durable registry `run_id` 会退化为 `resume:`；WIP 已改为 `codex-local:<operation>:<stable SHA-256 identity>`。

### 5.3 workflow-state 架构止血

这些改动已经在工作树中，但不是“SQLite 已完成”：

- 主 JSON 写入增加锁内 revision CAS，静默覆盖改成 `workflow_state_revision_conflict`；
- 9 个手工 workflow-state 备份入口收回中央 helper；
- 保留策略为最近 30 份 + 最近 30 个每日恢复点，去重后上限 60；
- 当前仍然整本读取、解析、clone、pretty serialize、sync、rename；
- 备份只有份数上限，没有字节预算；
- SQLite 没有切换，产品读写仍走 JSON。

## 6. 验证口径

### 6.1 已验证

- 真实 UI：单 worker、零写根、四动作闭环、advisory PASS、报告用户；
- 项目零写：7/7 SHA-256 一致；
- `cargo test --lib exec_process_registry::tests:: --quiet`：`9 passed; 0 failed`；
- `git diff --check`：通过；
- worker report JSON：`jq empty` 通过；
- 运行结束后 durable registry：0 entries。

### 6.2 最后一次完整回归基线

在最后的 process `run_id` 可追溯性小修之前，完整基线为：

- `cargo test --lib --quiet`：`892 passed; 0 failed; 43 ignored`，共 935；
- `npm run typecheck`：通过；
- `npm run test:offline-interaction`：15 项通过；
- `cargo check --offline`：通过；
- `cargo fmt --check`：只报历史 `codex_db.rs`、`codex_local_runner.rs`、`mcp/storage.rs` 三处漂移；
- `git diff --check`：通过。

用户要求 3b 完成后立即停，因此最后一处小修后没有刷新全库回归。接手者不得把“定向 9 条通过”写成“最终全库已刷新”。

## 7. 当前工作树边界

### 7.1 tracked WIP

主管/3b 主链主要涉及：

- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/consultant_agent.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_action_controller.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_session_launcher.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/worker_report.rs`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectJiaobanPanel.tsx`
- `prototypes/productized-desktop-shell/tests/jiaoban-supervisor-pilot-switch.test.tsx`

进程、派发与状态止血主要涉及：

- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/exec_process_registry.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/store_hygiene.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/utils/store_paths.rs`

工作树还包含 `director_agent.rs`、`secretary_agent.rs`、`index_host_app_entrypoints.rs`、`lib.rs` 等配套改动。接手时以 `git diff --name-only` 和逐文件 diff 为准，不要只按本节概括判断归属。

### 7.2 本线新增文档和证据

- `tasks/2026-07-12-orchestrator-station3b-readonly-real-project-mario-test-v1.md`
- `evidence/2026-07-13-orchestrator-station3b-mario-test-readonly-real-run-v1.md`
- `evidence/raw/2026-07-12-station3b-mario-test-readonly/`
- `docs/2026-07-13-architecture-review-v1.md`
- `evidence/2026-07-13-workflow-state-architecture-risk-remediation-v1.md`
- 本交接。

### 7.3 其它工作线资产

以下未跟随本轮 3b 处理，不能用 `git add -A` 顺手带入：

- `docs/research/2026-07-09-self-evolution-frontier-and-syn-design-v1.md`
- `docs/research/2026-07-09-spec-gate-atdd-agent-coding-design-v1.md`
- `docs/research/2026-07-09-syn-measurement-layer-design-v1.md`
- `prototypes/full-workbench-vision-mockup/`
- `prototypes/syn-adaptive-workbench-prototype/`
- `.claude/`
- `.playwright-cli/`

未得到用户指示前，不删除、不归档、不提交这些路径。

## 8. 架构线停在哪里

### 8.1 已完成审计，没有继续实现

当前主 JSON 为 5,897,201 bytes；备份 45 份，约 228 MiB。旧 SQLite 是 2026-06-15 快照，严重落后于 2026-07-13 JSON，不能直接翻闸。

确认的 SQLite P0：

1. 五组真实主状态数组未进入 importer/apply/schema/exporter：`execution_attempts`、`permission_requests`、`workflow_chain_runs`、`workflow_execution_controls`、`workflow_machine_runs`；
2. importer 接受部分 sidecar，但 apply 可能返回空记录或未知类型 `Ok(0)`，exporter 也不覆盖，存在“接受但丢弃”的假成功；
3. 三个主管持久账本没有迁移合同；`exec-process-registry` 是运行时租约，不能把旧 entry 当历史事实导入；
4. workflow-state 根仍有 91 个历史 txt，当前 preflight 会拒绝。

### 8.2 明确未做

- 没有修改 `workbench_sqlite_importer.rs`、`workbench_sqlite_apply.rs`、`workbench_sqlite_schema.rs` 或 `workbench_sqlite_exporter.rs`；
- 没有实现 M0 fail-closed 合同测试；
- 没有写 production SQLite；
- 没有 read-cut；
- 没有 stop-write JSON；
- 没有移动或删除真实历史 txt/备份。

若用户以后重开架构线，建议从 `docs/2026-07-13-architecture-review-v1.md` §八的 M0 开始；这只是建议，不是当前授权。

## 9. 当前权威、历史与暂停项

### 9.1 当前权威

- 活状态：`CURRENT.md`
- 当前排布：`docs/plans/2026-07-11-orchestrator-fast-path-five-stations-plan-v1.md`
- 3b 任务：`tasks/2026-07-12-orchestrator-station3b-readonly-real-project-mario-test-v1.md`
- 3b 结果：`evidence/2026-07-13-orchestrator-station3b-mario-test-readonly-real-run-v1.md`
- 架构风险：`docs/2026-07-13-architecture-review-v1.md`
- 当前交接：本文件。

### 9.2 历史

- attempt-1 至 attempt-3 是安全拦截和失败史，保留但不代表当前结果；
- `handoffs/2026-07-10-b2-execution-loop-closed-conversation-handoff-v1.md` 是 B2 收官历史；
- `docs/agent-work-summary.md` 是站 3a 工作总结；
- 站 3a PASS 仍有效，但不能替代站 3b 的真实项目证据。

### 9.3 暂停

- 站 4；
- SQLite M0-M6；
- 真实 follow-up 发射；
- 其它真实项目、写单、多 worker、自动连环；
- commit。

## 10. 下个对话的第一步

只做状态恢复，不默认开工：

```bash
cd /Users/yoyi/workspace/product-line
git status --short
git diff --check
sed -n '1,220p' CURRENT.md
sed -n '1,260p' handoffs/2026-07-13-station3b-real-loop-pass-and-architecture-pause-handoff-v1.md
```

然后等待用户确定新目标。如果用户只问“接下来做什么”，先给选项和风险，不自动修改文件。
