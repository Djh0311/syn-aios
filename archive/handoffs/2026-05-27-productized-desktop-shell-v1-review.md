# 产品化桌面壳一期总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-productized-desktop-shell-v1.md`
- 开发线：桌面应用线
- 开发线交接：`product-line/handoffs/2026-05-27-productized-desktop-shell-v1-result.md`
- 开发线 evidence：`product-line/evidence/2026-05-27-productized-desktop-shell-v1.md`
- 产物目录：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“产品化桌面壳一期”。

不接受为“完整桌面发布版”。

依据：

- 任务包目标明确是把 Tauri 最小能力探针推进为 Codex 治理工作台产品化桌面壳一期，不要求 release 发布版。
- `product-line/prototypes/productized-desktop-shell/src-tauri/tauri.conf.json` 中 `bundle.active=false`。
- `product-line/prototypes/productized-desktop-shell/package.json` 中 `tauri:dev` 仍依赖 `../tauri-capability-probe/.tauri-cli/bin/cargo-tauri dev`。
- 本轮复跑通过 `npm run typecheck`、`npm run build`、`cargo test --offline`。

## 先说薄弱点

- UI 点击链条没有做稳定自动化复核。依据：开发线 evidence 明确写明没有做自动 UI 点击验证；总指导线本轮也没有启动 Tauri 窗口做按钮级复测。
- 剪贴板内容没有独立核验。依据：开发线 evidence 明确写明没有读取系统剪贴板内容；总指导线也没有读取剪贴板，避免带出无关敏感内容。
- Finder 打开目录和定位文件没有本轮重新做按钮点击验证。依据：开发线 handoff 把它列为仍不确定项。
- release 打包关闭。依据：`src-tauri/tauri.conf.json` 的 `bundle.active=false`。
- 原型运行链仍复用探针目录里的 Tauri CLI 和 cargo 缓存。依据：`package.json` 的 `tauri:dev` 脚本和 README 运行说明。
- 依赖体积需要后续治理。依据：本轮复查 `productized-desktop-shell/` 约 71M，`node_modules/` 约 70M，`dist/` 约 220K；cargo 缓存复用探针目录，不属于新壳主体但仍占用本机空间。
- 总指导线本轮无法用 `pgrep` 复核同名进程残留。依据：当前环境 `pgrep -fl codex-governance-workbench` 返回 `sysmond service not found`。

## 已核对内容

- 应用目录存在：`product-line/prototypes/productized-desktop-shell/`。
- 技术栈落地：Tauri 2、Rust、React、TypeScript、Vite。
- 应用名落地：`CodexGovernanceWorkbench`。
- 窗口标题落地：`Codex 治理工作台`。
- 页面落地：首页、项目、会话、Skills / Plugins、任务线 / evidence / handoff、诊断。
- 前端本机动作先进入 `PermissionDialog`。
- 后端命令在执行前重新从索引构建白名单。
- 打开项目目录只允许索引内 `projects[].project_root`。
- 定位 rollout 只允许索引内 `threads[].rollout_path`。
- 复制路径只允许索引内项目路径或 rollout 路径。

依据：

- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 总指导线复跑验证

在 `product-line/prototypes/productized-desktop-shell/`：

```bash
npm run typecheck
npm run build
```

结果：

- `npm run typecheck` 通过。
- `npm run build` 通过。
- 构建输出仍为 `dist/index.html`、`dist/assets/index-CWBKuO92.css`、`dist/assets/index-C-Jz40GM.js`。

在 `product-line/prototypes/productized-desktop-shell/src-tauri/`：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home \
CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target \
cargo test --offline
```

结果：

- 3 个 Rust 测试通过。
- 测试项包括路径白名单、snapshot 不读取会话正文、读取真实静态索引摘要。

残留检查：

- `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无监听输出。
- `pgrep -fl codex-governance-workbench` 在当前环境不可用，不能作为残留进程结论依据。

## 安全边界判断

接受当前安全边界。

依据：

- 开发线 evidence 明确禁止事项复核：未写 `/Users/yoyi/.codex`，未改 Codex 状态库，未展示密钥、正文、工具输出、输入历史，未运行 harness。
- 总指导线核对源码时只看到 `pbcopy` 和 `open` 两类本机动作，且动作路径来自索引白名单。
- 源码中测试出现 `/Users/yoyi/.codex/auth.json` 是拒绝非白名单路径的测试样例，不是读取该文件。

## 当前接收范围

接收：

- 阶段 2 的产品化桌面壳一期。
- 静态索引读取和展示。
- 路径白名单策略。
- 本机动作确认弹层。
- 项目目录打开、rollout 定位、路径复制的后端命令和基础测试。

不接收：

- 完整发布版。
- release 打包、签名、图标、自动更新。
- 稳定 UI 点击验收。
- 剪贴板内容独立核验。
- 个人知识库、多 agent、向量搜索、模型调度、复杂画布编排。

## 下一步建议

建议下一步不要立刻扩展知识库或多 agent。

更稳的下一步是派给验证线：产品化桌面壳一期 UI 行为验证。

验证线任务应只覆盖：

- Tauri 窗口正文能读取索引。
- 6 个页面能切换。
- 打开目录、复制路径、定位 rollout 的确认弹层出现。
- 后端拒绝非白名单路径。
- 不展示正文、密钥、`.env`、工具输出。
- 复核 5173 和 Tauri dev 进程无残留。

release 打包建议单独排到后面，先定义签名、图标、bundle 体积和权限提示标准。
