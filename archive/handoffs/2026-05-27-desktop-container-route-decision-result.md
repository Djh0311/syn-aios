# 桌面容器路线决策交接

## 状态

总指导线任务已完成，可回收。

## 做了什么

- 比较继续纯网页、Electron、Tauri 三条路线。
- 明确静态网页壳继续保留为只读入口。
- 明确如果要做真桌面能力，优先推荐 Tauri 最小能力验证。
- 输出路线决策文档。

## 读取了哪些依据

- `product-line/tasks/2026-05-27-desktop-container-route-decision.md`
- `product-line/README.md`
- `product-line/STAGE_PLAN.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/handoffs/2026-05-27-desktop-app-static-index-shell-review.md`
- `product-line/handoffs/2026-05-27-desktop-app-static-shell-validation-review.md`
- `product-line/prototypes/desktop-app/README.md`

## 改了哪些文件

- `product-line/decisions/2026-05-27-desktop-container-route.md`
- `product-line/evidence/2026-05-27-desktop-container-route-decision.md`
- `product-line/handoffs/2026-05-27-desktop-container-route-decision-result.md`

## 推荐路线

推荐：

- 保留静态网页壳作为当前只读入口。
- 用户确认后，派桌面应用线做 Tauri 最小能力验证。

Tauri 最小能力验证只验证：

- 加载现有静态 UI。
- 读取同一个静态 `codex-index.json`。
- 显式打开项目文件夹。
- 显式定位 rollout 日志所在文件。
- 不写 `/Users/yoyi/.codex`。
- 不读取或展示 `.env`、密钥、授权文件。

## 不推荐路线

不推荐：

- 直接做完整 Electron 应用。
- 把纯网页继续称为最终桌面应用。
- 在静态壳里伪造打开文件夹或定位日志能力。

## 仍然未知的问题

- 本机 Tauri 工具链是否齐全。
- 是否需要可双击 app，还是开发模式足够。
- macOS 打包、签名、权限提示要做到什么程度。
- 路径动作是否只允许索引内已有路径。

## 风险

- 本轮没有安装或验证 Tauri 工具链。
- 本轮没有改应用源码。
- 本轮曾误写 `/Users/yoyi/workspace/evidence/SHOULD_NOT_EXIST.tmp`，已删除并复查不存在。

## 下一步建议

请用户确认是否接受“Tauri 最小能力验证”路线。

确认前不要派完整实现任务。
