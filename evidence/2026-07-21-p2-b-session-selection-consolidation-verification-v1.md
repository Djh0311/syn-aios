# P2-B 挑会话归置核验（2026-07-21）

## 一、结论

P2-B 已在当前工作树完成代码、离线交互、全量 Rust 回归与真实组件浏览器验收，尚未 commit。

- 每个执行节点默认新对话，不再自动选择最近会话。
- 整单级「给第一个节点预填对话」入口退场；兼容字段只镜像首节点，既有批准协议不扩改。
- 工序图只展示任务结构、依赖与会话结果，不提供挑选入口。
- 「怎么跑」统一承载逐节点选择已有对话；同一 proposal 重挂载保留，proposal 变化时全部重置为新对话。
- 旧公开 Tauri 绑定命令无前端调用，但本包不删；不把 UI 归置扩大为协议或测试清扫。

## 二、改动面

产品代码：

- `src/views/projects/ProjectJiaobanPanel.tsx`：取消最近会话自动默认；按 proposal 隔离缓存；「怎么跑」接逐节点 bindings；工序图关闭编辑入口。
- `src/views/projects/jiaoban/JiaobanAuthorizeStates.tsx`：「怎么跑」逐节点渲染共享 `JiaobanSessionPicker`，默认文案统一为「新对话（默认）」。
- `src/views/projects/jiaoban/jiaobanPreviewCanvas.tsx`：新增 `sessionBindingsEditable`，预演工序图可只读展示会话结果。
- `src/views/projects/projectWorkflowSidePanel.css`：仅补逐节点选择列表的网格间距与 `min-width: 0`。

回归断言：

- `tests/advice-only-authorize-face.test.tsx`：锁整单入口退场、逐节点入口与默认新对话。
- `tests/jiaoban-plan-preview-canvas.test.tsx`：锁工序图无挑会话入口。

## 三、机器验证

1. `npm run typecheck`（cwd=`prototypes/productized-desktop-shell`）：通过。
2. `npm run test:offline-interaction`（同 cwd）：全套通过；P2-B 新断言覆盖整单入口退场、逐节点选择、默认新对话、工序图无选择器。
3. `node scripts/harness/workbench-shape-gate.js --mode check`（仓根）：保持既有基线 **13 errors / 5 warnings / 5 infos**；exit 1 为历史债基线，`machine_face_on_ui`、`hardcoded_hex_on_ui`、`retired_style_family` 均零新增。
4. `cargo test --lib`（cwd=`prototypes/productized-desktop-shell/src-tauri`）：沙箱内 **1028 passed / 2 failed / 44 ignored**，两败均为既有进程/PID 权限探针；沙箱外原命令复跑 **1030 passed / 0 failed / 44 ignored**。

## 四、浏览器验收

真实 App 浏览器预览会在 `ProjectJiaobanPanel` 挂载时被既有 `loadFormalMemoryStore()` 非 Tauri 同步抛错阻断，故本包没有伪称完成整页浏览器验收。改用忽略目录中的临时 Vite 夹具，直接挂真实 `JiaobanPlanPreviewCanvas`、`JiaobanHowRunView`、`styles.css` 与 `projectWorkflowSidePanel.css`；不复制产品组件或交互逻辑。

Playwright 实测：

- 1280×900：`bodyScrollWidth=documentElement.scrollWidth=1280`；三节点均默认「新对话（默认）」；工序图没有 radio/选择器。
- 展开第一节点选择器并选择「知识库第一片与 P2-B 交接」后，仅第一节点更新为 existing，另外两节点仍为新对话；展开工序图第一节点仅见「已绑定」和「看原始对话」。
- 577×900 压测采用「选择器展开 + 工序图详情展开」最拥挤态：`bodyScrollWidth=documentElement.scrollWidth=577`，无横向溢出、遮挡或不可读重叠。

本地证据（已忽略、不进 commit）：

- `prototypes/productized-desktop-shell/output/playwright/2026-07-21-p2-b-desktop-1280x900.png`
- `prototypes/productized-desktop-shell/output/playwright/2026-07-21-p2-b-narrow-577x900.png`

控制台唯一 error 为无关 `favicon.ico 404`。

## 五、边界核对

- Rust 生产代码、安全闸、沙箱、批准语义、执行协议、H2 均零碰。
- 无新依赖、无新 Tauri command、无删除旧协议面。
- 全量 Rust 回归沙箱外全绿，说明前端类型改动没有带来后端连坐回归。

## 六、catch 结论

**catch:1**：完整浏览器预览抓到既有 `ProjectJiaobanPanel` 在非 Tauri 页面调用 `loadFormalMemoryStore()` 时同步抛错，调用点后接 `.catch()` 无法接住同步 throw。该问题与 P2-B diff 无关，本包只记入 `docs/harness-catch-log.md`，不顺手扩修。
