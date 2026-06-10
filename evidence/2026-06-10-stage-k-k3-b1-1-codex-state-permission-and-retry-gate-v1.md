# Stage K / K3-B1.1 Codex State Permission And Retry Gate Evidence v1

日期：2026-06-10

状态：已完成，结论为 `accepted_product_classification_retry_gate`。

## 结论

K3-B1.1 选择并完成路径 C：产品侧失败分类修补。K3-B1 已失败记录不删除、不覆盖；本轮没有再次执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`。

本轮接受为：

- Codex 原生 state db readonly / permission denied 类失败可被产品代码识别为 `codex_state_error`。
- `codex_state_error` 在 H4 unknown-result 边界下保持 `result_count=null`，不会显示成 0 条结果。
- Product Command failure / stop / retry summary 可把 `codex_state_error` 作为一等失败项，要求重新用户确认。
- 智能体页和运行中工作流页能显示中文“Codex 状态不可写”。
- K3-B1 retry 路径仍必须单独决策和记录。

本轮不接受为：

- K3-B1 已完成。
- K3-Level-B 已完成。
- K3-B2 已开始。
- Stage K 已完成。
- 真实 retry 已执行。

## 背景

K3-B1 已执行但失败分类为 `failed_classified_codex_state_readonly`。产品路径进入真实 Phase B runner，`prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`、`writes_project_files=false`；但当前环境访问 `/Users/yoyi/.codex/state_5.sqlite` 时返回 readonly database，runner exit code 为 `1`，readback 为 `readback_failed` / `result_count=null`，last-message 文件未生成。

非沙箱重跑申请曾被安全审查拒绝；主管线不得绕过。

## 代码改动

- `src-tauri/src/codex_local_runner.rs`
  - 新增 Phase B `codex_state_error` 分类。
  - 识别 stderr / failure message 中的 state db readonly / permission denied。
  - failure reason code 归一为 `codex_state_error`。
  - 新增测试覆盖 state db readonly 且 `result_count=null`。
- `src-tauri/src/real_execution_command.rs`
  - Product Command Phase B attempt status 保留 `codex_state_error`。
  - failure / stop / retry summary 增加 `codex_state_error` 项。
  - 新增测试覆盖 summary 中的 `codex_state_error`。
- `src-tauri/src/h4_execution_boundary.rs`
  - `codex_state_error` 纳入 unknown-result 状态，结果数保持 `null`。
- `src/views/AgentView.tsx`
  - attempt / runtime attention / automation unit 中文展示“Codex 状态不可写”。
- `src/views/RunningWorkflowsView.tsx`
  - run queue、failure classification、runtime status 中文展示“Codex 状态不可写”。

## 验证

已通过：

- `cargo test --lib codex_local_runner`：12 passed。
- `cargo test --lib real_execution_command`：36 passed / 7 ignored。
- `cargo test --lib project_workflow_automation`：15 passed / 4 ignored。
- `cargo test --lib h4_execution_boundary`：0 matched tests，命令通过。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，14 passed。
- `npm run build`：通过，仅保留既有 Vite chunk size warning。

说明：`npm run build` 重新生成了 `dist` 构建产物。

## 边界确认

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送 K3-B1 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、auth、keychain、OAuth、provider credential、full transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / 截图工具。
- 未修改 workflow state JSON 顶层结构。

## 下一步

K3-B1 retry 仍需单独决策：

1. 用户在本机可写 Codex 环境手动执行 K3-B1 exact command 并回交输出。
2. 或主管线在新的明确授权和允许环境中再次申请真实执行。
3. 若安全审查仍拒绝，停止，不绕过。

K3-B1 retry 成功并经主管线复核前，不得进入 K3-B2。
