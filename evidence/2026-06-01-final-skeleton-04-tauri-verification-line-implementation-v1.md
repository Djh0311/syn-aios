# Final Skeleton 04 Tauri Verification Line Implementation v1 Evidence

日期：2026-06-01

说明：任务文件使用 2026-06-01 命名；实际截图时 macOS 菜单栏显示为 2026/06/02 00:06-00:11，时区为当前桌面时区。

## 本轮结论

先说薄弱点：

- 本轮只跑通真实 Tauri 窗口的手动截图验收线，还不是自动化 Tauri UI 测试。
- 权限确认弹层没有截图；本轮没有稳定 fixture，也没有为了截图去触发写入或真实 Codex 路径。
- 第一次全屏截图误截到 Codex 主窗口，已覆盖为真实 Tauri 窗口区域截图；该失败过程记录在下面。

结论：

- `final-skeleton-04-tauri-verification-line-implementation-v1` 已完成最低验收。
- 已采集至少 3 张真实 Tauri 窗口截图。
- 已明确普通浏览器截图不能替代 Tauri 验收。
- 本轮未改产品功能，未执行真实 Codex，未读写 `/Users/yoyi/.codex`。

依据：

- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md` 第 662-704 行定义 Skeleton-04 目标、验收和输出。
- `tasks/2026-06-01-final-skeleton-04-tauri-verification-line-implementation-v1.md` 定义本任务执行步骤和截图清单。
- `src-tauri/tauri.conf.json` 窗口标题为 `Codex 治理工作台`，默认窗口大小为 1280x820。

## 实际执行

工作目录：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`

命令结果：

| 命令 | 结果 | 依据 |
|---|---|---|
| `npm run typecheck` | 通过。 | 当前回合重新执行，`tsc --noEmit` 退出码 0。 |
| `npm run build` | 通过。 | 当前回合重新执行，Vite 构建退出码 0。 |
| `npm run test:offline-interaction` | 通过。 | 当前回合补跑，输出 `offline interaction tests passed: 2`。 |
| `cargo test --lib` | 通过。 | 当前回合补跑，88 passed、0 failed、1 ignored。 |
| `npm run tauri:dev` | 成功启动真实 Tauri 窗口。 | 交接记录显示 Vite ready、Cargo built 并运行 `target/debug/codex-governance-workbench`；随后本轮用 macOS System Events 确认窗口。 |

既有 warning：

- `npm run build` 仍有 Vite chunk 超过 500 kB 的 warning，构建通过。
- `cargo test --lib` 仍有既有 Rust warning：`JsonRpcError::invalid_params` 未使用，测试通过。

## 启动和截图过程

| 步骤 | 结果 |
|---|---|
| 首次在沙箱内启动 `npm run tauri:dev` | 交接记录显示失败：Vite 监听 `127.0.0.1:5173` 遇到 `EPERM`。 |
| 提权重试 `npm run tauri:dev` | 交接记录显示失败：5173 端口已有项目 Vite 进程占用。 |
| 检查端口 | 交接记录显示 `lsof` 查到 PID `44909`，命令为项目内 `node .../vite --host 127.0.0.1`。 |
| 清理端口 | 交接记录显示已 kill PID `44909`。 |
| 再次启动 Tauri | 成功，Vite ready 于 `http://127.0.0.1:5173/`，Tauri app 启动。 |
| 本轮确认窗口 | `System Events` 返回进程 `codex-governance-workbench`，窗口标题 `Codex 治理工作台`，位置和尺寸为 `95, 43, 1280, 820`。 |
| 第一次截图 | 全屏截图误截到 Codex 主窗口，不作为验收证据。 |
| 后续截图 | 将 `codex-governance-workbench` 置前，并用 `screencapture -R95,43,1280,820` 截取真实 Tauri 窗口区域。 |
| 停止 Tauri | 已向运行会话发送 Ctrl-C；5173 端口复查未再监听。 |

进程残留说明：

- 停止本轮 Tauri dev 后，`pgrep -fl codex-governance-workbench` 仍能看到一个同名进程，但路径是 `/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target/debug/codex-governance-workbench`。
- 该进程不属于本轮 `productized-desktop-shell` dev 会话；本轮没有杀它，避免误伤别的窗口。

## 截图证据

截图目录：

- `evidence/tauri-verification/2026-06-01-final-skeleton-04/`

有效 Tauri 截图：

| 文件 | 内容 | 判定 |
|---|---|---|
| `01-home.png` | 首页，显示 `Codex 治理工作台` 窗口标题和首页总览。 | 真实 Tauri 窗口区域截图。 |
| `02-projects.png` | 项目页，显示项目方块入口和真实项目索引。 | 真实 Tauri 窗口区域截图。 |
| `03-project-workflow-canvas.png` | `workspace` 项目工作流页上半段，显示项目工作流主入口、运行前检查和项目黑板。 | 真实 Tauri 窗口区域截图。 |

辅助截图：

| 文件 | 内容 | 用途 |
|---|---|---|
| `03-project-detail.png` | `workspace` 项目详情总览页。 | 辅助证明从项目页进入项目详情的导航路径。 |
| `04-project-workflow-canvas.png` | 与 `03-project-workflow-canvas.png` 同内容。 | 保留最初采集文件；后续以 `03-project-workflow-canvas.png` 为正式路径。 |

截图尺寸：

- 每张窗口区域截图为 2560 x 1640 像素；原因是 macOS Retina 缩放，逻辑窗口尺寸为 1280 x 820。

## 普通浏览器与 Tauri 验收差异

| 项 | 普通浏览器截图 | 真实 Tauri 截图 |
|---|---|---|
| 原生窗口 | 不能证明。 | 可看到 `Codex 治理工作台` 原生窗口标题。 |
| Tauri WebView | 不能证明。 | 由 `codex-governance-workbench` 真实进程承载。 |
| 本地数据桥 | 普通浏览器可能缺 Tauri 数据桥。 | 本轮项目页显示真实项目索引，证明桌面壳数据路径可用。 |
| 窗口尺寸和壳集成 | 不能证明。 | 本轮按窗口位置和尺寸截取真实桌面窗口。 |

## 未覆盖项

| 项 | 状态 | 原因 |
|---|---|---|
| 会话页截图 | 未采集。 | 最低验收要求至少 3 张真实 Tauri 截图；本轮优先首页、项目页、项目工作流。 |
| 右侧栏独立截图 | 未单独采集。 | 首页和项目页截图中可见右侧竖栏；未单独切换右侧面板。 |
| 权限确认弹层截图 | 未采集。 | 没有稳定 fixture；不为了截图触发写入、真实 Codex 或权限动作。 |
| 自动化截图流程 | 未建立。 | 本轮目标是最小真实 Tauri 验收线，不是自动化 UI 测试框架。 |

## 禁止事项执行情况

| 禁止项 | 结果 |
|---|---|
| 不改产品功能 | 已遵守；本轮没有改产品代码。 |
| 不执行真实 `codex exec` / `codex exec resume` | 已遵守。 |
| 不读写 `/Users/yoyi/.codex` | 已遵守。 |
| 不读取 auth、token、`.env`、密钥、完整 transcript 或 rollout JSONL 正文 | 已遵守。 |
| 不把普通浏览器截图冒充 Tauri 验收 | 已遵守；正式截图来自 `codex-governance-workbench` 真实窗口。 |
| 不启动 MCP canvas run | 已遵守。 |
| 不写真实业务项目目录 | 已遵守。 |
| 不改 workflow state JSON | 已遵守。 |
| 不迁移数据库 | 已遵守。 |

## 不接受为

不接受为：

- Tauri UI 自动化测试体系已完成。
- 权限确认弹层已完成真实窗口截图验收。
- 会话页和所有右侧栏状态都已完整截图验收。
- 真实业务自动编排完成。
- MCP canvas run 已验证。
- 黑板候选持久状态 schema 已确认或实现。

## 下一步判断

可以进入下一普通小任务：

- `final-skeleton-05-canvas-reference-research-v1`

但下一步仍需遵守总包边界：

- 只做画布参考源复核和能力清单。
- 不启动 MCP canvas run。
- 不改真实工作流事实。
- 不执行真实 Codex。
