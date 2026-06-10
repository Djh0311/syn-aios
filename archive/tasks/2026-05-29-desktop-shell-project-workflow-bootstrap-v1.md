# 任务包：项目默认工作流草稿初始化 v1

## 任务名

在产品化桌面壳中，为选中的项目创建并展示一个默认工作流草稿。

## 所属开发线

桌面应用线。

当前派给桌面应用线，因为任务涉及 Tauri 后端状态文件写入、前端项目详情页展示和确认式本机写入。

## 背景

阶段 3 目标是 Codex 项目级可视化编排。

依据：

- `product-line/STAGE_PLAN.md`：阶段 3 要求项目打开后进入项目级可视化工作流，并能表达 Director、Codex 会话、任务包、handoff、evidence、review 的流转。
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`：已定义 workflow、node、edge、work_item、artifact、review、audit 等最小模型。
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`：本地工作流事实层 v0 用 JSON 文件保存。
- `product-line/handoffs/2026-05-28-desktop-shell-workflow-state-v0-review.md`：桌面壳已实现 v0 JSON 状态文件最小读写底座，但不是可编辑工作流。
- `product-line/handoffs/2026-05-29-desktop-shell-workflow-state-v0-validation-review.md`：真实窗口已看到 v0 面板和初始化确认弹层；原旁路观察式验证不适配真实试用。用户已明确不需要处理确认按钮防误触，继续推进工作流能力。

当前缺口：

- 工作台还不能把一个项目保存为本地工作流事实。
- 当前工作流画布仍主要来自索引候选和前端骨架。
- 还没有真正的项目 workflow、node、edge、audit 记录。

## 目标

实现最小可用的“项目默认工作流草稿初始化”。

用户在项目详情页选择一个项目后，可以通过确认式动作创建该项目的默认工作流草稿。

写入工作台自己的本地状态文件：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
```

至少写入：

- 一个 `projects[]` 记录，来自当前选中项目。
- 一个 `workflows[]` 记录，状态为草稿或等价状态。
- 一组默认 `nodes[]`：
  - 总指导 / Director
  - Codex 开发线
  - 验证线
  - 任务包
  - Handoff
  - Evidence
  - Review
- 一组默认 `edges[]`，表达：
  - 总指导派发任务包
  - 任务包交给 Codex 开发线
  - 开发线产出 handoff / evidence
  - 验证线验证产物
  - Review 回收判断
- 一条 `audit_events[]`，记录用户确认创建项目默认工作流草稿。

前端至少显示：

- 当前项目是否已有本地工作流草稿。
- 已有 workflow / node / edge 数量。
- 创建默认工作流草稿按钮。
- 创建前的确认弹层，说明写入的是工作台自己的状态文件，不写 `.codex`、不写 Codex 状态库、不写项目业务目录。
- 创建后的成功提示和刷新后的本地事实层数量。

## 不做

- 不做自动派发给真实 Codex 会话。
- 不做多会话编排执行。
- 不读取或展示 Codex 会话正文、工具输出、输入历史。
- 不自动生成真实任务包文件。
- 不登记真实回收意见。
- 不做节点拖拽编辑。
- 不做边编辑。
- 不做状态转换。
- 不自动运行 harness。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、LM 调度。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/tasks/README.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`
- `product-line/handoffs/2026-05-28-desktop-shell-workflow-state-v0-review.md`
- `product-line/handoffs/2026-05-29-desktop-shell-workflow-state-v0-validation-review.md`
- `product-line/prototypes/productized-desktop-shell/`
- `product-line/prototypes/index-kernel/codex-index.json`

允许做存在性检查，不读取内容：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

## 允许写入

- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/prototypes/productized-desktop-shell/package.json`
- `product-line/prototypes/productized-desktop-shell/tsconfig.json`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src/`
- `product-line/evidence/`
- `product-line/handoffs/`

真实运行时允许由 Tauri 后端在用户确认后写入：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不写项目业务目录。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不把索引候选自动升级成“已验证能力”。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、LM 调度。
- 不做 release 打包。
- 不拉取外网依赖。
- 不实现自动编排执行。

## 实现建议

后端：

- 新增 Tauri 命令，例如 `bootstrap_project_workflow`。
- 输入只接受索引内项目路径或项目标识，不接受任意输出路径。
- 后端从当前索引里确认项目存在。
- 如果状态文件不存在，可以先初始化最小 v0 状态，再写项目 workflow；或返回需要初始化，由前端引导。二选一，但必须记录清楚。
- 写入前如果已有状态文件，先备份。
- 写入使用临时文件 + 原子替换。
- 写入后重新读取校验。
- 如果该项目已有 workflow，不要重复创建；返回已有记录或明确 warning。

前端：

- 在项目详情页和工作流事实层面板之间建立关系。
- 创建按钮只对当前选中项目生效。
- 确认弹层文案用大白话说明：这是给工作台自己的小账本写入一个项目工作流草稿。
- 创建后刷新 snapshot 和 workflow-state snapshot。

## 验收标准

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- 新增 Rust 单测覆盖：
  - 状态文件不存在时的处理策略。
  - 为索引内项目创建默认 workflow。
  - 同一项目重复创建不产生重复 workflow。
  - 非索引项目被拒绝。
  - 写入前备份旧状态文件。
- 新增或更新离线前端交互测试：
  - 项目页可以打开创建默认工作流确认弹层。
  - 取消不会调用创建动作。
  - 确认动作显示目标路径和写入边界。
- evidence / handoff 说明真实状态文件任务前后是否存在。
- 如运行真实 Tauri 窗口，只允许在用户明确确认后创建或修改工作台状态文件。

## 必须回传

1. 薄弱点
2. 做了什么
3. 改了哪些文件
4. 新增或修改了哪些测试
5. 默认 workflow 写入了哪些对象
6. 如何避免重复创建
7. 如何保证只对索引内项目生效
8. 验证命令和结果
9. 真实状态文件任务前后是否存在
10. 是否触碰禁止事项
11. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否只实现项目默认工作流草稿初始化，没有扩大到自动编排执行。
- 是否没有写 `.codex`、Codex 状态库或项目业务目录。
- 是否不读取会话正文或敏感内容。
- 是否有防重复创建策略。
- 是否写入的 workflow / node / edge 符合最小模型。
- 是否适合下一步做任务包生成或节点状态流转。
