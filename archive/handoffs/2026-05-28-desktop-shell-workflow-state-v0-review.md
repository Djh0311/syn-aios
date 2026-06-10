# 桌面壳工作流事实层 v0 最小读写总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-desktop-shell-workflow-state-v0.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-28-desktop-shell-workflow-state-v0.md`
- Handoff：`product-line/handoffs/2026-05-28-desktop-shell-workflow-state-v0-result.md`
- 被回收产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“工作流事实层 v0 最小读写底座已实现”。

不接受为“可编辑工作流完成”，不接受为“节点/边编辑完成”，不接受为“工作项状态转换或 review 登记完成”，也不接受为“索引候选可自动变成本地事实”。

依据：

- Rust 后端新增 `load_workflow_state_snapshot` 和 `initialize_workflow_state` 命令。
- 状态文件不存在时返回 `exists=false`、`initialized=false` 和空计数，不创建目录或文件。
- 初始化写入生成 `workflow_state_v0` 最小 JSON。
- 初始化写入包含 `workflow_state_initialized` audit event。
- 旧文件存在时先备份到 `backups/workflow-state.v0.<timestamp>.json`。
- 写入使用临时文件 `.workflow-state.v0.<timestamp>.tmp` 和 `fs::rename` 原子替换。
- 写入后重新读取并校验 `schema_version` 和 `workflow_version`。
- 前端新增 `WorkflowStatePanel`，显示路径、存在状态、schema / workflow version、对象数量和未初始化 warning。
- 前端初始化动作生成 `initialize-workflow-state` 待确认动作，确认弹层显示目标路径、来源和写入边界。
- 前端调用 `initializeWorkflowState()` 时不传路径，Rust 后端使用 `AppState.workflow_state_path` 计算真实路径。
- 总指导线复跑验证通过，真实状态文件仍不存在。

## 先说薄弱点

- 这轮已经引入真实应用数据目录写入能力，但总指导线没有执行真实初始化。依据：真实状态文件存在性检查返回不存在。
- 前端 `WorkflowStatePanel` 在 `workflowState` 为空时有默认路径展示。依据：组件里有兜底路径；但写入不依赖前端路径，后端初始化命令不接收路径参数。
- 当前 schema 校验是最小手写校验，不是完整 JSON Schema。依据：evidence 风险说明和 `validate_workflow_state` 实现。
- 当前没有文件锁，多窗口并发写入仍未解决。
- `workspace_id` 当前固定为 `workspace:yoyi-workspace`，不是最终稳定 hash。
- 初始化会覆盖旧状态文件；虽然会先备份，但还没有迁移、冲突处理和用户级 diff 预览。

## 接受内容

接受 Rust 后端能力：

- 读取 v0 状态文件。
- 缺文件返回空状态，不自动创建。
- 用户确认后可初始化 v0 状态文件。
- 初始化写入最小 schema。
- 写入前备份旧文件。
- 写入包含 audit event。
- 临时文件写入 + 原子替换。
- 写入后重新读取校验。

接受前端能力：

- 项目页显示“本地事实层 v0”面板。
- 显示状态文件路径。
- 显示 `exists=true/false`。
- 显示 schema version / workflow version。
- 显示 workflows / nodes / edges / reviews / audit events / harness resources 数量。
- 显示未初始化 warning。
- 初始化动作必须经过确认弹层。
- 确认弹层显示写入边界：只写 `workflow-state.v0.json` 和同目录 `backups`，不写 `.codex`、Codex 状态库或项目业务目录。

接受测试新增：

- `missing_workflow_state_returns_empty_without_creating_file`
- `initializes_workflow_state_with_audit_event`
- `existing_workflow_state_is_backed_up_before_initialize`

## 总指导线复跑验证

在 `product-line/prototypes/productized-desktop-shell/` 复跑：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，输出 `offline interaction tests passed: 3`。
- `npm run build` 通过。

在 `product-line/prototypes/productized-desktop-shell/src-tauri/` 复跑：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home \
CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target \
cargo test --offline
```

结果：

- 6 个 Rust 单测通过。

真实状态文件复核：

```bash
test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'
```

结果：

- 返回码 1，无输出。
- 判断为真实状态文件仍不存在。

端口复核：

```bash
lsof -nP -iTCP:5173 -sTCP:LISTEN
```

结果：

- 无监听输出。

## 安全和范围判断

接受当前安全边界。

依据：

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未写项目业务目录。
- 未读取或展示 auth、env、密钥、令牌、授权文件内容。
- 未读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未自动创建真实状态文件。
- 未绕过用户确认写状态文件。
- 未自动运行 harness。
- 未把索引候选自动升级成本地事实。
- 未接入非 Codex agent。
- 未做知识库、向量搜索、LM 调度或 release 打包。

## 当前状态

这条任务从“待派发”改为“已回收”。

当前可以说：

- 桌面壳具备工作流事实层 v0 JSON 的最小读写底座。
- 缺文件不会自动创建。
- 初始化写入必须经过前端确认。
- 写入路径被后端固定在 Tauri 应用数据目录。

仍不能说：

- 可编辑工作流完成。
- 节点编辑、边编辑、任务状态转换完成。
- review 登记完成。
- 候选 harness resource 可以自动变成本地事实。
- 真实 Tauri 窗口里已经验证 v0 面板和确认弹层。

## 下一步

下一步派给验证线：做真实 Tauri 窗口 smoke 验证。

验证重点：

- Tauri 窗口能读取 v0 状态。
- 缺真实状态文件时显示 `exists=false` 和未初始化 warning。
- 初始化按钮能打开确认弹层。
- 确认弹层显示目标路径、Tauri 应用数据目录和写入边界。
- 不点击确认执行，不创建真实状态文件。
- 验证后复核状态文件仍不存在。
- 验证后复核 5173 无监听残留。
