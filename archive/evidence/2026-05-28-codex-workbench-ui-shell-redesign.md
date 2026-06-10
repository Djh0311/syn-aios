# Codex 工作台新版 UI 骨架实现证据

## 结论

薄弱点先说：

- 这轮只是 UI 骨架，不是完整工作台发布版。依据：没有新增真实最近打开事件、工作流状态机、Director / Review 数据、Skill 推荐模型或 Harness 验证状态。
- 首页最近项仍是临时口径。依据：当前索引没有真实最近打开、最近查看 skill、最近使用 harness 事件。
- 项目工作流是 React / CSS 画布骨架，不是可拖拽编排引擎。依据：任务包允许不引入 React Flow，并要求缺字段显式说明。
- Skill 管理和 Harness 管理是看板骨架。依据：本轮没有做 skill 删除、编辑、加载，也没有自动运行 harness。

可接受点：

- 已把产品化桌面壳主导航改为首页、Agent、项目、Skill 管理、Harness 管理。
- 首页只显示四个入口：Agent、项目、Skill 管理、Harness 管理；入口下方显示最近项，不显示数量。
- Agent 页只显示 Codex 可用卡片和 OpenClaw / VS Code / OpenCode / Claude Code 未接入空白位。
- 项目页实现项目列表和项目详情；项目详情是左侧窄功能列表、中间工作流画布、右侧详情面板。
- 项目级工作流表达项目中心、Codex 会话、Handoff、Evidence、Harness 候选，并显式显示 Director / Review / 边关系缺口。
- Skill 管理页改为关系看板骨架，包含分类、Agent 使用关系、项目使用关系、推荐关系占位和缺字段说明。
- Harness 管理页改为框架看板骨架，包含框架 / 类型、版本和来源、功能和场景、项目适配和验证入口、缺字段说明。
- 保留 Tauri 2 + Rust + React + TypeScript + Vite 技术底座。
- 保留只读索引读取、路径白名单和权限确认弹层。

## 本轮读取依据

- `product-line/tasks/2026-05-28-codex-workbench-ui-shell-redesign.md`
- `product-line/decisions/2026-05-28-codex-workbench-ui-ia-direction.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-ia-redesign-review.md`
- `product-line/evidence/2026-05-28-codex-workbench-ui-ia-redesign.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-ia-redesign-result.md`
- `product-line/prototypes/productized-desktop-shell/`

没有读取或展示：

- `auth.json`
- `.env`
- 密钥、令牌、授权文件内容
- Codex 会话正文、工具输出、命令输出、输入历史、记忆正文

## 修改文件

- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/HomeView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/SkillsBoardView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/HarnessBoardView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

旧视图文件仍保留，但不再作为主导航入口引用：

- `src/views/SessionsView.tsx`
- `src/views/TasksEvidenceView.tsx`
- `src/views/DiagnosticsView.tsx`
- `src/views/SkillsPluginsView.tsx`

## 字段来源

来自当前索引：

- 项目：`projects[].project_root`、`name`、`latest_updated_at_ms`、`thread_count`、`active_thread_count`、`archived_thread_count`
- 会话元数据：`threads[].title`、`project_root`、`updated_at_ms`、`archived`
- Handoff / Evidence：`projects[].handoff_files`、`projects[].evidence_files`
- Harness 候选：`projects[].harness_candidates[].entry_type`、`name`、`path`、`source`、`size_bytes`、`updated_at_ms`
- Skill：`skills[].skill_id`、`title`、`description`、`path`、`source_type`、`plugin_name`、`plugin_version`
- Plugin：`plugins[].plugin_name`、`plugin_version`、`skill_paths`、`has_apps`、`has_mcp_servers`

仍是缺口：

- 真实最近打开事件
- 真实最近查看或使用 skill 事件
- 真实最近使用 harness 事件
- Director 节点数据
- Review / 回收状态数据
- 工作流边关系、坐标和状态机
- Skill 到 agent / 项目的使用关系
- Skill 推荐关系和加载状态
- Harness 框架规范化、版本、来源仓库、功能说明、使用场景、关联命令语义、最近验证状态

## 验证

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`

浏览器检查：

- 普通 Vite dev server 页面可打开，主导航显示首页、Agent、项目、Skill 管理、Harness 管理。
- 普通浏览器不是 Tauri 窗口，所以数据读取区域显示“当前页面不在 Tauri 窗口中运行”。这是现有保护逻辑，不是 Tauri 验证。
- 项目工作流、Skill 看板、Harness 看板通过离线 React 组件测试覆盖。

端口清理：

- 验证后执行 `lsof -nP -iTCP:5173 -sTCP:LISTEN`，无监听输出。

## 禁止事项检查

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未读取或展示 auth、env、密钥、令牌、授权文件内容。
- 未读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未接入非 Codex agent。
- 未把 OpenClaw / VS Code / OpenCode / Claude Code 写成可用能力。
- 未做个人知识库、向量搜索或模型调度。
- 未做真实 skill 删除、编辑或加载。
- 未自动运行 harness。
- 未做 release 打包、签名、自动更新、托盘、通知或登录项。
- 未为 UI 拉取外网依赖。

## 风险

- 普通浏览器无法验证 Tauri invoke 成功读取索引；本轮以 Rust 测试和离线组件测试覆盖数据映射。
- 旧视图文件仍在代码库中，虽然不挂主导航，但后续如果继续保留可能造成认知噪音。
- 项目内左侧功能列表目前只有工作流视图真正落地，其他功能是迁移位置占位。
