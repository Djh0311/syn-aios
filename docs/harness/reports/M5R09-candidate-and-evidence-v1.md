# M5R09 内容候选与证据报告 v1

日期：2026-08-18

状态：`CANDIDATE_READY / AWAITING_INDEPENDENT_ACCEPTANCE / NOT_CLOSEOUT / NOT_M5_COMPLETE`

## Harness

- 唯一 current leaf 仍是 `M5R09-m1-enrollment-and-pre-closeout-hardening`；本报告不归档该 leaf、不关闭 stage-14、不激活 M6、stage-15 或壳采纳。
- M5R08 只按 `M5R08-20260818-1536.verdict.md` 放行范围完成生命周期迁移；其内容候选、记账和 scoped PASS 未被反写。
- 产品源码由 Grok 按 14 个白名单窄任务包串行写入，主管逐包审查并在不合格处另开修复包；主管只写合同增补、任务包、Harness 与交接文档。
- 最终内容候选：`c91d8fc72bcbf80186736caff841cb7a9b0660d1`，tree `fe2d982267d474631ca4ea7b3f90ed846f72a89d`。本报告及后续 leaf/plan/stage/current-state/audit 更新属于记账载体，不改变该内容候选。
- `authorization.json` 在节点为精确 closed 两字段。最终记账 SHA/tree 由包含本报告与状态投影的提交形成，并精确写入仓外节点请求。

## 产品

1. 普通产品现在有显式用户动作触发的 M1 enrollment command，来源只接受已安装 index 中唯一 exact project root；source-first 写入、registry 落盘、重复请求与重启解析均具幂等测试，冲突、未知或重复 root 零写拒绝。缺 source 的首次启动进入可恢复 `UNENROLLED`，M1 业务写继续 fail-closed；不自动导入、不按 path 派生。
2. command 已进入后端 registry、Tauri invoke DTO 与普通前端现有布局；UI 只提供最小未登记状态、root 输入、提交和结果展示，没有重画页面，也没有使用真实用户项目做证据。
3. memory entity/relation 与 mature pattern 的 nested owner/source carriers 在 exact legacy boundary 收敛为 canonical ProjectId；mixed/foreign owner 在提交前拒绝且零部分写。六条治理路径测试改用 canonical authority fixture，并由生产侧 authority/legacy migration 行为反例约束，不保留 path-derived test wrapper。
4. ordinary source 的 no-follow flag 与 symlink errno 使用 target-family cfg：Linux 直接测试通过；macOS/BSD 只形成静态 cfg 边界，不声称实机运行。
5. M5R08 报告已把“每个任务包”过大陈述改为 leaf 级候选流程事实；本叶 14 个 Grok 产品任务包均含精确完整矩阵命令，最终候选实际执行完整 `m5_`。
6. dispatch 重复调用精确断言 `dispatch_not_pending_delivery`；direct durable `persist_and_execute_workcell` 重入精确断言 `duplicate_effect`，fresh runtime 零 adapter event，operation/receipt/effect 数量保持各 1，持久 operation 字段不变。
7. protected WIP 已分活动 runtime 与静态 hash 两表；29 个同比静态路径 hash 不变，`commands.rs` 的候选外旧 WIP 继续以 59+/56- 留在 working tree，6 个 `m6_*.rs` 保持未跟踪。
8. M5→壳交接已明确 F3 尚未接收 driver 继承禁令、F5 尚无真实窗口像素证据，以及 F2 启动后的首项登记责任；F2/F3/F5 均未启动或验收。

## 证据

原始证据根：`/home/synadmin/workspace/.syn-gates/evidence/M5R09-c91d8fc/`

全部命令在候选 `c91d8fc` 的 detached disposable checkout 执行；Cargo 明确 `--offline`，前端复用本机已安装依赖。每项 stdout/stderr `.log` 与退出码 `.exit` 分开保留，`commands.txt`、`exit-summary.txt`、`summary.md`、`sha256sum.txt` 提供索引与完整性校验。

| 命令 | 结果 | exit |
|---|---|---:|
| `cargo check --lib --offline` | 完成；保留既有 warning debt | 0 |
| `cargo test --lib --offline m5r09_ -- --test-threads=1` | 23 passed / 0 failed | 0 |
| `cargo test --lib --offline memory_entity_relation_ -- --test-threads=1` | 14 / 0 | 0 |
| `cargo test --lib --offline mature_pattern_ -- --test-threads=1` | 14 / 0 | 0 |
| `cargo test --lib --offline m1_ordinary_identity_source_ -- --test-threads=1` | 4 / 0 | 0 |
| `cargo test --lib --offline m5_ -- --test-threads=1` | 188 / 0，1893 filtered | 0 |
| `npm run typecheck` | passed | 0 |
| 默认 `npm run build` | passed；保留既有 >500 kB chunk warning | 0 |
| 默认 bundle marker `rg -l` | 无匹配 | 1（预期） |
| `git diff --check 00e766a..c91d8fc` | 无输出 | 0 |
| `git diff --name-status 00e766a..c91d8fc` | 只含本叶允许路径及已授权生命周期迁移 | 0 |

验证后 disposable checkout 只有为离线前端依赖建立的 `node_modules` symlink 与 Tauri 生成的 untracked `linux-schema.json`；二者不是候选内容或 clean-tree 证据。checkout 已移除；`/tmp/syn-m5r09-cargo-target-c91d8fc` 仅是临时编译缓存，环境安全策略拒绝组合清理命令后原位保留，不是产品或证据载体。

## 载体

- 产品/内容候选：`c91d8fc72bcbf80186736caff841cb7a9b0660d1` / tree `fe2d982267d474631ca4ea7b3f90ed846f72a89d`。
- 记账载体：包含本报告、WIP 分层报告、leaf/plan/stage/current-state/audit 投影的最终提交；精确 SHA/tree 写入仓外节点请求。
- 证据载体：`/home/synadmin/workspace/.syn-gates/evidence/M5R09-c91d8fc/`。
- 证据只到 detached checkout 的离线/本地 Rust、TypeScript、production bundle 与静态边界；不是 GUI/Tauri 进程、真实窗口、真实项目/个人资料、真实 provider/账号/凭据、外部业务写、部署、发布或 M5 closeout。
