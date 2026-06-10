# Handoff: Stage H / H5-Level-B2 Mario Test Project Workflow Write Probe v1

日期：2026-06-08

## 结论

H5-Level-B2 已完成，接受为：

```text
accepted_as_h5_level_b2_single_project_workspace_write_real_dispatch_probe
```

证据：

- `evidence/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md`
- `evidence/2026-06-08-stage-h-h5-level-b2-supervisor-acceptance-review-v1.md`

## 结果摘要

- 真实执行：是。
- `prompt_sent`：true。
- `real_codex_executed`：true。
- `writes_codex_home`：true。
- 产品路径：通过工作台后端 continuation Phase B real runner / `RealCodexLocalPhaseBProcessRunner`，不是 direct CLI diagnostic。
- session：`019e798a-ac37-7771-b982-e38084fcd22e`。
- sandbox：`workspace-write`。
- 授权写入：只写 `/Users/yoyi/Documents/mario test/.workbench/h5-b2/real-dispatch-write-probe.md`。
- readback marker：`H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08`。
- 探针文件 sha256：`b3eaf99c09a786ab459721872f63bd7fd78f6e8dcd6d34b5e2c761103c5b69ae`。

## 关键 refs

```text
/Users/yoyi/workspace/product-line/tmp/h5-level-b2-real-dispatch/runs/run-1780892761807830000/workflow-state.v0.json
/Users/yoyi/workspace/product-line/tmp/h5-level-b2-real-dispatch/runs/run-1780892761807830000/session-continuations.v1.json
/Users/yoyi/workspace/product-line/tmp/h5-level-b2-real-dispatch/runs/run-1780892761807830000/runtime-logs.v1.json
/Users/yoyi/workspace/product-line/tmp/h5-level-b2-real-dispatch/runs/run-1780892761807830000/runtime/h2-phase-b/2026-06-08T01:31:00Z.becb82205578f5e1.last-message.txt
/Users/yoyi/Documents/mario test/.workbench/h5-b2/real-dispatch-write-probe.md
```

## 核心文件复核

`/Users/yoyi/Documents/mario test` 四个核心项目文件 hash 与 B1 主管复核记录一致：

```text
f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf  index.html
6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f  styles.css
814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd  game.js
02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5  README.md
```

## 过程偏差

开发线程已完成真实执行并落盘运行产物，但没有在开发线程内完成最终 evidence / handoff 回交。全局主管线基于落盘产物完成恢复式收口和复核，并补齐本 handoff。

复核还发现历史 sidecar 的 `command_preview` 仍带 `Level A preview only` 文案；主管线已修补后续 Phase B attempt 的 redacted command preview，避免真实执行记录继续继承 preview-only 字样。

## 验证

已通过：

```text
cargo test --lib session_continuation
cargo test --lib h5_project_dispatch_bridge
cargo test --lib codex_local_runner
cargo test --lib
rustfmt --check src/session_continuation_store.rs src/h5_project_dispatch_bridge.rs src/codex_local_runner.rs src/types.rs src/commands.rs
```

`cargo test --lib`：`257 passed; 0 failed; 5 ignored`。仅保留既有 `JsonRpcError::invalid_params` unused warning。

## 不接受范围

B2 不接受为 H5 通用产品化、H5 product command 正式化、H3-B 成功、H4-Level-B 真实失败 / 超时探针、自动重试、planned adapters 真实接入、provider/model verification、正式事实 / 正式记忆写入或阶段 H 完成。

## 下一步

建议不要继续拆 B3/B4 小探针。下一步应合并进入 H5 product command formalization / H5 acceptance checkpoint：把 B1 read-only 和 B2 workspace-write 证据收束为可复用产品 command、UI 权限入口和验收矩阵；仍不得直接声明 H5 通用产品化完成。
