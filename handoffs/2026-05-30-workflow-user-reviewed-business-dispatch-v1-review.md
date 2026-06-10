# Review：工作流用户审核业务派发 v1

## 结论

需要修改。

不是方向错，而是当前代码能力还不能作为“用户审核业务派发 v1 完成”验收。

## 薄弱点

- 没有执行真实 `codex exec resume`，所以不能证明真实会话写入可用。
- 业务派发 readback 路径存在明显断点。
- 超时字段已进入 schema，但真实 runner 没有使用。
- 失败回收还没有把业务派发失败写入 `workflow_execution_controls[]` / `execution_attempts[]`。
- `npm run build` 生成了 `dist/` 产物，任务包没有列为允许写入；需要下轮决定保留、忽略还是补进任务口径。

## 主要问题

### 1. 业务派发 readback 会被自己拒绝

位置：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs:2523`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs:2530`

问题：

`read_workflow_node_dispatch_result_at` 从 dispatch 里读出 `prompt_kind`，但构造 `prepare_request` 时把 `user_reviewed_instruction` 固定为 `None`。

如果 dispatch 是 `user_reviewed_instruction`，后续 `workflow_node_dispatch_context` 会要求完整业务指令 payload，于是 readback 会报“用户审核模式缺少完整派发字段”。

影响：

真实业务派发完成后，用户点读回 / 回收可能失败。

判断：

这是验收阻塞。

### 2. 超时字段没有真实执行

位置：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs:590`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs:618`

问题：

`CodexResumeRequestOptions` 有 `timeout_seconds`，但 `RealCodexResumeRunner` 只是 `spawn` 后 `wait()`，没有超时、kill、状态记录。

影响：

任务包要求的超时回收还没成立。长任务可能一直挂住。

判断：

这是协议缺口。可以不阻塞“参数传递”小目标，但阻塞“可控执行协议闭环”验收。

### 3. 失败路径没有写业务 execution attempt 分类

位置：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs:3080`

问题：

`write_failed_dispatch` 只把 dispatch 标记为 failed 和写 audit event，没有为 `user_reviewed_instruction` 补写 `workflow_execution_controls[]` / `execution_attempts[]`，也没有把 `target_path_not_writable`、`sandbox_read_only` 等分类落账。

影响：

真实失败时，总指导看不到本阶段要求的结构化失败回收。

判断：

这是验收阻塞。

### 4. `dist/` 产物边界需要处理

位置：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/index.html`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/assets/index-CoLiWPD6.js`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/assets/index-BTACVauc.css`

问题：

任务包没有单列 `dist/` 为允许写入，但 `npm run build` 更新了产物。

影响：

这不是功能 bug，但属于任务边界不干净。

判断：

下轮必须决定：保留构建产物并补任务口径，或清理 / 忽略构建产物。

## 可以接受的部分

- 后端不再只允许 `safe_probe`，已接入 `prompt_kind = user_reviewed_instruction`。
- runner 使用参数数组和 stdin，没有把业务 prompt 拼成 shell 字符串。
- 业务派发会传 `-C`、`--sandbox`、重复 `--add-dir`。
- UI 展示执行目录、沙箱、允许写入根目录、读写范围、禁止事项、超时和回传字段。
- safe probe 旧路径看起来保留了，离线测试覆盖了不传业务权限参数。
- 本轮没有真实执行 `codex exec resume`，没有写 `/Users/yoyi/.codex`。

## 验证记录

开发线回传：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，3 个测试。
- `cargo fmt`：通过。
- `cargo test --offline`：通过，60 passed，1 ignored。
- `npm run build`：通过。
- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：通过。

总指导只读复核：

- 关键代码路径已读取。
- evidence / handoff 已读取。
- 发现 readback、timeout、失败分类和 dist 边界问题。

## 回收决定

本轮接受为：

- `user_reviewed_instruction` 业务派发代码路径初步接入。
- UI 权限参数展示初步接入。
- safe probe 旧路径未明显回退。

本轮不接受为：

- 用户审核业务派发 v1 完成。
- 真实业务派发闭环完成。
- 超时 / 失败 / 权限分类完成。

## 下一步

写修正任务包，目标是：

1. 修复业务派发 readback payload 丢失。
2. 让超时字段真实生效，或明确降级为未实现并从 v1 验收移出。
3. 失败路径写入 `workflow_execution_controls[]` / `execution_attempts[]`，并记录失败分类。
4. 处理 `dist/` 产物边界。
5. 修完后再决定是否做真实 README 极小修改验证。
