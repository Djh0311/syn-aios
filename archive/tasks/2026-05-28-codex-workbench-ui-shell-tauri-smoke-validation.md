# 任务包：Codex 工作台新版 UI 骨架 Tauri 窗口 smoke 验证

## 任务名

验证新版 Codex 工作台 UI 骨架在真实 Tauri 窗口中的渲染和安全边界。

## 所属开发线

验证线。

这是现有验证线任务，不新增常设开发线。

## 背景

桌面应用线已经完成新版 UI 骨架实现，并由总指导线回收为阶段 3 起点。

已知结论：

- 新版首页四入口已经落地。
- Agent 页只显示 Codex 可用和未接入 agent 空白位。
- 项目详情工作流骨架已经落地。
- Skill 管理和 Harness 管理看板骨架已经落地。
- 前端类型检查、离线交互测试、前端构建和 Rust 单测已通过。

薄弱点：

- 上一轮只做了普通 Vite 页面检查，普通浏览器不是 Tauri 窗口。
- 还没有证明真实 Tauri 窗口能读取索引并渲染新版 UI。
- 还没有真实窗口层面的导航和页面内容 smoke 证据。

依据：

- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-redesign-review.md`
- `product-line/evidence/2026-05-28-codex-workbench-ui-shell-redesign.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-redesign-result.md`
- `product-line/STAGE_PLAN.md`

## 目标

- 启动 `product-line/prototypes/productized-desktop-shell/` 的 Tauri dev 窗口。
- 验证真实 Tauri 窗口能读取当前静态索引。
- 验证首页只有四个入口：Agent、项目、Skill 管理、Harness 管理。
- 验证首页入口不显示数量。
- 验证 Agent 页只展示 Codex 可用，其他 agent 未接入，不显示假能力。
- 验证项目页能进入项目详情，默认呈现工作流骨架。
- 验证项目详情包含左侧窄功能列表、中间工作流画布、右侧详情面板。
- 验证 Skill 管理页是关系看板骨架。
- 验证 Harness 管理页是框架看板骨架。
- 验证页面没有展示敏感正文、密钥、授权内容、会话正文或工具输出。
- 验证完成后清理 Tauri / Vite / cargo-tauri 进程，并确认 5173 无监听残留。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-28-codex-workbench-ui-shell-redesign.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-redesign-review.md`
- `product-line/evidence/2026-05-28-codex-workbench-ui-shell-redesign.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-redesign-result.md`
- `product-line/prototypes/productized-desktop-shell/`
- `product-line/prototypes/index-kernel/codex-index.json`

## 允许写入

- `product-line/evidence/`
- `product-line/handoffs/`

如验证确实需要临时日志，只能写入：

- `/private/tmp`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/` 下的临时验证输出

验证结束后必须说明是否留下临时文件。

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不读取系统剪贴板内容。
- 不执行真实 Finder 打开目录、定位 rollout 或复制路径，除非任务执行前单独得到用户明确确认。
- 不自动运行 harness。
- 不接入非 Codex agent。
- 不把 OpenClaw / VS Code / OpenCode / Claude Code 写成已可用能力。
- 不做个人知识库、向量搜索、模型调度。
- 不做 release 打包、签名、自动更新、托盘、通知或登录项。
- 不为了验证拉取外网依赖；如果本地工具不足，要记录缺口。

## 建议验证方式

- 优先使用现有本地命令：
  - `npm run typecheck`
  - `npm run test:offline-interaction`
  - `npm run build`
  - `cargo test --offline`
  - `npm run tauri:dev`
- 如需 GUI 自动化，可用本机已有工具；不要安装网络依赖。
- 如果 Tauri 窗口难以自动抓取内部 DOM，可以用以下证据组合：
  - 启动日志。
  - 窗口进程存在。
  - 前端状态文本或截图证据。
  - 手工或脚本化导航结果。
  - 进程清理和端口复核。

## 验收标准

- 有 evidence 和 handoff。
- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- Tauri dev 窗口能启动。
- Tauri 窗口能读取索引，页面不停留在“当前页面不在 Tauri 窗口中运行”的保护性失败状态。
- 首页只有 Agent、项目、Skill 管理、Harness 管理四个入口。
- 首页入口不显示项目数、会话数、skills 数、plugins 数等统计卡片。
- Agent 页不把未接入 agent 写成可用能力。
- 项目页默认能看到工作流骨架和缺字段说明。
- Skill 管理和 Harness 管理都是看板骨架。
- 未展示敏感内容或正文类内容。
- 验证后无 5173 监听残留。
- 若某项无法自动验证，必须说明原因和替代证据，不得写成已完全通过。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增了哪些 evidence / handoff
4. 使用了哪些验证命令，结果是什么
5. Tauri 窗口是否真实启动
6. Tauri 窗口是否真实读取索引
7. 首页四入口和不显示数量如何验证
8. Agent、项目、Skill、Harness 四页如何验证
9. 是否触碰任何禁止事项
10. 是否留下临时文件或残留进程
11. 哪些仍不确定，风险是什么

## 总指导回收重点

回收时必须判断：

- 是否接受为新版 UI 骨架真实 Tauri 窗口 smoke 验证。
- 是否仍不能包装成完整工作台发布版。
- 是否没有展示正文或敏感内容。
- 是否没有执行未经确认的系统动作。
- 是否没有把未接入 agent 写成可用能力。
- 是否清理了进程并复核 5173。
