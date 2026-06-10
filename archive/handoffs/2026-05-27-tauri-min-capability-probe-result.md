# Tauri 最小能力验证交接

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-tauri-min-capability-probe.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-27-tauri-min-capability-probe.md`
- 原型目录：`product-line/prototypes/tauri-capability-probe/`

## 结论

本轮不能接受为“已完成 Tauri 桌面能力验证”。

可以接受为“前置工具链检查结果”：Rust / Cargo / Node / npm 可用，但 Tauri CLI 不可用。任务包禁止安装网络依赖，所以没有创建可运行 Tauri 原型。

## 先说薄弱点

- 没有验证窗口加载本地 UI。依据：缺 Tauri CLI。
- 没有验证打开文件夹、复制路径、定位 rollout 日志。依据：没有可运行 Tauri 原型。
- npm 方式会尝试访问 registry。依据：`npm exec tauri -- --version` 请求 `https://registry.npmmirror.com/tauri` 并 DNS 失败。
- 当前只能说明环境缺口，不能证明 Tauri 路线可行或不可行。依据：Rust/Node 在，但 Tauri CLI 不在。

## 工具链是否可用

部分可用。

可用：

- Rust：`rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo：`cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- Node：`v23.11.0`
- npm：`10.9.2`

不可用：

- `cargo tauri`：未安装。
- `npm exec tauri`：会触发联网获取依赖，本轮 DNS 失败；任务包禁止安装网络依赖。

## 改了哪些文件

- `product-line/prototypes/tauri-capability-probe/README.md`
- `product-line/evidence/2026-05-27-tauri-min-capability-probe.md`
- `product-line/handoffs/2026-05-27-tauri-min-capability-probe-result.md`

## 验证了哪些桌面能力

没有验证真实桌面能力。

本轮只验证了前置工具链状态：

- Rust / Cargo 可运行。
- Node / npm 可运行。
- Tauri CLI 不存在。
- npm 获取 Tauri 包需要网络，当前失败。

## 哪些能力未验证

未验证：

- Tauri 窗口加载本地 UI。
- 读取 `codex-index.json` 到 Tauri 窗口。
- 打开索引内项目文件夹。
- 复制索引内路径到剪贴板。
- 定位索引内 rollout 日志所在文件。
- Tauri 权限配置。

原因：

- 缺 Tauri CLI。
- 任务包禁止安装网络依赖。

## 安全边界

符合任务包边界：

- 未写 `/Users/yoyi/.codex`。
- 未改 Codex 状态库。
- 未读取或展示密钥、令牌、授权文件、`.env`。
- 未展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未自动运行 harness。
- 未做完整产品化打包。
- 未安装网络依赖。
- 未把本轮说成最终桌面应用完成。

## 下一步建议

如果继续 Tauri 路线，需要先明确是否允许安装缺失依赖。

建议下一步只批准最小安装和验证：

- 安装或提供 Tauri CLI。
- 先跑 `cargo tauri --version`。
- 只创建最小能力原型。
- 所有本机动作只允许索引内已有路径，不接受任意输入路径。

如果不允许安装依赖，Tauri 路线当前只能停在工具链缺口记录，不能继续验证。
