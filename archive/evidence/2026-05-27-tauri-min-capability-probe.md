# Tauri 最小能力验证 evidence

## 对象

- 任务包：`product-line/tasks/2026-05-27-tauri-min-capability-probe.md`
- 开发线：桌面应用线
- 原型目录：`product-line/prototypes/tauri-capability-probe/`

## 这轮做了什么

- 读取任务包和上游路线决策。
- 检查本机 Rust / Cargo / Node / npm / Tauri CLI 可用性。
- 确认当前工具链缺 Tauri CLI。
- 因任务包禁止安装网络依赖，本轮没有创建可运行 Tauri 原型。
- 写入缺口说明和后续验证范围。

## 工具链检查结果

可用：

- `rustc --version`：`rustc 1.95.0 (59807616e 2026-04-14)`
- `cargo --version`：`cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- `node --version`：`v23.11.0`
- `npm --version`：`10.9.2`

路径：

- `rustc`：`/Users/yoyi/.cargo/bin/rustc`
- `cargo`：`/Users/yoyi/.cargo/bin/cargo`
- `node`：`/opt/homebrew/Cellar/node/23.11.0/bin/node`
- `npm`：`/opt/homebrew/Cellar/node/23.11.0/bin/npm`

不可用：

- `cargo tauri --version`：失败，Cargo 报 `no such command: tauri`。
- `cargo --list`：已安装命令列表中没有 `tauri`。

npm 路径：

- `npm config get registry`：`https://registry.npmmirror.com`
- `npm exec tauri -- --version`：尝试请求 `https://registry.npmmirror.com/tauri`，因 `getaddrinfo ENOTFOUND registry.npmmirror.com` 失败。

## 是否创建最小 Tauri 原型

没有。

依据：

- 任务包明确禁止安装网络依赖。
- 当前没有可用 Tauri CLI。
- 当前 `product-line` 下没有已有 `package.json`、`node_modules` 或 Tauri 工程可复用。
- 生成不可运行骨架不能验证窗口加载、打开文件夹、定位日志或剪贴板能力。

## 静态索引可提供的验证路径

检查 `product-line/prototypes/index-kernel/codex-index.json` 后，确认索引中已有可供后续验证的路径元数据：

- 示例项目路径：`/Users/yoyi/workspace`
- 示例 rollout 路径：`/Users/yoyi/.codex/sessions/2026/05/27/rollout-2026-05-27T03-08-59-019e65b0-b393-7501-b37c-6c61738fe4f8.jsonl`
- 索引生成时间：`2026-05-27T10:23:52Z`

说明：

- 本轮没有读取 rollout 文件正文。
- 本轮没有打开这些路径。
- 这些路径只用于证明索引里有后续能力验证所需的路径元数据。

## 验证了哪些桌面能力

未验证真实桌面能力。

原因：

- 缺 Tauri CLI，无法启动 Tauri 窗口。
- 任务包禁止安装缺失依赖。

已验证的只是前置条件：

- Rust / Cargo 可用。
- Node / npm 可用。
- Tauri CLI 不可用。
- npm 方式会触发联网获取依赖，当前网络/DNS 不可用。

## 安全边界

本轮符合任务包安全边界：

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件。
- 未展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未自动运行 harness。
- 未做完整产品化打包。
- 未安装网络依赖。
- 未把能力验证写成最终桌面应用已完成。

## 缺口

- 缺 Tauri CLI。
- npm registry 当前指向 `https://registry.npmmirror.com`，本轮 DNS 失败。
- 未验证 macOS 打开文件夹、定位文件、剪贴板动作。
- 未验证 Tauri 权限配置。
- 未验证加载现有静态 UI。

## 安装后第一条验证命令

如果后续允许安装 Tauri CLI，安装后第一条验证命令应是：

```bash
cargo tauri --version
```

如果选择 npm 方式，则先确认 registry 可访问，再运行：

```bash
npm exec tauri -- --version
```

## 下一步建议

- 先决定是否允许安装 Tauri CLI 及其依赖。
- 如果允许，下一轮只做最小工程，不做完整产品化打包。
- 最小工程的路径动作必须只接受索引内已有路径，不接受任意输入路径。
- 原型命令应显式区分：打开项目目录、复制路径、定位 rollout 文件所在位置。
