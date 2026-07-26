# L3 `knowledge_open` relay R1 blocker 指导线独立复核 v1

- 日期：2026-07-23
- 结论：**BLOCKER CONFIRMED / 当前 relay WIP 不得启动真实 App**
- 上游执行 evidence：`evidence/2026-07-23-l3-knowledge-open-host-owned-relay-offline-verification-v1.md`
- 当前返工入口：`tasks/2026-07-23-l3-knowledge-open-relay-secret-sink-closure-repair-package-v1.md`

## 1. 指导线核到的实际路径

### 1.1 Durable argv sink 成立

- `exec_process_registry.rs:243-275` 通过 `ps command` 获得完整 `ObservedProcess.cmdline`，直接赋给 `RegisteredProcess.cmdline_summary`。
- `exec_process_registry.rs:406-498` 随后将整个 store JSON 序列化并原子写入 `exec-process-registry.v1.json`。
- `manual_relay/conversation_transport.rs:416-445` 把 host-owned relay endpoint/grant 放入主管 MCP args，再编码进 Codex `-c mcp_servers.supervisor_orchestrator.args=...`。

因此真实主管进程登记会把 endpoint/grant 一并持久化。执行线停止是正确的。

### 1.2 Raw stdout/stderr sink 成立

- `manual_relay.rs:1814-1861` 在 spawn 前创建 `thread-events.stdout.jsonl` 与 `thread-events.stderr.txt`，child stdout/stderr 原样定向到这两个文件。
- `manual_relay.rs:1380-1402` 后续从磁盘读取完整字节；`bounded_stderr_summary` 是读取后的 receipt 投影，不是写盘前防护。

无法证明 CLI/MCP 错误永不回显配置，因此不能用“通常不回显”代替闭锁。

### 1.3 Pre-registration 与失败清理缺口成立

- 底层 manual relay 在 `spawn_running_codex_like_process` 内先创建 active attempt。
- `manual_relay/conversation_transport.rs:285-310` 在底层返回 raw `ManualRelayReceipt` 后，才写 conversation transport attempt record。
- `commands.rs:259-273` 的 generic raw poll/stop 保护依赖另一份 command-attempt map；`commands.rs:863-882` 又在 safe response 已生成后才插入。

这形成底层 attempt 已可见、raw guard 尚未知晓它的窗口。指导线还发现：如果后置 command-attempt 插入失败，当前路径直接返回错误，没有同步停止刚启动的底层 child/attempt；这是执行回报未单列的真实 cleanup catch。

## 2. 独立复跑

在 `prototypes/productized-desktop-shell/src-tauri`：

| 命令 | 指导线结果 |
| --- | --- |
| `cargo test knowledge_open_relay_tests --lib` | 6 passed；其中 raw receipt 测试只覆盖“已登记后”的拒绝，不覆盖 pre-registration 窗口 |
| `cargo test safe_receipt_omits_raw_command_and_process_material --lib` | 1 passed |
| `cargo check --lib` | 通过；598 条既有 warnings |
| `git diff --check` | 通过 |
| staged | 空 |

局部测试结论与执行线回报一致，但未覆盖真实 sink，不能提升为 R1 通过。

## 3. 裁决

1. 接受执行线 `BLOCKED`，不接受 R1/R3/真实 App 完成。
2. 扩白仅限 `exec_process_registry.rs` 与 `manual_relay.rs`，并复用原包已授权的 `commands.rs`、conversation transport 与测试面完成原子闭锁。
3. 修法冻结为：固定脱敏摘要 + 完整 observed cmdline hash；主管 relay 原始输出不落盘；spawn 前 safe-only 标记；所有后置登记失败逆序清理。
4. 本返工只做离线安全闭锁。真实 App、真实 store/vault、Gate 0 与十二项验收继续关闭。
5. 指导线新增 cleanup catch 已追加 `docs/harness-catch-log.md`；不是零 catch。

## 4. 当前状态

- 未启动 Syn、Codex CLI/MCP server、Obsidian 或真实 App。
- 未访问或修改真实 store/vault。
- 指导线只新增本复核 evidence、返工任务包并更新当前入口；不修改产品代码。
- 未 stage、commit、push、reset、clean 或 stash。
