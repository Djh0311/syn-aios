# Tauri CLI 后最小原型验证交接

## 结论

可以接受为“最小 Tauri 工具链和后端能力原型已建立”。

不能接受为“桌面应用完整完成”。补测后可以接受为“窗口、索引读取、路径白名单、复制按钮调用链、Finder 打开目录、Finder 定位文件已经有阶段性验证”。

## 先说薄弱点

- 依赖面大，占用数 GB 本地缓存和构建产物。
- 剪贴板内容没有独立读取核验。依据：读取系统剪贴板的权限请求被安全审查拒绝。
- 打开目录和定位文件已做真实 Finder 动作验证，但不是稳定 UI 按钮点击证据。依据：复制按钮点击后，Tauri 窗口辅助功能索引变得不稳定，后两项用索引路径执行等价 macOS 动作验证。
- 没有 release 打包、签名、权限提示验证。

## 做了什么

- 新增 Tauri CLI 后最小原型任务包。
- 本地安装 Tauri CLI 2.11.2 到原型目录。
- 创建 Tauri 最小工程。
- 新增静态 UI。
- 新增 Rust 后端命令和路径白名单。
- 更新原型 README。
- 生成 evidence。

## 工具链是否可用

可用。

依据：

```text
tauri-cli 2.11.2
```

## 改了哪些文件

主要新增或修改：

- `product-line/tasks/2026-05-27-tauri-min-prototype-after-cli.md`
- `product-line/tasks/README.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/prototypes/tauri-capability-probe/README.md`
- `product-line/prototypes/tauri-capability-probe/.gitignore`
- `product-line/prototypes/tauri-capability-probe/src-tauri/`
- `product-line/prototypes/tauri-capability-probe/ui/`
- `product-line/evidence/2026-05-27-tauri-min-prototype-after-cli.md`
- `product-line/handoffs/2026-05-27-tauri-min-prototype-after-cli-result.md`

## 安装或获取了什么依赖

- Tauri CLI：`tauri-cli 2.11.2`
- Tauri 应用依赖：通过 `cargo test` 拉取并锁定到 `src-tauri/Cargo.lock`

依赖缓存和构建产物：

- `.tauri-cli`：约 35M
- `.cargo-home`：约 478M
- `.cargo-target`：约 1.3G
- `src-tauri/target`：约 1.9G

## 验证了哪些能力

验证通过：

- `node --check ui/app.js`
- `cargo test --offline`：3 tests OK
- `cargo build --offline`：通过
- Tauri 调试窗口创建：系统事件读取到窗口标题 `Codex 治理工作台 Tauri 探针`
- 调试进程清理：`pgrep -x app` 无输出

## 哪些能力未验证或未充分验证

- 未独立核验剪贴板内容。
- 未通过稳定 UI 点击验证打开项目目录。
- 未通过稳定 UI 点击验证定位 rollout 文件。
- 未验证 release 打包。

原因：

- 读取系统剪贴板可能暴露无关敏感内容，权限请求被拒。
- 复制按钮点击后，Tauri 窗口辅助功能窗口索引变得不稳定。
- release 打包不在本轮任务范围。

## 安全边界

符合任务包：

- 不写 `/Users/yoyi/.codex`。
- 不改 Codex 状态库。
- 不展示密钥、`.env`、授权文件。
- 不展示会话正文。
- 不自动运行 harness。
- 不做完整产品化打包。

## 下一步建议

- 若继续补测，建议只针对“稳定 UI 点击打开目录 / 定位文件”和“剪贴板内容核验授权”开小任务。
- 若进入下一阶段，先设计桌面权限提示和路径展示策略，不要把最小探针当成完整应用。

## UI 与本机动作补测结果

记录时间：2026-05-27 21:51:03 CST。

用户已明确允许：

- 打开 Finder。
- 定位文件。
- 改系统剪贴板。

补测结果：

- Tauri CLI 可用：`tauri-cli 2.11.2`。
- `node --check ui/app.js` 通过。
- `cargo test --offline` 通过：3 tests OK。
- Tauri dev 窗口启动成功，窗口标题为 `Codex 治理工作台 Tauri 探针`，尺寸为 `1180, 760`。
- 窗口正文可读，显示项目 30、会话 296、Skills 50、Plugins 11、可打开项目路径 30、可定位 rollout 路径 296。
- UI 点击“复制路径”成功，页面状态文本变为 `已复制：/Users/yoyi`。
- 未独立读取剪贴板内容。原因：读取系统剪贴板的权限请求被安全审查拒绝，不能绕过。
- 用索引内项目路径 `/Users/yoyi` 执行 Finder 打开验证，Finder 前台目录为 `/Users/yoyi/`。
- 用索引内 rollout 路径 `/Users/yoyi/.codex/archived_sessions/rollout-2026-05-04T13-04-11-019df15f-5bd5-7c02-8e18-27503a548dd3.jsonl` 执行 Finder 定位验证，Finder selection 返回同一路径。
- Tauri 调试进程 `app` 已清理，PID 为 `63560`，dev 会话退出码 0。

补测后结论：

- 窗口正文显示问题已补证。
- 复制按钮 UI 调用链已补证，但剪贴板内容未独立核验。
- Finder 打开目录和定位文件能力已补证；后两项是同一索引路径约束下的 macOS 后端等价动作验证，不是稳定 UI 点击证据。
- 仍不能接受为完整桌面应用完成；可以接受为 Tauri 最小能力探针已覆盖窗口、索引读取、路径白名单、复制调用、打开目录、定位文件的阶段性验证。
