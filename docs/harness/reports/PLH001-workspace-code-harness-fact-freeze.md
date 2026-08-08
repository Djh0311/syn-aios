# PLH001 workspace、代码与 Harness 事实冻结报告

日期：2026-08-08

## 结论

唯一候选代码基线不应直接取现有 main，也不应把最新 Harness repair 分支整体当作产品完成线。最小、机械风险最低的候选顺序是：

1. 以干净 `main@e5269557` 为锚。
2. 在隔离树合入 `syn-fnd-002-dev@2a7229b` 的完整 24 个已提交提交。
3. 暂不带入其后的 3 个 I5 Harness repair 提交。
4. 两个 dirty worktree 原地保护，待已提交基线通过后分别收口，不直接覆盖或整批合并。

这只是 PLH002 的演练输入，不表示代码已经合并、产品已经验收或可以发布。

## Worktree 与分支事实

| worktree | branch / HEAD | 相对 main | 工作区 |
| --- | --- | --- | --- |
| `product-line-syn-integration-main` | `main@e5269557` | 基准；本地领先 origin/main 33 | clean |
| `product-line-lite-migration` | `codex/product-line-lite-migration@e5269557` | 与 main 同 HEAD | Lite bootstrap 21 个可见 untracked，staged 0 |
| `product-line-syn-fnd-001` | `codex/syn-fnd-001@e5269557` | 与 main 同 HEAD | clean |
| `product-line-syn-m1-baseline` | `81cf1a3` | main-only 2 / branch-only 2；已被 FND 链包含 | clean |
| `product-line-syn-fnd-002` | `2a7229b` | main-only 2 / branch-only 24 | 64 modified、14 untracked、staged 0 |
| `product-line-harness-i5-repair` | `e365717` | main-only 2 / branch-only 27 | clean；比 FND 多 3 个 Harness-only 提交 |
| `product-line` | `2bf9406` | main-only 5 / branch-only 0 | 86 modified、314 untracked、staged 0 |

提交祖先链为 `2bf9406 <= 0b257db <= 81cf1a3 <= 2a7229b <= e365717`。`main` 与 FND 从 `0b257db` 分开；两侧已提交路径交集为 0，所以先演练完整 merge 比选择性 cherry-pick 更可靠。

FND 已提交面共 65 个文件、`+12784/-185`；其中产品 runtime 38 个文件、9 个直接改 runtime 的提交。该链开头同时含两项旧 Harness 适配，PLH004/PLH005 会在代码基线形成后统一退出，不能把它们当产品验收。

repair 相对 FND 只有 3 个提交、3 个文件、`+1270/-0`，均为 I5 旧 Harness 修复材料；不进入产品代码基线。

除 `ui-wip-snapshot-2026-07-01` 外，其余旧 UI 分支都已是 main 祖先或同 HEAD。该 snapshot 只有 1 个独有提交，提交信息明确“未真机、勿当完成”，本阶段保留历史，不并入候选。

## 两个 dirty 现场

`product-line` 的状态指纹为 `9de7a6ac...98b`，tracked diff 指纹为 `a9b688c3...6fad`，untracked 内容清单指纹为 `0214a578...941`。其中有 25 个产品运行时路径，并与 FND 已提交/脏改发生重叠，禁止整批搬运。

`product-line-syn-fnd-002` 的状态指纹为 `60f1395f...f399`，tracked diff 指纹为 `996452ac...b2e`，untracked 内容清单指纹为 `00ea3446...29a`。其 64+14 项主要是 M2a 后续 WIP，含 45 个 Rust、2 个前端、1 个脚本路径；需在已提交基线之后独立验收。

## 旧 Harness 四类清单

审计范围共 106 个旧 Harness/bridge/template 文件：

- delete 51：`scripts/harness-v2/**` 47、`harness.config.json`、`docs/harness/components/high-risk-boundary.md`、`docs/task-packages/TEMPLATE.md`，以及最后才移除的 `.harness/manifest.json`。
- migrate 7：`AGENTS.md`、`CLAUDE.md`、旧 `AUTHORITY.md`、旧 `CURRENT.md`、`docs/code-map/README.md`、`docs/project-context.json`、`.githooks/commit-msg`。
- preserve 9：Code Map index+6 domain JSON，以及两份产品 bridge 代码/测试。
- history 39：旧 Harness history/runbook、20 个 bridge Markdown、17 个未接线 templates。

旧 manifest 共 58 条：53 created / 5 external，51 replace-managed / 7 create-if-missing。5 个内容漂移文件为 `AGENTS.md`、Code Map README、AUTHORITY、CURRENT、template pre-commit；必须迁移或降级，不能自动覆盖。

## 真实接线

- 唯一真实 Git Hook 是 `.githooks/commit-msg`，只要求 commit message 含 `catch:`，不调用旧 Harness runtime。
- 没有实际 CI workflow、pre-commit、pre-push、Claude settings 或 Codex hooks。
- Lite 当前只有 snippets，尚未注册宿主 Hook；不得宣称已经启用。
- 活动旧 v2 手工入口只剩 AGENTS 的 project-context/adapt，以及 Code Map README 的三条命令。
- 旧 schema-1 `scripts/harness/` 已不存在；相关 CI/templates 和 catalog 文字是历史漂移。

## 项目权威冲突

- 旧 CURRENT/AUTHORITY 明写 active-id NONE、没有 matching active package。
- `plans/v0.5.0/SYN-FND-001.md` 却仍写 R1 ACTIVE，不能单独恢复执行。
- `e526955` 已包含 R1 合同基线；旧 CURRENT 所称“未形成可复现 Git 基线”对这些已跟踪材料已经过期。
- 根 README 指向不存在的根级 CURRENT/AUTHORITY；CLAUDE 和 AGENTS 仍指向旧 Harness 路由。
- 旧 `SYN-FND-001` 已是 HISTORY/SUPERSEDED，不能作为 Lite current leaf 或授权来源。

## PLH002 固定输入与停止条件

- 固定输入：`main@e5269557`、`syn-fnd-002-dev@2a7229b`、共同祖先 `0b257db`。
- 只在 `/private/tmp/product-line-harness-lite-code-rehearsal-*` 演练 merge。
- 不带 repair 三提交、dirty WIP 或旧 UI snapshot。
- 先检查 merge/tree/diff，再只跑直接相关的小验证。
- 如果出现与只读盘点不一致的冲突、HEAD 漂移或需要创造新产品行为，立即停止。

## 本 leaf 验证边界

- 三个 Terra Ultra 子审计均只读完成：代码拓扑、旧 Harness inventory、项目权威映射。
- 六个既有 worktree 均未修改；两个 dirty 现场 staged 仍为 0。
- 未运行产品测试、服务、provider、数据库、浏览器、部署、发布或 push。
