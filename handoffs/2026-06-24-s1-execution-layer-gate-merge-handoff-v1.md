# 回交：S1 执行层合一「B 画布派发过 A 银行级强闸」· 执行线 → 主导线 v1

日期：2026-06-24　性质：**高危#3（改安全闸路径）实现回交**　任务包：`tasks/2026-06-24-s1-execution-layer-gate-merge-v1.md`

## 0. 一句话结论

按 **option A**（主导线拍板）实现：B 的 `execute_project_workflow_node_at` 在真起 runner 前，先过 A 的统一强闸 `decide_real_execution_command`，**path-lock 命中作 `authorization_complete` 的必要项**。逻辑接通 + 锁死（铁律/沙箱/A线/连环）经 stub 测试 + 4 路对抗复核验过。**执行线未 commit**。

> ⚠️ 按任务包 §8：§6 的「测试项目真跑端到端」是**单独后续步**、不在本包；本回交只算 **「逻辑接通 + 锁死验过」，不算端到端通**。

## 1. 改了什么（diff 范围）

```
 src/commands.rs  | +144            （S1 闸 + 3 个 helper）
 src/lib.rs       | +397 / -56      （5 个 S1 测试；-56 为 cargo fmt 顺带清的 pre-existing 区，fmt-only 见 §5）
 2 files changed
```

工作树 `git status --short` 仅 `M commands.rs` + `M lib.rs`（+ 未跟踪的任务包文档）。**无其它文件改动。**

### 1a. 闸插入点（commands.rs:2175-2223）

`execute_project_workflow_node_at` 内、构造 `exec_request` 之前插入一个 block：算 `path_lock_hit` / `duplicate_blocked` / `guard_blocked` → 调 `decide_real_execution_command` → `!gate.runner_call_allowed` 即 `return Err("real_execution_gate_blocked:...")`、不起 runner。

### 1b. 3 个 helper（commands.rs:2249-2344）

- `has_inflight_dispatch(state, workflow_id, node_id) -> bool`：算 `duplicate_blocked`。
- `canvas_node_guard_blocked(guard) -> bool` + 常量 `CANVAS_NODE_GUARD_AUTHORIZATION_REASONS`：算 `guard_blocked`（option A 的核心：排除 3 道授权 reason）。
- `build_canvas_node_codex_local_request(...)`：构造喂给 A guard 的 `CodexLocalExecutionRequest`。

## 2. 各判据怎么算（§2 映射表的落地）

| 字段 | B 的取值 | 出处 |
|---|---|---|
| `authorization_complete` | `path_lock_hit` = `workflow_engine_test_project_unsealed(project_root)` | commands.rs:2183, 2206 |
| `duplicate_blocked` | `has_inflight_dispatch`：查同 (workflow,node) 是否有 `state=="running"` 派发 | commands.rs:2185 |
| `guard_blocked` | `canvas_node_guard_blocked(inspect_codex_local_execution_guard(...))`：过 A 执行安全 guard、排除 3 授权 reason | commands.rs:2186-2204 |
| `readback_required` | `true`（B 走 readback_db 回读） | commands.rs:2212 |
| `diagnostics_blocked` | `false`（**见 ②**） | commands.rs:2210 |
| `stale_memory_blocked` | `false`（B 不走任务记忆包） | commands.rs:2211 |
| `user_rejected` | `false`（本包不引入逐次审批=S2） | commands.rs:2208 |
| `command_name`/`family`/`operation_id`/`h5_unified_product_command` | `"execute_project_workflow_node"` / `"workflow_real_execution"` / `"resume"` / `true` | commands.rs:2196-2199 |

### 三处需主导线过目的决策

**① option A 范围（guard 排除 3 授权 reason）** — A 的强 guard 会因「无 A 的授权产物」报 3 道 reason：`user_confirmation_required` / `authorization_scope_missing` / `audit_ref_missing`。B 的授权走 path-lock、不是 A 那套确认/范围/审计，**B 没有这些产物、不伪造**（伪造=授权绕过=不安全）。故只计「执行安全」reason（adapter/operation/路径/密钥/prompt 边界/readback/command_plan…）、排除这 3 道授权 reason。= **只加严（B 拿到执行安全检查）、不放松、不造假**。对抗复核确认：这 3 道的职能在 B 语境下被 path-lock（硬拦非测试）+ write_roots（沙箱据 sandbox_mode 强制、不读 authorization_scope_id）+ readback_required（主动回读）三层更强替代，伪造字段也突不破（详 §3 verdict②）。

**② `duplicate_blocked` 只数 `"running"`、不数 `"prepared"`** — `execute_workflow_node_dispatch_at` 每次派发都先 `write_prepared_dispatch` 留一条 orphan `"prepared"`（永不推进，真执行是另一条 started→completed 记录）。所以 `"prepared"` 每次残留、不是可靠在飞信号——数它会**误拦同节点合法重跑**（这也是一个生产 bug，初版数了 prepared 时 cap 测试 run2 即被误拦）。`"running"` 是真正执行中、同调用内推进到 completed/failed 不残留，才是准信号。

**③ `diagnostics_blocked` 取 `false` 的安全说明** — A 的 `diagnostics_blocked` 源自 A 自己的诊断降级输入，**B 派发上下文无此输入**（不接 A 的诊断源）。取 `false` 安全：它只是「少一道额外拦」，不影响铁律（path-lock）/沙箱/guard 三道硬闸；真要接需 B 侧先有诊断摘要数据源（无中生有的 `true` 会无依据拦正常派发）。同理 `stale_memory_blocked`（B 不走任务记忆包）、`user_rejected`（逐次审批=S2）取 `false`。

## 3. §3 死线证据（4 路只读对抗复核 verdict）

复核用 read-only `Explore` agent（无 Edit/Write，不会越界改码；已二次 `git status` 核实工作树仅 commands.rs+lib.rs）。

| § | 死线 | verdict | 要点 |
|---|---|---|---|
| 铁律 | `authorized ⟹ path-lock 命中` | **holds** | `authorization_complete` **全库唯一真代码赋值** = commands.rs:2206 `path_lock_hit`（其余 6 处均在 real_execution_command.rs test 模块）；所有到真 runner 的派发路径都被 path-lock 或本 S1 闸前置守护；`workflow_engine_test_project_unsealed` 严格等值 `/Users/yoyi/codex-workflow-mario-test`（commands.rs:1677-1678） |
| 沙箱 | `command_plan_for` 字节不动 | **holds** | `codex_local_runner.rs` diff = **0 行**；`command_plan_for` 据 request 的 `sandbox`/`allowed_write_roots` 构沙箱、不读授权字段 |
| option A | 排除 3 reason 不是漏洞 | **refuted（漏洞声明被驳）** | 3 职能被 path-lock+write_roots+readback 更强替代，伪造授权字段突不破（见 §2①） |
| 测试 | 不真跑 codex | **holds** | 5 个 `s1_` 测试 + cap 测试全用 stub `PermissiveExperimentRunner`(lib.rs:3707-3735) 或纯单元不调 runner；真 `RealWorkflowNodeCodexRunner` 仅 `#[ignore]` 测试(lib.rs:3623) |
| 不动 A 线 | `controlled_session_continuation`/H5 零变化 | **holds** | `session_continuation_store.rs` diff = **0 行** |
| 不开连环 | 4 护栏不削 | **holds** | `workflow_chain_controller.rs` diff = **0 行** |
| 不改判决体 | `decide_real_execution_command` 7 拦顺序不动 | **holds** | `real_execution_command.rs` diff = **0 行**（只喂输入） |

### 铁律正反测试（lib.rs，stub 验）

- `s1_gate_iron_law_path_lock_required_for_authorized`：`authorization_complete=false` → `!runner_call_allowed`（不授权）；非测试项目 `workflow_engine_test_project_unsealed` 恒 false；测试项目 + 各判据满足 → 授权。
- `s1_gate_blocks_dispatch_when_node_has_inflight_running`：注入 `"running"` 派发 → `execute_project_workflow_node_at` 返回含 `duplicate_blocked` 的 Err（新判据真拦，用 stub runner）。
- `s1_guard_blocks_prompt_with_secret_but_allows_clean`：含 `.env/token` 的 prompt → `guard_blocked=true`；干净 prompt → false。
- `s1_has_inflight_dispatch_counts_running_only` / `s1_canvas_node_guard_blocked_excludes_authorization_reasons`：钉死 ②③ 决策。

## 4. §4 全量闸输出

| 门 | 结果 |
|---|---|
| `cargo test --lib` | **580 passed / 0 failed / 25 ignored**（基线 575 + 5 S1 测试）✓ |
| `cargo fmt -- --check`（我的文件） | commands.rs / lib.rs **干净** ✓ |
| `npx tsc --noEmit` | 干净 ✓ |
| offline-interaction | 通过 ✓ |
| `vite build` | 通过 ✓ |
| `git diff --check` | 干净 ✓ |
| `workbench-shape-gate` | **脚本不存在**（`scripts/harness/` 空 — S0 清理 / AGENTS §四 harness 默认关）⚠️ 见 §5 |

> 基线 575：本会话并行主导线 commit `4414bac (S0 瘦身)` 删了孤儿死码+测试，基线从 580→575，与任务包 §4「基线 575/0」一致；S1 在其上 +5 测试 = 580。

## 5. 待办 / flags（主导线决断）

1. **shape-gate 脚本缺失** — 任务包 §4 列了 `node scripts/harness/workbench-shape-gate.js`，但 `scripts/harness/` 当前为空（S0 清理或从未落地）。此门无法跑、非 S1 引入。
2. **pre-existing fmt 债** — `cargo fmt -- --check` 在 **HEAD(4414bac) 即有 52 处** fmt 问题（`codex_db.rs` / `codex_local_runner.rs` / `lib.rs` 旧区 / `mcp/storage.rs`），多半并行 S0 commit 未 fmt。我**只修了自己 S1 文件**、未碰这 52 处：`codex_local_runner.rs` 受 §3 保护**不能** fmt（含 command_plan_for），已 `git checkout` revert；其余无关文件也 revert 以保最小 scope。**lib.rs 的 -56 行**是 cargo fmt 顺带清的 lib.rs pre-existing 旧区(~3624-3912)+cap 测试格式化，纯 fmt、无逻辑。→ 建议主导线另起一次 `cargo fmt` 全量整备（不与 S1 混）。
3. **path-lock 5-处收敛** — 任务包 §1 提「从抄 5 处收敛为进闸前算一次」。本包**新增**了进闸前算一次（合一），但**保留** `:1688/:1757/:1920/:2254` + `workflow_chain_controller.rs:304` 原有 path-lock 作纵深防御（defense-in-depth），**未删**。是否拆掉旧 5 处 = 后续收敛，建议单列（删防御层应单独谨慎验）。
4. **§6 测试项目真跑** — 端到端「测试项目真跑画布节点、验走强闸/path-lock 命中才放/改非测试 root 被拦/沙箱只动测试目录」是**本包之后单独一步**（轻档·测试项目），未做。

## 6. 主导线审实物 + commit 指引

- **核 diff**：`git diff` 看 commands.rs（闸+helper，自带注释）+ lib.rs（测试）；4 个 §3 文件 `git diff --stat -- <file>` 应 0 行。
- **重跑闸**：`cd .../src-tauri && cargo test --lib`（应 580/0）；前端 `npx tsc --noEmit` / offline / `vite build`。
- **扫高危**：确认未放开非测试项目、未碰 `command_plan_for`、未开连环、未改判决体、未动 A 线（§3 verdict 全 holds/refuted）。
- **commit**：执行线不 commit；由主导线带 `CURRENT.md` 回写（完成项挪①、刷新③下一步）做。
