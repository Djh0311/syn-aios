# Codex 工作台新版 UI 骨架 Tauri smoke 验证总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-codex-workbench-ui-shell-tauri-smoke-validation.md`
- 开发线：验证线
- Evidence：`product-line/evidence/2026-05-28-codex-workbench-ui-shell-tauri-smoke-validation.md`
- Handoff：`product-line/handoffs/2026-05-28-codex-workbench-ui-shell-tauri-smoke-validation-result.md`
- 被验证产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“新版 Codex 工作台 UI 骨架真实 Tauri 窗口 smoke 验证通过”。

不接受为“完整工作台发布版”，不接受为“release 打包验证”，不接受为“Finder / 剪贴板真实动作验证”，也不接受为“完整端到端 UI 自动化通过”。

依据：

- 验证线 evidence 记录真实 Tauri dev 窗口已启动，窗口标题为 `Codex 治理工作台`，尺寸为 `1280, 820`。
- 真实窗口文本显示 `已读取索引。所有本机动作仍需用户点击并确认。`。
- 真实窗口未出现 `当前页面不在 Tauri 窗口中运行`，说明不是普通浏览器保护性失败状态。
- 验证线用窗口文本级 smoke 方式覆盖首页、Agent、项目、Skill 管理、Harness 管理。
- 验证线记录未执行 Finder 打开、定位 rollout、复制路径或 `pbcopy`。
- 验证线记录清理了 `cargo-tauri dev`、`vite --host 127.0.0.1`、`codex-governance-workbench`，并复核 `5173` 无监听残留。
- 总指导线复跑 `npm run typecheck`、`npm run test:offline-interaction`、`npm run build`、`cargo test --offline` 均通过。
- 总指导线复核 `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无监听输出。

## 先说薄弱点

- 这不是完整 UI 自动化。依据：验证方式是 Tauri 启动日志、macOS System Events 窗口文本和命令验证组合，不是 DOM 级自动化，也不是截图级像素验收。
- 这没有验证真实系统动作。依据：验证线明确没有点击 `打开目录`、`复制路径` 或确认执行按钮，也没有执行 Finder、定位 rollout、`pbcopy` 或读取剪贴板。
- 这没有验证 release。依据：没有做打包、签名、自动更新、托盘、通知或登录项。
- 这没有证明可编辑自动化工作流完成。依据：Director、Review、边关系、状态机、任务包生成和回收写入仍没有完整数据模型和交互。
- Skill 页显示 skill 描述元数据。依据：验证线记录这是索引元数据，不是会话正文或工具输出；后续如果要更严格信息降噪，需要另开任务。
- 项目页显示索引内路径和会话标题元数据。依据：当前任务允许验证索引渲染；这不等于敏感红队测试已经完成。

## 接受的验证结果

接受以下 smoke 结论：

- Tauri dev 窗口能启动。
- Tauri 窗口能通过 invoke 读取当前静态索引。
- 页面没有停留在普通浏览器保护性失败状态。
- 首页显示四个入口：Agent、项目、Skill 管理、Harness 管理。
- 首页没有把项目数、会话数、skills 数、plugins 数作为统计卡片入口。
- Agent 页只把 Codex 显示为可用，OpenClaw / VS Code / OpenCode / Claude Code 均为未接入。
- 项目页能进入项目详情，显示左侧功能列表、中间工作流画布、右侧详情面板。
- 工作流骨架包含项目中心、Codex 会话、Handoff、Director、Review、Evidence、Harness 候选和缺少数据说明。
- Skill 管理页是关系看板骨架，明确只读 skill / plugin 元数据，不做删除、编辑或加载。
- Harness 管理页是框架看板骨架，明确不自动运行 harness，不写验证状态。
- 验证后无 5173 监听残留。

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

- 3 个 Rust 单测通过。

端口复核：

```bash
lsof -nP -iTCP:5173 -sTCP:LISTEN
```

结果：

- 无监听输出。

## 安全边界判断

接受当前安全边界。

依据：

- 没有写 `/Users/yoyi/.codex`。
- 没有改真实 Codex 状态库。
- 没有读取或展示授权文件内容、密钥、令牌、`.env` 内容。
- 没有读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 没有读取系统剪贴板。
- 没有执行 Finder 打开、定位 rollout、复制路径或 `pbcopy`。
- 没有自动运行 harness。
- 没有接入非 Codex agent。
- 没有把 OpenClaw / VS Code / OpenCode / Claude Code 写成可用能力。
- 没有做个人知识库、向量搜索、模型调度或 release 范围。
- 没有拉取外网依赖。

## 当前状态

这条验证任务从“待派发”改为“已回收”。

当前可以说：

- 新版 UI 骨架已完成桌面应用线实现。
- 新版 UI 骨架已完成真实 Tauri 窗口 smoke 验证。

仍不能说：

- 完整桌面发布版完成。
- 可编辑自动化工作流完成。
- Finder / 剪贴板真实动作验证完成。
- release 打包完成。
- 多 agent 接入完成。

## 下一步建议

下一步建议进入阶段 3 的最小工作流数据模型设计，不直接做复杂画布。

建议先由信息架构线或总指导线定义最小本地工作流模型：

- 项目。
- 角色：Director、Coder、Reviewer、Tester。
- 任务包。
- Handoff。
- Evidence。
- Review。
- 状态：待派发、执行中、等待用户、待回收、已接受、需修改、暂停。
- 节点关系和来源依据。

不建议马上做：

- 多 agent 接入。
- 知识库。
- 向量搜索。
- 自动运行 harness。
- 写 Codex 状态库。
