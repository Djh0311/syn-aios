# M5R09 M1 登记生产者与 closeout 前欠账加固

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

状态：`CURRENT` / `CANDIDATE_READY` / `AWAITING_INDEPENDENT_ACCEPTANCE` / `NOT_STAGE_CLOSEOUT` / `NOT_M5_COMPLETE`。本叶承接 M5R08 独立 PASS 结论列出的 8 项后续欠账；M5R08 的 scoped PASS 不反写、不重做。本叶不关闭 stage-14，不激活 M6、stage-15 或壳采纳。

来源收据：`u-7c1590b9c908f00b3110`；用户明确要求最新 PASS 后先完成生命周期迁移，把 verdict 欠账写入新 current leaf 的“做完的标准”，再推进至本叶标准真实通过。独立结论：`/home/synadmin/workspace/.syn-gates/verdicts/M5R08-20260818-1536.verdict.md`。

目标：在不接真实资料、provider、账号、凭据或外部业务写、不触碰 M6 与既有未归属 WIP 字节的前提下，为 M1 canonical ProjectId 建立普通产品内显式、可恢复的登记/迁移生产者，收敛 legacy 行级身份与测试入口，并关闭 M5 closeout 前仍可在本仓真实验证的 runtime、平台与记账欠账。

做完的标准：

1. 普通产品提供明确的 M1 canonical ProjectId 登记/迁移入口：来源显式、由用户明确动作触发、可重放、幂等；首次启动缺少 ordinary identity source 时进入可恢复但业务写 fail-closed 的未登记状态，不静默 fallback、不按 path 派生、不自动导入。登记完成后，同一项目重启解析为同一 canonical `ProjectId`，未知项目仍拒绝。命令必须注册到普通产品真实 command graph，前端有最小现有布局内入口；不用真实用户项目或资料做证据。
2. `memory_entity_relation_governance` 与 `mature_pattern_governance` 的 legacy store 不只迁移顶层 `project_id`：nested entity/relation/source-ref 等 owner 身份必须有明确、可测试的 canonical 迁移或兼容双读边界；foreign/mixed owner 在业务写前 fail-closed 且零部分写。
3. 六条 governance 路径的既有测试迁到 canonical 生产入口/受信 authority fixture，删除仅测试可用的 path-derived wrapper；直接反例必须在生产侧重新引入 path 派生或绕过 M1 authority 时变红，不能只靠易绕过的字符串扫描。
4. ordinary identity source 的 no-follow/open-error 处理使用目标平台正确的常量与 cfg；Linux 行为保持，macOS/BSD 不得套用 Linux `O_NOFOLLOW` / `ELOOP` 数值。能在当前 Linux 环境直接验证的行为跑测试；未有对应平台实机证据时只声明静态/交叉编译边界，不冒充 macOS 运行通过。
5. 修正 M5R08 报告关于完整 `m5_` 矩阵“已写入每个任务包”的过大表述为真实的候选流程规则，并把交节点前至少一次完整 `m5_` 矩阵写入本叶每个 Grok 产品任务包的交付验证段；不得改写 M5R08 已通过事实。
6. 把 duplicate-effect 反例拆清：一条精确验证 dispatch 状态门；另一条直达 durable `persist_and_execute_workcell` 重入并精确断言 `duplicate_effect`，同时机械证明同一 attempt/effect 没有第二 operation、receipt 或业务 effect；不把多个可能错误码的宽松“或”断言当作持久幂等证据。
7. 将 protected-WIP 归责中的活动 Harness runtime 文件与静态 WIP 分成两张表：只有静态表承诺可复核内容 hash；活动 `.observed.json` / `.observed.jsonl` 如实记录观察时点与漂移边界。既有 WIP 继续原位保全，不暂存、不覆盖、不删除，`m6_*.rs` 保持未跟踪候选。
8. M5→壳交接继续明确欠账接收边界：F3 不继承 M5R07 acceptance driver；真窗口像素证据由新壳 F5 获取；并明确“`syn-shell` F2 启动时第一件事登记第 2、3 节”为仍未完成的接收方责任。本叶只保证交接与未完成状态可追踪，不进入 `syn-shell`、不建立 F2/F3/F5 leaf、不声称接收或像素证据已发生。
9. Grok 产品改动按窄任务包串行实施；主管逐包复核。最终候选在 disposable checkout 至少通过 `cargo check --lib --offline`、直接相关 M1/enrollment/memory/mature/runtime 测试、完整 `cargo test --lib --offline m5_ -- --test-threads=1`、前端 typecheck/build、默认 bundle gate、`git diff --check`；原始日志绑定候选 SHA/tree。
10. 全部本叶标准真实通过后，authorization 保持精确 closed 两字段，在 `.syn-gates/open/` 写唯一 M5R09 节点请求并停止；不得自行归档本叶、关闭 stage-14、宣布 M5 完成、进入 M6 或壳采纳。

候选事实（不等于独立验收）：

- 内容候选 `c91d8fc72bcbf80186736caff841cb7a9b0660d1` / tree `fe2d982267d474631ca4ea7b3f90ed846f72a89d` 已覆盖上述 1–8 项，并保持本叶允许写域。
- detached disposable checkout 原始证据位于 `/home/synadmin/workspace/.syn-gates/evidence/M5R09-c91d8fc/`：`cargo check --lib --offline` 0；`m5r09_` 23/23；memory/mature 各 14/14；ordinary source 4/4；完整 `m5_` 188/188；前端 typecheck/default build 0；默认 bundle marker 零命中；candidate-range `git diff --check` 0。
- `M5R09-protected-wip-attribution-v1.md` 与更新后的 M5R08 manifest 分开活动 runtime 和静态 hash；`commands.rs` 候选外 WIP 仍为 59+/56-，6 个 `m6_*.rs` 仍未跟踪。
- 当前只请求本 leaf 的独立验收；本叶仍留在 `leaves/`，不自行归档或前进到下一 leaf。

允许动：

- `docs/contracts/m1-project-enrollment-addendum-v1.md` [新增；只补充登记/迁移生产入口，不改 M1–M4 冻结合同正文或旧 hash]
- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`（仅登记/迁移 authority、缺失源可恢复 fail-closed、平台正确 no-follow 与直接测试）
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（仅 ordinary product M1 未登记状态与登记 authority 的必要 AppState 接线）
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`（仅 M1 登记 command 与本叶直接测试）
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`（仅登记上述 command）
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`（仅登记 command DTO/invoke）
- `prototypes/productized-desktop-shell/src/App.tsx` 与 `prototypes/productized-desktop-shell/src/components/ActiveWorkbenchView.tsx`（仅现有布局内的最小 M1 登记入口与状态展示，不重画布局）
- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs`（仅 nested legacy identity 收敛与 canonical 测试入口）
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs`（仅 nested legacy identity 收敛与 canonical 测试入口）
- `prototypes/productized-desktop-shell/src-tauri/src/lib_memory_lint_mature_pattern_tests.rs`（仅把相关既有测试迁到 canonical authority fixture）
- `prototypes/productized-desktop-shell/src-tauri/src/m5_controlled_execution.rs`（仅精确 duplicate-effect 重入测试；必要时最小修正直接暴露出的同范围缺陷）
- `prototypes/productized-desktop-shell/src-tauri/src/m5_product_commands.rs`（仅 dispatch 状态门精确反例）
- `prototypes/productized-desktop-shell/src-tauri/src/m5_ordinary_control_acceptance.rs`（必要相邻路径；仅把“missing ordinary identity source 必须拒绝构造”的过时反例改为本叶约定的 `UNENROLLED` 可恢复启动与 M1 业务 fail-closed 反例，不改 M5R07/M5R08 已通过的执行链或验收 driver）
- `tasks/2026-08-18-syn-m5r09-*` [新增]
- `docs/harness/reports/M5R08-candidate-and-evidence-v1.md`（仅修正“每个任务包”过大表述）
- `docs/harness/reports/M5R08-protected-wip-attribution-v1.md`（仅拆分活动 runtime 与静态 WIP 的 hash 承诺）
- `docs/harness/reports/M5R09-*` [新增]
- `handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md`（仅补记接收方尚未发生与 F2 首项责任，不激活下游）
- `docs/harness/authorization.json`、`docs/harness/plan.md`、`docs/current-state.md`、`docs/harness/stages/stage-14.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/done/2026-08/`、`docs/harness/audit/2026-08.jsonl`

不许动：

- M5R07/M5R08 已通过的候选、原始证据与 scoped PASS；不得反写、重做或扩大其标准
- M1–M4 冻结合同正文与旧 hash；`worker_report.rs`；页面整体布局；与本叶欠账无关的 execution kernel
- `m6_*.rs`、`stage-12`、D0C04、D0C05、Headless Core、Primary/epoch、M6/M7–M11
- 既有未归属 WIP 的内容；只允许只读 hash/归责记账，不暂存、不覆盖、不删除
- `syn-shell` 仓库、F2/F3/F5 实施、真实窗口像素声称
- 真实资料/项目写、真实模型/provider/message/connector、账号、凭据、外部网络业务写
- push、merge、rebase、deploy、release、reset、stash、clean、`git add -A`
- 伪造 Hook receipt、authorization、stage/leaf、测试、Tauri 或窗口证据
