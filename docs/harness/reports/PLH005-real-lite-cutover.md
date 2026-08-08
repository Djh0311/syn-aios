# PLH005 真实卸载旧 Harness 并完成 Lite 适配

日期：2026-08-08

## 实际结果

在 `codex/product-line-lite-migration@c9d53a9` 真实应用了 PLH004 已验收的切换方案。产品源码没有改动；变化只发生在开发 Harness、项目权威入口、项目 Hook、静态 Code Map 的 Harness domain 和历史状态标记。

旧 Adaptive Harness 已退出活动面：

- manifest 生成的 50 个 `created + replace-managed` 文件：50/50 不存在。
- 漏账的 `scripts/harness-v2/active-path-audit.test.js`：不存在。
- `.harness/manifest.json`：最后移除。
- `harness.config.json` 与 `scripts/harness-v2/**`：没有剩余文件。
- 活动 Hook/宿主 carrier 对旧 runtime 的引用：0。
- AGENTS/CLAUDE/README/project-context/Code Map README 等当前入口对旧路由的引用：0。

总退出集为 52 个旧 carrier。旧 `AUTHORITY.md` / `CURRENT.md` 没有丢弃，已移至 `docs/harness/history/adaptive-v0.5/` 并明确为历史、不可授权。

## Lite 与项目适配

- Lite core 0.3.0，ownership 44 项：40 项和安装 hash 一致，4 项为安装后由项目维护的 `plan.md`、`stage-01.md`、`MISTAKES.md`、`checks.json`，0 missing。
- `.gitignore` 只放开 `.claude/settings.json` 与 `.claude/harness-lite/**`，其他 Claude 本地内容仍忽略。
- `.claude/settings.json` 与 `.codex/hooks.json` 已从各自 snippet 人工合并；所有命令目标文件实际存在。
- `.githooks/pre-commit` 已从旧 v2 gate 改为项目自有 staged whitespace check，并恢复 0755；独立临时 index 运行通过。
- `.githooks/commit-msg` 和 `docs/harness-catch-log.md` 原样保留。
- AGENTS/CLAUDE 保留项目特有协作边界，只把开发生命周期改为 Lite 的 plan → stage → leaf → current authorization。
- `TASK_TEMPLATE.md` 变成产品补充模板；新工作必须先有 Lite leaf。
- 旧 v0.5 包、tasks README 和产品计划 README 都明确降级为历史输入，不再从 ACTIVE/READY/next 恢复工作。

`checks.json` 现在登记：quick 的 Git diff check；Harness、Rust、前端的 path-scoped task checks；显式 full Rust lib tests；真实 App manual 项。Stop 不运行这些项目检查。

## 验证

- `hl chain` / `progress` / `auth`：PASS，当前 PLH005，新 stage-01 授权有效。
- 7 个项目/Hook JSON：解析 PASS；注册 Hook 目标全部存在。
- Lite runtime 全部 JavaScript：`node --check` PASS。
- pre-commit / commit-msg：`sh -n` PASS。
- 重复安装：写 0、跳过/保护 46；前后 status 指纹同为 `9f013f27...0cbf`。
- `hl check quick`：PASS。
- `hl check task AGENTS.md`：PASS。
- `git diff --check`：PASS。
- `prototypes/**` diff：0。
- 真实迁移树 staged：0。
- 两处受保护 WIP status 指纹：`9de7a6ac...98b`、`60f1395f...f399`，与 PLH001 一致。

没有运行 full、real Codex、真实 App、provider、数据库、浏览器、部署、发布或 push。

## Git 与回滚

切换前已冻结 165-entry 文件快照：`/private/tmp/product-line-harness-lite-real-cutover-20260808-before.tar`，SHA-256 `85c491ef...7216`。tracked 旧文件还可直接从 `c9d53a9` 恢复。

本 leaf 没有暂存或提交：当前 tracked 为 54 deleted、13 modified，新增 Lite/历史/报告共 67 个 untracked，全部交给 PLH006 做最终范围审计、恢复探针和精确 staging/commit。不会使用 `git add -A`。
