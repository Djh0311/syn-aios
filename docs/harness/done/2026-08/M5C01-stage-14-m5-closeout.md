# M5C01 stage-14 / M5 closeout

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

状态：`DONE` / `M5 SCOPED PRODUCT-CHAIN PASS` / `STAGE_14_CLOSED` / `NO_PRODUCT_SOURCE_CHANGE` / `M6_NOT_ACTIVE`。本叶只执行 M5R09 独立 PASS 后的 stage-14 / M5 生命周期收口；不是第四个加固叶，不重开任何已通过候选，也不激活 M6、stage-15、F2/F3/F5 或壳采纳。

来源收据：`u-974ab17ddf70da3ff6f4`。当前用户明确要求最新 PASS 后先完成 M5R09 生命周期迁移，并以 2026-08-18 18:40 纪律直接进入 stage-14 / M5 closeout；除非“不修则普通产品对真实用户不可用”，其余 verdict 欠账只进入 unfinished 记录，不阻塞 closeout。独立结论：`/home/synadmin/workspace/.syn-gates/verdicts/M5R09-20260818-1836.verdict.md`。

目标：在不改产品源码、不吸收用户自有 OSS 门面或既有未归属 WIP 的前提下，精确绑定 M5 已接受候选和证据，同步权威状态与 M6 输入交接，关闭 stage-14，并在 M6 继续保持未激活时停到独立验收节点。

做完的标准：

1. 据 `M5R09-20260818-1836.verdict.md` 只归档 M5R09：绑定内容候选 `c91d8fc72bcbf80186736caff841cb7a9b0660d1` / tree `fe2d982267d474631ca4ea7b3f90ed846f72a89d` 与记账 `8e6f59f48d2d90891d3c02396378921e4a2f5d6e` / tree `2043660c9547c6c102ae24414674918ca8215eb0`；不把该 PASS 扩大成发布、真实资料、真实 provider、真窗口或跨平台实机证据。
2. 逐条处置 verdict 的 8 项欠账，并服从 18:40 纪律：`c1025ba` 的 7 路径 OSS 门面按用户自有独立载体记账；canonical ProjectId 后续消费与 relation source owner typed 判别记录为 M6 前置 unfinished；`UNENROLLED` 主动提示记录为新壳 F3 unfinished；测试 helper、dead code、warning 分类和历史 worktree 注册项记录为非阻塞工程 unfinished。它们不进入本叶产品修正，也不阻塞 closeout。
3. 新建可携带 closeout 报告与 protected-WIP/载体清单，精确区分：M5 产品候选、M5R09 记账、M5C01 closeout 内容、用户自有 `c1025ba`、活动 Harness runtime、静态未归属 WIP。只有静态表承诺内容 hash；活动 runtime 只记录观察时点与漂移边界。
4. M6 输入交接精确绑定已接受的 ProjectSummary/QueryPort、完整 execution identity envelope、旧执行入口 compatibility/rollback 边界与 M5 候选 SHA；明确 canonical ProjectId 扩面等 unfinished 前置，且 M6/stage-15 仍未激活。
5. `docs/current-state.md`、master、M5 阶段计划、M5/M6 修正计划、计划索引与 Harness plan 据实统一为 `M5 SCOPED PRODUCT-CHAIN PASS / STAGE-14 CLOSED`，同时保留证据上限：Linux WSL 的 detached/local/synthetic/ordinary Tauri 组合证据，不是发布、真实用户资料、真实 provider、真窗口像素或 macOS/BSD 实机。
6. 只读证明各 leaf 按独立放行时的生命周期惯例唯一存在：REC-00、M5R01–M5R09 在 `done/2026-08/`，M5R00 按 `M5R00-20260818-0305.verdict.md` 明确放行的旧惯例唯一留在 `archive/`；不为统一目录而重写已接受生命周期。stage-12 与 D0C04/D0C05 原位不变；OSS-01 保持 unfinished；M6/stage-15/F2/F3/F5 无 current leaf、无激活事实。
7. closeout 文档候选只包含本叶允许路径，`git diff --check`、冻结合同 tree/hash 对比、候选/记账 SHA-tree 对应、authorization 精确 closed 两字段、Git index 精确 staging、既有 dirty WIP hash 保全和 lifecycle 结构检查全部通过；原始日志写入 `.syn-gates/evidence/M5C01-<短SHA>/`。
8. 最终以独立 closeout 内容提交和 lifecycle 记账提交承载；把本 leaf 与 `stage-14.md` 原子归档到 `docs/harness/done/2026-08/`，Harness plan 只把阶段14标为关闭，阶段15仍未激活。随后确认 `.syn-gates/open/` 无未处理请求，写唯一 M5C01 节点请求，authorization 保持精确 closed，并停止；不进入 M6、壳采纳、push、部署或发布。

允许动：

- `docs/harness/authorization.json`
- `docs/harness/plan.md`
- `docs/harness/stages/stage-14.md` 与退场目标 `docs/harness/done/2026-08/stage-14.md`
- `docs/harness/leaves/M5R09-m1-enrollment-and-pre-closeout-hardening.md` 与归档目标 `docs/harness/done/2026-08/M5R09-m1-enrollment-and-pre-closeout-hardening.md`
- `docs/harness/leaves/M5C01-stage-14-m5-closeout.md`、其 unfinished 来源与归档目标 `docs/harness/done/2026-08/M5C01-stage-14-m5-closeout.md`
- `docs/harness/unfinished/`（仅新增 verdict 欠账后续叶；既有 OSS-01、D0C04、D0C05 只读保全）
- `docs/harness/audit/2026-08.jsonl`
- `docs/harness/reports/M5C01-*` [新增]
- `docs/current-state.md`
- `docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`
- `docs/plans/2026-08-01-syn-stage-5-project-supervisor-and-execution-loop-plan-v1.md`
- `docs/plans/2026-08-16-syn-m5-m6-fact-reconciliation-and-product-closure-plan-v1.md`
- `docs/plans/README.md`
- `handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md`
- `/home/synadmin/workspace/.syn-gates/evidence/M5C01-*` 与 `/home/synadmin/workspace/.syn-gates/open/M5C01-*`
- `refs/heads/main`（仅本地精确提交；不得 push）

生命周期只读例外：`docs/harness/archive/M5R00-m1-ordinary-project-identity-prerequisite.md` 只用于证明旧惯例归档唯一性，不移动、不改写。

只读保全：

- 用户自有 OSS 门面提交 `c1025ba81b6c7885a16529b8f66c919655db48e4` 的精确 7 路径：`README.md`、`LICENSE`、`CONTRIBUTING.md`、`SECURITY.md`、`prototypes/productized-desktop-shell/package.json`、`prototypes/productized-desktop-shell/src-tauri/Cargo.toml`、`docs/harness/unfinished/OSS-01-public-push-and-codex-oss-application.md`
- M1–M4 冻结合同正文与旧 hash、M5R00–M5R09 已接受产品候选与证据、所有产品源码
- stage-12、D0C04、D0C05、既有未归属 WIP、活动 Harness runtime

不许动：

- 任何产品源码、测试或构建配置；不得用 closeout 顺手修 verdict 的非阻塞欠账
- 用户自有 OSS 门面 7 路径与 OSS-01 生命周期；不得纳入 M5C01 候选、提升为 current、push 或提交外部申请
- M1–M4 冻结合同正文和旧 hash；M5R00–M5R09 已通过候选、原始 evidence 与 scoped PASS
- `m6_*.rs`、stage-12、D0C04、D0C05、M6/M7–M11、Headless Core、Primary/epoch、F2/F3/F5、`syn-shell`
- 真实资料/项目写、真实模型/provider/message/connector、账号、凭据、外部网络业务写
- push、merge、rebase、deploy、release、reset、stash、clean、`git add -A`
- 伪造 Hook receipt、authorization、stage/leaf、测试、Tauri、窗口或发布证据

## 收口结果

- closeout 内容候选 `de98d69a363ff82281330fb3b82de82c03a9b484` / tree `b90244a8535c829e96341d42fef39602ef499f6d` 精确 5 路径，零产品源码变化；全部结构、写域、冻结物、载体和 WIP 检查最终通过。
- verdict 8 项欠账全部按 18:40 纪律进入用户载体或 3 个 unfinished 文件，没有开启产品返修或第四个加固 leaf。
- M5 产品锚仍为 `c91d8fc`，M5R09 记账仍为 `8e6f59f`；本叶只关闭 stage-14 生命周期和同步事实，不改变证据边界。
- 原始证据：`.syn-gates/evidence/M5C01-de98d69/`；可携带报告：`M5C01-closeout-input-and-debt-routing-v1.md`、`M5C01-protected-wip-and-carrier-attribution-v1.md`、`M5C01-stage-14-closeout-and-evidence-v1.md`。
- authorization 保持精确 closed；M6/stage-15/F2/F3/F5/壳采纳、OSS-01 push/申请、部署和发布均未激活或发生。
