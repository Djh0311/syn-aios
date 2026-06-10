# Codex 工作台新版 UI 骨架实现交接

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-codex-workbench-ui-shell-redesign.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-28-codex-workbench-ui-shell-redesign.md`

## 结论

建议接受为新版 UI 骨架实现。

不建议接受为完整工作台发布版。依据：真实最近使用事件、工作流状态机、Director / Review 数据、Skill 推荐关系、Harness 验证状态仍不存在。

## 做了什么

- 把主导航从旧六入口改成首页、Agent、项目、Skill 管理、Harness 管理。
- 首页改成四个主入口卡片：Agent、项目、Skill 管理、Harness 管理。
- 首页入口下方显示最近项，不显示数量。
- 新增 Agent 页：只展示 Codex 可用卡片，其他 agent 为空白未接入。
- 重做项目页：左侧项目列表，右侧项目详情。
- 项目详情采用左侧窄功能列表、中间工作流画布、右侧详情面板。
- 工作流画布表达项目中心、Codex 会话、Handoff、Evidence、Harness 候选，并显示 Director / Review / 边关系缺口。
- 新增 Skill 管理关系看板骨架。
- 新增 Harness 管理框架看板骨架。
- 更新离线交互测试，覆盖首页四入口、Agent 空白位、项目工作流、Skill 看板、Harness 看板和路径动作权限弹层。
- 后端只补充读取当前索引已有字段：skill 描述、harness 更新时间、harness 大小。

## 改了哪些文件

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
- `product-line/evidence/2026-05-28-codex-workbench-ui-shell-redesign.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-redesign-result.md`

## 新增或更新了哪些测试

更新：

- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

覆盖内容：

- 首页包含 Agent、项目、Skill 管理、Harness 管理四个入口。
- 首页不显示旧统计数量文案。
- Agent 页包含 Codex 和未接入 agent 空白位。
- 未接入 agent 不出现操作能力。
- 项目详情包含工作流、Codex 会话、Handoff、Evidence、Harness 候选、Director、Review 和缺少数据说明。
- Skill 管理包含分类、Agent 使用关系、项目使用关系、推荐关系占位、缺少字段说明。
- Harness 管理包含框架 / 类型、版本和来源、功能和场景、项目适配、不自动运行 harness。
- 项目打开目录、复制路径仍触发待确认动作，并由权限弹层展示目标路径和路径来源。

## 新版首页如何实现

- `HomeView.tsx` 改成四张入口卡片。
- 四个入口分别调用主应用导航：Agent、项目、Skill 管理、Harness 管理。
- 不显示项目数、会话数、skill 数、plugin 数等统计卡片。
- 最近项目用 `projects.latest_updated_at_ms` 近似最近活跃。
- 最近 Skill 用 `skills` 静态索引候选。
- 最近 Harness 用 `harness_candidates.updated_at_ms` 近似最近修改。
- Agent 当前只显示 Codex。
- 页面显式说明这些最近项不是可靠真实使用事件。

## 项目级工作流如何实现

- `ProjectsView.tsx` 内部保留项目列表。
- 选中项目后进入 `ProjectDetail`。
- `ProjectDetail` 采用三栏：
  - 左侧窄功能列表：工作流、会话、任务包、Handoff / Evidence、Skills、Harness、设置。
  - 中间 `WorkflowCanvas`：React / CSS 节点和连线骨架。
  - 右侧 `ProjectSidePanel`：显示当前项目元数据和缺口。
- 工作流节点包括项目中心、Codex 会话、Handoff、Evidence、Harness 候选。
- Director、Review 和缺边关系用缺口节点显示，不假装已有数据。

## Skill 管理看板如何实现

- 新增 `SkillsBoardView.tsx`。
- 按 `source_type` 做分类。
- Agent 使用关系只把 Codex 标成当前可展示。
- 其他 agent 不参与推荐或加载。
- 项目使用关系显示缺字段说明。
- 推荐关系显示占位，不自动推荐。
- Plugin 信息只作为来源元数据展示。

## Harness 管理看板如何实现

- 新增 `HarnessBoardView.tsx`。
- 从项目的 `harness_candidates` 汇总候选。
- 按 `entry_type` 展示框架 / 类型。
- 版本和来源列显示已有 source、path、updated_at_ms，同时说明版本和来源仓库缺失。
- 功能和场景列说明索引没有功能说明、场景标签或命令语义。
- 项目适配列显示哪些项目有 harness 候选。
- 明确不自动运行 harness，也不写验证状态。

## 字段来源和缺口

来自当前索引：

- 项目、会话元数据、handoff 候选、evidence 候选、harness 候选、skill 元数据、plugin 元数据。

仍是缺口：

- 真实最近打开 / 最近使用事件。
- Director、Review、工作流边关系、状态机和坐标。
- Skill 使用关系、推荐关系、加载状态。
- Harness 版本、来源仓库、功能说明、使用场景、关联命令语义、最近验证状态。

## 验证命令和结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`

浏览器检查：

- `npm run dev` 在沙箱内绑定 5173 失败，提权启动后成功。
- 普通浏览器打开 `http://127.0.0.1:5173/`，确认主导航为首页、Agent、项目、Skill 管理、Harness 管理。
- 普通浏览器不是 Tauri 窗口，数据读取失败提示符合现有保护逻辑。

端口清理：

- 已停止 Vite。
- `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无监听输出。

## 禁止事项

未触碰禁止事项：

- 未写 `/Users/yoyi/.codex`。
- 未改 Codex 状态库。
- 未读取或展示 auth、env、密钥、令牌、授权文件内容。
- 未读取或展示会话正文、工具输出、命令输出、输入历史、记忆正文。
- 未接入非 Codex agent。
- 未把 OpenClaw / VS Code / OpenCode / Claude Code 写成可用能力。
- 未做知识库、向量搜索、模型调度。
- 未做真实 skill 删除、编辑或加载。
- 未做完整 harness 多仓库多版本管理。
- 未自动运行 harness。
- 未做 release 打包、签名、自动更新、托盘、通知或登录项。
- 未拉外网依赖。

## 风险和下一步

- 旧视图文件仍存在但不挂主导航。后续建议在确认新版入口稳定后清理或迁移旧文件，避免误用。
- 普通 Vite 页面不能验证 Tauri invoke 读取索引。后续如要做视觉验收，建议启动 Tauri 窗口验证真实数据渲染。
- 项目内除工作流外的左侧功能项仍是占位。后续应逐步把会话、任务包、Handoff / Evidence、Skills、Harness、设置迁入项目详情内部。
- 若要做真正工作流，需要先补索引或本地事实库字段，不应继续靠前端推断。
