# 项目默认工作流草稿初始化 v1 证据

## 结论

薄弱点先说：

- 这轮只实现“项目默认工作流草稿初始化”，不是自动编排执行。依据：没有派发给真实 Codex 会话，没有生成任务包文件，没有状态转换，没有节点/边编辑。
- 默认节点和边是草稿骨架。依据：写入 `state=draft`，并通过 audit 记录用户确认创建。
- 如果状态文件不存在，本轮选择在用户确认创建项目工作流时同时创建最小 v0 状态，再写项目 workflow。依据：任务允许二选一，但必须记录清楚。
- 真实状态文件本轮没有被创建。依据：任务前和任务后都只做存在性检查，`test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'` 均返回不存在。

可接受点：

- 后端新增 `bootstrap_project_workflow` 命令。
- 命令只接受索引内项目路径，不接受任意输出路径。
- 后端从当前 `codex-index.json` 确认项目存在。
- 状态文件不存在时，用户确认的 bootstrap 写入会先构造最小 v0 状态，再写项目默认 workflow。
- 已有状态文件写入前会备份。
- 写入使用临时文件 + 原子替换。
- 写入后重新读取校验。
- 同一项目重复创建不会产生重复 workflow。
- 前端项目详情页显示当前项目是否已有本地工作流草稿、workflow / node / edge 数量和创建按钮。
- 创建按钮走确认弹层，说明写入工作台自己的状态文件，不写 `.codex`、不写 Codex 状态库、不写项目业务目录。

## 本轮读取依据

- `product-line/tasks/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- `product-line/handoffs/2026-05-28-desktop-shell-workflow-state-v0-result.md`
- `product-line/evidence/2026-05-28-desktop-shell-workflow-state-v0.md`
- `product-line/handoffs/2026-05-29-desktop-shell-workflow-state-v0-validation-review.md`
- `product-line/prototypes/productized-desktop-shell/`

没有读取或展示：

- `auth.json`
- `.env`
- 密钥、令牌、授权文件内容
- Codex 会话正文、工具输出、命令输出、输入历史、记忆正文
- 真实工作流状态文件内容

## 修改文件

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-result.md`

## 新增或修改了哪些测试

Rust 新增覆盖：

- 状态文件不存在时，用户确认 bootstrap 会初始化最小 v0 状态并写入项目 workflow。
- 为索引内项目创建默认 workflow。
- 同一项目重复创建不产生重复 workflow。
- 非索引项目被拒绝。
- 写入已有状态文件前会备份旧状态文件。

前端离线测试新增覆盖：

- 项目详情页显示“项目工作流草稿”区。
- 未创建时显示创建默认工作流草稿按钮。
- 点击按钮只打开待确认动作。
- 取消确认不会调用创建动作。
- 确认弹层显示目标路径和写入边界。

## 默认 workflow 写入了哪些对象

写入 `projects[]`：

- `project_id`
- `display_name`
- `root_path`
- `source_kind=codex_index`
- `permission_level=read_only`
- `created_at`
- `updated_at`
- `warnings`

写入 `workflows[]`：

- `workflow_id`
- `workflow_version=1`
- `project_id`
- `title`
- `state=draft`
- `source_kind=workspace_state`
- `permission_level=user_confirmed_write`
- `model_policy=none`
- `created_at`
- `updated_at`

写入 7 个 `nodes[]`：

- 总指导 / Director
- Codex 开发线
- 验证线
- 任务包
- Handoff
- Evidence
- Review

写入 6 个 `edges[]`：

- 总指导派发任务包：`decomposes_to`
- 任务包交给 Codex 开发线：`assigned_to`
- 开发线产出 handoff：`produces`
- 开发线产出 evidence：`produces`
- 验证线验证 evidence：`validates`
- Review 回收 handoff：`reviews`

写入 `audit_events[]`：

- `event_type=project_default_workflow_bootstrapped`
- `actor_ref=user_confirmed_desktop_shell`
- `permission_level=user_confirmed_write`
- `before_state=missing_project_workflow`
- `after_state=draft`

## 如何避免重复创建

默认 workflow id 由项目根路径生成：

- `workflow:<stable-project-root-id>:default`

写入前检查 `workflows[]` 是否已有同一 `workflow_id`。

如果已存在：

- 返回已有状态 snapshot。
- 不追加 workflow。
- 不追加 nodes。
- 不追加 edges。
- 不追加 audit event。
- 返回 `no-op:existing-workflow`。

## 如何保证只对索引内项目生效

Tauri 命令输入只接受项目路径。

后端处理：

- 读取当前 `codex-index.json`。
- 在 `projects[].project_root` 中查找输入路径。
- 找不到时拒绝，返回“项目不在当前索引内，已拒绝创建本地工作流草稿”。
- 不接受任意输出路径。
- 写入路径固定为工作台应用数据目录中的 `workflow-state.v0.json`。

## 验证

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`

真实状态文件存在性：

- 任务前：不存在。
- 任务后：不存在。

端口检查：

- `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无监听输出。

## 禁止事项检查

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未写项目业务目录。
- 未读取或展示 auth、env、密钥、令牌、授权文件内容。
- 未读取或展示会话正文、工具输出、命令输出、输入历史、记忆正文。
- 未自动运行 harness。
- 未把索引候选自动升级成已验证能力。
- 未接入非 Codex agent。
- 未做知识库、向量搜索、LM 调度。
- 未做 release 打包。
- 未拉取外网依赖。
- 未实现自动编排执行。

## 风险

- 默认节点和边仍是草稿骨架，不能代表真实任务执行状态。
- 当前 `stable_id` 是简单路径归一化，不是最终 hash；极端路径可能碰撞。
- 当前没有文件锁，多窗口并发写入需要后续处理。
- 后续做任务包生成或节点状态转换时，仍必须走用户确认、audit、备份和原子替换。
