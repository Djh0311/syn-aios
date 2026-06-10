# 任务包：项目内 Agent 会话入口 v1

## 所属开发线

桌面应用线。

协作开发线：Codex 会话线、信息架构线。

## 背景

当前工作台已纠偏为 Codex 会话管理和 Codex 工作流编排，不再把任务包管理器作为主流程中心。

已完成：

- Agent 页已实现 Codex 会话中心只读 UI v1。
- 后端已有 `load_codex_session_transcript(thread_id)` 只读命令。
- 项目内 Agent 会话架构已决策：项目页也能打开单独 Agent 会话，但必须复用 Agent 页同一套会话能力，不能做第二套聊天系统。

依据：

- `product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-codex-session-center-readonly-v1-result.md`
- 用户明确要求：“项目里也能打开单独的agent会话”。

## 已知、未知和假设

已知：

- `SessionRecord.project_root` 可用于按项目过滤会话。
- Agent 页已能展示会话列表并读取单个 transcript。
- 当前只验证了只读 transcript，没有验证发送、resume、多轮聊天。

未知：

- 项目归属是否总能由 `project_root` 准确表示。
- 未来用户绑定、工作流绑定与索引推断如何合并到同一个项目会话列表。
- 项目页最终会话布局是中央面板、右侧抽屉还是标签页。

假设：

- v1 先用索引推断的 `project_root` 做项目过滤。
- v1 不新增用户绑定写入。
- v1 不做发送消息、新建会话、resume、删除、移动。
- v1 复用 Agent 页已有只读 transcript 能力。

## 目标

在项目详情页增加项目内 Agent 会话入口：

- 左侧项目功能列表出现 `Agent 会话`。
- 进入该项后显示当前项目关联的 Codex 会话。
- 可以在项目页内打开单个 Codex 会话 transcript。
- 会话时间线、工具调用、工具结果、命令输出和 warning 展示复用 Agent 页同一套组件或同一套展示逻辑。
- 明确显示项目归属来源为 `索引推断`。
- 当前不支持发送消息、resume、新建会话、删除、移动。

大白话：项目页里能打开这个项目下的 Codex 会话，但不是再造一套聊天系统。

## 非目标

- 不做真实 Tauri 窗口 smoke。
- 不做发送消息。
- 不做持续多轮聊天。
- 不做 `codex resume`。
- 不做新建会话 UI。
- 不做删除、移动、归档会话。
- 不写 `/Users/yoyi/.codex`。
- 不改 Codex 状态库。
- 不运行 Codex CLI。
- 不运行 harness。
- 不做多 agent。
- 不做知识库。

## 建议实现

建议从现有 `AgentView.tsx` 抽出或复用展示组件：

- `AgentSessionCenter`
- 会话列表展示
- `SessionReader`
- transcript 时间线展示

如果当前组件还不便复用，可以做最小改造：

- 导出项目页需要的只读会话组件。
- 让组件支持 `scope="global" | "project"` 或通过 props 控制标题和空状态文案。
- 项目页只传入过滤后的 sessions。

项目页改动建议：

- 在 `ProjectDetail` 左侧功能列表增加 `Agent 会话`。
- 选中 `Agent 会话` 时展示项目会话面板。
- 过滤逻辑：

```text
session.project_root === project.project_root
```

- 会话为空时显示：

```text
当前项目没有索引推断关联的 Codex 会话。
```

- 有会话时显示：
  - 会话标题
  - thread id 短号
  - 更新时间
  - rollout 状态
  - 项目归属来源：索引推断
  - 打开 / 读取正文入口

## 允许读取

- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-codex-session-center-readonly-v1-result.md`
- 当前静态 `codex-index.json` 中的会话元数据。

如需 transcript 读取，只能通过现有后端命令读取用户在 UI 中点选的单个索引内会话。

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 授权文件内容
- 密钥文件内容
- 与本任务无关的业务会话正文

## 允许写入

允许写入：

- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/evidence/2026-05-29-desktop-shell-project-agent-session-entry-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-agent-session-entry-v1-result.md`

如确实需要，为复用类型或命令可写：

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

但本任务不应新增写 `.codex` 的后端命令。

## 禁止事项

- 不执行 Codex CLI。
- 不创建新 Codex 会话。
- 不发送 prompt。
- 不运行 `codex resume`。
- 不运行 `codex fork`。
- 不删除、移动、归档任何 Codex 会话。
- 不读取 `auth.json`、`.env`、授权文件、密钥文件。
- 不把完整 transcript 写入 evidence / handoff。
- 不运行 harness。
- 不把任务包管理器重新作为主流程中心。
- 不复制一套独立项目聊天系统。

## 验收标准

- 项目页左侧功能列表有 `Agent 会话`。
- 项目页能按当前项目过滤 Codex 会话。
- 项目页能打开单个会话 transcript。
- 项目页会话时间线展示与 Agent 页保持一致或复用同一组件。
- UI 明确显示项目归属来源为 `索引推断`。
- 没有发送消息、新建会话、resume、删除、移动入口。
- 空项目会话时有明确空状态。
- 前端离线测试覆盖项目内 Agent 会话入口。
- 类型检查、构建、离线交互测试通过。
- 如改 Rust，Rust 单测通过。

## 建议验证

建议运行：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline
```

本任务不要求真实 Tauri 窗口 smoke。是否派验证线另行决定。

## 必须回传

1. 薄弱点先说。
2. 做了什么。
3. 改了哪些文件。
4. 是否复用了 Agent 页会话组件。
5. 项目页如何过滤会话。
6. 是否显示项目归属来源。
7. 是否新增发送、resume、删除、移动等危险入口。
8. 验证命令和结果。
9. 是否读取授权、密钥或业务会话正文。
10. 下一步建议。

## 总指导回收重点

回收时重点检查：

- 是否真的在项目页能打开单独 Agent 会话。
- 是否复用同一套会话能力，没有复制第二套聊天系统。
- 是否仍然只是只读能力，没有偷偷加发送 / resume。
- 是否把索引推断显示成候选来源，而不是用户确认事实。
- 是否没有偏回任务包管理器。
