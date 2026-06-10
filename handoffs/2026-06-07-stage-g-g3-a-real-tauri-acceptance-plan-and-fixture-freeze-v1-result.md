# Handoff: Stage G / G3-A Real Tauri Acceptance Plan And Fixture Freeze v1

日期：2026-06-07

## 回收结论

G3-A 可接受为：

```text
real_tauri_acceptance_plan_and_fixture_freeze_completed
```

下一步：

```text
G3-B Real Tauri Manual Screenshot Acceptance 待开始
```

## 接受范围

- G3 被拆为 G3-A / G3-B / G3-C。
- G3-B 截图目录、文件命名、覆盖清单、fixture、降级规则和授权边界已冻结。
- 普通浏览器 smoke 不能替代真实 Tauri 的规则已写清。
- 权限弹层、项目页、项目工作流画布、节点详情、智能体、记忆、知识库、通知 / 待办 / 运行中、管理 runtime log + diagnostics 均进入 G3-B 清单。

## 不接受范围

- 不接受为真实 Tauri 已启动。
- 不接受为截图已采集。
- 不接受为 G3 整体完成。
- 不接受为 G4 回放、G5 最终冻结或阶段 G 完成。
- 不接受为真实 Codex 执行、真实 prompt 发送、读写 `/Users/yoyi/.codex` 或读取 secret / transcript。

## 验证

本轮未改产品代码，未运行 npm / cargo。验收方式是文档和入口扫描。

过程说明：

- G3-A worker 已先创建三份 G3-A 文档。
- 主管主线程在等待超时后误判 worker 卡住，随后接管同名文件创建；经后续核对，当前三份文件内容完整。
- 后续只应补入口同步，不应覆盖 G3-A 主体内容。

建议全局主管复核：

- G3-A task / evidence / handoff 是否存在。
- 权威入口是否统一为 G3-A 已完成，G3-B 待开始。
- 是否不存在 G3-B / G3 / G4 / G5 已完成冒领。

## 下一步注意

G3-B 执行前必须确认是否授权：

- 启动真实 Tauri。
- 使用 macOS 截图或等价截图工具。
- 检查 / 清理必要端口和残留进程。

如果任一项不可用，必须记录为真实窗口 / 截图验收未完成，不得回收 G3-B。
