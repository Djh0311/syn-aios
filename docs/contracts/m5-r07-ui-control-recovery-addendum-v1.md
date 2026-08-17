# M5R07 U01b 安全有限 execution control 补充合同 v1

- 版本：v1（2026-08-17）
- 状态：**ADDITIVE / 非冻结正本 / 不构成 M5 或 stage-14 完成**
- 关系：补充 `m5-isolated-app-and-candidate-v1.md`、`m5-controlled-execution-and-runtime-conformance-v1.md`、`m5-r07-product-path-correction-v1.md`。**不改 M1–M4 冻结合同正文与 hash，不改 shared-isolated authority/profile。**
- 范围：**只覆盖 U01b 安全有限控制**。默认入口 `jiaoban` 已由 `f962038e725ba4e24b2699a46cd1a8d274f13ae6` 独立通过，本包不再重做。U02 ordinary positive runner、M1 ordinary GUI composition、complex terminal retry、M6、closeout 均未完成。

## 1. 本包不重做的已落地项

默认入口保持 `ProjectsView` / `ProjectWorkspaceShell` 的 `jiaoban`，左侧唯一正式 `ProjectSupervisorPanel`，overview 无第二实例。该结果属于 `f962` 默认入口包，不是本控制包的完成声明。

## 2. 正式 command

生产表面只新增：

- `load_m5_execution_control`
- `apply_m5_execution_control`

### 2.1 Request

- Load request **只**允许 `binding_id` / `project_id`。
- Apply request **只**允许 `binding_id` / `project_id` / `action` / `expected_control_revision`。
- `action` 只允许 `STOP` | `RETRY` | `RESUME`。
- 两端 request 必须 `deny_unknown_fields`。不得接 `operation` / `grant` / `dispatch` / `attempt` / `effect` / `fault` / `allowed_commands` / worker session / 按钮权限。
- TS / renderer 不得选择或传递 fault / authority 字段。

### 2.2 Response

响应由服务端从已核对真源派生，至少含：

`control_revision`、`phase`、`durable_state`、`attempt_state`、`retry_count`、`max_retries`、`can_stop`、`can_retry`、`can_resume`、`blocked_reason`、`last_receipt_id`、`replayed`。

TS/UI 只按服务端 `can_*` 显示与禁用，不得自推状态。

## 3. 真源与 join

每次 load/apply：

1. 只从已安装 M1 解析 **registered canonical project**。
2. 重新 load 新鲜 M3 supervisor / worker view；不得缓存、不得由 renderer 传入 RoleSession。
3. `m5_formal_progress` **只是 pointer**，不是 authority。必须用 pointer 重新 load 并 exact join：
   Grant、Dispatch、Attempt、outbox/effect、DurableOperation。
4. 直接链错或 authority 失效（Grant revoked / expired / hash drift；session inactive / permission drift；cross-project）一律 **写前拒绝**。
5. 本包消费既有 join，不扩建通用防篡 / 攻击矩阵。

## 4. Control 真源、CAS、幂等

- Control 真源必须有 durable `control_revision` 与精确幂等 receipt。
- 同 `binding_id` + `project_id` + `action` + `expected_control_revision` 精确 replay：返回同一 receipt，`replayed=true`，零新增。
- 同 revision 不同 action，或 stale / forged revision：稳定拒绝，零写。
- 禁止 `INSERT OR REPLACE` 覆盖同 effect。
- Control command **本身绝不调用 runtime**。生产 execute 入口仍唯一 `run_m5_authorized_runtime_with_state`。

## 5. 本包实际开放的有限动作

这是安全但有限的 control candidate，不追求完美恢复：

- **STOP**：仅无外部 effect 的 `CREATED` / `PAUSED` 可直接标 `CANCELLED`。`RUNNING` / `LEASED` 没有 authoritative cancel readback，因此 `can_stop=false`。
- **RESUME**：仅 `PAUSED` + 已持久 checkpoint 且无外部 effect。生产路径目前没有通用 checkpoint writer；没有可证明 checkpoint 时必须 `can_resume=false`。
- **RETRY**：本包不建立新 Attempt / Grant / Dispatch / effect lineage。`OUTCOME_UNKNOWN` 与 terminal Attempt 一律 `can_retry=false` 并写 blocked reason，不得盲 retry / 复活。

宁可 server `can_*=false` 并写明剩余缺口，也不可翻状态冒充恢复。

## 6. 明确未关闭

本合同 **不** 宣布下列事项完成：

- U02 ordinary disposable positive Tauri runner / server-owned fixture
- M1 ordinary `ProjectRecord` → canonical/exact alias 的可信创建/迁移/ordinary GUI composition
- shared-isolated 正向 scene / window / restart
- `RUNNING` / `LEASED` 的 authoritative cancel readback
- 新 Attempt / Grant / Dispatch / effect lineage 上的 RETRY
- `OUTCOME_UNKNOWN` 的同 effect reconcile
- M5 / stage-14 closeout / M6 激活

既有候选实现或定向测试不得写成 M5 / stage 完成。
