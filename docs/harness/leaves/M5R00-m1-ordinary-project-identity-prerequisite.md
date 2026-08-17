# M5R00 M1 普通项目身份前置（按真实缺口重开）

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

目标：给老项目一条可信的"真实身份"路径——在普通产品启动路径上真实取得 M1 正式项目身份，不靠测试 fixture 预登记、不靠 `project_id()` 的路径字符串派生。本叶只补前置，不做 M5R07 收尾，不改 execution kernel。

状态：`CURRENT` / `NOT_ACCEPTED` / `NOT_CLOSEOUT`。M5R07 已挂起在 unfinished，其已有 scoped PASS 全部保留。stage-14 仍开；authorization closed；M6 与壳采纳未激活。

来源收据：用户 2026-08-18 明确"M1 纳入验收前置"，并批准 M5R07 只砍界面类证据的降级标准。交接见 `handoffs/2026-08-18-syn-new-bus-director-m1-prerequisite-and-reduced-m5r07-standard-v1.md`。

为什么重开：REC-00 前置矩阵曾把 M1 项记为 PASS、M5R00 记为 `NOT_NEEDED`。复核事实——非测试代码里登记 M1 精确别名的只有 `m5_ordinary_control_acceptance.rs`（env 门控验收 fixture）与 `#[cfg(test)]` 内调用；普通启动路径没有任何真实项目登记入口；老项目身份仍由 `lib.rs::project_id()` 从项目路径规整而来。按 8-16 事实重整计划，矩阵出现 GAP 必须先走 M5R00。普通 `AppState` 已安装 M1 项目索引权威（`install_ordinary_product`，8-17 落地），因此本叶缺的是登记与迁移入口，不是权威地基。

产品：一份 M1 增补合同；老项目到 M1 正式身份的创建 / 迁移入口，来源显式、可重放、幂等；普通启动路径上真实调用；不可用时 fail-closed。

做完的标准：

1. M1 增补合同新建，不改任何冻结合同正文与旧 hash；
2. 创建 / 迁移路径来源显式，可重放，重复执行幂等；
3. 普通启动路径真实调用该入口；缺失、损坏或不可用时 fail-closed，不静默 fallback、不 path 派生、不自动导入 legacy index；
4. 定向测试覆盖首次登记、重复登记幂等、重启后同一解析、缺失与损坏拒绝；
5. `cargo check --lib --offline` 与相关定向测试在 disposable checkout 上通过；
6. 独立内容提交，写域精确，`git diff --check` 通过；
7. 到此停下写节点请求文件，等独立验收，不自行进入 M5R07。

证据：本叶只在 disposable checkout 上产出定向证据，绑定候选 SHA。不做 GUI、不做窗口截图、不做 computer use。

载体：本叶新增的合同、`m1_project_index.rs`、`lib.rs` 登记调用点、任务包与本叶报告；一次独立内容提交加一次记账提交。

允许动：

- `docs/contracts/`（仅新增 M1 增补合同；不改冻结合同正文）
- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（仅普通启动登记调用点与必要 `AppState` 接线，不泛化为任意 lib 改写）
- `prototypes/productized-desktop-shell/src-tauri/src/m3_project_role_identity_source.rs`（仅因 M1 身份来源变化必须同步时）
- `tasks/2026-08-18-*`
- `docs/harness/stages/stage-14.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/audit/2026-08.jsonl`、`docs/harness/reports/M5R00-*`、`docs/current-state.md`

不许动：

- M5R07 已有 scoped PASS 结论，不得反向写成 FAIL；不得在本叶做 M5R07 收尾
- M1–M4 冻结合同正文与 hash
- `m5_*.rs` execution kernel、`worker_report.rs`、页面布局
- `m6_*.rs`、`stage-12`、`unfinished/D0C04`、`unfinished/D0C05`
- 21 个未归属的 `src-tauri/src/*.rs` 与 `gen/schemas/linux-schema.json`（本轮只读保全）
- 自行关闭 stage-14、宣布 M5 完成、进入 M6 或壳采纳
- 伪造 receipt、authorization、stage/leaf、测试或 App 证据
- push、merge、rebase、部署、发布
