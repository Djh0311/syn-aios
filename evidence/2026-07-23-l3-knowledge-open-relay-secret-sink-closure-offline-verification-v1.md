# L3 `knowledge_open` relay secret-sink closure：离线验证 v1

- 日期：2026-07-23
- 状态：**OFFLINE COMPLETE / GUIDANCE ACCEPTED**
- 对应任务包：`tasks/2026-07-23-l3-knowledge-open-relay-secret-sink-closure-repair-package-v1.md`
- 边界：只验证 host-owned supervisor relay 的离线安全/cleanup 合同；**未启动 Syn、Codex CLI/MCP server、Obsidian 或真实 App，未访问真实 store/vault，未进入 R4。**

## 1. 红合同与转绿

任务包 §4 冻结了六类原始缺口：durable argv 落盘、reaper 身份放宽、stdout/stderr/JSON 跨 chunk 落盘、pre-spawn raw 可见窗口、durable/stdin/outer-registration 后置失败未逆序清理，以及既有 agent/manual relay 回归。开发先以这些真实缺口为红合同，未删除断言或改 fixture 逃避。

实现后的本包绿门实数如下；它们覆盖六类合同和随后指导线追加的 host recovery / persistent-cleanup 闭锁。

| 离线门 | 实际结果 |
| --- | --- |
| `cargo test knowledge_open_relay_tests --lib` | 7 passed, 0 failed |
| `cargo test safe_receipt_omits_raw_command_and_process_material --lib` | 1 passed, 0 failed |
| `cargo test exec_process_registry --lib` | 13 passed, 0 failed |
| `cargo test conversation_transport --lib` | 30 passed, 0 failed |
| `cargo test manual_relay --lib` | 54 passed, 0 failed, 2 ignored |
| `cargo test supervisor_sentinel_temp_artifacts_are_byte_clean_before_cleanup --lib` | 1 passed, 0 failed |
| `cargo test outer_command_attempt_collision_reaps_running_safe_only_transport --lib` | 1 passed, 0 failed |
| `cargo test outer_command_attempt_registry_unavailable_reaps_running_safe_only_transport --lib` | 1 passed, 0 failed |
| `cargo test outer_registry_unavailable_keeps_a_host_recovery_route_until_trusted_stop --lib` | 1 passed, 0 failed |
| `cargo test outer_collision_keeps_safe_resources_until_a_trusted_retry_settles_them --lib` | 1 passed, 0 failed |
| `cargo test poisoned_registry_allows_only_host_owned_supervisor_recovery_routes --lib` | 1 passed, 0 failed |

未把上述定向绿门说成全仓测试或真实 App 验收。

## 2. Durable registry 与精确 reaper

仅 host-owned supervisor conversation process 使用脱敏 durable 身份：

- `cmdline_summary` 固定为 `host_owned_supervisor_conversation_process`；
- `observed_cmdline_sha256` 保存完整 observed command line 原始字节的 SHA-256（64 个十六进制字符）；
- endpoint、grant、relay path、`mcp_servers...args` 与 raw command line 不进入 sidecar；
- reaper 只有在 started_at、process-group leader/PGID 和完整 observed command-line hash 都精确匹配时才回收。任一不匹配不扩大 kill 面，维持 fail-closed 行为。

`r0_host_owned_supervisor_registry_must_not_persist_relay_command_secrets` 与 `host_owned_supervisor_reaper_requires_exact_started_at_pgid_and_cmdline_hash` 随 registry 定向组通过。

## 3. Supervisor capture 与 sentinel

host-owned supervisor 改为 supervisor-only 有界内存捕获：

- 总输入上限 64 KiB，单 stdout JSON frame 上限 8 KiB，最多 128 个 live events；
- stdout 只增量解析受限 JSONL；stderr 仅作为状态，不把原文投影到 receipt；
- overflow、parse 或 I/O 失败均 fail-closed，同时仍走 child/process-group 回收；
- 终态/Drop 清空 capture state；不创建 supervisor 的 stdout/stderr/last-message capture 文件。

sentinel 回归覆盖 stdout、stderr、JSON error 与跨 chunk 边界，并在 cleanup 前扫描测试临时 registry/capture/error/receipt 普通文件。定向 sentinel 门 1/1 通过、零命中；safe receipt、raw command result 和错误文本也不投影 raw sensitive material。

## 4. Attempt 生命周期与失败清理

状态机先在 spawn 前通过 `safe-only` marker 和 visibility gate 闭锁 generic raw poll/stop，再保留 active slot、启动 child、登记 durable process、启动 memory capture，并交由 trusted supervisor transport 操作。

每一个后置失败都按逆序处理：

1. durable registration、stdin write、outer safe-attempt registration、terminal normalization、stop 与 app-shutdown 都先停止 child/process group；
2. 成功停止后注销 durable registration，清理 active、protected marker 与 memory capture；
3. trusted cleanup 未结算时保留 child/durable/active/marker/capture 与 host-owned recovery route，generic raw 路径仍被拒绝；
4. trusted retry 结算后才移除 inner/outer record 与 recovery route；registry poison 也只允许 host-owned recovery。

定向 collision、registry-unavailable、persistent-cleanup 和 shutdown 回归均通过。generic writer 也不能在 retained supervisor cleanup 窗口覆盖相同 attempt id。

## 5. 一次性 rustfmt 收口

执行前 HEAD 为 `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`，暂存区为空；下列三个冻结 SHA 与任务包 §6.1 完全一致：

| 文件 | 格式化前 SHA-256 | 格式化后 SHA-256 |
| --- | --- | --- |
| `manual_relay.rs` | `a138f920f6e8a68d93010a8afac65abf2c45e05eeed421f7690a5113b1b010d5` | `62d97639ea14ba6800f5c3106a98d7051da43a564a8dbe5918dba9779906f06f` |
| `manual_relay/conversation_transport.rs` | `6a8de491b1ddf298633453ae56229e5461bd345eb0dac3758fcb40fffb12f247` | `a69018176f0b6b1db8899198577d9453fcfebaea3b58325b6228291036a10749` |
| `commands.rs` | `56ab0c5f77827ae2545f8edaac763220e97a6410632e6d0754cad8c455d7a5df` | `e9f98ea7c340c8f871e227505a962905298f345aebb7d5ddbccf904a78005126` |

执行命令为 `rustfmt --edition 2021 src/manual_relay.rs src/commands.rs`。前者只递归其唯一的已授权外置子模块 `conversation_transport.rs`。预检、目标后置 `rustfmt --check`、同一修改时间窗口和独立只读审计共同确认：只发生授权的 25 + 1 + 13 = **39** 个唯一机械 hunk，未见语义、注释、测试合同、UI、capability 或白名单外源文件改动。

取证限定：格式化前的脏工作树没有另存为 Git blob，因此无法在事后进行逐字节的 39-hunk 重建；结论基于执行前预检、冻结/后置 SHA、唯一命令、后置格式门、路径/mtime 取证和独立审计。

## 6. 其余离线门与历史债

- `cargo check --lib`：通过；598 条项目既有 warning。
- 定向 Rust 测试输出合计仍显示 18 条既有 warning；未归因给本包。
- 五个目标 Rust 文件的 `rustfmt --check`：通过。
- `git diff --check`：通过。
- `git diff --cached --name-only`：空。
- shape baseline：exit 0，`Status: pass`，17 errors / 5 warnings / 5 info。
- shape check：exit 1，`Status: fail`，同为 17 / 5 / 5；当前 finding 清单未显示本包新增类别，但此前最后已知聚合读数是 16/5/5，且本包没有开工前 shape 快照，所以 post-change baseline 不能证明整包绝对零净增，也不能写成绿色。

## 7. 实际范围、catch 与下一步

本包代码路径为：

- `prototypes/productized-desktop-shell/src-tauri/src/exec_process_registry.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay/conversation_transport.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/knowledge_open_relay_tests.rs`

本轮新增了一条真实 catch：完整 `manual_relay` suite 抓到 shared safe-only fixture 的并行隔离漏洞与一个测试夹具的 marker 顺序错误。已为共享全局状态的夹具增加测试锁，并将 marker reservation 前置；该 catch 已按本包授权追加到账本。

暂存区为空；没有 commit、push、reset、clean 或 stash；没有启动真实服务，也没有读写真实 store/vault。下一步是等待指导线独立核 diff 并复跑 secret/hash/cleanup 测试；在此之前不恢复上游 R1-R4。

## 8. 指导线独立验收

2026-07-23 指导线独立复核并接受本包离线结论：

- 三个格式化后 SHA 与第 5 节一致；五个目标 Rust 文件格式检查、`git diff --check` 与 staged 空检查通过。
- 独立复跑 `exec_process_registry` 13/13、完整 `manual_relay` 54/54（2 ignored）、`knowledge_open_relay_tests` 7/7、safe receipt 1/1。
- 独立复跑 outer collision、outer registry unavailable、host-only recovery retained/retry、poison recovery 五项，各 1/1。
- 独立 `cargo check --lib` 通过，598 条 warning 未被伪报为绿色。
- 静态复核确认 durable registry、完整身份 hash、spawn 前 raw 闭锁和未结算 cleanup 的 host-only recovery route 与任务合同一致。

验收范围只到离线安全返工；未启动 Syn、Codex CLI/MCP、Obsidian 或真实 App，未访问真实 store/vault，也未证明 `knowledge_open opened=true` 或 N6 十二项通过。
