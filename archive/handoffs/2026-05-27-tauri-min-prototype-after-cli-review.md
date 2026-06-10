# Tauri CLI 后最小原型验证回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-tauri-min-prototype-after-cli.md`
- 开发线：桌面应用线
- 原型目录：`product-line/prototypes/tauri-capability-probe/`
- Evidence：`product-line/evidence/2026-05-27-tauri-min-prototype-after-cli.md`
- Handoff：`product-line/handoffs/2026-05-27-tauri-min-prototype-after-cli-result.md`

## 结论

接受为 Tauri 最小能力探针阶段性验证结果。

边界：

- 接受：Tauri CLI 可用、最小工程可编译、后端能读取真实索引、路径白名单已实现、窗口能创建、窗口正文能读取、复制按钮调用链已通过 UI 点击补证、打开目录和定位文件已通过索引路径等价 Finder 动作补证。
- 不接受：完整桌面应用完成、剪贴板内容已独立核验、打开目录和定位文件已通过稳定 UI 点击验证、release 打包完成。

## 先说薄弱点

- 剪贴板内容没有独立核验。依据：读取系统剪贴板可能暴露无关敏感内容，权限请求被安全审查拒绝。
- 打开目录和定位文件不是稳定 UI 按钮点击证据。依据：复制按钮点击后 Tauri 窗口辅助功能索引变得不稳定，后两项使用索引内路径执行等价 Finder 动作验证。
- 本地依赖和构建产物体积大。依据：`.cargo-home`、`.cargo-target`、`src-tauri/target` 合计数 GB。
- 路径动作会影响系统状态。依据：打开目录和定位文件会打开 Finder，复制路径会改剪贴板。
- 当前原型应用名仍是调试二进制 `app`，不是正式产品名。依据：系统事件进程名显示为 `app`。

## 已满足验收项

- 有 Tauri CLI 可用性结果。
- 有最小原型文件。
- 有窗口创建证据。
- 有窗口正文补证，显示项目 30、会话 296、Skills 50、Plugins 11。
- 有真实索引读取测试。
- 有路径白名单测试。
- 有 UI 点击“复制路径”调用链补证，页面状态返回 `已复制：/Users/yoyi`。
- 有 Finder 打开索引内项目路径补证。
- 有 Finder 定位索引内 rollout 文件补证。
- 有安全边界说明。
- 有 evidence 和 handoff。

## 未满足或未充分满足项

- 剪贴板内容未独立核验。
- 打开目录和定位文件未通过稳定 UI 按钮点击验证。
- release 打包、签名、权限提示未验证。

这些不退回桌面应用线。当前结果足够支撑“最小能力探针阶段性通过”，但不能支撑“完整桌面应用完成”。

## 状态

已回收，接受为阶段性验证结果。

下一步建议：

- 不再单独派“Tauri 探针 UI 与按钮行为验证”作为当前阻塞项。
- 若要进入产品化桌面应用线，先设计权限提示、路径展示策略、正式应用名和 release 打包边界。
- 若要补剪贴板内容核验，必须单独确认读取系统剪贴板的权限和敏感信息风险。
