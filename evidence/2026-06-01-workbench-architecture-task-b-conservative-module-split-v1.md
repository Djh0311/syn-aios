# Workbench Architecture Task B Conservative Module Split v1

日期：2026-06-01

## 1. 本轮边界

本轮执行 Task B 的保守切片。

允许范围：

- 拆类型定义。
- 拆 Tauri command 包装。
- 拆 workflow 读模型。
- 拆 WorkbenchSnapshot 组装。
- 拆前端纯类型。

实际执行范围：

- 后端拆了类型定义文件。
- 后端拆了 Tauri command 包装文件。
- 前端拆了独立可编辑画布纯类型。
- workflow 读模型函数和 WorkbenchSnapshot 组装函数本轮没有搬。

没有执行：

- 没有改状态机。
- 没有改 workflow state JSON。
- 没有碰真实 Codex resume 执行逻辑。
- 没有碰工作流机器。
- 没有碰 MCP 可编辑画布运行逻辑。
- 没有改任务包产品规则。
- 没有读取 `/Users/yoyi/.codex`。
- 没有执行 `codex exec` / `codex exec resume`。

依据：

- `decisions/2026-06-01-architecture-module-split-guardrail-v1.md:7-25` 只允许第一批低风险拆分，并禁止状态机、真实 Codex resume、工作流机器、MCP 画布运行逻辑和任务包规则变更。
- `decisions/2026-06-01-project-workflow-canvas-authority-v1.md:7-15` 明确项目工作流事实源是项目 workflow state，独立 `CanvasView` / `src-tauri/src/mcp/**` 不是当前项目工作流权威事实源。
- `evidence/2026-06-01-workbench-architecture-readonly-audit-v1.md:262-288` 建议 Task B 只做保守拆分。

## 2. 改动文件

后端：

| 文件 | 动作 | 说明 |
|---|---|---|
| `prototypes/productized-desktop-shell/src-tauri/src/types.rs` | 新增 | 从 `lib.rs` 搬出后端数据结构、请求响应结构、任务包预览结果等类型定义。 |
| `prototypes/productized-desktop-shell/src-tauri/src/commands.rs` | 新增 | 从 `lib.rs` 搬出 Tauri command 包装和同一段里的 command helper。 |
| `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` | 修改 | 删除已搬出的类型和 command 包装正文，改为 `include!("types.rs")` 和 `include!("commands.rs")`。 |

前端：

| 文件 | 动作 | 说明 |
|---|---|---|
| `prototypes/productized-desktop-shell/src/lib/types/canvas.ts` | 新增 | 从 `src/lib/types.ts` 搬出 editable canvas 纯类型。 |
| `prototypes/productized-desktop-shell/src/lib/types.ts` | 修改 | 保留原类型入口，通过 `export type { ... } from "./types/canvas"` 继续对外导出 canvas 类型。 |

生成产物：

- `npm run build` 生成/更新了 `prototypes/productized-desktop-shell/dist/index.html` 和 `dist/assets/*`。

## 3. 后端拆分方式

本轮选择 `include!`，不是完整 Rust namespace 模块化。

原因：

- `lib.rs` 里大量类型字段仍由同文件函数直接构造和读取。
- 如果改成 `mod types; use types::*;`，需要批量调整结构体和字段可见性，改动面会扩大。
- 保护决策要求第一批保持 Tauri command 名字、请求字段、响应字段、serde 字段名和测试语义不变。

判断：

- 这是“保守文件拆分”，不是最终模块边界。
- 下一轮如果要做真正 Rust 模块边界，需要单独处理可见性，并配套更细的回归测试。

## 4. 未搬原因

| 候选 | 本轮未搬原因 |
|---|---|
| workflow 读模型函数 | `derive_workflow_read_model` 串联 `inspect_workflow_run_check_from_value`、任务包、账本、汇报、审查、异常、状态机、接口边界等一组 helper。直接搬会迫使开放大量私有函数，风险高于本轮保守切片。 |
| WorkbenchSnapshot 组装函数 | `build_snapshot` 依赖 `AppState`、`SessionSourceMode`、`codex_db`、路径白名单、项目/会话/技能/插件解析等。直接搬会牵涉读取策略和敏感路径边界。 |
| MCP canvas/run 相关文件 | 决策明确不纳入 Task B 第一批。 |
| 工作流机器 | 决策明确第一批不能碰。 |
| Codex resume runner | 决策明确第一批不能碰。 |

## 5. 验证

已通过：

| 命令 | 结果 | 备注 |
|---|---|---|
| `npm run typecheck` | 通过 | TypeScript 类型检查通过。 |
| `npm run test:offline-interaction` | 通过 | 输出 `offline interaction tests passed: 2`。 |
| `npm run build` | 通过 | Vite build 通过，有 chunk 大于 500 kB 的既有构建警告。 |
| `HOME=/private/tmp/codex-task-b-home RUSTUP_HOME=/Users/yoyi/.rustup CARGO_HOME=/Users/yoyi/.cargo cargo test --lib` | 通过 | 81 passed，0 failed，1 ignored。临时 `HOME` 避免读取真实 `/Users/yoyi/.codex/state_5.sqlite`。 |
| `rustfmt --check src/types.rs src/commands.rs` | 通过 | 新增 Rust 文件格式检查通过。 |

未通过但未修改：

| 命令 | 结果 | 处理 |
|---|---|---|
| `cargo fmt --check` | 未通过 | 会要求重排既有 `lib.rs` 大段代码和 `src-tauri/src/mcp/**` 文件。本轮禁止碰 MCP 可编辑画布运行逻辑，所以没有执行批量格式化。 |

说明：

- 第一次用 `HOME=/private/tmp/codex-task-b-home cargo test --lib` 时，rustup 因临时 HOME 下没有默认 toolchain 失败；随后显式设置 `RUSTUP_HOME` 和 `CARGO_HOME` 后测试通过。
- 没有执行真实 Codex。
- 没有读取 `/Users/yoyi/.codex`。

## 6. 行为风险判断

本轮改动应为无行为变化。

依据：

- 后端类型字段没有改名。
- Tauri command 函数名没有改。
- `tauri::generate_handler!` 里的命令名没有改。
- workflow state JSON 读写函数没有改。
- 状态机函数没有改。
- Codex resume runner 没有改。
- 工作流机器函数没有改。
- MCP canvas/run 文件没有改。
- 前端现有 `../lib/types` / `./types` import 入口没有改。

剩余风险：

- `include!` 只是文件拆分，不是最终模块边界。
- 真正拆 workflow 读模型和 WorkbenchSnapshot 组装时，还会遇到私有 helper 和可见性问题。
- `cargo fmt --check` 仍显示既有格式问题；如果下一轮想清理格式，必须另开纯格式化任务，并确认是否允许触碰 MCP 文件文本。
