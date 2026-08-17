# M5R00 M1 普通项目身份前置候选报告

状态：`CANDIDATE_READY / AWAITING_INDEPENDENT_ACCEPTANCE / NOT_CLOSEOUT`。

内容候选：`99a5afc678949de50abd63876c57732024e53b18`

内容 tree：`08669b04e8a899e42bad53b7b8cecee95554bfaf`

## Harness

- 唯一 current leaf 仍为 M5R00；本报告只把完成标准投影为候选事实，不归档 leaf、不关闭 stage-14、不激活 M6 或壳采纳。
- `docs/harness/authorization.json` 在开发与节点停靠时均保持精确 closed 两字段；没有补造 authorization、Hook receipt、App receipt 或独立验收结论。
- M5R07 继续位于 unfinished；其 implementation、evidence tip、U01a/U01b/U01c/U02 的 scoped PASS 原样保留，仍不是 evidence-binding 或 closeout。本叶没有修改 M5R07 产品或结论。
- 按交接第 7 节补记 2026-08-16 末条之后已发生的事实。交接写 M1I01 与 M3O01–M3O03 系列“共 13 个提交”，当前 Git first-parent 在 `88cb02e…288fd3` 实见 17 个：10 个产品/返修提交和 7 个证据/记账提交；审计据 Git 实见记录该差异，没有删减或补造时间线。

## 产品

- 新增非破坏性的 M1 增补合同，固定 ordinary app-data 下的显式身份来源文件、严格 schema、创建/显式 legacy 迁移模式、稳定 fail-closed 错误和幂等重放语义；M1–M4 冻结合同正文未修改。
- `m1_project_index.rs` 在既有独占 registry lock 内读取并验证显式来源；首次登记铸造 opaque UUID v4，重复来源不改写，同一 app-data 重启后解析同一身份。来源、registry 或 schema 缺失/损坏时拒绝，不静默 fallback、不从 path 派生、不自动导入 legacy index。
- 普通 Tauri `AppState` 构造在产品组合前调用该重放入口；生产 wrapper 使用仓库已有 index/tasks seeds。非测试共享 helper 让同一组合路径可用隔离合成 seeds 验证，测试未物化仓库中带个人路径的 index。
- 定向测试覆盖首次登记、重复幂等、重启稳定，以及来源/registry 缺失和损坏拒绝。两次主管复核发现并返修：真实 Tauri seed 路径错误；测试 fallback 与个人路径 fixture 边界。最终候选不含这两项缺口。

## 证据

全部命令在 detached disposable checkout `99a5afc` 上运行，使用独立 Cargo target；原始日志位于 `/home/synadmin/workspace/.syn-gates/evidence/M5R00-99a5afc/`。

1. 候选元数据、精确写域、冻结合同 hash、`git diff --check HEAD^..HEAD`：exit 0，日志 `00-candidate-metadata.log`。
2. `CARGO_TARGET_DIR=/tmp/syn-m5r00-99a5afc-target.eQhRWB cargo test --lib m1_project_index --offline`：exit 0；25 passed、0 failed、1997 filtered，日志 `01-cargo-test-m1-project-index.log`。
3. `CARGO_TARGET_DIR=/tmp/syn-m5r00-99a5afc-target.eQhRWB cargo check --lib --offline`：exit 0；884 个既有 warning，日志 `02-cargo-check-lib.log`。
4. 验后 SHA/tree、候选文件 hash 与 diff integrity：exit 0，日志 `03-post-verify-integrity.log`。
5. 构建生成的 disposable-only `gen/schemas/linux-schema.json` 不属于候选，已在移除 checkout 前精确删除；checkout 随后 clean 并移除，Cargo target 保留在 `/tmp/syn-m5r00-99a5afc-target.eQhRWB`。日志 `04-teardown-precheck.log`、`05-teardown.log`，均 exit 0。

证据上限：仅离线、合成、disposable 的代码与构造路径证据；没有 GUI、窗口截图、computer use、真实个人资料、真实用户项目、真实 provider/账号/凭据、外部网络业务写、部署或发布。静态校验后读取之间的 symlink/文件替换 TOCTOU 是后续非阻断加固项，不属于本 leaf 完成标准，也未被写成已解决。

## 载体

- 独立内容提交 `99a5afc678949de50abd63876c57732024e53b18`，tree `08669b04e8a899e42bad53b7b8cecee95554bfaf`，精确 6 个允许路径：1 份新合同、`m1_project_index.rs`、`lib.rs` 和 3 份窄任务包。
- 本报告、current-state、stage/leaf 投影与 audit 补记组成单独记账提交；其 SHA/tree 由节点请求在提交后绑定。
- 仓库开始时既有未归属 Rust、usage、M6 与 `linux-schema.json` WIP 继续原样保全，未暂存、未归责、未 reset/stash/clean/覆盖。
- 外部原始日志和保留的 `/tmp` Cargo target 是验证载体，不是产品提交、发布物或真实运行。

仍未完成：独立验收、M5R00 leaf 归档、M5R07 恢复与其降级后组合类验收、M5/stage-14 closeout、M6 与壳采纳。节点请求只申请独立验收 M5R00 内容候选和本报告所列证据边界。
