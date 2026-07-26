# S1B-H2-R4B 安全离线 reconcile probe 验证 v1

- 日期：2026-07-22（+0800）
- 任务包：`tasks/2026-07-22-s1b-h2-r4b-safe-offline-reconcile-probe-package-v1.md`
- 结论：**`GREEN_PROJECT_PROPOSALS_OFFLINE_RECONCILE`**；R4B 的离线诊断已完成，未进行任何真实修复或现场验收。

## 结果边界

在全新 Gate 0 后，执行器只把五个指定 JSON sidecar 和 SQLite DB/WAL/SHM 复制到一个新建的仓外 0700 私有目录。临时、ignored 的 `#[cfg(test)]` probe 直接构造离线 `DbPrimaryJsonProjectionConfig` 并调用既有 `reconcile_db_vs_json`；它没有调用 startup、replay、audit append、export、apply 或 Tauri command。

目标表的脱敏结果如下：

| table | DB | JSON | matched | DB-leading | JSON-leading | hash mismatch | stored hash shape |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `project_proposals` | 74 | 74 | 74 | 0 | 0 | 0 | 74/74 为 64 位小写十六进制 |

三种 key group 均为空，故没有输出任何 key 或 key-set digest；probe 只输出了允许的 table/count/direction/hash-shape 字段。可控内存 fixture 先验证同 key、同 hash 的 green（1/1/1），再验证同 key、canonical hash 发散时仅以 `hash_mismatch_count=1` 与 SHA-256 digest/tail 表达，不打印键、记录正文或原始错误。

这证明**本轮冻结的最小离线副本中，`project_proposals` 不存在 shared-key canonical-hash/freshness 分支的 DB-leading、JSON-leading 或 hash mismatch**。它不证明真实 App 已通过，也不把 R4-R2 的启动期 fail-closed 信息重写为单一根因；R4B 没有启动 App，且没有把全表 report 冒充为只含五个 sidecar 的现场 startup 结论。

## Gate 0、复制与不变量

- Gate 0：已确认上轮 PID 不存在，Workbench/dev/Codex/MCP scoped process、store/DB/WAL/SHM holder、lock 全部为 0；registry 为 `revision=1132`、`entries=0`。SQLite ordinary read-only integrity 为 `ok`。
- 仓库：HEAD=`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`，staged=0；38 个既有脏项被冻结。R4 八个冻结源码 hash 均匹配；固定测试项目 full-file manifest 仍为 `679e35df…52675782`。
- 白名单副本：8 个输入的 source/copy aggregate manifest 都是 `e6950e…77de2f8`，复制前后相等；source 与 copy 两侧 SQLite integrity 均为 `ok`。
- 临时 probe 仅出现于 `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_storage_mode_m5c_tests.rs`，运行后已用同一 patch 删除。该文件 SHA-256 已恢复 Gate 0 值 `83779244…466049`，Git status 为 clean，probe marker 为 0。
- 结束复核：R4 八源码 hash、固定测试项目 full-file manifest、真实 8 输入 manifest 与 SQLite integrity 均未变。一次后检脚本使用了错误的前端相对路径而使聚合检查假失败；纠正为 `src/views/projects/jiaoban/useJiaobanConversationState.ts` 后，八个 hash 全部与 Gate 0 一致。未把这项脚本错误归为数据或源码漂移。
- 临时目录只包含本轮白名单副本，已删除；确认其不存在。没有复制 whole store、`.codex`、runner output、auth/token 或私有 home 内容。

## 离线闸

| 检查 | 结果 |
| --- | --- |
| temporary fixture probe（green + hash-divergence red） | 1/0 |
| 离线 copy probe | 1/0 |
| `cargo test --lib m5a_reconciliation_is_read_only -- --nocapture` | 1/0 |
| `cargo test --lib m5f1_ -- --nocapture` | 3/0 |
| `cargo test --lib m5b_ -- --nocapture` | 10/0 |
| `cargo test --lib m5c_ -- --nocapture` | 5/0 |
| `cargo check --lib` | exit 0 |

编译/测试有 warning 输出；通过结论仅依据各命令的 exit status 与断言结果，不把 warning 当作正确性证明。未修改 M5 DB-primary/CAS/fallback、H2 单工具预批准、approval/sandbox/read-only、watchdog、invalid-resume、进程组清理或消息运输路。

## 未执行与下一步

- 未启动真实 App、未 build/冻结真实现场 binary、未发送 H2 首句或第二句、未创建/点击/批准 Pending 卡、未启动 chain 或 worker。
- 未写真实 JSON/DB/WAL/SHM，未重 seed、迁移、恢复、rollback 或修改 storage mode；未修改固定测试项目；未 stage、commit、push、reset、clean 或 stash。
- 未输出 proposal/user 正文、完整 ID、record JSON、原始 stderr、token/auth、私有路径或私有 home 内容。
- 本轮未发现新的 harness interceptor，故未追加 catch log。

R4B 的 green 结果只允许进入一份**新的、用户在场的 R4C 真实 App 包**：重新 Gate 0、重新构建并冻结当前裸 debug binary，再严格按“首句一次、成功才第二句一次、一张 Pending 卡即停”的现场合同验证。该授权不由本轮离线 green 自动获得；见 `tasks/2026-07-22-s1b-h2-r4c-fresh-gate0-real-app-pending-card-verification-package-v1.md`。
