# Codex 工作台新版 UI 骨架 Tauri smoke 验证 evidence

任务包：`product-line/tasks/2026-05-28-codex-workbench-ui-shell-tauri-smoke-validation.md`

验证时间：2026-05-28

## 结论

新版 Codex 工作台 UI 骨架通过真实 Tauri 窗口 smoke 验证。

边界先说清楚：

- 这不是完整工作台发布版。
- 这不是 release 打包、签名、自动更新或托盘验证。
- 这不是 Finder / 剪贴板真实动作验证。
- 这不是完整端到端 UI 自动化；验证方式是 Tauri 窗口启动日志、macOS System Events 读取窗口文本、基础命令和清理复核的组合证据。

## 验证命令

在 `product-line/prototypes/productized-desktop-shell/` 执行：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 3`。
- `npm run build`：通过。

在 `product-line/prototypes/productized-desktop-shell/src-tauri/` 执行：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline
```

结果：

- `path_whitelist_accepts_only_index_projects_and_rollouts`：通过。
- `snapshot_keeps_metadata_without_session_body`：通过。
- `reads_real_static_index_summary`：通过。

启动 Tauri dev：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target npm run tauri:dev
```

结果：

- Vite ready：`http://127.0.0.1:5173/`
- Tauri 运行：`/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target/debug/codex-governance-workbench`
- 窗口标题：`Codex 治理工作台`
- 窗口尺寸：`1280, 820`

## Tauri 窗口是否读取索引

真实 Tauri 窗口中读取到：

- `Codex 工作台`
- `新版 UI 骨架`
- `四入口首页、项目级工作流、Skill 和 Harness 关系看板`
- `已读取索引。所有本机动作仍需用户点击并确认。`

没有停留在普通浏览器保护性失败状态：

- 未出现 `当前页面不在 Tauri 窗口中运行`

结论：真实 Tauri 窗口成功通过 Tauri invoke 读取静态索引，并渲染新版 UI 骨架。

## 首页 smoke

真实窗口首页读取到：

- 标题：`只保留四个入口`
- 四个入口按钮：
  - `Agent`
  - `项目`
  - `Skill 管理`
  - `Harness 管理`
- 首页入口文案：
  - `当前只显示 Codex`
  - `按最近活跃近似`
  - `索引候选`
  - `按最近修改近似`

未看到旧统计卡片作为首页主入口：

- 没有把项目数、会话数、skills 数、plugins 数作为首页四入口展示。

## Agent 页 smoke

切换 `Agent` 后，真实窗口读取到：

- `当前只编排 Codex`
- `Codex`
- `可用`
- `只读索引`
- `路径白名单`
- `权限确认弹层`
- `OpenClaw` / `未接入`
- `VS Code` / `未接入`
- `OpenCode` / `未接入`
- `Claude Code` / `未接入`
- `当前没有接入协议、健康检查、会话索引或可操作能力。`

结论：

- Codex 是唯一可用 agent。
- 未接入 agent 没有被写成可用能力。

## 项目页 smoke

切换 `项目` 后，真实窗口读取到：

- `项目列表`
- `项目详情`
- `打开目录`
- `复制路径`
- 左侧窄功能列表：
  - `工作流`
  - `会话`
  - `任务包`
  - `Handoff / Evidence`
  - `Skills`
  - `Harness`
  - `设置`
- 中间工作流画布：
  - `项目级工作流画布`
  - `项目中心`
  - `Codex 会话`
  - `Handoff`
  - `Director`
  - `Review`
  - `Evidence`
  - `Harness 候选`
  - `缺少数据`
- 右侧详情面板：
  - `详情面板`
  - `会话元数据`
  - `Handoff 候选`
  - `Evidence 候选`
  - `Harness 候选`

结论：

- 项目页能进入项目详情。
- 默认呈现工作流骨架。
- 三栏结构能在真实 Tauri 窗口中渲染。

## Skill 管理页 smoke

切换 `Skill 管理` 后，真实窗口读取到：

- `关系看板骨架`
- `只读 skill 和 plugin 元数据；不做删除、编辑或加载。`
- `分类`
- `Agent 使用关系`
- `Codex`
- `当前唯一可用 agent；可读取索引中的 skill 元数据。`
- `其他 agent`
- `未接入；不显示可用、加载、推荐或操作能力。`
- `项目使用关系`
- `项目映射缺失`
- `推荐关系占位`
- `不自动推荐`
- `来源和缺字段`
- `缺少字段`

说明：

- Skill 页会显示 skill 描述文本，这是索引元数据。
- 本轮没有发现会话正文、工具输出、命令输出或授权内容展示。

## Harness 管理页 smoke

切换 `Harness 管理` 后，真实窗口读取到：

- `框架看板骨架`
- `只显示候选入口和缺口；不自动运行 harness。`
- `框架 / 类型`
- `版本和来源`
- `版本字段缺失`
- `功能和场景`
- `场景无法可靠判断`
- `验证入口`
- `当前只展示候选路径；不自动运行，也不写验证状态。`
- `最近验证缺失`
- `项目适配`
- `来源和缺字段`
- `不是完整管理`
- `当前边界`
- `不自动运行 harness，不做多仓库多版本管理，不判断候选是否有用或废弃。`

结论：Harness 页是框架看板骨架，不自动运行 harness。

## 敏感内容和禁止事项扫描

执行源码扫描：

```bash
rg -n 'auth\.json|\.env|secret|token|authorization|first_user_message|payload\.content|stdout|stderr|raw_memories|MEMORY\.md|writeFile|child_process|exec\(|spawn\(|OpenClaw.*可用|VS Code.*可用|OpenCode.*可用|Claude Code.*可用|个人知识库|向量搜索|模型调度' product-line/prototypes/productized-desktop-shell/src product-line/prototypes/productized-desktop-shell/src-tauri/src product-line/prototypes/productized-desktop-shell/src-tauri/tauri.conf.json
```

命中说明：

- `spawn()`：后端 `pbcopy` 实现存在，但本轮没有点击确认执行，也没有执行复制动作。
- `auth.json`：只在 Rust 单测中作为非白名单拒绝样本字符串。
- `当前只治理 Codex；不做知识库、多 agent、向量搜索或模型调度。`：边界文案。

未发现：

- 读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容的代码路径。
- 读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文的代码路径。
- 把 OpenClaw / VS Code / OpenCode / Claude Code 写成可用能力的代码路径。

## 禁止事项核对

本轮未做：

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 未读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未读取系统剪贴板内容。
- 未执行 Finder 打开目录、Finder 定位 rollout、复制路径或 `pbcopy`。
- 未自动运行 harness。
- 未接入非 Codex agent。
- 未把 OpenClaw / VS Code / OpenCode / Claude Code 写成已可用能力。
- 未做个人知识库、向量搜索、模型调度。
- 未做 release 打包、签名、自动更新、托盘、通知或登录项。
- 未拉取外网依赖。

## 进程和端口清理

清理前本轮相关进程：

- `cargo-tauri dev`：PID 14182
- `vite --host 127.0.0.1`：PID 14359
- `codex-governance-workbench`：PID 14387

已定向清理上述 PID。

清理后复核：

- `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无输出。
- `ps` 只剩查询命令和 `rg` 自身，没有 `codex-governance-workbench`、`vite --host 127.0.0.1`、`cargo-tauri dev` 残留。

未留下临时验证文件。

## 风险

- 这次 smoke 依赖 macOS System Events 读取窗口文本，不是完整 DOM 级或截图级 UI 自动化。
- Skill 页显示 skill 描述元数据，虽然不是会话正文或工具输出，但仍属于较长文本，后续如要更严格的“摘要化 UI”，需要另开任务收敛。
- 项目页会显示索引内项目路径、handoff/evidence 路径和会话标题元数据；本轮按任务允许验证索引渲染，但不等同于敏感正文展示。
- 本轮没有点击 `打开目录`、`复制路径` 或任何确认执行按钮。
