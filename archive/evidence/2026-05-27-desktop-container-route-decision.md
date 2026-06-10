# 桌面容器路线决策证据

## 结论先说

薄弱点：

- 本轮没有安装 Electron、Tauri 或任何网络依赖。依据：任务包禁止安装依赖，本轮只写决策文档。
- 本轮没有检查本机是否具备 Tauri 工具链。依据：任务目标是路线决策，不是环境探测。
- 决策推荐的是“Tauri 最小能力验证”，不是直接进入完整 Tauri 实现。
- 本轮曾误创建 `/Users/yoyi/workspace/evidence/SHOULD_NOT_EXIST.tmp`，随后立即删除并复查不存在。这个误写不应被忽略。

可用结论：

- 静态网页壳继续保留，作为只读入口。
- 如果第一版要兑现打开文件夹、定位日志、复制路径等本机动作，推荐优先做 Tauri 最小能力验证。
- 不推荐直接做完整 Electron。
- 不推荐继续把纯网页叫最终桌面应用。

## 本轮读取

- `product-line/tasks/2026-05-27-desktop-container-route-decision.md`
- `product-line/README.md`
- `product-line/STAGE_PLAN.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/handoffs/2026-05-27-desktop-app-static-index-shell-review.md`
- `product-line/handoffs/2026-05-27-desktop-app-static-shell-validation-review.md`
- `product-line/prototypes/desktop-app/README.md`

## 本轮写入

- `product-line/decisions/2026-05-27-desktop-container-route.md`
- `product-line/evidence/2026-05-27-desktop-container-route-decision.md`
- `product-line/handoffs/2026-05-27-desktop-container-route-decision-result.md`

待更新：

- `product-line/tasks/README.md`
- `product-line/README.md`

## 误写处理

误写：

- `/Users/yoyi/workspace/evidence/SHOULD_NOT_EXIST.tmp`

处理：

- 已删除。
- 已用 `test ! -e /Users/yoyi/workspace/evidence/SHOULD_NOT_EXIST.tmp` 复查不存在。

影响：

- 没有保留越界文件。
- 但这是一次允许写入范围外的误操作，作为本轮风险记录。

## 决策文件

路线决策文件：

- `product-line/decisions/2026-05-27-desktop-container-route.md`

核心判断：

- 继续纯网页：适合作为当前只读入口，不适合作为最终桌面能力方案。
- Electron：前端迁移成本低，但权限面和依赖面较大，不推荐作为第一优先路线。
- Tauri：更适合把本机动作收敛成少量显式命令，推荐作为真桌面能力的优先验证路线。

## 仍然未知

- 本机是否已经有 Tauri 所需工具链。
- macOS 打包和权限提示要做到什么程度。
- 第一版是否必须打包成可双击 app，还是本机开发模式即可。
- 打开文件夹和定位日志是否只允许索引内已有路径，还是允许用户输入路径。

## 下一步

需要用户确认是否接受“Tauri 最小能力验证”路线。

确认前不派完整实现任务。
