# Codex 工作台新版 UI 骨架 Tauri smoke 验证 handoff

任务包：`product-line/tasks/2026-05-28-codex-workbench-ui-shell-tauri-smoke-validation.md`

回收时间：2026-05-28

## 回收结论

验证线任务已完成。新版 Codex 工作台 UI 骨架可以接受为“真实 Tauri 窗口 smoke 验证通过”。

边界：

- 不能包装成完整工作台发布版。
- 不能包装成 release 打包验证。
- 不能包装成 Finder / 剪贴板真实动作验证。
- 不能包装成完整端到端 UI 自动化。

## 做了什么

1. 复跑基础命令：
   - `npm run typecheck`
   - `npm run test:offline-interaction`
   - `npm run build`
   - `cargo test --offline`
2. 启动真实 Tauri dev 窗口：
   - Vite 监听 `127.0.0.1:5173`。
   - Tauri 进程 `codex-governance-workbench` 启动。
   - 窗口标题为 `Codex 治理工作台`。
3. 用 macOS System Events 读取真实窗口文本：
   - 证明窗口读取索引成功。
   - 证明窗口没有停在普通浏览器保护性失败状态。
   - 验证首页、Agent、项目、Skill 管理、Harness 管理 smoke 内容。
4. 扫描敏感内容和禁止项。
5. 清理 Tauri / Vite / cargo-tauri 进程并复核 5173。

## 改了哪些文件

没有修改产品源码。

新增：

- `product-line/evidence/2026-05-28-codex-workbench-ui-shell-tauri-smoke-validation.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-tauri-smoke-validation-result.md`

## 验证结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，输出 `offline interaction tests passed: 3`
- `npm run build`
- `cargo test --offline`，3 个 Rust 单测通过

真实 Tauri 窗口：

- 已启动。
- 标题：`Codex 治理工作台`
- 尺寸：`1280, 820`
- 页面显示：`已读取索引。所有本机动作仍需用户点击并确认。`
- 未出现：`当前页面不在 Tauri 窗口中运行`

## UI smoke 结果

首页：

- 显示 `只保留四个入口`。
- 四入口为 `Agent`、`项目`、`Skill 管理`、`Harness 管理`。
- 首页入口不以项目数、会话数、skills 数、plugins 数作为统计卡片展示。

Agent 页：

- Codex 显示 `可用`。
- OpenClaw / VS Code / OpenCode / Claude Code 都显示 `未接入`。
- 未接入 agent 文案为没有接入协议、健康检查、会话索引或可操作能力。

项目页：

- 能进入项目详情。
- 默认呈现工作流骨架。
- 包含左侧窄功能列表、中间 `项目级工作流画布`、右侧 `详情面板`。
- 工作流包含项目中心、Codex 会话、Handoff、Director、Review、Evidence、Harness 候选和缺少数据说明。

Skill 管理页：

- 是 `关系看板骨架`。
- 显示分类、Agent 使用关系、项目使用关系、推荐关系占位、来源和缺字段。
- 明确只读 skill / plugin 元数据，不做删除、编辑或加载。

Harness 管理页：

- 是 `框架看板骨架`。
- 显示框架 / 类型、版本和来源、功能和场景、项目适配、来源和缺字段。
- 明确不自动运行 harness，不写验证状态。

## 禁止事项状态

未触碰：

- 没写 `/Users/yoyi/.codex`。
- 没改真实 Codex 状态库。
- 没读取或展示授权文件内容、密钥、令牌、`.env` 内容。
- 没读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 没读取系统剪贴板。
- 没执行 Finder 打开、定位 rollout、复制路径或 `pbcopy`。
- 没自动运行 harness。
- 没接入非 Codex agent。
- 没把 OpenClaw / VS Code / OpenCode / Claude Code 写成可用能力。
- 没做个人知识库、向量搜索、模型调度或 release 范围。
- 没拉取外网依赖。

## 清理状态

已清理本轮启动的：

- `cargo-tauri dev`
- `vite --host 127.0.0.1`
- `codex-governance-workbench`

最终复核：

- `5173` 无监听残留。
- 无同名 Tauri / Vite / cargo-tauri 进程残留。
- 未留下临时验证文件。

## 风险

- System Events 只能作为窗口文本级 smoke 证据，不是完整 UI 自动化。
- Skill 页显示 skill 描述元数据，后续如果要求更严格的信息降噪，需要单独收敛。
- 项目页显示索引内路径和会话标题元数据；本轮未发现正文或工具输出，但也不是敏感红队测试。
