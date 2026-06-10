# Tauri 最小能力验证回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-tauri-min-capability-probe.md`
- 开发线：桌面应用线
- 原型目录：`product-line/prototypes/tauri-capability-probe/`
- 回传 evidence：`product-line/evidence/2026-05-27-tauri-min-capability-probe.md`
- 回传 handoff：`product-line/handoffs/2026-05-27-tauri-min-capability-probe-result.md`

## 结论

接受为 Tauri 前置工具链检查结果。

不接受为 Tauri 桌面能力验证完成。没有可运行 Tauri 原型，也没有验证窗口加载、打开文件夹、复制路径或定位 rollout 日志。

## 先说薄弱点

- 没有创建可运行 Tauri 原型。依据：缺 Tauri CLI，任务包禁止安装网络依赖。
- 没有验证任何真实桌面能力。依据：窗口加载、打开文件夹、复制路径、定位 rollout 日志都未执行。
- 当前只能证明工具链缺口，不能证明 Tauri 路线可行或不可行。依据：Rust/Cargo/Node/npm 可用，但 Tauri CLI 不可用。
- npm registry 指向 `https://registry.npmmirror.com`，后续 npm 方式会依赖网络可用性。依据：回收线复跑 `npm config get registry`。

## 复核结果

回收线复跑结果：

- `rustc --version`：`rustc 1.95.0 (59807616e 2026-04-14)`
- `cargo --version`：`cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- `node --version`：`v23.11.0`
- `npm --version`：`10.9.2`
- `cargo --list | rg '^    tauri'`：无输出，说明未发现 cargo tauri 子命令。
- `cargo tauri --version`：失败，`error: no such command: tauri`。
- `npm config get registry`：`https://registry.npmmirror.com`

## 当前生效结论

- Tauri 路线暂时卡在缺 Tauri CLI。
- 不能继续派 Tauri 实现任务，除非用户允许安装或提供 Tauri CLI。
- 当前静态网页壳仍是可用只读入口。
- 后续如果允许安装，第一条复核命令应是 `cargo tauri --version`。

## 状态

已回收，接受为工具链缺口结果；等待用户确认是否允许安装或提供 Tauri CLI。
