# Stage K / K3-B1.1 Codex State Permission And Retry Gate v1

日期：2026-06-10

状态：已完成，结论为 `accepted_product_classification_retry_gate`。本文是 K3-B1 失败后的环境 / 权限 / retry gate；本轮选择路径 C：先做产品侧失败分类修补，不直接执行真实 Codex，不发送 prompt，不读写 `/Users/yoyi/.codex`。K3-B1.1 的目标是决定如何安全重跑 K3-B1，而不是绕过安全审查。

## 1. 背景

K3-B1 已执行但失败分类为 `failed_classified_codex_state_readonly`：

- 产品路径进入真实 Phase B runner。
- `prompt_sent=true`。
- `real_codex_executed=true`。
- `writes_codex_home=true`。
- runner exit code 为 `1`。
- readback 为 `readback_failed`，`result_count=null`。
- last-message 文件未生成。
- `/Users/yoyi/Documents/mario test` 四个核心文件 hash 前后一致。

失败原因是当前执行环境访问 `/Users/yoyi/.codex/state_5.sqlite` 时遇到 readonly database。

非沙箱重跑申请被安全审查拒绝；主管线不得绕过。

## 2. 目标

- 明确 K3-B1 retry 的可接受执行方式。
- 确认 `.codex` 写入权限和风险告知。
- 决定是由用户手动执行、由工作台未来 UI 执行，还是由主管线在新的明确授权和允许环境中执行。
- 如果需要产品修补，先新增 runner failure classification，而不是再次真实执行。

## 3. 可选路径

### 路径 A：用户手动执行

用户在本机可写 Codex 环境中手动运行 K3-B1 exact command，并把输出 / run dir / hash 结果交回主管线复核。

要求：

- 使用 K3-B1 任务包中的 exact env 和 test。
- 不改 prompt。
- 不改项目文件。
- 输出必须包含 exit code、last-message path、readback marker、sidecar path。

### 路径 B：再次申请主管线真实执行

只有在用户明确接受以下风险后才可申请：

- 会发送 K3-B1 prompt 到真实 Codex runner。
- 会写 `/Users/yoyi/.codex`。
- 可能产生 Codex 原生日志和状态副作用。
- 失败仍按分类记录，不包装成成功。

若安全审查仍拒绝，立即停止。

### 路径 C：先做产品修补

在不执行真实 Codex 的情况下，补 runner failure classification：

- 将 Codex state readonly / permission denied 从 generic `runner_failed` 分类为更明确的 `codex_state_error`。
- 保持 readback failed / unavailable 的 `result_count=null`。
- UI / runtime log 显示“Codex 原生状态不可写，需要用户在可写环境重试”。

该路径不完成 K3-B1，只改善失败体验。

本轮已选择并完成路径 C：

- `CodexLocal` Phase B runner 现在会把 `state db` readonly / permission denied 类失败分类为 `codex_state_error`。
- H4 unknown result 边界把 `codex_state_error` 保持为 `result_count=null`。
- Product Command failure / stop / retry summary 将 `codex_state_error` 作为一等失败项显示，要求重新用户确认。
- 智能体页和运行中工作流页新增中文展示“Codex 状态不可写”。
- 不重跑 K3-B1，不发送 prompt，不读写 `/Users/yoyi/.codex`。

## 4. 禁止

- 禁止绕过 Product Command 直接跑裸 CLI。
- 禁止为了通过测试修改 readback marker 或 prompt。
- 禁止把 K3-B1 已失败记录删除或覆盖。
- 禁止在未经明确风险确认时再次发送 prompt。
- 禁止进入 K3-B2。

## 5. 验收

K3-B1.1 可接受为：

- retry 路径已明确；
- 风险边界已记录；
- 如果选择产品修补，测试通过且不执行真实 Codex；
- 如果选择真实 retry，必须另有 K3-B1 retry evidence / handoff。

K3-B1.1 不接受为：

- K3-B1 已完成；
- K3-Level-B 已完成；
- K3-B2 已开始；
- Stage K 已完成。

## 6. 记录

- Evidence：`../evidence/2026-06-10-stage-k-k3-b1-1-codex-state-permission-and-retry-gate-v1.md`
- Handoff：`../handoffs/2026-06-10-stage-k-k3-b1-1-codex-state-permission-and-retry-gate-v1-result.md`
