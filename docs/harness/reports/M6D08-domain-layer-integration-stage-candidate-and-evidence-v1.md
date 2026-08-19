# M6D08 domain-layer integration stage candidate and evidence v1

状态：`SUPERVISOR_SELF_REVIEW_PASS / STAGE_CANDIDATE_PENDING_INDEPENDENT_ACCEPTANCE / DOMAIN_LAYER_ONLY / NOT_RELEASED`

## Harness

- stage：`stage-15`；leaf：`M6D08 M6 域层集成回归与阶段候选`；执行来源 receipt：`u-bf5cbe4117b9c087c7f1`。
- 内容候选为 `a3d575975033f7eb5ec92dab18c24fe97ddb8001` / tree `d710e6f38be216e813dbb66482d87e8bc80ce923`；parent 为 M6D07 收口记账 `22f4a11ef488342f070a6fc91e36994f92f17493`。
- Grok 是优先产品执行者而非唯一写者。对 Grok 的窄派发因私有仓库代码出站策略被系统拒绝，未产生 Grok 修改；同一长驻 Codex 在 current leaf 精确写域内接管。施工期只有一个产品源码写者，内容提交按实际贡献只署 Codex trailer。
- 候选精确包含 6 个允许产品路径：`lib.rs`、新增 `m6_org_domain_integration.rs`，以及 `m6_org_consult_handoff.rs`、`m6_org_global_role_session.rs`、`m6_org_member_directory.rs`、`m6_org_temporary_agent_projection.rs`。没有 `commands.rs`、registry、前端、manifest、合同、M1–M5、用户载体或禁止域产品变化。
- 本叶归档后 `docs/harness/leaves/` 为空，authorization 回精确 closed；stage-15 仍保持 active/not accepted。本报告是主管逐叶自复核，不代替最终 Cursor Opus stage verdict，不自行关闭 stage-15 或宣布 M6 完成。

## 产品

1. 新增一个普通 `AppState` 的非 GUI 集成回归：以两个 synthetic ProjectSummary、真实 M3 Secretary/Global RoleSession、M6-owned stores 和 fake provider/runtime 串联 advisory、Secretary consult、stable member、temporary history、multi-view 与 adoption。它证明冲突与来源链接可重建、consult replay 幂等、stable contact 可读、temporary/stable/runtime child 分型、stale availability 排除，以及 adoption/consultation 最多只形成 `PENDING` / `PENDING_USER_DECISION`，不形成 project command、grant 或 fact。
2. 集成 fixture 对项目 domain store、event、audit、outbox、sidecar、compatibility projection、相关文件和 spawn settings 做递归 hash/write-spy；组合运行前后快照相同。可变化载体只在 M3/M6 协调 owner 的隔离 app-data，项目侧零写。
3. legacy/rollback 反例保留旧 single-project global review 与 Agent Center registry/display 载体，目录 export/rebuild 精确往返；生产 M6 跨项目路径仍只消费 `ProjectSummaryQueryPort`，没有 `project_root`、raw project read 或 ownerless process-fact 输入。
4. ordinary startup 行为明确选择 **fail-closed**：普通 `AppState` 安装 Global Supervisor RoleSession 时若候选歧义，返回稳定 `m6_org_global_role_session_ambiguous`，不降级成 `Unavailable`、legacy 或 default session；只有原本明确的 isolated/legacy composition 保留显式 unavailable slot。
5. consult Handoff 在 decide 前已被外部推进 revision 时，accept/reject 都返回同一稳定 `m6_org_consult_handoff_revision_conflict`；M3 readback 仍可读 winning status/revision，M6 不把 revision 1 前提静默冒充通用语义。
6. temporary-agent `refresh`、`search`、`promote` 在访问 M5 或 M6 store、做 replay 或写投影之前统一经过 Global Supervisor RoleSession gate。真实形状 M5 carrier 必须同时存在 `m5_claims`、`m5_work_items`、`m5_prepared_attempts`、`m5_execution_grants`、`m5_dispatches`、`m5_worker_role_session_bindings`、`m5_execution_attempt_readbacks`、`m5_command_receipts`、`m5_events`、`m5_audit_records`、`m5_durable_operations`；缺表稳定报 `m6_org_temporary_agent_m5_schema_carrier_mismatch`，不静默当作无历史。
7. temporary source 状态显式区分 `CompatibleExecutionHistory` 与 `NoExecutionHistory`。完整 envelope 的 receipt/event/audit/durable-operation exact join 继续 fail-closed；缺 `m5_durable_operations` 或 durable row 不能降级为无 ChildRunRef。
8. stable-member 回归使用两个真正独立、各有自身调用计数的 fake runtime 实例；替换 provider/runtime 后 `MemberId`、memory/capability refs 与 export/rebuild 结果均不漂移。

## 证据

原始证据根：`/home/synadmin/workspace/.syn-gates/evidence/M6D08-a3d5759/`。全部 cargo 命令在候选 SHA 的 detached checkout `/tmp/syn-m6d08-evidence.ukalbe`、独立 target `/tmp/syn-m6d08-target-a3d5759` 上执行；`exit-codes.tsv` 记录每条退出码，`SHA256SUMS` 固定原始日志。

- `cargo test --lib m6 --offline -- --nocapture`：exit 0；82 passed / 0 failed。
- `m6d08`：6/6；`m6d07`：8/8；`m6d06`：8/8；`m6d05`：7/7；`m6d04`：4/4；`m6d03`：13/13；`m6d02`：16/16；各命令 exit 0。
- `m6p00`：21/21；指定 `m1_project_index_restart_restores_same_project_id`：1/1；完整适用 `m1_project_index_`：18/18；`m4c05`：9/9；`m3c05`：43/43；各命令 exit 0。
- M6D02 的本候选过滤数为 16，不沿用原叶报告的 15；新增的一项是最终候选中的 ordinary-startup fail-closed 回归。综合 `m6` 与逐叶过滤有重叠，不相加冒充独立总数。
- `cargo check --lib --offline`：exit 0；rustc 汇总 888 warnings，与当前已知基线一致，不写成零 warning。
- `git diff --check 22f4a11..a3d5759`、冻结合同/所选 M1/M3/M5 路径 diff、production-chain probe、主工作树 WIP snapshot、保护载体 hash snapshot、disposable clean status 均 exit 0。
- cargo 在 disposable checkout 生成的未跟踪 `gen/schemas/linux-schema.json` 只在该 disposable checkout 被精确删除；最终 `git status --porcelain=v1 -uall` 为空。主工作树同名受保护文件没有修改、暂存、清理或恢复。
- 主工作树 6 个受保护 `m6_*.rs`（含 `.bak`）与 `linux-schema.json` 的 SHA-256 仍分别为 `620faa27…`、`2c576d9b…`、`6cd604b4…`、`147bd08e…`、`6155c26a…`、`7db42ba1…`、`7e51a7ed…`，等于本叶前基线。

主管七项判据：

1. 写域：PASS。内容候选只有 6 个 current leaf 明示的 `m6_org_*` / `lib.rs` 路径，零能力扩张、零前端、零合同和零禁止域。
2. 冻结物：PASS。M1–M5 冻结合同与所选核心路径 diff 为空；ExecutionGrant、WorkerReport、receipt/audit/quarantine、guarded legacy 和完整 envelope 没有放宽。
3. WIP 保全：PASS。7 个受保护载体 hash 不变且未进 index/commit；四个无关 tracked WIP 与 Harness usage/report 噪声均未暂存、归责、清理或覆盖。
4. 独立重跑：PASS。候选 SHA 绑定的 M6/M1/M3/M4 定向回归、cargo check、diff/frozen/production/status/hash 检查全部 exit 0，原始日志与校验和齐全。
5. 实质：PASS。测试从普通 `AppState` 穿过已接入的 M3/M5/M6 production functions 和真实 SQLite owner stores；关键 gate、revision conflict 与 carrier 检查在 production code 生效，不是 fixture-only 文档声明。
6. 不越级：PASS。证据只到 WSL local/offline/synthetic 的 M6 域层与 ordinary composition；不证明新壳 UI、双项目 isolated App、窗口像素、真实 provider/model/account/project、部署、发布或完整 M6。
7. 欠账路由：PASS。ORG-007、新壳 UI 与 21 个 M6 command 的 renderer consumption 在 `M6S01`；warning/legacy ownerless command/旧 prototype/worktree hygiene 在 `ENG-01`；advisory/member/source-ref 给 M7 的只读输入写入具名 handoff，但 M7 未激活；真实系统证据继续逐项另批。

## 欠账与去处

- `docs/harness/unfinished/M6S01-dual-project-isolated-app-acceptance-on-new-shell.md`：ORG-007、顶层 Global Supervisor/成员/会诊 UI、21 个 command 的真实 renderer 消费、双 scratch-project isolated App、source deep link、跨重启与真实窗口像素；等待 stage-15 独立 PASS、F2/F3 和用户另行开始。
- `docs/harness/unfinished/ENG-01-post-m5-nonblocking-hardening-and-worktree-hygiene.md`：ownerless `record_project_director_process_fact_decision` cutover/default-workflow fallback、888 warning 分类、增补合同索引、受保护 prototype 归属与历史 worktree/target 精确清理。M6D08 只证明该 legacy command 未进入 M6 输入，没有实施 cutover。
- `handoffs/2026-08-19-syn-m6-to-m7-domain-event-input-v1.md`：M7 只能消费 typed event/ref/hash 和原 owner receipt；不得把 advisory、member、consultation 或 `DecisionRequest` 自动升级为 Memory/PersonalFact，且该 handoff 不激活 M7。
- 真实 provider/model/message/account、真实项目只读分析、真实外部 effect、部署、发布和长期日用：均未进入 stage-15，必须按各自阶段与新授权另包。

## 载体

- 产品载体是候选 `a3d5759` 的 Rust M6 域层、普通 `AppState` composition 与本地 SQLite owner stores；不是正在运行的 GUI、新壳、真实 provider 集成或发布产物。
- 本报告、M6→M7 handoff、M6S01/ENG-01 路由、归档 leaf、stage/plan/current-state/audit 与 closed authorization 是 Harness/交接记账，不改变产品候选 tree。
- 当前结论严格为 `M6D08 SUPERVISOR SELF-REVIEW PASS / STAGE-15 DOMAIN-LAYER CANDIDATE / PENDING INDEPENDENT VERDICT / NOT RELEASED`。
