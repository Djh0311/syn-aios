# Tauri UI 与本机动作补测回收 evidence

## 对象

- 回收线：总指导线
- 被回收结果：Tauri 探针 UI 与本机动作补测
- 关联 evidence：`product-line/evidence/2026-05-27-tauri-min-prototype-after-cli.md`
- 关联 handoff：`product-line/handoffs/2026-05-27-tauri-min-prototype-after-cli-result.md`
- 回收时间：2026-05-27 21:59:09 CST

## 先说薄弱点

- 剪贴板内容没有独立核验。依据：读取系统剪贴板可能暴露无关敏感内容，权限请求被安全审查拒绝。
- 打开目录和定位文件不是稳定 UI 按钮点击证据。依据：复制按钮点击后 Tauri 窗口辅助功能索引不稳定，后两项用索引内路径执行等价 Finder 动作验证。
- 这仍不是完整桌面应用。依据：没有 release 打包、签名、权限提示、正式应用名、完整产品壳。

## 回收判断

接受为 Tauri 最小能力探针阶段性验证结果。

接受范围：

- 窗口正文显示已补证。
- 索引读取已补证。
- 路径白名单已有测试。
- UI 点击“复制路径”调用链已补证。
- Finder 打开索引内项目路径已补证。
- Finder 定位索引内 rollout 文件已补证。
- Tauri 调试进程已清理。

不接受范围：

- 不接受为完整桌面应用完成。
- 不接受为剪贴板内容已独立核验。
- 不接受为 release 打包完成。

## 已更新文件

- `product-line/handoffs/2026-05-27-tauri-min-prototype-after-cli-review.md`
- `product-line/tasks/README.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/evidence/2026-05-27-tauri-ui-action-validation-recovery.md`
- `product-line/handoffs/2026-05-27-tauri-ui-action-validation-recovery-result.md`

## 当前队列状态

- Tauri 探针 UI 与按钮行为验证不再作为当前阻塞待派发任务。
- 当前阶段暂无阻塞任务。
- 下一步如果继续推进桌面应用，应先由总指导线派发产品化桌面壳任务包。

## 下一步建议

产品化桌面壳任务包必须先明确：

- 权限提示。
- 路径展示策略。
- 正式应用名。
- release 打包边界。
- 剪贴板是否允许独立核验。
- Finder 打开/定位是否继续作为用户点击动作。
