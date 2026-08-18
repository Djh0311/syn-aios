# M5R08 M1 消费面、attempt-scoped runtime 幂等与验收欠账收敛

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

状态：`CURRENT` / `IN_PROGRESS` / `NOT_STAGE_CLOSEOUT` / `NOT_M5_COMPLETE`。本叶只承接 M5R07 独立 PASS 结论明确列出的后续欠账；不反写 M5R07，不关闭 stage-14，不激活 M6 或壳采纳。

来源收据：`u-68fc6ed5b0c72851083c`；用户明确要求在最新 PASS 后先完成生命周期迁移，并把 verdict 欠账写入新 current leaf 后推进至本叶标准真实通过。独立结论：`/home/synadmin/workspace/.syn-gates/verdicts/M5R07-20260818-1344.verdict.md`。

目标：在不接真实资料、provider、账号、凭据或外部业务写、不触碰 M6 与旧 WIP 字节的前提下，统一剩余普通生产入口的 M1 canonical ProjectId，修正同项目不同 attempt 的 runtime 幂等身份，关闭 M5R07 verdict 的本仓工程与记账欠账，并为只能由后续壳阶段完成的事项留下精确交接。

做完的标准：

1. `mature_pattern_governance` 与 `memory_entity_relation_governance` 被六个已注册生产 command 消费的 preview/write 路径全部改为由服务器已安装的 M1 read authority 解析 canonical `ProjectId`；业务写前 fail-closed，无 path-derived fallback、静默默认或自动导入。已有 path-derived stable-id 数据必须有明确、可测试的迁移或兼容双读边界，不制造第二 owner。
2. 真实 runtime 的 `workcell_id`、durable `operation_id` 与 `receipt_id` 至少绑定 Attempt 或 Grant，而不是只绑定 Project。同一项目两个不同合法 attempt 各自留下不同 operation / receipt / effect；重复执行同一 attempt 保持零第二 effect，旧 lineage 不改写。
3. 把“任一 scoped M5 candidate 交节点前至少跑一次完整 `m5_` 矩阵”固化为任务包/候选流程规则，并在本叶候选上实际执行，不再只记口头约定。
4. 普通前端构建不再无条件携带或启动 M5R07 acceptance driver；验收启用必须是显式构建/运行边界，默认产品构建静默关闭且服务端 gate 保留。此项只清理旧壳验收载荷，不启动 lightcode F3/F5 或重画页面。
5. `try_new_with_tauri_app_data_root` 的 tasks seed 从无效 `../../tasks/README.md` 修到仓库根 `../../../tasks/README.md` 这一已发生生产行为变更，补入既有 M5R07 产品路径增补合同与本叶报告，不把它继续留作来源不明改动。
6. 本叶报告与最终节点请求完整列出 M5R07 的实际载体序列：产品 `ab5c46e` → `7cab372`，记账 `0b7b5e1` → `a85278a`，以及本叶新载体；不得再只报最后一个记账提交。
7. 关闭 M1 ordinary identity source 静态校验与读取之间的 TOCTOU：校验与解析必须绑定同一已打开对象/字节快照，并保持缺失、损坏、symlink/替换攻击 fail-closed；补直接竞态/替换反例，不改冻结 M1–M4 合同正文。
8. 将 acceptance driver 的新壳 F3 继承禁令与真桌面窗口像素证据的 F5 责任写入 M5→壳交接；只记账，不在本叶进入 `syn-shell` 或声称已取得像素证据。
9. 对当前 34 项未归属 WIP 逐项形成路径、Git 状态、内容 hash、来源/语义归属与后续 disposition 的只读 manifest。它们继续原位保全，不暂存、不覆盖、不删除；`m6_*.rs` 明确保持未跟踪候选，不能被 M5 候选或任何 clean 吞掉。
10. Grok 产品改动按窄任务包串行实施；主管逐包复核。候选在 disposable checkout 至少通过 `cargo check --lib --offline`、直接相关 M1/identity/memory/runtime 测试、完整 `cargo test --lib --offline m5_ -- --test-threads=1`、前端 typecheck/build、`git diff --check`；原始日志绑定候选 SHA/tree。
11. 全部标准真实通过后，authorization 保持精确 closed 两字段，在 `.syn-gates/open/` 写唯一 M5R08 节点请求并停止；不得自行归档本叶、关闭 stage-14、宣布 M5 完成、进入 M6 或壳采纳。

允许动：

- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs`（仅 M1 canonical ProjectId 消费与兼容读取）
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs`（仅 M1 canonical ProjectId 消费与兼容读取）
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`（仅上述六个 command 的 M1 解析与 fail-closed 接线）
- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`（仅 ordinary source TOCTOU 修复与直接测试）
- `prototypes/productized-desktop-shell/src-tauri/src/m5_product_commands.rs`（仅 attempt/grant-scoped workcell identity）
- `prototypes/productized-desktop-shell/src-tauri/src/m5_controlled_execution.rs`（仅 attempt-scoped operation/duplicate-effect 语义与测试）
- `prototypes/productized-desktop-shell/src-tauri/src/m5_agent_runtime.rs`（仅 attempt-scoped receipt identity 与测试）
- `prototypes/productized-desktop-shell/src-tauri/src/m5_runner_entry_registry.rs`（仅直接静态守卫）
- `prototypes/productized-desktop-shell/src/main.tsx` 与其既有 M5R07 acceptance driver 模块（仅显式 build/runtime gate 与默认 bundle 剥离；不改布局）
- `docs/contracts/m5-r07-product-path-correction-v1.md`（仅补记已经发生且已独立核实的 tasks seed 路径修正）
- `tasks/2026-08-18-syn-m5r08-*` [新增]
- `docs/harness/reports/M5R08-*` [新增]
- `handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md` [新增，只记录依赖与边界，不激活下游]
- `docs/harness/authorization.json`、`docs/harness/plan.md`、`docs/current-state.md`、`docs/harness/stages/stage-14.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/done/2026-08/`、`docs/harness/audit/2026-08.jsonl`

不许动：

- M5R07 已通过的候选、证据与 scoped PASS；不得反写、重做或扩大其标准
- M1–M4 冻结合同正文与旧 hash；`worker_report.rs`；页面布局；与本叶两项修正无关的 execution kernel
- `m6_*.rs`、`stage-12`、D0C04、D0C05、Headless Core、Primary/epoch、M6/M7–M11
- 34 项既有 WIP 的内容；只允许只读 hash/归责记账，不暂存、不覆盖、不删除
- `syn-shell` 仓库、F2/F3/F5 实施、真实窗口像素声称
- 真实资料/项目写、真实模型/provider/message/connector、账号、凭据、外部网络业务写
- push、merge、rebase、deploy、release、reset、stash、clean、`git add -A`
- 伪造 Hook receipt、authorization、stage/leaf、测试、Tauri 或窗口证据
