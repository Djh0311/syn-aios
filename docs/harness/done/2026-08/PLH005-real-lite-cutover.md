# PLH005 真实卸载旧 Harness并完成 Lite 适配

阶段：stage-01 代码事实收敛、唯一权威与 Lite 切换

目标：把 PLH004 已通过的逐文件切换应用到迁移 worktree，只修改 Harness 与权威入口，不开发产品功能。

干完的标准：旧 Adaptive Harness 活动 runtime/配置/接线退出；Lite ownership 有效；AGENTS/CLAUDE/README 与 docs/harness 是同一权威；项目代码与保留 worktree 不变。

允许动：

- `.harness/`
- `scripts/harness-v2/`
- `harness.config.json`
- `.claude/harness-lite/`
- `.claude/settings.json`
- `.codex/harness-lite/`
- `.codex/hooks.json`
- `.gitignore`
- `.githooks/pre-commit`
- `docs/harness/`
- `AGENTS.md`
- `CLAUDE.md`
- `README.md`
- `TASK_TEMPLATE.md`
- `tasks/README.md`
- `docs/project-context.json`
- `docs/plans/README.md`
- `plans/v0.5.0/README.md`
- `plans/v0.5.0/SYN-FND-001.md`
- `docs/code-map/README.md`
- `docs/code-map/domains/development-harness.json`
- `docs/current-state.md`
- `docs/decisions.md`
- `docs/task-queue.md`
- `docs/open-questions.md`
- `docs/sprint-contract.md`
- `docs/harness-catalog.md`
- `docs/task-packages/TEMPLATE.md`
- `templates/hooks/pre-commit`
- `templates/hooks/pre-push`
- `templates/ci/github-actions/harness.yml`
- `templates/ci/gitlab/harness.yml`

必须保留：

- `.githooks/commit-msg`
- `docs/harness-catch-log.md`
- `docs/code-map/`（只允许适配上面精确列出的两个文件）
- 产品源码、合同、决定、证据、任务历史和用户 WIP

## 步骤

1. 复核 PLH003/PLH004 的 HEAD、dirty、manifest/hash 和逐文件计划没有漂移。
2. 先写 Lite 项目事实，适配真实 `.githooks/pre-commit`，再退出旧 runtime 和活动引用。
3. 核对 ownership；人工合并宿主 hook，不覆盖项目入口。
4. 运行 chain/progress/auth、quick/task、旧活动引用扫描和重复安装检查。
5. 本 leaf 不 push；是否 commit 单独交代。

精确删除集：旧 manifest 的 50 个 `created + replace-managed` 文件、未入 manifest 的 `scripts/harness-v2/active-path-audit.test.js`，最后删除 `.harness/manifest.json`，共 52 项。
