# Tauri 最小能力验证

## 当前结果

已在本目录内安装本地 Tauri CLI，并创建最小 Tauri 探针。

薄弱点：

- 这不是完整桌面应用。
- 没有做产品化打包。
- 窗口正文已有补测证据，复制按钮调用链已有 UI 点击证据；打开目录和定位文件是等价 Finder 动作验证，不是稳定 UI 点击证据。
- 没有独立读取系统剪贴板内容，不能断言剪贴板内容已核验。
- 本地依赖和构建缓存占用数 GB。

## 已验证

- 本地 Tauri CLI：`.tauri-cli/bin/cargo-tauri`，版本 `tauri-cli 2.11.2`。
- Rust 后端测试：`cargo test --offline` 通过，3 tests OK。
- Rust 后端构建：`cargo build --offline` 通过。
- 前端脚本语法：`node --check ui/app.js` 通过。
- Tauri 调试进程可启动，系统事件能看到进程 `app` 和窗口标题 `Codex 治理工作台 Tauri 探针`。
- 窗口正文可读，显示项目 30、会话 296、Skills 50、Plugins 11、可打开项目路径 30、可定位 rollout 路径 296。
- UI 点击“复制路径”可触发后端调用，页面状态返回 `已复制：/Users/yoyi`。
- Finder 可打开索引内项目路径 `/Users/yoyi`。
- Finder 可定位索引内 rollout 文件。
- 调试进程验证后已关闭。

## 原型范围

原型文件：

- `ui/index.html`
- `ui/app.js`
- `ui/styles.css`
- `src-tauri/src/lib.rs`
- `src-tauri/tauri.conf.json`

后端只允许这些动作：

- 读取 `../../index-kernel/codex-index.json` 并返回摘要。
- 复制索引内项目路径或 rollout 路径。
- 打开索引内项目目录。
- 定位索引内 rollout 文件。

路径限制：

- 项目路径白名单来自索引里的 `projects[].project_root`。
- rollout 路径白名单来自索引里的 `threads[].rollout_path`。
- 不接受任意输入路径执行本机动作。

## 运行方式

从本目录运行：

```bash
CARGO_HOME="$PWD/.cargo-home" CARGO_TARGET_DIR="$PWD/src-tauri/target" ./.tauri-cli/bin/cargo-tauri dev --no-watch --no-dev-server-wait
```

或从 `src-tauri` 运行：

```bash
CARGO_HOME="../.cargo-home" CARGO_TARGET_DIR="./target" cargo run --offline
```

## 验证命令

```bash
./.tauri-cli/bin/cargo-tauri --version
node --check ui/app.js
cd src-tauri && CARGO_HOME="../.cargo-home" CARGO_TARGET_DIR="./target" cargo test --offline
cd src-tauri && CARGO_HOME="../.cargo-home" CARGO_TARGET_DIR="./target" cargo build --offline
```

## 未验证或未充分验证

- 没有独立读取系统剪贴板内容。读取剪贴板可能暴露无关敏感内容，权限请求被拒。
- 没有稳定通过 UI 点击验证“打开目录”“定位文件”；这两项已用索引路径执行等价 macOS 动作验证。
- 没有验证最终 `.app` 打包、签名、权限提示。
- 没有验证 release 模式。

## 安全边界

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示密钥、令牌、授权文件、`.env`。
- 不展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
