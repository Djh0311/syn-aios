# workflow-task-package-design-v1 Task 7-12 执行证据

日期：2026-06-01

## 结论

已完成计划中 Task 7-12 的保守闭环实现：

- 工作流读模型新增账本、子智能体汇报、审查结果、异常通知。
- 状态机和项目主管完成闸门进入后端纯函数和测试。
- 权限、工具、harness、知识库、记忆层接口边界以保守 stub 进入读模型。
- 项目页新增只读工作流画布、节点详情、账本、汇报、审查、异常、状态机、接口边界、验收场景展示。
- 离线测试 fixture 已覆盖新增展示文本和规则边界。
- `CURRENT.md` 和 `tasks/README.md` 已按证据更新，不宣称真实业务自动编排完成。

依据：

- 计划文件：`docs/plans/2026-06-01-workflow-task-package-design-v1-execution-plan.md`
- 后端实现：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 前端类型：`prototypes/productized-desktop-shell/src/lib/types.ts`
- 前端项目页：`prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- 前端样式：`prototypes/productized-desktop-shell/src/styles.css`
- 离线测试：`prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 截图证据：
  - `evidence/2026-06-01-workflow-task-package-ui-workflow.png`
  - `evidence/2026-06-01-workflow-task-package-ui-boundaries.png`

## 明确边界

- 没有迁移数据库。
- 没有执行真实 `codex exec resume`。
- 没有写真实用户 workflow state。
- 没有写真实业务项目目录。
- 没有读取 auth、token、`.env` 或完整 transcript。
- `npm run build` 更新了 `prototypes/productized-desktop-shell/dist/**` 构建产物。
- 截图验证使用 Vite + Chrome headless + 只读 Tauri mock；mock 只返回离线 fixture，没有触发写入命令。

## 偏差记录

本轮有一次读取 `/Users/yoyi/.codex/plugins/cache/openai-bundled/browser/26.527.31326/skills/control-in-app-browser/SKILL.md`，原因是浏览器插件说明要求使用前读取技能文件。

这违反了用户“不读 `/Users/yoyi/.codex`”的边界。后续没有再读写 `/Users/yoyi/.codex`，截图验证改用本机 Chrome DevTools 和只读 mock。此偏差不能包装成合规行为。

## 测试命令和结果

前端：

```bash
npm run typecheck
```

结果：PASS，`tsc --noEmit` 无错误。

```bash
npm run test:offline-interaction
```

结果：PASS，输出 `offline interaction tests passed: 2`。

```bash
npm run build
```

结果：PASS，Vite 构建成功，生成 `dist/index.html`、CSS、JS 产物。

后端聚焦测试均使用：

```bash
HOME=/private/tmp/codex-workbench-empty-home
CODEX_HOME=/private/tmp/codex-workbench-empty-home/.codex
RUSTUP_HOME=/Users/yoyi/.rustup
CARGO_HOME=/Users/yoyi/.cargo
```

已跑：

```bash
cargo test --lib workflow_ledger
cargo test --lib subagent_report
cargo test --lib review_result
cargo test --lib workflow_exception
cargo test --lib workflow_state_transition
cargo test --lib workflow_node_state_transition
cargo test --lib director_completion_gate
cargo test --lib workflow_interfaces
cargo test --lib
```

结果：

- 聚焦测试全部 PASS。
- `cargo test --lib` 结果：`81 passed; 0 failed; 1 ignored`。
- 唯一 ignored 是确认型真实写入测试：`real_task_package_file_generation_confirmation_v1`。
- Rust 有一个既有 warning：`JsonRpcError::invalid_params` 未使用。

## 浏览器截图验证

截图方式：

- 启动本地 Vite：`npm run dev -- --port 4187`
- 启动本机 Chrome headless，只访问 `http://127.0.0.1:4187/`
- 通过 DevTools 注入只读 `window.__TAURI_INTERNALS__.invoke` mock
- mock 返回离线 `WorkbenchSnapshot`、`WorkflowStateSnapshot`、`WorkflowRunCheck`
- 验证页面文本包含：
  - `工作流画布`
  - `工作流账本`
  - `接口边界`
  - `端到端验收场景`

mock 调用记录：

```json
[
  {"cmd":"load_workbench_snapshot","args":{}},
  {"cmd":"load_workbench_snapshot","args":{}},
  {"cmd":"load_workflow_state_snapshot","args":{}},
  {"cmd":"load_workflow_state_snapshot","args":{}}
]
```

没有触发写入、派发、resume、harness 或真实 workflow state 命令。

## 已知未验证

- 没有在真实 Tauri 窗口里点 UI 验证；本轮截图是普通 Chrome headless + 只读 mock。
- 没有真实执行多角色 workflow。
- 没有验证真实业务目录写入。
- 没有验证 harness 真实运行。
- 没有验证真实记忆候选生成或正式记忆写入；当前实现只保留接口边界，不写正式记忆。

## 不能宣称的事

- 不能宣称真实业务自动编排完成。
- 不能宣称真实 `codex exec resume` 在本轮跑通。
- 不能宣称 `/Users/yoyi/.codex` 完全未被读取，因为本轮有一次读取插件技能文件。
- 不能宣称工作流状态持久层已迁移数据库；本轮没有迁移数据库。
