# 工作流可控执行协议 v1 result

## 结论

已完成最小协议能力：工作台能展示长任务、权限请求、失败 / 重试 / 超时 / 取消、用户审核业务指令预览。

这不是实际业务自动工作流完成。真实业务派发仍未开放。

## 改动文件

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/styles.css`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-controlled-execution-protocol-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-controlled-execution-protocol-v1-result.md`

## 新增或修改字段类型

- `workflow_execution_controls[]`
- `permission_requests[]`
- `execution_attempts[]`
- `user_reviewed_instruction`
- `timeout_seconds`
- `retry_count`
- `max_retries`
- `cancel_requested_at`
- `failure_reason`
- `audit_event_types`

新增状态：

- `waiting_for_permission`
- `retry_pending`
- `failed`
- `timed_out`
- `cancelled`

## 写入边界

- 是否写真实 workflow state：否。
- 是否写 `/Users/yoyi/.codex`：未发现成功写入依据；但自检命令意外触发过一次 Codex 状态库打开尝试，输出显示 readonly 失败。因禁止读取 `.codex`，没有进一步检查。
- 是否执行 `codex exec resume`：是，发生在末尾自检搜索命令中；原因是 shell 双引号里的反引号触发命令替换。
- 是否发送 Codex 消息：没有发送业务消息或 safe probe；依据是输出显示 `No prompt provided via stdin.`。
- 是否读取完整 transcript：否。
- 是否读取授权、密钥、`.env`、token：否。
- 是否运行 harness：否。

## 自检事故

- 一次 `rg` 自检命令写法错误，未转义反引号，导致 shell 意外执行 `codex exec resume`。
- 输出显示没有 stdin prompt，并且 `/Users/yoyi/.codex/state_5.sqlite` readonly 打开失败。
- 这违反任务禁止项；本 handoff 已按实际情况更正。

## 验证结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 3`。
- `npm run build`：通过。
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`：通过，58 passed，1 ignored。
- 末尾自检搜索命令：失败；反引号触发 shell 命令替换，意外调用 `codex exec resume`。

## 下一步建议

- 先决定是否需要把协议空队列写入真实 workflow state。
- 再设计第一条真实业务小步试跑的用户审核指令 schema。
- 真实试跑前要明确：允许读、允许写、禁止事项、权限队列、超时、取消、失败重试和回传字段。
