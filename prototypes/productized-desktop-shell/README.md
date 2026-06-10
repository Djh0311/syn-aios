# Codex 治理工作台产品化桌面壳一期

## 定位

这是阶段 2 的产品化桌面壳一期，不是完整发布版。

应用名：

- `CodexGovernanceWorkbench`
- 窗口标题：`Codex 治理工作台`

## 技术栈

- Tauri 2
- Rust
- React
- TypeScript
- Vite

## 运行

先安装前端依赖：

```bash
npm install
```

开发模式：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home \
CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target \
npm run tauri:dev
```

说明：

- 当前复用 `tauri-capability-probe` 的本地 Tauri CLI 和 cargo 缓存。
- release bundle 关闭，未做完整产品化打包。

## 已实现页面

- 首页总览
- 项目页
- 会话页
- Skills / Plugins 页
- 任务线 / evidence / handoff 页
- 诊断页

## 本机动作

动作都必须由用户点击触发，并先显示权限确认弹层：

- 打开索引内项目目录。
- 复制索引内项目路径或 rollout 路径。
- 定位索引内 rollout 文件。

后端会再次检查路径白名单。

## 安全边界

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件。
- 不展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不接受任意用户输入路径执行本机动作。
- 不做个人知识库、多 agent、向量搜索、模型调度、复杂画布编排。

## 验证

```bash
npm run typecheck
npm run build
cd src-tauri
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home \
CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target \
cargo test --offline
```
