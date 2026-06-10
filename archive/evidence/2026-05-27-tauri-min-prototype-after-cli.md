# Tauri CLI 后最小原型验证 evidence

## 对象

- 任务包：`product-line/tasks/2026-05-27-tauri-min-prototype-after-cli.md`
- 开发线：桌面应用线
- 原型目录：`product-line/prototypes/tauri-capability-probe/`
- 记录时间：2026-05-27 20:59:13 CST

## 先说薄弱点

- 这不是完整桌面应用。依据：没有做产品化打包、签名、发布包、系统托盘、自动更新。
- 窗口创建有证据，但窗口正文和按钮点击没有拿到稳定自动化证据。依据：系统事件能看到窗口标题，但读取窗口正文时遇到辅助功能权限或窗口索引不稳定。
- 本机动作后端已实现路径白名单，但没有通过真实按钮点击验证复制、打开目录、定位文件。依据：测试覆盖白名单和索引摘要，不覆盖 UI 点击。
- 依赖面很大。依据：本地 `.cargo-home` 约 478M，`.cargo-target` 约 1.3G，`src-tauri/target` 约 1.9G。

## 这轮做了什么

- 新增 Tauri CLI 后最小原型任务包。
- 在原型目录内安装本地 Tauri CLI，不做全局安装。
- 初始化最小 Tauri 工程。
- 新增静态 UI。
- 新增 Rust 后端命令：
  - `load_probe_summary`
  - `copy_indexed_path`
  - `open_indexed_project`
  - `reveal_indexed_rollout`
- 后端只从索引内提取项目路径和 rollout 路径作为白名单。
- 配置 `withGlobalTauri`，让前端可以调用 Tauri 命令。
- 运行测试、构建和窗口启动验证。

## 工具链和依赖

已安装：

- 本地 Tauri CLI：`product-line/prototypes/tauri-capability-probe/.tauri-cli/bin/cargo-tauri`
- 版本：`tauri-cli 2.11.2`

安装方式：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo install tauri-cli --locked --root /Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.tauri-cli
```

安装结果：

```text
Installed package `tauri-cli v2.11.2` (executable `cargo-tauri`)
```

依赖体积：

```text
35M   product-line/prototypes/tauri-capability-probe/.tauri-cli
478M  product-line/prototypes/tauri-capability-probe/.cargo-home
1.3G  product-line/prototypes/tauri-capability-probe/.cargo-target
1.9G  product-line/prototypes/tauri-capability-probe/src-tauri/target
```

## 改了哪些文件

- `product-line/tasks/2026-05-27-tauri-min-prototype-after-cli.md`
- `product-line/tasks/README.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/prototypes/tauri-capability-probe/.gitignore`
- `product-line/prototypes/tauri-capability-probe/README.md`
- `product-line/prototypes/tauri-capability-probe/src-tauri/Cargo.lock`
- `product-line/prototypes/tauri-capability-probe/src-tauri/Cargo.toml`
- `product-line/prototypes/tauri-capability-probe/src-tauri/build.rs`
- `product-line/prototypes/tauri-capability-probe/src-tauri/capabilities/default.json`
- `product-line/prototypes/tauri-capability-probe/src-tauri/icons/`
- `product-line/prototypes/tauri-capability-probe/src-tauri/src/lib.rs`
- `product-line/prototypes/tauri-capability-probe/src-tauri/src/main.rs`
- `product-line/prototypes/tauri-capability-probe/src-tauri/tauri.conf.json`
- `product-line/prototypes/tauri-capability-probe/ui/index.html`
- `product-line/prototypes/tauri-capability-probe/ui/app.js`
- `product-line/prototypes/tauri-capability-probe/ui/styles.css`

## 验证结果

Tauri CLI：

```text
tauri-cli 2.11.2
```

前端脚本：

```text
node --check product-line/prototypes/tauri-capability-probe/ui/app.js
```

结果：通过，无输出。

Rust 测试：

```text
running 3 tests
test tests::extracts_only_indexed_project_and_rollout_paths ... ok
test tests::builds_summary_without_reading_session_body ... ok
test tests::reads_real_static_index_summary ... ok

test result: ok. 3 passed; 0 failed
```

Rust 构建：

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.49s
```

窗口验证：

```text
System Events 可见进程包含 app
窗口标题：Codex 治理工作台 Tauri 探针
窗口属性曾显示 size:1180, 760
```

清理：

```text
pgrep -x app
```

结果：无输出，退出码 1，表示调试进程未残留。

## 已验证能力

- Tauri CLI 可用。
- 最小 Tauri 工程可编译。
- Rust 后端能读取真实静态索引并生成摘要。
- 后端路径白名单只来自索引里的项目路径和 rollout 路径。
- Tauri 调试窗口能创建，窗口标题可被系统事件读取。

## 未充分验证能力

- 未稳定验证窗口正文是否显示所有索引数字。依据：辅助功能读取窗口正文失败或不稳定。
- 未自动化验证按钮点击。依据：没有成功读取页面按钮可访问文本，也没有执行 UI 点击。
- 未真实触发 `open_indexed_project` 和 `reveal_indexed_rollout`。依据：为避免打开 Finder 干扰，只做后端编译和白名单测试。
- 未验证 release 打包。依据：任务范围不做完整产品化打包。

## 安全边界

符合任务包边界：

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件。
- 未展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未自动运行 harness。
- 未做完整产品化打包。

仍需注意：

- `open_indexed_project` 和 `reveal_indexed_rollout` 使用 macOS `open`，后续点击验证会打开 Finder。
- `copy_indexed_path` 使用 `pbcopy`，后续点击验证会改变系统剪贴板。
- 这些动作应继续保持用户点击触发，不应自动执行。

## UI 与本机动作补测

记录时间：2026-05-27 21:51:03 CST。

用户本轮明确允许：

- 打开 Finder。
- 定位文件。
- 改系统剪贴板。

### 先说薄弱点

- 剪贴板按钮通过 UI 点击触发，Tauri 页面状态返回成功，但没有独立读取剪贴板内容。依据：尝试提升权限读取 `pbpaste` 被安全审查拒绝，理由是读取系统剪贴板可能暴露无关敏感内容。
- “打开目录”和“定位文件”完成了真实 macOS 动作验证，但由于 UI 窗口在复制后辅助功能窗口索引变得不稳定，后两项用等价后端命令路径执行验证，而不是继续强行点 UI。
- 原型调试进程名仍是 `app`，不是正式产品名。依据：System Events 读取进程名为 `app`。

### 复核命令和结果

Tauri CLI：

```bash
./.tauri-cli/bin/cargo-tauri --version
```

结果：

```text
tauri-cli 2.11.2
```

前端语法：

```bash
node --check ui/app.js
```

结果：通过。

Rust 离线测试：

```bash
cd src-tauri
CARGO_HOME="../.cargo-home" CARGO_TARGET_DIR="./target" cargo test --offline
```

结果：

```text
test result: ok. 3 passed; 0 failed
```

启动 Tauri 探针：

```bash
CARGO_HOME="$PWD/.cargo-home" CARGO_TARGET_DIR="$PWD/src-tauri/target" ./.tauri-cli/bin/cargo-tauri dev --no-watch --no-dev-server-wait
```

结果：

- dev 构建完成：`Finished dev profile ... target(s) in 33.92s`
- 调试进程运行：`Running target/debug/app`

### 窗口和正文验证

System Events 读取窗口：

```text
窗口标题：Codex 治理工作台 Tauri 探针
窗口尺寸：1180, 760
```

窗口正文可读，关键内容包括：

- 状态文本：`已读取索引。桌面动作仍需用户点击触发。`
- 项目数：`30`
- 会话数：`296`
- Skills：`50`
- Plugins：`11`
- 索引文件：`/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/src-tauri/../../index-kernel/codex-index.json`
- 可打开项目路径：`30`
- 可定位 rollout 路径：`296`
- Warning：`无顶层 warning`
- 项目路径：`/Users/yoyi`
- rollout 路径：`/Users/yoyi/.codex/archived_sessions/rollout-2026-05-04T13-04-11-019df15f-5bd5-7c02-8e18-27503a548dd3.jsonl`

### 复制路径验证

触发方式：

```applescript
tell application "System Events" to tell process "app" to click button "复制路径" of group 6 of group 1 of UI element 1 of scroll area 1 of group 1 of group 1 of window 1
```

结果：

- UI 点击返回成功。
- 页面状态文本变为：`已复制：/Users/yoyi`。

未完成：

- 未独立读取剪贴板内容。原因：读取系统剪贴板的提升权限请求被拒，不能绕过。

结论：

- UI 到 Tauri 后端 `copy_indexed_path` 的调用链可用。
- 系统剪贴板内容未做独立内容核验，只能记录为页面返回成功。

### 打开项目目录验证

索引内项目路径：

```text
/Users/yoyi
```

验证动作：

```bash
open /Users/yoyi
```

Finder 结果：

```text
/Users/yoyi/
```

结论：

- macOS 打开 Finder 目录能力可用。
- 路径来自索引内项目路径。

### 定位 rollout 文件验证

索引内 rollout 路径：

```text
/Users/yoyi/.codex/archived_sessions/rollout-2026-05-04T13-04-11-019df15f-5bd5-7c02-8e18-27503a548dd3.jsonl
```

验证动作：

```bash
open -R /Users/yoyi/.codex/archived_sessions/rollout-2026-05-04T13-04-11-019df15f-5bd5-7c02-8e18-27503a548dd3.jsonl
```

Finder selection 结果：

```text
/Users/yoyi/.codex/archived_sessions/rollout-2026-05-04T13-04-11-019df15f-5bd5-7c02-8e18-27503a548dd3.jsonl
```

结论：

- macOS Finder 定位文件能力可用。
- 路径来自索引内 rollout 路径。
- 本轮只读取 Finder 选中路径作为验证结果，没有读取 rollout 文件正文。

### 清理

清理方式：

- System Events 读取 Tauri 调试进程 `app` 的 PID：`63560`。
- 执行 `kill 63560`。
- Tauri dev 会话随后退出，工具会话返回退出码 0。

### 本轮补测结论

已补强：

- 窗口正文已可读。
- UI 复制按钮已触发并返回成功状态。
- Finder 打开目录能力已验证。
- Finder 定位 rollout 文件能力已验证。

仍不能说：

- 不能说剪贴板内容已独立核验。
- 不能说后两项是通过 UI 按钮点击完成；它们是同一索引路径约束下的 macOS 后端等价动作验证。
- 不能说最终桌面应用完成，仍只是 Tauri 最小能力探针。
