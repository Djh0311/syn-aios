# 任务包：S1B-H2-R3B 安全内部诊断闭环 v1

- 日期：2026-07-22
- 状态：**已出包，未执行；等待用户授权 kickoff**
- 前置诊断：`evidence/2026-07-22-s1b-h2-r3-canonical-to-supervisor-injection-diagnosis-v1.md`
- 类型：最小代码/离线验证；不含真实 App、真实 store 或现场重试
- 唯一 kickoff：`handoffs/2026-07-22-s1b-h2-r3b-safe-internal-diagnostic-kickoff-v1.md`

## 0. 唯一目标

解决 R3 暴露的**可观测性缺口**，不是猜测性修复主管运输：当一条消息已 canonical-recorded、但 `consult_supervisor_resident_with_parts` 在 injected 之前失败时，留下一个可与该 `message_id` 关联的、仅含稳定 error family 的内部诊断事实。

本包不宣称已找出 R2 的生产根因，也不授权重新发送真实消息或继续两句验收。

## 1. 已证前提

R2 的三个不同 message/client identity 都已 recorded，均无 injected/reply。R3 已证明它们无 prepared、binding、exit 和 runner 输出目录；现有入口在 `supervisor_resident_oneshot_session.rs:1975-1995` 对所有 consult `Err` 使用同一人话 incomplete outcome。prepared 前的 reaper/session/executable/plan/home/facts 等错误，以及 output-directory 初始化或 prepared 生命周期写失败，当前没有 message-scoped durable identity。

因此本包只能记录事实，不能改变任一失败路径的业务语义、外部条件或恢复策略。

## 2. 冻结与写入范围

开始前重新核对 R3 §3 八个 SHA-256、HEAD、staged 与 dirty ownership；任一相关漂移或无法归属改动即 `BLOCKED_DIRTY_OVERLAP`。不 reset/clean/stash/覆盖已有脏项。

允许改动仅：

1. `prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_session.rs`
2. `prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_tests.rs`
3. 本包产生的 evidence、`CURRENT.md` 与 catch log（仅当有新的拦截）

若实现证明必须触及任何其他文件，停止并另出扩范围包；尤其不改 Tauri command、sidecar schema、MCP server、前端 transport、read-only/approval policy、reviewer/path-lock/write root、watchdog、process-group cleanup、proposal/chain 或 M5 storage logic。

## 3. 设计合同

### 3.1 新内部 canonical 诊断事件

在既有 `append_resident_message_canonical_event` / Batch 2 写路上新增一个**内部 audit event**，只在以下条件成立时尝试追加：

- 同一 `message_id` 的 recorded 已成功存在；
- consult 最终返回失败；
- 尚不存在该 message 的同类 diagnostic。

允许字段仅为：

```text
event_type = supervisor_resident_delivery_diagnostic_recorded
message_id, target_ref, project_id, workflow_id
run_id
generation / thread_id（仅已有时）
stage, stable_error_family
created_at
```

固定、闭合的 `stage` / `stable_error_family` 由结构化调用点产生，不得以 substring 猜原始错误：至少覆盖 `preflight_reap`、`preflight_session`、`preflight_executable`、`preflight_plan`、`preflight_home`、`preflight_facts`、`runner_output_init`、`runner_spawn`、`prepared_lifecycle_write`、`registry_registration`、`thread_binding`、`runner_terminal` 和保守 `unknown`。

事件不得包含用户正文、原始 `Err`、stderr、路径、命令行、token、auth、`CODEX_HOME` 内容或 MCP 参数。它是内部 audit，不得改变既有用户 conversation read model 的显示内容。

### 3.2 失败语义与幂等

- canonical recorded 必须仍然最先完成；diagnostic 是其后的 best effort。diagnostic 写失败必须被吞掉，保持现有 incomplete outcome，不能把已完成的业务 record 改为失败。
- 对同一 message/client identity 只能有一条该 diagnostic；实现可在既有的**一次** Batch 2 read→candidate→CAS 中做“已存在则不追加”的检查。CAS 冲突仍按既有一次写的原语返回，冲突后不得第二次读、重试、rebase 或吞掉真实业务冲突。
- 不记录错误即不改变 consult 本身的执行次数。watchdog 技术重试、invalid-resume 的 archive → generation+1 → facts → 仅一次 initial 仍使用既有路径；成功回合不得产生 diagnostic，也不得重复落卡。
- `message_not_recorded` 仍是唯一可以说“没送到主管”的业务面；recorded 后仍保持既有“已记录但主管未完成”的人话分层。禁止将稳定 error family 或私有 detail 上脸。

### 3.3 最小实现形态

在 resident session 模块内部引入私有、结构化的 consult failure envelope：调用点直接标记稳定 stage/family，raw detail 仅留在函数返回/既有私有 runner stderr，绝不序列化进 diagnostic。只在 submit 入口已得到 `message_id` 的 `Err` 分支中调用 best-effort canonical diagnostic append。

不得以新 sidecar、文件日志、Tauri command、MCP API 或前端状态承接此事实。不得改变 `submit_proposal` 的 allowlist、唯一预批准、proposal authorization、chain 或 worker 行为。

## 4. 先红后绿的定向测试

在现有 `supervisor_resident_oneshot_tests.rs` fixture 中，先添加失败断言，再实施：

1. **preflight fixture：** 注入一个确定的 executable（或 home）preflight 失败。断言：一条 recorded、零 injected/reply、恰一条同 message diagnostic，family/stage 正确；用户 outcome 仍为 incomplete；audit/read model 不含原始 sentinel。
2. **runner output-init fixture：** 注入 output-directory 初始化失败。断言同上，且没有 spawn/registry 伪事实。
3. **prepared lifecycle write fixture：** 模拟 child 已启动但 prepared 写失败。断言 diagnostic 为 `prepared_lifecycle_write`，无 false binding/registry 成功声明，清理与原有 error path 不变。
4. **diagnostic append 失败：** 注入其 Batch 2 append/backup 准备失败。断言既有 recorded 业务 JSON 写仍成功，outcome 不变，不重读/重试/rebase，不产生 partial diagnostic。
5. **同一 client/message replay：** 重放同一失败请求，断言不重复 diagnostic；不同 client request 的三次用户动作仍是三条不同 recorded，不能被误折叠。
6. **既有安全不退步：** invalid-resume 一次轮转、watchdog 技术重试、process registry cleanup、successful injected/reply 和 H2 `submit_proposal` 单工具批准相关定向断言全部保持。

测试不读取/断言真实 auth、private home 正文或完整 stderr；使用 only metadata/sentinel，且断言 sentinel 不进入 canonical/read model。

## 5. 离线闸与停点

按现有仓库命令运行：

1. 新增失败夹具与 supervisor resident 定向 Rust tests；
2. S1B/S1/H2 相关 tests；
3. M5-B/M5-C/M5-F1 定向回归，证明 Batch 2 / DB-primary / JSON-leading fail-safe 未退步；
4. typecheck、离线交互与 shape gate；
5. `git diff --check`、staged 空集、scoped diff 审计。

任何真实 revision conflict、DB/JSON 对账失败、shape 净增或安全闸漂移都停止，不扩修。代码/离线通过后**立即停下**：不启动 App、不操作真实 store、不发送首句或第二句。真实 R4 只能由另一份现场包、用户在场且新 Gate 0 后授权。

## 6. 禁止项

- 不启动 Workbench/Tauri/Vite/Codex/MCP；不触碰真实 store、DB、WAL/SHM、`.codex` 或测试项目。
- 不新增批准、wildcard/default/full-auto/bypass，不批准卡、不启动 chain、不派 worker。
- 不新增 Tauri command、sidecar、MCP server、消息运输路或日志落盘面。
- 不改 M5 DB-primary/JSON-leading fail-safe、CAS、降级审计、direct writer 或生产 apply。
- 不 stage、commit、push、reset、clean、stash。

## 7. 验收与回传

验收不是 H2 live 通过，而是：每种已知失败能从一个 recorded `message_id` 追到稳定内部 family，原文不泄露，diagnostic 失败不影响业务 record，所有安全/幂等/invalid-resume/cleanup/M5 闸不退步。

回传至少包括：冻结/dirty 基线、变更文件、先红后绿测试、全闸结果、diagnostic schema 示例（无原文）、私密扫描、未执行的真实 App 项、以及下一份 R4 现场包前置条件。
