# Handoff: Stage K / K0 Scope, Permission, And Acceptance Matrix Freeze v1

日期：2026-06-10

状态：已完成。

## 1. 本轮完成

新增 Stage K 日常可用产品化计划：

- `docs/plans/2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md`

新增 K0 任务包：

- `tasks/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1.md`

新增 K0 evidence / handoff：

- `evidence/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1.md`
- `handoffs/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1-result.md`

K0 冻结了：

- K1-K6 任务顺序。
- 真实执行授权矩阵。
- 测试项目矩阵。
- `.codex`、prompt、transcript、secret 边界。
- 记忆捕获策略。
- UI 普通层 / 详情层 / 开发者层。
- 多会话协作分线职责。
- checkpoint 文档同步规则。
- 候选测试项目登记字段。
- 真实执行点字段工作表。

## 2. 当前边界

本轮没有改产品代码，没有执行真实 Codex，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有启动 Tauri / Browser / Chrome，没有创建隔离测试项目。

K0 不授权：

- 真实 `codex exec`。
- 真实 `codex exec resume`。
- 通用自由 Codex 控制台。
- 任意目录无限制执行。
- planned adapters 真实接入。
- provider credential / model verification。
- 自动 retry / stop / restart。
- 自动写正式记忆。

## 3. 验证

已做文案 / 边界扫描：

- `Stage K 已完成` 命中只在“不接受为”或验收清单中。
- `通用自由 Codex 控制台已开放` 命中只在“不接受为”中。
- `任意目录无限制执行已开放` 命中只在“不接受为”中。
- “不授权直接执行新的真实 / 不授权直接读写 `.codex`”出现在 Stage K 计划状态说明中，语义为禁止。

未跑代码测试：

- K0 不改产品代码。
- K0 不改 UI。
- K0 不改 Rust / Tauri。

## 4. 复核线

已派既有复核线只读审查 Stage K 计划和 K0：

- Thread：`019eabfc-7e22-70b3-860e-8017c46919f4`
- 要求：只读，不改文件，不启动 GUI，不执行真实 Codex，不读写 `/Users/yoyi/.codex`

回交结论：

- P0：无。
- P1：无。
- P2：K0 任务包应把测试项目、真实执行授权、readback marker、`.codex` 副作用、dirty worktree 和 stop/restart proposal 等字段补成可验收表格。

主管线已补：

- `tasks/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1.md` 的 `5.1 候选测试项目登记表`。
- `tasks/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1.md` 的 `5.2 真实执行点字段工作表`。

## 5. 下一步

K0 已可收口。下一步：

1. 同步 `CURRENT.md`、`tasks/README.md`、`STAGE_PLAN.md`、`README.md`、`docs/plans/README.md` 的 K0 checkpoint 入口。
2. 启动 K1 / K2 并行准备：
   - K1 UI 线：智能体对话页日常可用重构，不授权真实执行。
   - K2 Execution 线：通用 Codex `resume` / `new session` 产品入口任务包，真实执行点必须单独授权。

## 6. 给下一任全局主管

不要把 K0 说成 Stage K 完成。K0 只是阶段范围和验收矩阵冻结。

不要继承 H / J / PCR 的历史真实执行授权。K2 / K3 / K5 的每个真实执行点都必须重新列明 target、allowed roots、`.codex` 最小范围、prompt summary/ref/hash、readback、runtime log、audit 和 rollback。

不要把 UI 小修补拆成大量任务包。Stage K 已明确：入口文档只在 checkpoint 完成、阻断或阶段边界变化时同步。
