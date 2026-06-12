# Handoff: Root Treatment / R2-T11 Rust Task Package Dispatch Preparation Test Extraction v1

日期：2026-06-12

状态：实现和本地验证已完成，复核待进行。

任务包：`tasks/2026-06-12-root-treatment-r2-t11-rust-task-package-dispatch-preparation-test-extraction-v1.md`

Evidence：`evidence/2026-06-12-root-treatment-r2-t11-rust-task-package-dispatch-preparation-test-extraction-v1.md`

Planning baseline commit：`d45fd0adaf437eecfbbb1258ff0f095b502b45f1`

Task package commit：`9e1b6abbf46c7203cb90b906900226d9e3a592fe`

Task package hash backfill commit：`5ea9fba80b609b49fee110e32363e44f137c9d18`

Implementation commit：`6a0640a9088445f196282f8c0b657c4ec079872b`

Review result：待复核。

## 1. 完成内容

R2-T11 按新策略继续做能降低 `lib.rs` 棘轮指标的低风险 inline tests 迁移：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/lib_task_package_dispatch_preparation_tests.rs`
- `lib.rs` 原位置保留 `include!("lib_task_package_dispatch_preparation_tests.rs");`
- `scripts/harness/workbench-shape-gate.js` 的 `lib.rs` waterline 更新为 `6544`

迁移内容为 32 个任务包生成 / 任务记忆注入 / dispatch readiness / field correction tests。共享 helper / fixture builder 继续留在 `lib.rs`。

## 2. 形状指标

- `lib.rs`：`8045 -> 6544`，下降 `1501` 行。
- 新 include 文件：`1503` 行，低于 `.rs` 新文件上限 `3000`。
- shape gate：pass，0 errors，0 warnings；`lib.rs: 6544/6544 (same)`。
- 字节级对比：旧测试块与新 include 文件完全一致，`byte_exact_match: True`。

## 3. 验证

已通过：

- `cargo test --lib workflow_run_check`：2 passed。
- `cargo test --lib task_package`：29 passed，1 ignored；ignored 为既有 confirmation 测试，本轮未执行。
- `cargo test --lib task_memory_injection`：5 passed。
- `cargo test --lib dispatch_field_correction`：5 passed。
- `cargo test --lib`：471 passed，16 ignored。
- `cargo fmt -- --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

保留既有 warning：

- `src/mcp/protocol.rs` 中 `JsonRpcError::invalid_params` dead_code warning。本轮未触碰该文件。

## 4. 边界确认

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`
- 发送 prompt
- 读写 `/Users/yoyi/.codex`
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript
- 启动 Tauri / Browser / Chrome / Vite / 截图工具
- 修改 UI / CSS / TS
- 修改 Tauri command、DB/schema、sidecar schema、workflow state JSON schema

过程备注：

- 一次旧口径扫描命令曾因双引号内 Markdown 反引号触发 shell 命令替换，输出 `zsh: command not found: lib.rs`；该误扫没有执行 Codex、没有触碰 `/Users/yoyi/.codex`，随后已用单引号固定短语重跑扫描。

## 5. 复核待办

复核线需只读确认：

- 新 include 只包含任务包允许的 32 个 tests。
- 旧 `lib.rs` 被删测试块与新 include 文件内容一致。
- helper / fixture builder 没有迁移。
- workflow node dispatch execute/readback、workflow machine、runner/stub、K3、real-state、memory/formal/cross-store adoption、offline role dispatch tests 没有迁入新 include。
- shape gate waterline `6544` 与当前 `wc -l lib.rs` 一致。

## 6. 不接受为

本轮不接受为：

- `lib.rs <= 3,000`
- R2 全部完成
- R3 Level B 执行
- 生产 SQLite 迁移 / read-cut / stop-write
- 多 agent 并行真实执行解锁
- 真实 Codex 执行
- task package / task memory injection / dispatch readiness 产品语义变更或新增能力
- workflow node dispatch execute/readback 迁移完成
- workflow execution runner / workflow machine / K3-B guard 迁移完成
- memory candidate adoption 迁移完成
- formal memory adoption 迁移完成
- UI / 产品行为修改
- backlog 功能解冻
