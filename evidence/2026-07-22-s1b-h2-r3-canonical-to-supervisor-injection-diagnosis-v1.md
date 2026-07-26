# S1B-H2-R3 canonical → 主管注入/回复链只读诊断 v1

日期：2026-07-22（+0800）  
任务包：`tasks/2026-07-22-s1b-h2-r3-canonical-to-supervisor-injection-diagnosis-package-v1.md`  
上游：`evidence/2026-07-22-s1b-h2-r2-real-app-pending-card-verification-v1.md`

状态：**完成只读取证；裁决 D / `NEEDS_SAFE_INTERNAL_DIAGNOSTIC`。没有授权、也没有执行代码修复或真实 App 重试。**

## 结论先行

三次用户发送都已经以不同的 client/message identity 成功落入 canonical，因而不是“消息没有送到”，也不是产品重复落账。三条随后都停在同一个可证区间：**canonical `recorded` 之后、首个持久化 resident lifecycle 事实（成功 `turn_prepared`）之前**。本窗没有 prepared、process binding、runner exit、injected 或 reply；私有 runner 也没有为该窗创建可关联的 output 目录或文件。

但这个区间仍包含多个生产分支：preflight 的 reaper/session/executable/plan/home/facts、runner output-directory 初始化失败，以及 child spawn 后 prepared 生命周期写失败。入口将这些 `Err` 全部折叠为同一 `message_recorded_supervisor_incomplete` 结果，且在 prepared 前没有 message-scoped 失败审计。现有证据不能诚实地把三条归为其中任何一个具体分支，故不能裁决 A、B 或 C。

下一步只应实施安全的内部诊断：以既有 canonical / Batch 2 生产写路为每条已 recorded 的消息记录稳定错误族和最小关联 identity；不得把原始错误、stderr、认证或私有 home 内容送入用户面或仓库。任务包见 `tasks/2026-07-22-s1b-h2-r3b-safe-internal-diagnostic-package-v1.md`。

## Gate 0：现场关闭与冻结

- Workbench、dev、Codex、MCP、Vite 相关进程均无匹配；workflow-state、production DB/WAL/SHM 无 holder；registry entries 为 `0`。没有 kill、启动或写入。
- HEAD 为 `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`，staged 集为空。R3 指定的八个源码 SHA-256 全部精确匹配；既有脏项不与冻结范围重叠，未触发 `BLOCKED_DIRTY_OVERLAP`。
- 现场 workflow revision=`289`；target workflow canonical 为 `11/3/3`；proposal/Pending/chain=`74/17/40`。DB immutable 只读 integrity 为 `ok`，业务投影与 JSON 一致，无新增 JSON-only degradation。

完整脱敏 hash、mtime、计数与冻结源码清单见 [Gate 0 raw](raw/2026-07-22-s1b-h2-r3-diagnosis/gate0-freeze.md)。

## Gate 1：三条案发消息

| # | message_id | client hash / tail | recorded_at（+0800） | text hash | injected | reply |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | `user:1784666691878362000` | `ceed5246…21df95` / `872523ab` | 07-22 04:44:51.878 | `8576e1e…0d72079` | 0 | 0 |
| 2 | `user:1784666696842452000` | `eca80860…adede0` / `879c0847` | 07-22 04:44:56.842 | `8576e1e…0d72079` | 0 | 0 |
| 3 | `user:1784666700190151000` | `0d75c566…c954f2` / `7f0f9ba4` | 07-22 04:45:00.190 | `8576e1e…0d72079` | 0 | 0 |

三条均匹配 R2 规定首句的 hash，但 client/message identity 各不相同。逐条按 `message_id`、`reply_to_message_id` 和 `target_ref` 交叉扫描后，每条只关联到自身 recorded event；DB projection 逐条一致。这与用户确认的三次发送一致，**不构成重复落账**。详情见 [canonical raw](raw/2026-07-22-s1b-h2-r3-diagnosis/canonical-message-matrix.md)。

## Gate 2/3：逐条生命周期、session、registry 与私有 runner

| 消息 | recorded | prepared | registry | durable binding / `thread.started` | runner exit | injected | reply | 最早可证失败边界 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `…1878362000` | 是 | 无 | 无历史证据 | 无 | 无 | 无 | 无 | recorded 后、成功 output 初始化 / prepared 前 |
| `…6842452000` | 是 | 无 | 无历史证据 | 无 | 无 | 无 | 无 | 同上 |
| `…0019151000` | 是 | 无 | 无历史证据 | 无 | 无 | 无 | 无 | 同上 |

两类独立持久化证据（supervisor JSON 与 SQLite）均显示：R2 时间窗内 resident audit、新 session、prepared、created/reused/replaced、invalid-resume、binding、turn exit 与 MCP tool call 均为零。前一历史 session 的 generation=`6`、host PID=`0`、状态为 `resident_exited`，不是 `resident_turn_cleanup_failed`；它不足以将后两条判为 cleanup fail-closed。由于三条都没有 `prepared.active_message_id`，该历史 session 也不是任何一条的 run 归属证据；它只说明若 session load 成功返回其非空 thread，代码会进入 resume 预检。

当前 registry 空且本窗无 registry audit，但正常 register/unregister 不留历史 audit，所以不能仅凭 registry 说“从未 spawn”。私有面补足了更窄的事实：对 g6-resume、以及若 invalid-resume 后会出现的 g7-initial/g7-resume，本窗都没有 output directory、stderr 或 last-message 的元数据；现存相关文件全部早于案发约 65 小时。它排除了把历史 invalid-resume artifact 当作三条案发错误的做法，但不区分更早 preflight 与 output-directory 创建本身失败。

详细脱敏矩阵见 [lifecycle/private raw](raw/2026-07-22-s1b-h2-r3-diagnosis/lifecycle-and-private-matrix.md)。

## Gate 4：根因裁决

**裁决：D / `NEEDS_SAFE_INTERNAL_DIAGNOSTIC`。**

### 支持 D 的两类以上独立证据

1. **canonical + DB/JSON 证据：** 三个唯一 recorded identity 都没有 injected/reply；同时 DB/JSON 中没有任何能把其连接到 run 的 prepared、binding 或 terminal 事实。
2. **私有 runner 元数据：** 本窗不存在可关联的 runner output directory 或文件；历史产物在时间上早约 65 小时，不能倒推到这三条。
3. **源码控制流：** recorded 后调用 consult，但 `Err(_)` 在入口被统一吞掉；prepared 前有多个不同的 fallible gate，而 prepared 是首个携带 `active_message_id` 的 durable bridge。现有 registry 也不保存正常 register/unregister 历史。

### 反证检查

- **A（单一代码根因）不成立：** 没有三条共享的稳定 error family、同一私有 artifact 或 message-scoped terminal record；把某个源码候选当作根因会是猜测。
- **B（首次根因 + 后续 fail-closed）不成立：** 历史 session 为 PID=0 的 `resident_exited`，本窗没有 cleanup-failed、stale-reap 或新的 lifecycle 状态；无法把第 2/3 条归为第 1 条残留拒绝。
- **C（外部条件）不成立：** 没有 quota/provider/CLI/权限等受控错误族或 runner artifact；外部归因同样无证据。

## 最小后续设计（不实施）

R3B 仅增加内部、非用户面 diagnostic：每当一条已 canonical-recorded 的消息在 consult 失败返回时，通过既有 Batch 2 canonical 写路，尽力且幂等地记录 `message_id`、`run_id`、generation/thread（已知时）、稳定 `error_family` 与 stage。原始错误不写 canonical/read model；runner stderr 仍仅留既有私有路径。diagnostic 写失败必须被忽略，不能覆盖已完成的 recorded 业务事实，也不得触发重读、重试或 rebase。

不增加 Tauri command、sidecar、MCP server、批准能力或消息运输路；不触碰 H2 单工具预批准、read-only、approval policy、watchdog、invalid-resume 单次轮转、进程组清理、proposal/chain 或真实 store。详细合同与 kickoff：

- `tasks/2026-07-22-s1b-h2-r3b-safe-internal-diagnostic-package-v1.md`
- `handoffs/2026-07-22-s1b-h2-r3b-safe-internal-diagnostic-kickoff-v1.md`

## 现场与私密边界

- 未启动 App、Codex CLI、MCP server、Tauri/Vite 或构建；未发送消息，未操作真实 store，未改 Rust/TypeScript/测试/配置。
- evidence 只含 opaque identity、hash、mtime、计数与稳定受控分类；未写入用户原文、完整 stderr、auth/token 或私有 `CODEX_HOME` 正文。
- 未 stage、commit、push、reset、clean 或 stash。
