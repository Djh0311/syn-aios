# 任务包：final-skeleton-04-tauri-verification-line-implementation-v1

## 状态

已完成。

原因：

- 用户已用“运行”确认执行真实 Tauri GUI 验收。
- 已启动真实 Tauri 桌面窗口并截图。
- 依据见 `evidence/2026-06-01-final-skeleton-04-tauri-verification-line-implementation-v1.md` 和 `handoffs/2026-06-01-final-skeleton-04-tauri-verification-line-implementation-v1-result.md`。

## 目标

跑通一条真实 Tauri 窗口截图和交互验收路径，明确区分真实 Tauri 验收和普通浏览器验收。

## 允许

- 在原型目录运行 `npm run typecheck`。
- 在原型目录运行 `npm run build`。
- 用户确认后运行 `npm run tauri:dev` 启动真实 Tauri 窗口。
- 用户确认后使用 macOS 截图能力保存真实 Tauri 窗口截图。
- 新增 evidence 截图目录。
- 新增 evidence / handoff。

## 禁止

- 不改产品功能。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth、token、`.env`、密钥、完整 transcript 或 rollout JSONL 正文。
- 不把普通浏览器截图冒充 Tauri 验收。
- 不启动 MCP canvas run。
- 不写真实业务项目目录。

## 设计来源

- `evidence/2026-06-01-final-skeleton-03-tauri-verification-line-design-v1.md`
- `handoffs/2026-06-01-final-skeleton-03-tauri-verification-line-design-v1-result.md`

## 建议截图目录

- `evidence/tauri-verification/2026-06-01-final-skeleton-04/`

## 建议截图清单

| 编号 | 页面或状态 | 文件名 |
|---|---|---|
| 1 | 首页 | `01-home.png` |
| 2 | 项目页 | `02-projects.png` |
| 3 | 项目工作流画布 | `03-project-workflow-canvas.png` |
| 4 | 会话页 | `04-sessions.png` |
| 5 | 右侧栏 / 通知或运行入口 | `05-right-rail.png` |
| 6 | 权限确认弹层，若有稳定 fixture | `06-permission-dialog.png` |

## 执行步骤

1. 运行 `npm run typecheck`。
2. 运行 `npm run build`。
3. 启动 `npm run tauri:dev`。
4. 等待窗口标题为 `Codex 治理工作台` 的真实 Tauri 窗口出现。
5. 对截图清单逐项导航并截图。
6. evidence 中记录：
   - 命令。
   - 截图路径。
   - 哪些是真实 Tauri 窗口。
   - 哪些页面未能稳定打开。
   - 普通浏览器验收和 Tauri 验收的差异。

## 验收

- 至少有 3 张真实 Tauri 窗口截图。
- 不能只有普通浏览器截图。
- evidence 记录限制和失败项。

## 必须输出

- `evidence/2026-06-01-final-skeleton-04-tauri-verification-line-implementation-v1.md`
- `handoffs/2026-06-01-final-skeleton-04-tauri-verification-line-implementation-v1-result.md`

## 停止条件

出现以下情况停下来：

- 启动 Tauri 需要额外安装工具链。
- 截图需要新增未确认权限。
- 需要读取或写入敏感目录。
- 需要执行真实 Codex。
