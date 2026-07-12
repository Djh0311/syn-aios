# 站 3a 无人值守收口证据 v1

日期：2026-07-12  
任务包：`tasks/2026-07-12-orchestrator-station3a-unattended-closure-v1.md`  
结论：`HISTORICAL__SUPERSEDED`（当时为 `READY_FOR_UNLOCKED_UI`；现行完成证据是 `evidence/2026-07-12-orchestrator-station3a-control-core-bridge-v1.md` 的 v7 PASS。本文件中的旧 `approval_policy = "never"`、42 字节 proof 和旧 app 构建只保留历史，不代表当前实现。）

## 本轮范围与修改

- 本轮没有修改 3a 实现代码；静态审查未发现新的确定性缺陷。
- 本轮仅新增本 evidence 文件。
- 未操作工作台 UI，未新增或修改固定测试项目文件，未触碰真实项目或站 3b，未更新 `CURRENT.md`，未执行 `git add` / commit / push。
- 子任务接口不支持指定 `5.6-terra-极高`，未声称已设置该模型；本轮按最高审查强度执行。

## 静态审查结果

- P0：无。
- P1：无。
- P2：无确定性缺陷。

核对证据：

- 缺失或空 `allowed_write` 不再回退到项目根：`h5_project_dispatch_bridge.rs:44-48`。
- worker 启动前先核对任务包精确写根以及 project/workflow/node/work item/authorization 和 authorized check：`commands.rs:2427-2439,2755-2822`。
- 模型抄错 work item ID 时，只从同一 project/workflow/node/authorization 的唯一 prepared work item 恢复正本，并记录 canonicalized warning：`mcp/supervisor_orchestrator.rs:233-259,356-390`。
- active authorization 同时核对 project/workflow/role/write root：`mcp/supervisor_orchestrator.rs:1251-1278`。
- 站 3a 写根锁定固定测试项目；planned task ID 基于完整 authorization 的 SHA-256 摘要：`supervisor_session_launcher.rs:477-484,583-589`。
- 主管命令固定 `-C /Users/yoyi/codex-workflow-mario-test --sandbox read-only`；临时 `CODEX_HOME` 只写 `approval_policy = "never"` 和唯一 `supervisor_orchestrator` MCP：`supervisor_session_launcher.rs:699-718,751-769,973-1001`。

## 自动化验证

1. `cargo test --lib mcp::supervisor_orchestrator::tests --quiet`
   - PASS：11 passed；0 failed；0 ignored。
2. `cargo test --lib station3a_ --quiet`
   - PASS：3 passed；0 failed；0 ignored。
3. `cargo test --lib s3_director_dispatch_integration_stub --quiet`
   - PASS：1 passed；0 failed；0 ignored。
4. `cargo test --lib --quiet`
   - PASS：832 passed；0 failed；43 ignored；共 875 tests。
5. `npm run typecheck`
   - PASS：`tsc --noEmit` 退出码 0。
6. `npm run test:offline-interaction`
   - PASS：`offline interaction tests passed: 15`，所列离线 DOM/状态断言均通过。
7. `cargo check --offline`
   - PASS：退出码 0。仓库仍输出既有 unused/dead-code warnings，不是本包新增失败。
8. `cargo fmt --check`
   - 基线非零：仅命中任务包允许的既有 `src/codex_db.rs`、`src/codex_local_runner.rs`、`src/mcp/storage.rs`；本包 3a 文件无格式漂移。
9. `git diff --check`
   - PASS：退出码 0，无输出。
10. `../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug --bundles app`
    - PASS：退出码 0；前端生产构建和 Rust debug 构建成功。
    - 最新 app：`prototypes/productized-desktop-shell/src-tauri/target/debug/bundle/macos/CodexGovernanceWorkbench.app`
    - app 与内部二进制时间：`2026-07-12 04:14:21`。
    - 构建有既有 Rust warnings 和 Vite 大 chunk warning，但没有构建失败。

## 固定测试项目只读证据

证明文件：`/Users/yoyi/codex-workflow-mario-test/station3a-supervisor-write-proof.txt`

```text
$ wc -c station3a-supervisor-write-proof.txt
42 station3a-supervisor-write-proof.txt

$ xxd -g 1 station3a-supervisor-write-proof.txt
00000000: 73 74 61 74 69 6f 6e 33 61 20 73 75 70 65 72 76  station3a superv
00000010: 69 73 6f 72 20 77 6f 72 6b 65 72 20 77 72 69 74  isor worker writ
00000020: 65 20 76 65 72 69 66 69 65 64                    e verified
```

以上证明内容精确为 `station3a supervisor worker write verified`，42 字节，末字节 `64`，无末尾换行。

`git status --short`：

```text
 M README.md
 M index.html
?? chain-proof.txt
?? jiaoban-plan-a-proof.txt
?? jiaoban-proof.txt
?? jiaoban-slice2-proof.txt
?? s1-step3-proof.txt
?? s2-3-loop-proof.txt
?? station3a-supervisor-write-proof.txt
?? workflow-fulldispatch-proof.txt
?? workflow-real-run-proof.txt
```

与任务包记录的既有基线相比，只多 `station3a-supervisor-write-proof.txt`。

## 解锁后仍需完成

本 evidence 只证明自动检查和最新 `.app` 已准备好，不把既有证明文件冒充本次主管闭环。解锁后仍必须从工作台 UI 的“允许并开始”发射一次，并在真实账本中看到同一 run 的：

1. worker 派发成功；
2. `read_worker_report` 成功；
3. `final_mark: pass`；
4. `report_user` 与临时 `CODEX_HOME` 清理完成。

这些真实账本证据出现前，站 3a 不能宣布完成。
