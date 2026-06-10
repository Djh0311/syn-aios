# 任务包：工作流用户审核业务派发 v1

## 任务名

工作流用户审核业务派发 v1。

## 所属开发线

桌面应用线 / Codex 会话线 / 总指导线。

验证线按需复核。

## 当前判断

工作流已经证明可以真实写入一次测试项目，但还不是桌面壳正式业务派发能力。

依据：

- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-mario-test-project-real-execution-v1-result.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-mario-test-project-real-execution-v1.md`
- `/Users/yoyi/workspace/product-line/CURRENT.md`

大白话：

上一轮能成功，是因为总指导手工把 `-C /Users/yoyi --sandbox workspace-write` 加进了 `codex exec resume`。工作台自己还不会把这些权限参数变成用户可审核、后端可执行、结果可回收的业务派发。

## 薄弱点

- 当前后端正式 `execute_workflow_node_dispatch` 仍只允许 `safe_probe`，会拒绝真实业务 prompt。
- 用户审核业务指令虽然有协议字段，但没有真正接入后端执行路径。
- 权限参数还没有产品化：`execution_cwd`、`sandbox_mode`、`allowed_write_roots`、`--add-dir`。
- 前两次测试失败说明：只给目标路径不够，只改工作根也不够。
- 不能把上一轮 CLI 手动成功包装成工作台按钮已经可用。

## 目标

把 `user_reviewed_instruction` 真实业务派发接入桌面壳最小闭环：

1. 后端允许 `prompt_kind = "user_reviewed_instruction"` 进入真实派发路径。
2. 派发请求必须携带用户审核过的业务指令对象。
3. 指令对象必须包含：
   - `execution_cwd`
   - `sandbox_mode`
   - `allowed_write_roots`
   - `allowed_reads`
   - `allowed_writes`
   - `forbidden_actions`
   - `timeout_seconds`
   - `max_retries`
   - `required_return`
4. UI 确认弹层展示这些权限参数。
5. 后端调用 Codex 时能传入：
   - `-C <execution_cwd>`
   - `--sandbox <sandbox_mode>`
   - 必要时 `--add-dir <allowed_write_root>`
6. 执行结果写入：
   - `workflow_execution_controls[]`
   - `execution_attempts[]`
   - `permission_requests[]`，如有权限阻塞
   - `audit_events[]`
7. 回收时能区分：
   - 权限不足
   - 只读沙箱
   - 成功写入
   - 允许范围外写入
   - Codex 执行失败
   - 超时

大白话目标：

让工作台自己知道“这条真实任务要在哪里跑、能写哪里、用什么沙箱”，用户看完确认后，后端按这些参数派发，而不是总指导手工拼 CLI。

## 非目标

- 不做多 agent 调度。
- 不做复杂工作流画布。
- 不做项目团队工作区 v1 表达层。
- 不做 harness 自动运行。
- 不读取完整 transcript。
- 不读取 `auth.json`、`.env`、密钥、token、授权文件。
- 不把这一步说成真实业务自动编排全部完成。
- 不默认删除、移动、归档任何 Codex 会话。

## 允许读取

允许读取项目内：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-mario-test-project-real-execution-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-mario-test-project-real-execution-v1-result.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`

允许只读真实 workflow state 的必要结构：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

允许只读测试项目文件清单和本轮要验证的文件内容：

- `/Users/yoyi/codex-workflow-mario-test/index.html`
- `/Users/yoyi/codex-workflow-mario-test/styles.css`
- `/Users/yoyi/codex-workflow-mario-test/game.js`
- `/Users/yoyi/codex-workflow-mario-test/README.md`

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文

## 允许写入

允许写入项目内代码：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/styles.css`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`

允许写 evidence / handoff：

- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-user-reviewed-business-dispatch-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-v1-result.md`

允许更新当前权威和任务队列：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`

如执行真实业务派发验证，用户明确确认后允许写：

- `/Users/yoyi/.codex`，通过 `codex exec resume`
- `/Users/yoyi/codex-workflow-mario-test/`，仅限用户审核指令允许的测试文件
- 真实 workflow state 和 backups：
  - `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
  - `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

## 禁止事项

- 禁止未获用户确认就执行 `codex exec resume`。
- 禁止未获用户确认就写 `/Users/yoyi/.codex`。
- 禁止读取完整 transcript。
- 禁止读取授权、密钥、`.env`、token。
- 禁止修改 `/Users/yoyi/gameai/agent world`。
- 禁止修改业务测试项目允许范围外文件。
- 禁止运行 harness。
- 禁止联网安装依赖。
- 禁止使用 `--dangerously-bypass-approvals-and-sandbox`。
- 禁止把 CLI 手动验证说成桌面壳正式能力。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号的文本时必须使用单引号或 `rg -F`。

## 建议实现

### 后端

扩展派发请求类型：

- `prompt_kind`
- `user_reviewed_instruction`
- `execution_cwd`
- `sandbox_mode`
- `allowed_write_roots`
- `timeout_seconds`
- `max_retries`

扩展 runner：

- safe probe 继续走现有路径。
- `user_reviewed_instruction` 走新路径。
- 新路径必须根据审核字段构造 Codex 参数：
  - `codex exec`
  - `-C <execution_cwd>`
  - `--sandbox <sandbox_mode>`
  - `--add-dir <dir>`，按需多次
  - `resume <thread_id> -`

注意：

- 不要把业务 prompt 拼成 shell 字符串直接执行；优先通过参数数组和 stdin。
- 不要在状态里保存完整 transcript。
- 保存最终回复摘要、退出码、文件清单统计、warning 分类即可。

### 前端

在“派发指令 / 用户审核业务指令”区域展示：

- 执行目录
- 沙箱模式
- 允许写入根目录
- 允许读取范围
- 禁止事项
- 超时
- 最大重试次数
- 预期回传格式

确认弹层必须明确：

- 会执行 `codex exec resume`
- 会写 `/Users/yoyi/.codex`
- 会写哪些业务路径
- 不读取哪些敏感内容

### 回收分类

建议 warning / failure 分类：

- `target_path_not_writable`
- `sandbox_read_only`
- `allowed_write_roots_missing`
- `created_expected_files`
- `unexpected_files_created`
- `codex_resume_exit_nonzero`
- `plugin_auth_warning`
- `mcp_shutdown_warning`
- `timeout`

## 建议验证

先跑代码级验证：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --offline
python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json
```

再做一条真实验证，但必须先获得用户确认：

目标建议：

- 对 `/Users/yoyi/codex-workflow-mario-test/README.md` 做一个极小修改，例如追加一行 `Workflow dispatch smoke passed.`
- 允许写入范围只给 README。
- `execution_cwd = /Users/yoyi`
- `sandbox_mode = workspace-write`
- `allowed_write_roots = ["/Users/yoyi/codex-workflow-mario-test"]`

验证后必须回传：

- 是否执行 `codex exec resume`
- 是否写 `/Users/yoyi/.codex`
- 是否写真实 workflow state
- 是否只改了允许文件
- 是否读取敏感文件
- 最终回复摘要
- workflow state control / attempt / audit id

安全搜索要求：

- 搜索固定文本使用 `rg -F '固定文本' ...`。
- 搜索包含反引号的文本必须使用单引号或 `rg -F`。
- 禁止用 shell 双引号包住未转义反引号。

## 验收标准

必须满足：

- `safe_probe` 旧路径不回退。
- `user_reviewed_instruction` 能进入真实派发路径。
- UI 能让用户看清执行目录、沙箱和可写范围。
- 后端能把 `execution_cwd`、`sandbox_mode`、`allowed_write_roots` 传给 Codex。
- 权限不足 / 只读沙箱 / 成功写入 / 越界写入能被区别记录。
- 不保存完整 transcript。
- 不读取敏感文件。
- 代码验证通过。

真实验证只有在用户确认后才算验收的一部分；如果没有确认，只能验收代码能力和阻止态。

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
9. 验证命令和结果。
10. 新增 evidence / handoff。
11. 下一步建议。

## 总指导回收重点

总指导回收时必须判断：

- 是否把权限参数真正产品化，而不是仍靠手工 CLI。
- 是否保留 safe probe 的安全边界。
- 是否把真实业务 prompt 和用户确认绑定起来。
- 是否准确记录了 `/Users/yoyi/.codex` 和 workflow state 写入。
- 是否没有读取敏感文件和完整 transcript。
