# Evidence: Unified Product Command Routing PCR9 Supervisor Acceptance Review v1

日期：2026-06-09

## 结论

全局主管复核结论：

```text
accepted_as_pcr9_specified_mario_test_codex_local_session_unified_product_command_level_b_real_probe_with_p2
```

PCR9 接受为：

- 指定项目 `/Users/yoyi/Documents/mario test` 的统一 Product Command Level B 真实探针完成。
- 指定 `codex-local` session `019e798a-ac37-7771-b982-e38084fcd22e` 的 read-only B1 和 workspace-write B2 真实 `resume` 探针完成。
- B1/B2 完成证据来自 `run_real_execution_product_command_phase_b_at` / product command attempt，不来自 legacy H5、direct CLI 或裸底层 continuation Phase B。
- B1 未写项目文件；B2 只写允许文件 `.workbench/pcr9/real-product-command-write-probe.md`。
- prompt 正文未持久化到 product sidecar、continuation sidecar 或 runtime log；运行产物只保留 summary/ref/hash 和 safe summary。

PCR9 不接受为：

- 任意项目自由执行完成。
- 通用真实 send / resume 产品化全部完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 自动 retry / stop / restart 产品化完成。
- 真实 Tauri / Browser / screenshot 验收完成。
- 最终蓝图完成。

## 复核对象

- PCR9 task：`tasks/2026-06-09-unified-product-command-routing-pcr9-level-b-real-probe-authorization-and-fixture-freeze-v1.md`
- PCR9 evidence：`evidence/2026-06-09-unified-product-command-routing-pcr9-level-b-real-probe-v1.md`
- PCR9 handoff：`handoffs/2026-06-09-unified-product-command-routing-pcr9-level-b-real-probe-v1-result.md`
- B1 run：`tmp/pcr9-real-product-command/runs/b1-run-1780975668202642000/`
- B2 run：`tmp/pcr9-real-product-command/runs/b2-run-1780975711294856000/`
- 只读复核线程：`019ea33a-23c4-7c10-8db3-95b8cf910fe7`

## 复核线结论

只读复核线最终结论为：带 P2 通过，无 P0/P1。

复核线建议主管线接受 PCR9 为“指定 `mario test` / 指定 `codex-local` session 的统一 Product Command Level B 真实探针完成”，并允许进入 PCR10 checkpoint。

复核线确认：

- B1/B2 均存在 product command attempt。
- B1/B2 均不是 legacy H5、direct CLI 或裸 continuation Phase B 完成证据。
- B1/B2 prompt canonical hash 与 evidence 一致。
- B1/B2 run sidecar、当前 `mario test` hash 与 evidence 一致。
- run 目录未命中 prompt 正文关键句。
- runtime log 是 redacted safe summary，未持久化 prompt 正文或原始 transcript。

## P0 / P1

无。

## P2

1. B1 product sidecar 在 `sandbox=read-only` 下仍记录 `allowed_write_roots` 为项目根。实际 `writes_project_files=false`，核心文件 hash 未变，不构成阻断。PCR10 需要补充口径：read-only sandbox 下该字段不等于项目写授权。
2. B1/B2 product attempt warnings 继承底层 continuation 标签，例如 `product_command:run_controlled_session_continuation_real_resume_phase_b`。外层 attempt 已是统一 product command，非 P1；PCR10 或后续命名收敛任务应降低误读风险。

## 主管线独立复核

主管线只读复核到 B1 product command Phase B attempt：

```text
product_command_id: real-exec-command:dispatch:pcr9-b1:mario-test:codex-dev:read-only-probe:v1
attempt_id: real-exec-command-attempt:phase-b:real-exec-command:dispatch:pcr9-b1:mario-test:codex-dev:read-only-probe:v1:4
status: phase_b_real_resume_executed
prompt_sent: true
real_codex_executed: true
writes_codex_home: true
writes_project_files: false
readback: succeeded
result_count: 1
marker: PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_READ_ONLY_OK_2026_06_09
```

主管线只读复核到 B2 product command Phase B attempt：

```text
product_command_id: real-exec-command:dispatch:pcr9-b2:mario-test:codex-dev:write-probe:v1
attempt_id: real-exec-command-attempt:phase-b:real-exec-command:dispatch:pcr9-b2:mario-test:codex-dev:write-probe:v1:4
status: phase_b_real_resume_executed
prompt_sent: true
real_codex_executed: true
writes_codex_home: true
writes_project_files: true
readback: succeeded
result_count: 1
marker: PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_WRITE_PROBE_OK_2026_06_09
```

主管线复核到 B2 项目写入仅为：

```text
/Users/yoyi/Documents/mario test/.workbench/pcr9/real-product-command-write-probe.md
```

当前 probe 文件 sha256：

```text
2dddea8eedf9cbbe56012742b0761b6b7d3290701512b33a3a58adff665386b2
```

## 项目核心文件 Hash 复核

主管线重新计算 `/Users/yoyi/Documents/mario test` 四个核心文件 hash，与 PCR9 evidence 一致：

```text
f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf  /Users/yoyi/Documents/mario test/index.html
6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f  /Users/yoyi/Documents/mario test/styles.css
814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd  /Users/yoyi/Documents/mario test/game.js
02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5  /Users/yoyi/Documents/mario test/README.md
```

结论：B1/B2 未修改核心项目文件。

## Prompt 持久化边界

主管线窄扫 `tmp/pcr9-real-product-command/runs`：

- 未命中 canonical prompt 正文关键句。
- 命中仅为 prompt summary、prompt ref、risk acknowledgement 等安全摘要字段。
- runtime log 明确记录 command 和 prompt body omitted。

结论：PCR9 运行证据支持“prompt body 只作为运行时输入，不持久化到 product sidecar / continuation sidecar / runtime log”。

## 验证沿用

PCR9 evidence 记录以下验证已通过：

```text
cargo fmt -- --check
cargo test --lib real_execution_command
cargo test --lib codex_local_runner
cargo test --lib session_continuation
cargo test --lib h5_project_dispatch_bridge
cargo test --lib runtime_log
cargo test --lib diagnostic
cargo test --lib workflow_authorization
cargo test --lib
npm run typecheck
npm run test:offline-interaction
npm run build
```

主管线本轮没有重跑真实 Codex，也没有重跑全量 npm/cargo；主管线做了只读 evidence / sidecar / hash / runtime log 复核。

## 边界确认

PCR9 真实探针执行阶段在用户明确“全部授权”后，确实触发真实 `codex exec resume` 并允许 Codex 原生运行时对 `/Users/yoyi/.codex` 做最小必要写入。

本轮主管验收阶段：

- 未再次执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex` 或 `.codex/plugins/cache`。
- 未读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout。
- 未启动 Browser / Chrome / Tauri / Vite / screenshot。
- 未同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`；这些保留给 PCR10 checkpoint。

## 后续

下一步允许进入 PCR10 checkpoint。PCR10 必须：

- 同步权威入口和计划口径。
- 记录 PCR9 带 P2 通过。
- 明确 PCR9 不是通用真实自动化完成。
- 明确 read-only `allowed_write_roots` 口径。
- 明确底层 continuation warning 标签仍是后续命名债。
