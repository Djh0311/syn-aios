# SYN-M5R07 control registry 全量矩阵回归返修窄包 v1

日期：2026-08-18
阶段：stage-14 / current leaf M5R07
基线候选：`ab5c46e`
写者：Grok `grok-4.6 --reasoning-effort high`
状态：REPAIR TASK / NOT ACCEPTANCE / NOT CLOSEOUT

## fresh 失败事实

候选 `ab5c46e` 的 detached disposable checkout 上执行：

```bash
cargo test --lib --offline m5_ -- --test-threads=1
```

结果 exit 101：179 passed / 1 failed。唯一失败：

`m5_runner_entry_registry::tests::control_commands_do_not_call_runtime_and_are_registered`

失败断言是 `assert!(!src.contains("run_admitted_workcell"))`。现有测试把整个 `production_prefix(m5_controlled_execution.rs)` 放进“control command 不得调用 runtime”的循环，但同文件的正式 runtime 必须且已经合法包含 `run_admitted_workcell`；同一测试模块的 `production_cannot_pass_caller_grant_to_runtime_execute` 也明确要求正式产品 runtime 保持在 admitted workcell。这个全文件断言扩大了 control 函数的源码边界，造成自相矛盾的假阳性。

原始日志：`/home/synadmin/workspace/.syn-gates/evidence/M5R07-ab5c46e/cargo-test-lib-offline-m5_.log`。

## 目标与写域

只修改：

- `prototypes/productized-desktop-shell/src-tauri/src/m5_runner_entry_registry.rs`

只修上述测试的源码边界：

- 保留 `load_m5_execution_control_with_state` 和 `apply_m5_execution_control_with_state` 两个 product command slice 的禁止 runtime 断言；
- 对 `m5_controlled_execution.rs`，若继续做静态断言，只能精确切出 control load/apply 函数本身，不能把包含正式 runtime 的整个 production prefix 当成 control 区段；
- 仍须证明 control load/apply 不调用 `run_admitted_workcell`、`run_authorized_workcell`、`run_m5_authorized_runtime_with_state` 或 `.execute(workcell`；
- 不删除或弱化对正式 runtime 必须走 admitted workcell、raw helper 保持 test-only 的既有独立测试；
- 不改任何产品执行语义、command registry、controlled execution、DTO、页面或其他文件。

不许 stage/commit，不许 reset/stash/clean，不许覆盖混合 WIP。

## 验证

使用仓外 target 运行：

```bash
cd /home/synadmin/workspace/syn/prototypes/productized-desktop-shell/src-tauri
CARGO_TARGET_DIR=/tmp/syn-m5r07-control-regression-target cargo test --lib control_commands_do_not_call_runtime_and_are_registered --offline
CARGO_TARGET_DIR=/tmp/syn-m5r07-control-regression-target cargo test --lib --offline m5_ -- --test-threads=1
```

必须报告真实 exit 与通过数；不得用 0 tests 或仅修改期望字符串掩盖真实 runtime 调用。
