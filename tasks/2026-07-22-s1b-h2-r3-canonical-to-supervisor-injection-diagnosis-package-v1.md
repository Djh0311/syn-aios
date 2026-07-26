# 任务包：S1B-H2-R3 canonical → 主管注入/回复链只读诊断 v1

- 日期：2026-07-22
- 状态：**已出包，未执行；等待用户发送 kickoff**
- 类型：真实现场只读取证与修复设计；不含代码修改，不含真实 App 重试
- 权威 kickoff：`handoffs/2026-07-22-s1b-h2-r3-canonical-to-supervisor-injection-diagnosis-kickoff-v1.md`
- 直接上游：`evidence/2026-07-22-s1b-h2-r2-real-app-pending-card-verification-v1.md`
- 原 H2 合同：`tasks/2026-07-19-s1b-h2-supervisor-syn-natural-information-flow-package-v1.md`

## 0. 任务结论

R2 已证明三次用户发送都成功落为 canonical `supervisor_resident_user_message_recorded`，但三次均没有对应的 `supervisor_resident_user_message_injected` 或 `supervisor_resident_supervisor_message_recorded`。因此故障边界不是“消息未送达 canonical”，也不能再把 `recorded +3` 解释为产品重复落账。

本包只做一件事：在不启动 App、不发送新消息、不修改真实 store 和代码的前提下，将三条 recorded 消息逐一关联到主管回合生命周期、会话 binding、进程登记和私有 runner 产物，确定每条消息最早在哪个生产边界停止，并形成证据充分、范围明确的最小修复包。

本包不得宣称 H2 已修复或真实 App 已通过。

## 1. 已知事实

1. R2 Gate 0/1 已通过：当前源码对应的裸 debug binary 已重建并实际启动，七个冻结源码 hash 在构建前后匹配。
2. 用户明确确认规定首句共发送三次。canonical 计数为：

   ```text
   recorded: 8 → 11 (+3)
   injected: 3 → 3  (+0)
   replied:  3 → 3  (+0)
   ```

3. proposal/Pending/chain 始终为 `74/17/40`；未见 `submit_proposal` handler acceptance，未发第二句，未批准卡，未启动 worker。
4. 固定测试项目未变化；DB-primary 没有新增 degradation；App 正常关闭后 registry、holder 和相关进程均清零。
5. 生产代码的顺序已由源码确认：
   - 先 `append_resident_user_message_recorded`；
   - 再调用 `consult_supervisor_resident_with_parts`；
   - 该调用任意 `Err` 当前都在用户消息入口被折叠为 `message_recorded_supervisor_incomplete`；
   - 只有取得成功 turn 后才写 `user_message_injected`，再写 supervisor reply。
6. 所以 `injected +0` 只能证明失败发生在“成功 turn 之后写 injected”之前或写 injected 本身；仅凭三个计数无法区分 preflight、spawn、thread binding、CLI/runner、watchdog、cleanup、turn protocol、lifecycle audit或 injected canonical 写失败。

## 2. 未知项

必须用现场证据回答，禁止猜测：

- 三条新增 recorded event 的精确 `message_id`、`client_request_id`、时间和顺序；
- 每条消息是否到达 `resident_turn_prepared`；
- 是否真正启动过 `codex exec` / `resume`，以及 PID/进程组是否成功登记；
- 是否收到真实 stdout `thread.started`，绑定了哪个 thread/generation；
- 是否发生 invalid-resume、watchdog、quota、protocol、cleanup 或 lifecycle audit 失败；
- 当前私有 resident home 中是否仍有可关联的 stderr/last-message/manifest/hash；
- 三次失败是否同一根因，还是第一条失败后遗留状态导致后两条被 fail-closed；
- 是否有成功 turn 但 `append_resident_user_message_injected` 写失败的证据；
- 现有审计能否完成归因；如果不能，具体缺失哪一个安全内部事实。

## 3. 出包时冻结基线

- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- staged：空
- 相关文件 SHA-256：

```text
86bae55ccc9cd9e1499eae9396b987ea9ef18a31c43f872ad97c0e5e79db2da3  prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_session.rs
82b15432fa35e47b4b6bcc26cab1a20906f8f307b491b8d326602b1bb7ea9c58  prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_tests.rs
d13a9ac9b5b4d0ed9e8fb9d55e713495be48ddc8073bc0b742e946a2aaa56845  prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator_resident_session.rs
7f382cadf799f9dc6e4a34e86b22aca666d9bb8983dee717c235d85c2e03252e  prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model_entrypoints.rs
4057ea384e46c39d2c8f101213e48cf8f4e76fc5ce68522a4a6e1ba13c9ee848  prototypes/productized-desktop-shell/src-tauri/src/exec_process_registry.rs
47ac7053f55403c55d0a467703937b865c01fe001413bec81dc9776e46558bd2  prototypes/productized-desktop-shell/src/views/projects/jiaoban/useJiaobanConversationState.ts
1fa3f464ecc827fda5ed7e6c7c9d99060a4034efbd50f8c357864993d2144c6d  prototypes/productized-desktop-shell/tests/jiaoban-conversation-center.test.tsx
6130ee77e3b6ce4a3730fd049adc2b9bc18718ae49d2401af8d2c035d351962b  prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator_submit_proposal.rs
```

执行者开始时必须重算。相关文件漂移或出现不明归属改动时停止并报告 `BLOCKED_DIRTY_OVERLAP`。本包不允许借诊断之名覆盖任何脏项。

## 4. 权限边界

### 允许

- 只读检查仓库、R2 evidence/raw、`CURRENT.md`、catch log 和相关源码；
- 只读检查已关闭现场的 workflow-state、DB/JSON、resident session、audit events、进程登记历史和私有 resident home；
- 读取与三次发送直接相关的 stderr/last-message 等私有产物，用于本地归因；
- 生成脱敏后的诊断 evidence、修复任务包、kickoff，并最小更新 `CURRENT.md` / catch log；
- 如需稳定取证，可在仓外创建权限为 `0700` 的只读副本目录；只把 manifest、hash 和脱敏结论写入仓库，不把私有正文带入仓库。

### 禁止

- 不启动 Workbench、Tauri dev、Vite、Codex CLI 或 MCP server；
- 不发送首句、第二句或任何真实 App 消息；
- 不读取后再写回真实 store，不执行 reconcile、apply、reseed、迁移或 CAS；
- 不修改 Rust、TypeScript、测试、配置、schema、依赖、命令或脚本；
- 不 stage、commit、push、reset、clean 或 stash；
- 不 kill 进程。若发现 holder/残留，报告 `BLOCKED_LIVE_HOLDER` 并由用户决定；
- 不把用户原文、完整 stderr、auth、token、私有 CODEX_HOME 内容写进 evidence 或用户 read model；
- 不把“源码里某个分支可能失败”写成“现场就是这个根因”。

## 5. Gate 0：现场仍保持关闭

1. 重算第 3 节 hash，并记录 HEAD、porcelain、staged 集。
2. 只读确认 Workbench/dev/Codex/MCP 残留、registry 与 store holder 均为空。
3. 若现场重新被打开或文件正被持有，立即停止；本包不得在活动 store 上做取证副本。
4. 记录真实 store 与 R2 最终证据的只读 hash/mtime/revision，确认取证对象没有被后续 App 回合覆盖。
5. 若 R2 后发生新的 canonical/主管事件，必须单独列出并重新建立时间边界，不能把新事件混进三条案发消息。

## 6. Gate 1：锁定三条案发消息

从 canonical audit 中提取三条新增 recorded event，并形成以下表格：

| 序号 | message_id | client_request_id（仅 hash/末段） | recorded_at | 文本 hash | 对应 injected | 对应 reply |
|---|---|---|---|---|---|---|
| 1 |  |  |  |  |  |  |
| 2 |  |  |  |  |  |  |
| 3 |  |  |  |  |  |  |

要求：

- 不在仓库证据中重复用户原文；只记录“与 R2 规定首句 hash 匹配”；
- 每条消息独立关联，不以总计数替代；
- 明确确认三条 message/client request 是否各不相同；
- 若无法唯一识别三条案发消息，停止并报告 `BLOCKED_EVENT_IDENTITY_AMBIGUOUS`。

## 7. Gate 2：逐条重建主管回合生命周期

对每个 `message_id` 按生产顺序关联：

```text
user_message_recorded
  → resident_turn_prepared
  → process registration
  → session created / reused / replaced
  → stdout thread.started / durable binding
  → runner completion or failure
  → resident_turn_exited / cleanup state
  → user_message_injected
  → supervisor_message_recorded
```

每条都填写：

- 是否存在、事件 ID、时间、run_id；
- PID/进程组（如有）、generation、thread_id；
- launch 类型：created/reused/replaced；
- terminal reason：turn_completed、turn_failed、invalid_resume、watchdog、cleanup 等；
- 与前后事件的时间顺序和 identity 是否一致；
- 最早缺失或失败的边界。

不可把“没有 canonical injected event”直接等同于“未启动 runner”。必须用 lifecycle/进程/私有产物证明。

## 8. Gate 3：私有 runner 产物与审计交叉验证

1. 定位 R2 实际使用的 resident run/home/generation；冻结目录、关键文件 hash、mtime 和权限。
2. 只在本机查看与案发窗口相关的 stderr、last-message、manifest 和 session binding；不得复制正文到仓库。
3. 从私有原文中只提取受控错误家族和必要的非敏感参数，例如：
   - `preflight`
   - `spawn_or_registry`
   - `thread_binding`
   - `invalid_resume`
   - `quota_or_provider`
   - `watchdog_or_timeout`
   - `turn_failed_or_protocol`
   - `cleanup`
   - `canonical_injected_write`
   - `unknown_observability_gap`
4. 与 canonical lifecycle audit、session active message、generation/thread 和 R2 时间窗交叉验证。
5. 如果三次原始产物被同一路径覆盖，只能对尚有证据的回合下结论；不得用最后一次错误倒推前三次。
6. 若 raw detail 与用户面分层一致，保持原文私有；若发现原始错误泄露，另列安全缺陷，但不在本包修复。

## 9. Gate 4：根因裁决

必须在以下结论中选一个，并给出反证检查：

### A. 单一根因已证实

三条消息在同一最早边界失败，且 canonical、lifecycle、私有 runner 产物三方至少两方相互印证。输出：

- 精确失败函数/分支；
- 触发条件；
- 为什么离线测试未覆盖或为何真实状态不同；
- 为什么这能解释 `+3/+0/+0`；
- 排除的相邻原因。

### B. 首次根因 + 后续 fail-closed 已证实

第一条触发原始故障，后两条因残留 lifecycle/session/cleanup 状态被拒绝。必须分别说明三条，不得合并成“一样失败”。

### C. 外部条件阻断

例如额度、CLI/provider、系统权限或文件持有者。需证明代码没有独立回归证据，并明确恢复条件；不得为外部条件改产品语义。

### D. 现有证据不足

必须精确指出：哪个函数吞掉了什么分类、现有审计缺少哪个 identity/terminal fact、为什么不能从现有文件恢复。结论写 `NEEDS_SAFE_INTERNAL_DIAGNOSTIC`，不得猜根因。

## 10. Gate 5：最小修复设计，不实施

基于 Gate 4 输出独立修复包草案，至少包含：

1. 唯一根因和失败夹具；
2. 最小允许改动文件；
3. 先失败后通过的定向测试；
4. 是否需要一个新的**内部**诊断事件；若需要，必须：
   - 走既有 canonical/Batch 2 生产写路；
   - 只记录稳定 error family、message_id、run_id、generation/thread（若已知）；
   - 原始 stderr 只留私有 detail，不进用户 read model；
   - 不因诊断写失败覆盖原业务事实；
5. canonical-first、同一 client request 幂等、invalid-resume 单次轮转、进程组清理和用户真话分层如何保持不变；
6. 离线闸与下一次真实 App R4 验收边界。

若 Gate 4 是 C，输出“现场恢复包”而不是代码修复包；若是 D，只允许出安全内部诊断修复包，不得夹带猜测性业务修复。

## 11. 诊断验收标准

本包只有在以下全部成立时算完成：

1. 三条案发 message_id 已逐条锁定；
2. 每条 record→prepared→binding→exit→injected→reply 的证据矩阵已完成；
3. 结论明确落在 A/B/C/D 之一；
4. 关键结论至少有两类独立证据，或诚实标为 D；
5. 没有启动 App、发送消息、写真实 store 或修改代码；
6. 私有原文和凭证未进入仓库；
7. 形成新的诊断 evidence 与下一份精确修复/恢复任务包；
8. `CURRENT.md` 最小更新，catch log 仅在新增拦截时 EOF 追加；
9. scoped `git diff --check` 通过；
10. 未 stage、未 commit。

## 12. 交付物

- `evidence/2026-07-22-s1b-h2-r3-canonical-to-supervisor-injection-diagnosis-v1.md`
- `evidence/raw/2026-07-22-s1b-h2-r3-diagnosis/`：只放脱敏表格、命令、hash/mtime、计数与 exit；不放私有正文
- 根据 Gate 4 生成其中一类：
  - `tasks/2026-07-22-s1b-h2-r3b-<root-cause>-repair-package-v1.md`
  - `tasks/2026-07-22-s1b-h2-r3b-live-condition-recovery-package-v1.md`
  - `tasks/2026-07-22-s1b-h2-r3b-safe-internal-diagnostic-package-v1.md`
- 对应 kickoff
- `CURRENT.md` 最小状态更新
- 必要时向 `docs/harness-catch-log.md` EOF 追加

## 13. 十项回传

1. Gate 0 是否保持全绿；
2. HEAD、staged、脏基线及冻结 hash；
3. 三条 message_id/client request hash/时间矩阵；
4. 三条生命周期证据矩阵；
5. session generation/thread/active message 与进程登记关联；
6. 私有 runner 产物的脱敏错误家族与交叉验证；
7. Gate 4 裁决 A/B/C/D 及反证；
8. 最小修复/恢复设计和新任务包路径；
9. evidence/CURRENT/catch log 变更与私密信息检查；
10. diff-check、stage、commit 状态，以及所有未执行项。

