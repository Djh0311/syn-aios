# Final Skeleton 03 Tauri Verification Line Design v1 Evidence

日期：2026-06-01

## 本轮结论

先说薄弱点：当前还没有真实 Tauri 窗口验收线。已有的是普通前端构建、React 离线渲染测试和一些普通浏览器/截图证据，不能证明真实桌面壳里的页面状态。

本轮只做设计，不启动 Tauri、不截图、不改业务代码。

结论：

- 可以设计一条最小真实 Tauri 验收线。
- 实现时需要用户确认，因为要启动 GUI 桌面窗口并保存截图。
- 普通浏览器截图只能作为补充证据，不能替代 Tauri 验收。

## 读过的依据

| 文件 | 结论 |
|---|---|
| `package.json` | 当前有 `tauri:dev`：`../tauri-capability-probe/.tauri-cli/bin/cargo-tauri dev`。 |
| `src-tauri/tauri.conf.json` | Tauri dev 会启动 `npm run dev`，devUrl 是 `http://127.0.0.1:5173`；窗口标题是 `Codex 治理工作台`，默认 1280x820。 |
| `scripts/run-offline-interaction-test.mjs` | 当前离线测试是 esbuild 打包后 Node 环境渲染，不是真实 Tauri。 |
| `tests/offline-permission-dialog.test.tsx` | 有可复用的 React fixture，可用于后续构造权限弹层状态，但不能直接证明真实 Tauri。 |
| `evidence/**/*.png` | 已有截图多为普通浏览器或历史 UI 证据，需在新 evidence 中明确区别。 |

## 当前启动方式

| 项 | 当前值 |
|---|---|
| 原型目录 | `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell` |
| 前端 dev | `npm run dev` |
| 前端 dev URL | `http://127.0.0.1:5173` |
| Tauri dev | `npm run tauri:dev` |
| Tauri CLI 路径 | `../tauri-capability-probe/.tauri-cli/bin/cargo-tauri` |
| 窗口标题 | `Codex 治理工作台` |
| 默认窗口大小 | `1280x820` |
| 最小窗口大小 | `1040x680` |

## 验收对象

| 对象 | 验收目的 | 证据要求 |
|---|---|---|
| 首页 | 证明真实 Tauri 窗口能加载主入口。 | Tauri 窗口截图。 |
| 项目页 | 证明项目是工作台主轴之一。 | Tauri 窗口截图。 |
| 项目工作流画布 | 证明项目工作流主入口可见，不用独立 CanvasView 冒充事实源。 | Tauri 窗口截图。 |
| 会话页 | 证明 Codex 会话中心在 Tauri 中可见。 | Tauri 窗口截图。 |
| 右侧栏 / 运行入口 | 证明通知、运行或权限入口在真实窗口中没有遮挡。 | Tauri 窗口截图。 |
| 权限确认弹层 | 证明确认弹层在真实窗口中可用。 | 如有稳定 fixture，则截图；否则记录缺口。 |

## 建议截图路径和命名

目录：

- `evidence/tauri-verification/2026-06-01-final-skeleton-04/`

文件：

| 文件名 | 内容 |
|---|---|
| `01-home.png` | 首页。 |
| `02-projects.png` | 项目页。 |
| `03-project-workflow-canvas.png` | 项目工作流画布。 |
| `04-sessions.png` | 会话页。 |
| `05-right-rail.png` | 右侧栏 / 通知或运行入口。 |
| `06-permission-dialog.png` | 权限确认弹层，若能稳定打开。 |

## 建议命令

前置验证：

```bash
npm run typecheck
npm run build
```

真实 Tauri 启动：

```bash
npm run tauri:dev
```

截图方式建议：

- 优先使用 macOS 原生窗口截图，确保截到标题为 `Codex 治理工作台` 的真实窗口。
- 如果无法稳定定位窗口，则可以先用全屏截图记录，并在 evidence 中明确说明限制。
- 不使用普通浏览器截图冒充 Tauri 证据。

## Evidence 模板

后续实现 evidence 至少包含：

| 项 | 内容 |
|---|---|
| 启动命令 | 实际运行的 `npm run typecheck`、`npm run build`、`npm run tauri:dev`。 |
| 截图清单 | 每张截图路径和对应页面。 |
| 真 Tauri 判定 | 说明截图来自标题为 `Codex 治理工作台` 的 Tauri 窗口。 |
| 普通浏览器差异 | 明确 Chrome/headless 不能证明 Tauri 壳、窗口尺寸、原生 WebView 状态。 |
| 失败项 | 哪些页面或弹层未能稳定打开。 |
| 禁止事项 | 未执行真实 Codex、未读写 `.codex`、未启动 MCP canvas run。 |

## 后续任务包

已新增任务包草案：

- `tasks/2026-06-01-final-skeleton-04-tauri-verification-line-implementation-v1.md`

状态：

- 待用户确认后执行。

原因：

- Skeleton-04 前置要求 Skeleton-03 已完成并被用户接受。
- 实现需要启动真实 GUI 窗口并截图。

## 禁止事项执行情况

| 禁止项 | 结果 |
|---|---|
| 不改业务代码 | 已遵守。 |
| 不执行真实 Codex | 已遵守。 |
| 不读写 `/Users/yoyi/.codex` | 已遵守。 |
| 不读取 auth、token、`.env`、密钥、完整 transcript | 已遵守。 |
| 不把 Chrome headless 当完整验收 | 已遵守，本轮没有产出截图。 |
| 不启动 MCP canvas run | 已遵守。 |

## 验证

本切片只写设计和后续任务包，没有跑代码测试。

原因：

- 未改业务代码。
- 未改前端或后端逻辑。
- 未启动 Tauri。
