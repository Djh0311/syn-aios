# Workbench Architecture Task B Conservative Module Split v1 Result

日期：2026-06-01

## 1. 这轮做了什么

执行了 Task B 的保守切片。

完成：

- 后端类型定义从 `lib.rs` 拆到 `src-tauri/src/types.rs`。
- 后端 Tauri command 包装从 `lib.rs` 拆到 `src-tauri/src/commands.rs`。
- 前端 editable canvas 纯类型从 `src/lib/types.ts` 拆到 `src/lib/types/canvas.ts`。
- `src/lib/types.ts` 保持原类型入口继续转导出 canvas 类型。

没有完成：

- workflow 读模型函数没有搬。
- WorkbenchSnapshot 组装函数没有搬。

原因：

- 两者依赖大量私有 helper；本轮强拆会扩大可见性改动，风险超过“保守切片”。

## 2. 改了哪些文件

源码：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/types/canvas.ts`

验证生成产物：

- `prototypes/productized-desktop-shell/dist/index.html`
- `prototypes/productized-desktop-shell/dist/assets/*`

## 3. 验证结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `HOME=/private/tmp/codex-task-b-home RUSTUP_HOME=/Users/yoyi/.rustup CARGO_HOME=/Users/yoyi/.cargo cargo test --lib`
- `rustfmt --check src/types.rs src/commands.rs`

未通过：

- `cargo fmt --check`

原因：

- 它要求重排既有 `lib.rs` 大段代码和 `src-tauri/src/mcp/**` 文件。
- 本轮禁止碰 MCP 可编辑画布运行逻辑，所以没有批量格式化。

## 4. 边界声明

本轮没有：

- 改状态机。
- 改 workflow state JSON。
- 碰真实 Codex resume。
- 碰工作流机器。
- 碰 MCP 可编辑画布运行逻辑。
- 改任务包产品规则。
- 读取 `/Users/yoyi/.codex`。
- 执行 `codex exec` / `codex exec resume`。
- 迁移数据库。

## 5. 下一步建议

先不要继续扩大拆分。

下一步如果继续 Task B，建议只选一个方向：

- 方向一：把后端类型从 `include!` 过渡到真正 `mod types`，配套可见性调整和测试。
- 方向二：只拆 workflow 读模型函数，并先列出需要开放的 helper。
- 方向三：只拆 WorkbenchSnapshot 组装，并先处理 `codex_db` 读取边界，避免误读真实 `/Users/yoyi/.codex`。

不建议下一步做：

- 不要同时拆读模型、快照、状态机和 Codex runner。
- 不要为了 `cargo fmt --check` 顺手格式化 MCP 文件，除非单独确认。

## 6. 手动检查清单

这轮没有改 UI 行为，手动检查以“应用仍能打开关键页面”为主：

- 启动桌面壳或前端开发环境。
- 打开首页，确认项目、会话、运行中摘要能正常显示。
- 打开项目页，确认项目工作流区域还能读取 workflow state。
- 打开独立可编辑画布入口，确认页面能加载，不要求开工运行。
- 不点击任何会触发真实 Codex resume 的按钮。
- 不启动 MCP 可编辑画布 run。

## 7. 本轮 evidence

- `evidence/2026-06-01-workbench-architecture-task-b-conservative-module-split-v1.md`
