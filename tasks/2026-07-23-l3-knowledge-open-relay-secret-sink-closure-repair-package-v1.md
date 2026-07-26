# 任务包：L3 `knowledge_open` relay secret sink 与 attempt 登记闭锁返工 v1

- 日期：2026-07-23
- 状态：**OFFLINE COMPLETE / GUIDANCE ACCEPTED**
- 负责人：现有 Codex 开发线（`gpt-5.6-terra`，reasoning=`ultra`）
- 指导/验收：当前总指导对话
- 上游暂停包：`tasks/2026-07-23-l3-knowledge-open-host-owned-relay-and-real-app-acceptance-package-v1.md`
- 阻塞复核：`evidence/2026-07-23-l3-knowledge-open-relay-r1-blocker-guidance-review-v1.md`
- 格式裁决：**AUTHORIZED ONCE / 仅限 §6.1 冻结的 3 文件与 39 个机械 hunk**

## 对齐块

- `authority_chain`：`AGENTS.md` → `CURRENT.md` → `AUTHORITY.md` → `decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md` → 本任务包。
- `plan_anchor`：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md` 的 N6，以及暂停上游包的 R1-R3。
- `existing_before_new`：复用现有 `exec_process_registry.v1`、manual relay process lifecycle、conversation transport safe receipt 与 host-owned relay WIP；不另造 transport、registry 或日志系统。
- `capabilities_touched`：只触及主管 `knowledge_open` relay 的进程身份、输出捕获、attempt 可见性与失败清理；不改知识 read/write 能力、binding、角色或 allowlist。
- `forbidden_alternatives`：FD/socketpair、环境传 secret、新 sidecar、先落原文后删除、通用日志重构、放开 raw endpoint、放宽 reaper identity、启动真实 App。

## 0. Kickoff

- 任务：关闭 host-owned supervisor relay 的三类真实 secret/lifecycle sink，再把 R1-R3 离线门跑绿。
- 负责人：现有 `gpt-5.6-terra / ultra` 开发线。
- 交付物：进程登记脱敏身份、主管 relay 无原始磁盘捕获、spawn 前 raw receipt 闭锁与全失败路径清理、红/绿测试、离线验证 evidence。
- 完成标准：endpoint、grant、完整 supervisor argv 与原始 stderr/stdout 不进入 durable registry、普通捕获文件、raw Tauri receipt、UI 或错误文本；任何登记失败都回收本包启动的 child/attempt；orphan reaper 仍只对启动时间、PGID 与完整命令身份都一致的进程组执行回收。

本包是高危安全闸的精确修补。**不启动 Syn、Codex CLI/MCP server、Obsidian 或真实 App，不访问真实 store/vault，不进入 R4 十二项验收。** 离线绿后先回总指导验收，再恢复上游包。

## 1. 已核实的 blocker

1. `exec_process_registry.rs` 当前把 `ps command` 全文写入 `cmdline_summary`，随后序列化到 `exec-process-registry.v1.json`。主管 MCP 的 `-c mcp_servers...args=...` 已包含 relay endpoint/grant，因此真实启动会持久化敏感值。
2. `manual_relay.rs` 当前在 spawn 前创建 `thread-events.stdout.jsonl` 与 `thread-events.stderr.txt`，并把 child stdout/stderr 原样重定向到文件。不能以“CLI 通常不回显配置”作为安全保证。
3. generic raw poll/stop 的保护登记发生在底层 manual attempt 已启动并返回 receipt 之后，存在 pre-registration 回读窗口。
4. 指导线新增 catch：底层 child/attempt 已启动后，如果外层 safe-attempt 登记失败，当前路径直接返回错误，没有同步停止底层 attempt、注销 durable process registration 和清理捕获材料。

现有 6 项 relay 测试、1 项 safe receipt 测试与 `cargo check --lib` 通过，只证明 WIP 的局部合同，不覆盖上述 sink。

## 2. 冻结修法

### 2.1 Durable process identity

- 只对可信宿主选择的 host-owned supervisor conversation process 使用新身份模式；不得由前端、用户文本、tool arguments 或环境变量选择。
- durable entry 只保存固定脱敏摘要和 `sha256(raw observed cmdline)` identity，不保存或拼接 raw cmdline、endpoint、grant、内联 MCP config、绝对 relay 路径。
- hash 输入必须是实际 `ps command` 的完整字节文本；不得只 hash 固定摘要。相同启动时间、PGID、完整 command hash 才可 reaper kill。
- 旧 v1 条目维持现有 fail-closed 兼容：不能核实就只清/留登记，不得扩大 kill 面；不要为了迁移读取或改写真实 sidecar。
- 不新增第二个 registry、relay sidecar、DB/JSON workflow schema；复用既有 hash helper，不加依赖、不改 Cargo 文件。

### 2.2 Supervisor output capture

- host-owned supervisor relay 不得把原始 child stdout/stderr 写入磁盘。优先使用**有界内存捕获 + 增量 JSONL 解析**；如采用其他方案，必须用测试证明原始敏感值在写盘前已被处理，并覆盖跨 chunk 边界，不能先落原文再删除。
- existing agent/manual relay 路径保持原行为；不要把本包扩成通用日志系统重构。
- 内存捕获必须有总字节上限、单帧上限和终态清理；超限 fail-closed，但不能吞掉 child 回收。
- safe receipt 只保留现有分层字段与中立错误 family；不得把 raw stderr、argv、endpoint、grant 或内部路径投影到 UI。

### 2.3 Attempt visibility 与失败清理

- supervisor conversation attempt 在 child spawn 前即标为 `safe-receipt-only`；generic raw poll/stop 从第一个可观察时刻起拒绝该 attempt。
- safe transport 内部 poll/stop 仍可通过受信入口操作同一 attempt；不得靠放开 generic raw endpoint 实现。
- 保护标记、底层 active attempt、外层 command attempt 任何一步失败，都必须按逆序清理：停止本包 child/process group、注销 durable registration、清除 pending/attempt 标记与内存捕获。
- registration failure、stdin write failure、outer safe-attempt collision/poison、terminal parse failure、Stop、App shutdown 的清理都要有注入测试。失败时不能遗留“raw 可读但 safe map 不存在”的半状态。

## 3. 精确写入白名单

### 新扩白

- `prototypes/productized-desktop-shell/src-tauri/src/exec_process_registry.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`

### 原包内 merge-only

- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay/conversation_transport.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/knowledge_open_relay_tests.rs`
- 如测试必须接现有模块：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 只允许机械模块登记，不改业务逻辑。

### 文档

- 本任务包：只更新实际状态和实际白名单。
- `evidence/2026-07-23-l3-knowledge-open-relay-secret-sink-closure-offline-verification-v1.md`（新增）。
- `docs/harness-catch-log.md`：只有执行线再发现新 catch 才追加；不得重复记指导线已记录的 outer-registration 清理缺口。

除此之外一律不写。尤其不改 Cargo、前端、知识文件、binding/DB/JSON、capability allowlist、主管 profile、CURRENT/AUTHORITY。

### 实际写入（2026-07-23）

- 本包代码面：`exec_process_registry.rs`、`manual_relay.rs`、`manual_relay/conversation_transport.rs`、`commands.rs`、`knowledge_open_relay_tests.rs`；未触碰 Cargo、前端、知识文件、binding/DB/JSON、capability/profile/allowlist 或 `CURRENT.md`/`AUTHORITY.md`。
- 本次 §6.1 一次性格式化只写入 `manual_relay.rs` 25、`manual_relay/conversation_transport.rs` 1、`commands.rs` 13 个机械 hunk，共 39 个。
- 文档只写本任务包、下列离线验证 evidence，以及一条本包新抓到且已修复的 shared-fixture 隔离 catch。

## 4. 红合同

实现前先让以下测试因当前缺口失败：

1. 以 sentinel endpoint/grant 构造 observed supervisor cmdline，登记后 sidecar 全字节不含 sentinel、relay 路径、`mcp_servers...args` 或 raw cmdline；只含固定摘要和 64 个十六进制字符的 identity。
2. reaper 对相同 started_at/PGID/hash 精确回收；任一不同只注销/保留登记且不 kill。
3. supervisor child 输出在 stdout、stderr、JSON error、跨 chunk 边界分别回显 sentinel，所有普通文件、safe receipt、raw command result 和错误文本均零命中。
4. raw poll/stop 与 safe start 并发时，从 spawn 前保护登记开始，raw 入口始终返回固定 protected family。
5. durable registration、stdin write、outer safe-attempt registration 分别注入失败，child/process group 均被停止，durable/active/protected/capture 四类状态均清零。
6. agent/manual relay 既有 capture、poll/stop 与 receipt 行为不回归。

红灯必须来自真实缺口，不得删除既有断言或改 fixture 逃避。

## 5. 实施与停止顺序

1. 冻结当前 HEAD、status、staged 空状态和白名单文件 hash。
2. 写红合同并记录预期失败。
3. 先完成 process registry 脱敏身份与 reaper 精确匹配。
4. 再完成 supervisor-only 内存捕获和 attempt 原子闭锁/逆序清理。
5. 跑第 6 节离线门；全绿后写 evidence 并停止。

若实现必须新增线程/缓冲基础设施，应保持在 `manual_relay.rs` 内的 supervisor-only 最小结构；如需要新文件、依赖、schema bump、环境秘密或 FD/socketpair，立即停止回交，不自行扩白。

## 6. 必跑验证

1. 新增 process registry secret/hash/reaper 测试。
2. 新增 supervisor capture、并发可见性和四类失败清理测试。
3. `cargo test knowledge_open_relay_tests --lib`
4. `cargo test safe_receipt_omits_raw_command_and_process_material --lib`
5. `cargo test exec_process_registry --lib`
6. conversation transport、manual relay process lifecycle 相关定向回归。
7. `cargo check --lib`
8. 目标 Rust `rustfmt --check`
9. 对测试临时 registry/capture/error/receipt 做 sentinel 全字节扫描，零命中。
10. `git diff --check`
11. `git diff --cached --name-only` 为空。
12. shape gate 如实报告；历史聚合债务不写成绿色，本包不得新增类别。

不要求 `npm run typecheck` 或前端 runner，因为本包禁止前端改动；若实际 diff 触碰前端即越界。

### 6.1 指导线一次性目标格式授权

2026-07-23 指导线已独立执行目标 `rustfmt --check` 并复核实际 diff。当前 product-line 唯一仍承担代码写入的线程是本包现有开发线；并行对话审计线为已停止的只读线程，本开发线也已明确请求完成该格式门，因此当前 WIP owner 同意条件成立。

授权前冻结 SHA-256：

- `manual_relay.rs`：`a138f920f6e8a68d93010a8afac65abf2c45e05eeed421f7690a5113b1b010d5`
- `manual_relay/conversation_transport.rs`：`6a8de491b1ddf298633453ae56229e5461bd345eb0dac3758fcb40fffb12f247`
- `commands.rs`：`56ab0c5f77827ae2545f8edaac763220e97a6410632e6d0754cad8c455d7a5df`

一次性授权范围：

- `manual_relay.rs`：25 个 rustfmt hunk；
- `manual_relay/conversation_transport.rs`：1 个 rustfmt hunk；
- `commands.rs`：13 个 rustfmt hunk；
- 共 39 个唯一机械 hunk，只允许 rustfmt 改写空白、换行、缩进、import 排序和等价括号布局，不允许顺手改名、改注释、改测试或改控制流。

执行前必须复核三份 SHA 与上述值完全一致、staged 为空；任一漂移立即停止。执行后必须确认目标 `rustfmt --check` 通过，并复跑本包安全/cleanup 定向测试、`cargo check --lib`、sentinel 扫描、`git diff --check` 与 staged 空检查。若 formatter 出现第 40 个 hunk、触碰其他文件，或复跑结果与当前报告不一致，立即停止且不得写成功 evidence。

## 7. 立即停止条件

- 任一敏感 sentinel 进入 durable/普通文件/UI/raw receipt/error。
- reaper 需要放宽 started_at/PGID/完整命令身份任一核对。
- 需要读取或改写真正 `exec-process-registry.v1.json`、真实 store 或 vault。
- 需要新增 sidecar、DB/JSON schema、依赖、环境秘密、通用命令/文件能力。
- agent/manual relay 行为被不必要改变，或真实 App/CLI/MCP 被启动。
- 出现白名单外写入、无法归属的并行 hunk、staged 非空。

## 8. 回交格式

1. 红合同预期失败与转绿实数；
2. registry 持久化字段与 reaper identity 算法；
3. supervisor output 的内存/边界策略；
4. attempt 从 spawn 前到终态的状态机和每个失败清理结果；
5. sentinel 扫描范围与零命中证据；
6. 完整验证实数、warnings 与 shape 三数；
7. 实际修改文件和白名单核对；
8. staged/进程/真实 store/vault 状态；
9. 新 catch 与剩余问题。

收到回传不等于验收通过。指导线需独立核 diff、复跑 secret/hash/cleanup 测试后，才决定是否恢复上游 R1-R4。

## 9. 指导线独立验收（2026-07-23）

结论：**ACCEPTED / 只结算本包离线安全返工，不代表真实 App 或 N6 十二项通过。**

指导线独立确认：

- 三个格式化目标的后置 SHA 与 evidence 完全一致，目标五个 Rust 文件 `rustfmt --check`、`git diff --check` 通过，staged 为空；
- `exec_process_registry` 13/13、完整 `manual_relay` 54/54（2 ignored）、`knowledge_open_relay_tests` 7/7、safe receipt 1/1 通过；
- outer collision、outer registry unavailable、host recovery retained/retry 与 poisoned registry 五条关键门各 1/1 通过；
- `cargo check --lib` 通过，仍为 598 条既有 warning；
- 静态 diff 复核确认 host-owned durable entry 不保存 raw argv，reaper 仍要求 started_at、PGID=PID 与完整 observed command hash；safe-only marker 在 spawn 前闭锁，未结算 cleanup 只保留 host-only recovery route，generic raw 路径保持拒绝。

shape 当前复跑为 17 errors / 5 warnings / 5 info；此前最后已知读数为 16/5/5，且本包没有开工前 shape 快照，因此不能用本包的 post-change baseline 证明整包绝对零净增。当前 finding 清单未显示本包新增 finding 类型，这一事实只支持“未新增类别”，不支持“绝对零净增”。

本包至此停止。上游 `tasks/2026-07-23-l3-knowledge-open-host-owned-relay-and-real-app-acceptance-package-v1.md` 可按其既有边界恢复；R4 的真实运行结论仍须单独取证。
