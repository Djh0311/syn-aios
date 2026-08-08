# AGENTS.md — product-line 开发协作规则（Harness Lite 适配）

全程中文和大白话。当前用户指令决定要做什么；本文件保留项目自己的协作边界，Harness Lite 只管理开发工作的生命周期、授权与最小相关验证，不承载产品功能。

## 唯一开发入口

每次开始或上下文压缩后依次运行：

```text
node .claude/harness-lite/bin/hl.js chain --target .
node .claude/harness-lite/bin/hl.js progress --target .
node .claude/harness-lite/bin/hl.js auth --target .
```

开发生命周期唯一链是：

```text
AGENTS.md
→ docs/harness/plan.md
→ docs/harness/stages/ 中唯一当前 stage
→ docs/harness/leaves/ 中唯一 current leaf
→ docs/harness/authorization.json（只核对当前用户授权）
```

`docs/plans/`、`docs/contracts/`、`tasks/`、`handoffs/`、`archive/`、`plans/v0.5.0/` 和旧治理状态只提供产品输入或历史证据，不授予执行权限。根 README 只做产品介绍。

## 五类硬边界

只有以下事情进入重档：

1. 进入远端、服务器、生产、真实 App、真实账号或真实消息，并造成真实副作用。
2. 读取凭据，或写用户级 `/Users/yoyi/.codex`。
3. 删除、覆盖、清理或做其他难恢复操作。
4. 修改或结束当前工作，或修改 Harness 的授权、守门和审计。
5. 开启非测试真实项目的自动连环、多项目接力、push、发布或部署。

明确授权已经覆盖的具体动作直接执行；未覆盖的硬边界才停。普通本地开发按当前 leaf 的精确范围继续。

## 验证和事实

- 做状态判断前先核实 Git、代码和直接验证，不照搬旧计划、handoff 或历史任务包的完成标记。
- 完成时必须说明怎么验、实际证据和证明边界；局部测试不等于真实 App、生产或发布通过。
- 含 Rust production 路径的改动不能只跑 `cargo test`，必须同时跑 `cargo check --lib` 或等价 non-test build。
- Stop 不跑项目测试；task 只选与本次改动有关的小检查；full 只在显式调用时运行。
- 范围扩大、事实漂移、共享所有权冲突或需要创造未授权产品行为时停止。

## 保护现有工作

- 任何已有 dirty、untracked 或 ignored 内容都视为用户工作；不 reset、clean、stash、覆盖、整批归因或整批暂存。
- 不使用 `git add -A`。只列出本次精确文件。
- `git add` / `git commit` 需要当前用户指令或当前 Harness 授权明确覆盖；执行子 agent 不提交。
- push、共享分支合并、发布、部署和删除分支/worktree 仍需单独授权。

## 项目 Git 约定

- `.githooks/commit-msg` 保留项目既有 `catch:` 标记；`docs/harness-catch-log.md` 保留为历史与项目拦截账本。这是项目 Git 约定，不是另一套生命周期。
- `.githooks/pre-commit` 只做 staged diff 的机械检查，不解释任务、授权或产品事实。
- 完成 leaf 使用 `hl done`，完成整个 stage 使用 `hl close-stage`；不再回写旧 `CURRENT.md` / `AUTHORITY.md`。

## 协作方式

- 两条线：主导线负责统筹、对接用户、核实实物；执行线负责范围内实现。执行线自报完成不等于主导线接受。
- 重要任务只认 Lite 当前 leaf。项目任务补充材料可以使用 `TASK_TEMPLATE.md`，但模板和字段齐全都不激活工作。
- 用户要求多 agent 时，同一 stage 授权可供主 agent 与子 agent 使用；子 agent 仍受相同文件范围和硬边界约束。

## 产品边界

Syn 是服务当前用户的全能个人 AI 工作台。项目是复杂工作的主要事实、权限与执行边界；秘书和全局主管是顶层入口，项目主管在项目内部。对话、知识、记忆、任务、工作流、Agent、连接器、工具、审计和日报是协作能力。工作流能力保留，但工作流界面不是产品中心；也不要把产品退回任务包管理器。

产品 truth、SQLite/store、provider、Codex 会话、NPC/Agent、资产和 runtime 都留在产品代码与产品资料中，不搬进 Harness Lite。

## 完成和退场

每个 leaf 先交代：做出什么、验证跑了什么、改了哪些文件、遗留什么。最后一个 leaf 完成后关闭 stage，并交代入口、实际结果、遗留和接手人、改动位置、是否并主线、分支/worktree、测试材料、下一入口，以及记录是否真的在提交里。
