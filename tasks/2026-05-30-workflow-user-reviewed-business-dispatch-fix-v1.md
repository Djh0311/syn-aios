# 任务包：工作流用户审核业务派发修正 v1

## 任务名

工作流用户审核业务派发修正 v1。

## 所属开发线

桌面应用线 / Codex 会话线。

总指导线回收，验证线按需复核。

## 当前判断

上一轮 `workflow-user-reviewed-business-dispatch-v1` 不能验收为完成。

依据：

- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-v1-review.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-user-reviewed-business-dispatch-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-v1-result.md`

大白话：

业务派发“能跑起来”的代码路径已经有了，但读回、超时、失败分类还没闭合。现在要修这些洞，不急着真实派发。

## 薄弱点

- 业务派发 readback 目前会丢 `user_reviewed_instruction` payload，导致读回时自己拒绝。
- `timeout_seconds` 进入了请求，但真实 runner 没有使用。
- 失败路径没有写 `workflow_execution_controls[]` / `execution_attempts[]`。
- 失败 warning 还没有分成 `sandbox_read_only`、`target_path_not_writable` 等结构化分类。
- `npm run build` 会写 `dist/`，上一任务包没有声明；这轮要明确处理。

## 目标

修正用户审核业务派发 v1 的闭环缺口：

1. 修复业务派发 readback：
   - 从 dispatch 里恢复 `user_reviewed_instruction`。
   - 不再因为 readback 缺 payload 拒绝业务派发结果。
2. 处理超时：
   - 优先让 `timeout_seconds` 真实生效。
   - 如果本轮无法安全实现真实 kill/timeout，必须明确从 v1 验收移出，并在 UI / evidence 里标为未实现，不得继续假装完成。
3. 修复失败路径：
   - `user_reviewed_instruction` 失败时也写 `workflow_execution_controls[]`。
   - 写 `execution_attempts[]`，状态为 `failed` 或 `timed_out`。
   - 写 failure reason 和 warning 分类。
4. 增加失败分类：
   - `sandbox_read_only`
   - `target_path_not_writable`
   - `allowed_write_roots_missing`
   - `codex_resume_exit_nonzero`
   - `codex_resume_spawn_failed`
   - `timeout`
5. 处理 `dist/`：
   - 如果运行 `npm run build`，必须在 evidence / handoff 说明 `dist/` 是否变更。
   - 如果 `dist/` 是预期构建产物，任务包允许写入 `dist/`。
   - 不要把 `dist/` 变化藏起来。

## 非目标

- 不执行真实业务派发。
- 不执行真实 `codex exec resume`，除非用户另行明确批准。
- 不写 `/Users/yoyi/.codex`。
- 不修改 `/Users/yoyi/codex-workflow-mario-test`。
- 不修改 `/Users/yoyi/gameai/agent world`。
- 不读取完整 transcript。
- 不读取 `auth.json`、`.env`、密钥、token、授权文件。
- 不运行 harness。
- 不做多 agent 调度。
- 不做项目团队工作区 v1。

## 允许读取

允许读取项目内：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-v1-review.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-user-reviewed-business-dispatch-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-v1-result.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/`，仅用于判断构建产物边界
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`

允许只读真实 workflow state 的必要结构：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文

## 允许写入

允许写入代码：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`

允许写构建产物：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/`

说明：

`dist/` 只有在执行 `npm run build` 时允许变化；必须在回传里说明。

允许写 evidence / handoff：

- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-user-reviewed-business-dispatch-fix-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-fix-v1-result.md`

允许更新：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`

## 禁止事项

- 禁止未获用户确认就执行真实 `codex exec resume`。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止读取完整 transcript。
- 禁止读取授权、密钥、`.env`、token。
- 禁止修改 `/Users/yoyi/codex-workflow-mario-test`。
- 禁止修改 `/Users/yoyi/gameai/agent world`。
- 禁止运行 harness。
- 禁止联网安装依赖。
- 禁止使用 `--dangerously-bypass-approvals-and-sandbox`。
- 禁止把未真实验证的能力说成真实业务闭环已完成。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号的文本时必须使用单引号或 `rg -F`。

## 验收标准

必须满足：

- 业务派发 readback 不因 payload 缺失失败。
- 失败路径对 `user_reviewed_instruction` 写入 execution control 和 attempt。
- 失败 warning 至少覆盖只读沙箱 / 目标不可写 / exit nonzero / spawn failed。
- 超时要么真实生效，要么明确降级为未实现并从 v1 完成口径移出。
- safe probe 路径不回退。
- 不保存完整 transcript。
- 不读取敏感文件。
- `dist/` 变化有说明。

## 建议验证

代码验证：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
cargo fmt
cargo test --offline
python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json
rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md
```

测试建议：

- 增加 Rust 离线测试：业务派发 readback 使用 dispatch 内 payload。
- 增加 Rust 离线测试：业务派发失败写 execution attempt。
- 增加 Rust 离线测试：safe probe 不受业务权限参数影响。
- 增加前端离线测试：确认弹层仍显示执行目录、沙箱和可写根。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 做了什么。
3. 改了哪些文件。
4. 是否执行真实 `codex exec resume`。
5. 是否写 `/Users/yoyi/.codex`。
6. 是否写真实 workflow state。
7. 是否读取敏感文件或完整 transcript。
8. 是否修改 `/Users/yoyi/codex-workflow-mario-test`。
9. `dist/` 是否变化，以及如何处理。
10. 验证命令和结果。
11. 新增 evidence / handoff。
12. 下一步是否可以进入真实 README 极小修改验证。

## 总指导回收重点

总指导回收时必须判断：

- readback 断点是否真的修掉。
- 失败路径是否真的落账。
- 超时是否真实实现，还是被明确降级。
- `dist/` 产物是否被诚实处理。
- 是否仍然没有读取敏感文件或完整 transcript。
