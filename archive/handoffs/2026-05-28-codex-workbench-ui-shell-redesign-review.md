# Codex 工作台新版 UI 骨架实现总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-codex-workbench-ui-shell-redesign.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-28-codex-workbench-ui-shell-redesign.md`
- Handoff：`product-line/handoffs/2026-05-28-codex-workbench-ui-shell-redesign-result.md`
- 被回收产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“Codex 工作台新版 UI 骨架实现”。

不接受为“完整工作台发布版”，也不接受为“可编辑自动化工作流已经完成”。

依据：

- 主导航已收敛为首页、Agent、项目、Skill 管理、Harness 管理。
- 首页只保留四个入口，入口下方显示最近项口径，不显示数量。
- Agent 页只把 Codex 标为可用，OpenClaw / VS Code / OpenCode / Claude Code 均为未接入空白位。
- 项目页已改成项目列表和项目详情；项目详情包含左侧窄功能列表、中间工作流画布、右侧详情面板。
- 工作流画布表达项目中心、Codex 会话、Handoff、Evidence、Harness 候选，并把 Director、Review、边关系和状态机标成缺口。
- Skill 管理和 Harness 管理已经从旧列表改成看板骨架。
- 总指导线复跑 `npm run typecheck`、`npm run test:offline-interaction`、`npm run build`、`cargo test --offline` 均通过。
- 端口复核 `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无监听输出。

## 先说薄弱点

- 这仍是 UI 骨架，不是完整工作台。依据：真实最近打开事件、Director 拆解、Review 回收、工作流边关系、状态机、Skill 推荐、Harness 验证状态都还没有数据模型。
- 工作流画布是 React / CSS 节点和连线骨架，不是可拖拽、可编辑、可执行的编排引擎。依据：本轮没有引入 React Flow，也没有新增工作流状态存储。
- 项目内左侧功能列表目前主要是结构占位。依据：代码中左侧按钮没有切换会话、任务包、Handoff / Evidence、Skills、Harness、设置等子视图。
- 本轮没有做 Tauri 窗口真实视觉验收。依据：开发线只记录普通 Vite 页面检查，普通浏览器不是 Tauri 窗口，不能验证 Tauri invoke 真实读取索引后的完整 UI。
- 离线交互测试当前输出为 `offline interaction tests passed: 3`。依据：总指导线复跑结果；它覆盖一个综合 shell 场景和两个项目路径动作场景，不等于旧 UI 的 4 个路径动作场景。
- 旧视图文件仍存在。依据：`SessionsView.tsx`、`TasksEvidenceView.tsx`、`DiagnosticsView.tsx`、`SkillsPluginsView.tsx` 仍保留，但不挂主导航。

## 接受内容

接受桌面应用线本轮实现范围：

- `App.tsx` 主导航改为五个主视图：首页、Agent、项目、Skill 管理、Harness 管理。
- `HomeView.tsx` 实现四入口首页，最近项使用索引近似口径，并明确说明不是真实使用事件。
- `AgentView.tsx` 只展示 Codex 可用能力，其他 agent 显示未接入，不展示操作能力。
- `ProjectsView.tsx` 实现项目列表、项目详情、左侧功能列表、工作流画布和右侧详情面板。
- `SkillsBoardView.tsx` 实现 Skill 分类、Agent 使用关系、项目使用关系、推荐关系占位和缺字段说明。
- `HarnessBoardView.tsx` 实现 Harness 类型、版本和来源、功能和场景、项目适配和验证入口看板骨架。
- `lib.rs` 继续保持只读索引读取、路径白名单、复制路径、打开项目、定位 rollout 的后端边界。
- `offline-permission-dialog.test.tsx` 更新为新版 UI 骨架的离线组件验证。

## 验证复跑

总指导线在 `product-line/prototypes/productized-desktop-shell/` 复跑：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，输出 `offline interaction tests passed: 3`。
- `npm run build` 通过，生成 `dist/index.html`、`dist/assets/*.css`、`dist/assets/*.js`。

总指导线在 `product-line/prototypes/productized-desktop-shell/src-tauri/` 复跑：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home \
CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target \
cargo test --offline
```

结果：

- 3 个 Rust 单测通过。

端口复核：

```bash
lsof -nP -iTCP:5173 -sTCP:LISTEN
```

结果：

- 无监听输出。

## 安全边界判断

接受当前安全边界。

依据：

- 本轮回收没有写 `/Users/yoyi/.codex`。
- 没有改真实 Codex 状态库。
- 没有读取或展示 `auth.json`、`.env`、密钥、令牌或授权文件内容。
- 没有读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 没有接入非 Codex agent。
- 没有把 OpenClaw / VS Code / OpenCode / Claude Code 写成已可用能力。
- 没有做个人知识库、向量搜索、模型调度或 release 打包。
- 没有自动运行 harness。

## 当前状态

这条任务从“待派发”改为“已回收”。

当前可作为阶段 3 的起点：

- 新版首页四入口已经落地。
- 项目详情工作流骨架已经落地。
- Skill 管理和 Harness 管理看板骨架已经落地。

但还不能进入“可编辑自动化工作流”口径。

## 下一步建议

下一步建议派给验证线做 Tauri 窗口真实渲染 smoke test。

建议验证范围：

- Tauri 窗口能读取真实静态索引。
- 首页只显示四个入口，且不显示数量。
- Agent 页只显示 Codex 可用和其他 agent 未接入。
- 项目详情默认进入工作流画布。
- Skill 管理和 Harness 管理是看板骨架。
- 不展示正文和敏感内容。
- 验证后无 5173 残留监听。

暂不建议继续加功能。理由是当前 UI 骨架还缺真实 Tauri 视觉验收，直接进入工作流状态机容易把未验证 UI 当作稳定底座。
