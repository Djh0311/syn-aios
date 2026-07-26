# L3 `knowledge_open` host-owned relay 离线停点证据 v1

- 日期：2026-07-23
- 执行线结论：**BLOCKED / R1 不能安全验收，未进入 R3 或真实 App。**
- 对应任务包：`tasks/2026-07-23-l3-knowledge-open-host-owned-relay-and-real-app-acceptance-package-v1.md`
- 说明：这是执行线的停点记录，不替代指导线验收。

## 1. 已完成的离线合同与最小接线

- R0 先新增 relay 红合同；最初定向 Rust 测试因 `knowledge_open_relay` 模块不存在而以 `E0583` 失败，未破坏既有断言。
- 当前 WIP 包含 host-owned UDS relay、短租约 grant、固定 request/ack schema、Active binding 复核、固定 vault Markdown 二次读取，以及同一 intent 的 UI ack 链。
- 主窗口事件现仅向 `main` Webview 发射；ack command 同样拒绝非 `main` Webview。
- 已登记 conversation transport 的 generic raw manual poll/stop 已被拒绝；safe transport receipt 的序列化断言不包含 relay endpoint 或 grant。

这些局部成果不足以宣称 R1 通过，因为下列 secret sink 仍在本包白名单外。

## 2. 最早安全停点

1. `exec_process_registry.rs` 将运行中 Codex 的完整 `ps command` 写入既有 durable sidecar。当前固定 MCP argv 中包含 relay endpoint 和 grant，因而会持久化敏感值。
2. `manual_relay.rs` 将 supervisor 子进程 stdout/stderr 原样写进临时捕获文件；MCP/CLI 错误可能回显 argv 或内部配置，不能以“通常不会回显”作为安全证明。
3. raw manual receipt 的 managed-attempt guard 在 transport 返回 safe receipt 后才登记；在 hostile renderer 模型下存在极短的 pre-registration 回读窗口。

这三项均违反任务包 §3 中 endpoint、grant、argv、环境与内部错误不得进入 ordinary log/UI/canonical evidence 的合同。继续真实 App 会把静态 WIP 变成真实秘密暴露，故停止。

## 3. 已跑验证

| 检查 | 结果 |
| --- | --- |
| `cargo test knowledge_open_relay_tests --lib` | 6 passed, 0 failed |
| `cargo test safe_receipt_omits_raw_command_and_process_material --lib` | 1 passed, 0 failed |
| `cargo check --lib` | 通过；未发现 relay 测试 harness 新增 warning。项目既有 aggregate warning 未在本停点当作绿色。 |
| 新增 relay Rust 文件 `rustfmt --check` | 通过 |
| 真实 App / Gate 0 / vault 操作 | 未启动、未访问、未执行 |

未运行 R3 的全量 Rust/TypeScript/offline runner/shape gate；它们在 secret sink 修复前不能构成可接受的阶段验收。

## 4. 最小下一包

需要指导线明确扩白到下列文件，且只允许对应修补：

- `src-tauri/src/exec_process_registry.rs`：对带固定 supervisor MCP marker 的条目持久化固定脱敏摘要与 `sha256(raw_cmdline)` identity；reaper 仍以启动时间、PGID 与 hash 精确匹配。旧条目维持现有 fail-closed 行为。
- `src-tauri/src/manual_relay.rs`：对 host-owned supervisor relay 的捕获文件和 raw receipt 生命周期作最小收紧，阻断 stderr/argv 落盘与 pre-registration race；不得调整 supervisor binding、权限、capability allowlist 或知识写边界。

不建议改成 inherited-FD/socketpair：它偏离任务包冻结的 listener + host-written MCP config 数据流，且本机 Codex Node shim 不保留该 FD。

## 5. 安全与运行状态

- 未新增 binding/DB/JSON schema、relay sidecar、知识写能力或 supervisor 权限。
- 未启动 Syn、Codex CLI/MCP、Obsidian 或任何真实 App；未访问 vault、未生成截图、未创建验收命名空间。
- staged 仍为空；未 commit、push、reset、clean、stash 或删除任何用户文件。
