# S1B-H2-R4A DB/JSON 全语义预检诊断 v1

日期：2026-07-22（+0800）
任务包：`tasks/2026-07-22-s1b-h2-r4a-db-json-reconciliation-preflight-diagnosis-package-v1.md`
结论：**D / `NEEDS_SAFE_OFFLINE_RECONCILE_PROBE`**；已完成本轮只读诊断，未修复、未重 seed、未启动 App。

## Gate 0：新鲜现场卫生与冻结

- 用户已正常关闭 R4-R2 的 PID `26611`；本轮独立复核该 PID 不存在，Workbench/dev/Codex/MCP scoped process、递归 store holder、明确 JSON/SQLite/WAL/SHM holder 与根 lock 均为 `0`，registry 为 `revision=1132`、`entries=0`。
- HEAD 为 `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`，staged 为 `0`；35 个既有脏项仅冻结，dirty manifest SHA-256 为 `ddf9c53b…e3d76583`。R4 八个源码 SHA-256 均仍匹配任务包冻结值，未发现可归属外的相关源码漂移。
- 普通 read-only SQLite `integrity_check=ok`；storage mode 仍是 `db_primary_json_projection`。本轮开始与结束两次比较的 workflow、proposal、supervisor、registry、storage config、DB/WAL/SHM SHA-256 均相同；固定测试项目 full-file manifest 亦相同。
- 这是数据诊断所需的进程卫生绿；它与 R4-R2 已处理的正常 Quit 后进程残留是两件事，不将其混作数据根因。

## 脱敏 mismatch 矩阵

| 层面 | 只读现场摘要 | 源码语义 | 可证结论 |
| --- | --- | --- | --- |
| `project_proposals` 自然键 | JSON=`74`、DB=`74`、shared=`74`；双方排序集合 SHA-256 均为 `7dfb6f52…e70e0b6`；key-set 差集的 DB/JSON leading 均为 `0` | `proposal_id` 是 JSON typed record 与 DB row 的自然键，随后进入 `reconcile_table` | 不是少行、错键或单边 natural-key leading。count-level 相等在本例还被提升为 key-set 相等，但仍不等于 record hash 相等。 |
| DB row hash | `74/74` 为非空、64 位小写十六进制；hash multiset 摘要=`e7db7416…106eda29` | DB 读取会先验证 stored hash 可为 repository 或 canonical 形态，随后重新算 canonical hash | hash 形状绿不证明它等于 typed JSON 的 canonical hash。 |
| typed JSON normalisation | 6 个 serde-default 字段在 JSON 与 DB raw record 中均 `74/74` 存在；`tasks` 两侧均为 71 absent、3 nonempty、0 empty，3 个 present-key 的集合摘要均为 `69b03e06…d74731e4` | proposal JSON 经 typed `ProjectConsultationProposal` 再 `serde_json::to_value`；`tasks` 为空时 skip，其他兼容字段有 default | 已排除本轮可安全统计到的缺字段、空 `tasks` 或 `tasks` 归属差异；不能据此推断所有嵌套值／数组顺序相等。 |
| import metadata | `project_proposal` source=`1/1 accepted`，schema=`project_consultation_proposal_store.v1`，revision=`131`；latest source hash 与当前 proposal 文件 hash 相等 | exporter/importer 用该 source metadata 记录 sidecar schema/revision | 不是“当前完整 sidecar 从未被接受导入”的简单陈账；metadata 仍不能证明每个 DB `record_json` 与当前 typed value 相同。 |
| full reconciliation | R4-R2 在启动期已报告稳定家族 `db_json_reconciliation_not_green`，范围为 project-proposals；本轮未重启 App | private `reconcile_db_vs_json` 把 DB generic `record_json` 与 typed JSON `Value` 的 canonical hashes 比较；同键 hash 不同还会经 freshness 排序判 DB/JSON leading 或 hash mismatch | 共享键的内容／canonical-hash／freshness 分支仍是最早未被本轮安全 read surface 展开的边界。 |

## 为什么不能用现有 production 入口完成最后一步

- `reconcile_db_vs_json` 是 crate-private；其 DB load 本身为 read-only，但唯一 production startup 路径是 `initialize_for_startup`。
- startup 路径在 DB-leading 情况会 replay JSON projection，并会追加 startup audit；它不是本包允许的纯读入口。
- canonical hash 递归排序 object keys、保留 array order，并以 Rust `serde_json` 序列化字符串后 SHA-256。`jq -S` 或其他语言的普通 JSON 序列化不能被声称为等价实现。

因此，本轮不以临时脚本、App 启动、build 或非等价 hash 把“可能”伪装为实际 mismatch 列表。

## 裁决与停点

**裁决为 D / `NEEDS_SAFE_OFFLINE_RECONCILE_PROBE`。**

最早可证实的边界是：`project_proposals` 的 74 个 shared `proposal_id` 在 DB generic `record_json` 与 typed-then-reserialized JSON 值进入 canonical hash / freshness 比较之前。自然键、字段存在形态、导入元数据和 stored-hash 格式均不足以说明哪一个 shared key、哪一段嵌套值或哪一种 freshness 分支触发了 R4-R2 的 fail-closed 结果。

没有证据把该状态归因为 H2、MCP、主管 transport、单一代码写路或需要立即重 seed。下一步只能在新精确授权下，对最小离线副本直接调用既有 pure reconciliation 函数并仅回传脱敏 counts/digests；见 `tasks/2026-07-22-s1b-h2-r4b-safe-offline-reconcile-probe-package-v1.md`。

## 本轮未执行

- 未启动 App、未 build、未发送 H2 两句、未创建 client/message identity；
- 未写真实 JSON/DB/WAL/SHM、未重 seed、迁移、恢复或修改 storage mode；
- 未修改源码、H2 tool approval、approval/sandbox/read-only、watchdog、invalid-resume、进程组清理或 M5 逻辑；
- 未输出 proposal/user 内容、完整 ID、`record_json`、原始错误、auth/token、私有家内容或绝对私有路径；
- 未修改固定测试项目，未 stage、commit、push；无新的 harness interceptor，故未追加 catch log。
