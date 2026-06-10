# Evidence: Stage H / H4 Supervisor Acceptance Review v1

日期：2026-06-08

## 1. 结论

全局主管复核 H4 开发线回交后，H4 接受为：

```text
accepted_as_h4_level_a_non_real_productization_after_supervisor_review
```

接受范围：

- H4 Level A 非真实产品化完成。
- `result_count = null` 统一规则接入 runner、continuation store、runtime attention。
- `readback_unavailable`、`readback_failed`、`readback_timed_out`、`timed_out`、`not_attempted`、`blocked_by_guard`、`duplicate_blocked`、`stale_cancelled` 等 unknown-result 状态不会被写成真实 0 条结果。
- duplicate guard 扩展到同 adapter + operation + continuation / work item / workflow node / target session 范围；blocked path 写 attempt / audit / runtime log，且不调用 runner。
- stale cleanup 只处理工作台自有 active attempt，要求 expected revision，写 audit / runtime log，状态为 `stale_cancelled`。

不接受为：

- 真实 `codex exec` / `codex exec resume` 已执行。
- prompt 已发送。
- 真实 Codex session 已创建。
- `/Users/yoyi/.codex` 已读写。
- H3-B retry 已授权或已执行。
- H3-B 成功。
- H4-Level-B 真实失败 / 超时探针完成。
- H5 项目工作流真实派发完成。
- 阶段 H 完成。

## 2. 主管复核动作

本轮全局主管线完成：

- 读取 H4 开发线回交状态。
- 复核 H4 evidence / handoff。
- 复核 H4 代码关键点：
  - `h4_execution_boundary.rs`
  - `codex_local_runner.rs`
  - `session_continuation_store.rs`
  - `runtime_session_attention.rs`
  - `types.rs`
  - `lib.rs`
- 复核 H4 开发线记录的验证结果。
- 派发 H5-Level-A 只读准备复核线，并读取回交建议。
- 修补两个旧事实口径：
  - `tasks/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md`
  - `tasks/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-v1.md`

## 3. 修补说明

H4 任务包本体修补：

- 状态从“已创建，待执行”修为“已完成 Level A 非真实执行产品化；H4-Level-B 必须另行授权”。
- H3-B 从“未授权、未执行”修为“已执行一次真实 fixture probe，但 `failed_classified`，产品路径已补 `--skip-git-repo-check`，未二次真实执行”。
- 禁止声称项从“不能说 H3-B 已授权或已执行”修为“不能说 H3-B retry 已授权或已执行 / H3-B 已成功”。

H5 任务包草案修补：

- H3-B 当前事实改为 `failed_classified`，不再写成未授权未执行。
- H4 当前事实改为 Level A 非真实产品化已完成，H4-Level-B 仍未授权。
- H5-Level-B 前置改为 H3-B retry 或明确 resume 路径不依赖 new session 的执行点授权。
- 禁止声称项改为不能声称 H3-B retry 已授权 / 已执行 / 已成功，不能声称 H4-Level-B 真实失败 / 超时探针完成。

## 4. 验证依据

沿用并复核 H4 开发线验证记录：

```text
cargo test --lib codex_local_runner -- --nocapture
cargo test --lib session_continuation -- --nocapture
cargo test --lib runtime_log -- --nocapture
cargo test --lib runtime_session_attention -- --nocapture
cargo test --lib g2_diagnostic -- --nocapture
cargo test --lib
rustfmt --check src/h4_execution_boundary.rs src/codex_local_runner.rs src/session_continuation_store.rs src/runtime_log_store.rs src/runtime_session_attention.rs src/types.rs src/lib.rs
```

记录结果：

- `codex_local_runner`: 11 passed。
- `session_continuation`: 16 passed / 2 ignored。
- `runtime_log`: 5 passed。
- `runtime_session_attention`: 2 passed。
- `g2_diagnostic`: 1 passed。
- `cargo test --lib`: 254 passed / 3 ignored。
- `rustfmt --check`: passed。

本轮主管线补充验证：

```text
rg -n 'H4 已创建但待执行|H4 .*待执行|H4 readback .*待执行|H3-B 仍未授权|仍未授权、未执行|H4 readback / failure / timeout / cancel / duplicate guard 完整产品化尚未完成|H3-B 任务包已创建但仍未授权|H3-B 未授权 / 未执行|H4 failure / timeout / duplicate guard 已完成|H3-B 已授权或已执行' ...
```

结果：无命中。

## 5. 多线程协作记录

- H4 开发线：`019ea381-1e3c-7e50-b507-c46eac2039be`，已回交。
- H5-Level-A 只读准备复核线：`019ea394-3cc3-7b01-b278-0af41a8e05fb`，已回交。

H5 准备线未改文件、未跑测试、未执行真实 Codex、未读写 `.codex`。其主要发现是 H5 草案仍有 H3-B / H4 旧状态词；本轮主管线已修补。

## 6. 边界确认

本轮主管线没有：

- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送 prompt。
- 创建真实 fixture run。
- 读写 `/Users/yoyi/.codex`。
- 读取 auth、token、secret、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 改产品代码。
- 改 UI。

## 7. 下一步

允许进入：

- H5-Level-A 协议 / 产品路径集成设计。

仍需另行授权：

- H3-B retry。
- H4-Level-B 真实失败 / 超时探针。
- H5-Level-B 真实项目工作流派发。

