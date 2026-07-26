# 任务包：S1B-H2-R4B 安全离线 reconcile probe v1

- 日期：2026-07-22
- 状态：**已出包，未执行；须由用户另行精确授权**
- 前置证据：`evidence/2026-07-22-s1b-h2-r4a-db-json-reconciliation-preflight-diagnosis-v1.md`
- 类型：离线副本上的最小只读诊断；不含生产修复、重 seed、App 或 H2 对话
- 唯一 kickoff：`handoffs/2026-07-22-s1b-h2-r4b-safe-offline-reconcile-probe-kickoff-v1.md`

## 0. 唯一目标

在**脱离真实 store 的最小私有副本**上，直接调用既有 `reconcile_db_vs_json`，只回传 `project_proposals` 的表级 counts、方向、脱敏 key-set digest 与匹配数，确定 R4-R2 的 shared-key canonical-hash/freshness 差异；不实施任何修复。

## 1. 为什么必须另授权

R4A 已证明 74 个自然键相同、普通字段 presence 对齐、最新 accepted import hash 与当前 proposal sidecar 相等，但 production 没有纯读 reconciliation command。`initialize_for_startup` 会 replay/append audit，不能调用；非等价 JSON 工具不能代替 Rust canonical hash。

本包将创建一份含私有业务数据的离线副本，且需要 test-only probe 执行 crate-private 函数。这是新的复制／构建／离线运行能力，不由 R4A 的只读授权自动取得。

## 2. 精确授权与 Gate 0

用户授权必须明确包含：

1. App、Workbench/dev/Codex/MCP、registry 和 JSON/DB/WAL/SHM holder 已全空；若任一不空，`BLOCKED_LIVE_HOLDER`，不 kill。
2. 仅复制 reconciliation 必需的五个输入：workflow state、proposal、authorization、supervisor action、supervisor orchestrator sidecar，以及 SQLite DB/WAL/SHM；不复制 `.codex`、runner output、临时家、auth/token 或整个 App support root。
3. 副本只可建在新建、权限 `0700` 的仓外私有临时目录；源与副本均生成 manifest/hash。副本原文不得进仓库、终端回传或 evidence；删除／保留副本也须在 kickoff 中获得明确处置授权。
4. 重新冻结 HEAD、staged、相关 dirty、R4 八源码 hash、真实 source file hashes、SQLite integrity 与固定测试项目 manifest。任何无法归属的相关漂移均为 `BLOCKED_DIRTY_OVERLAP`。

## 3. 最小 probe 合同

- 仅允许一个 `#[cfg(test)]` 的脱敏 probe，直接构造离线 `DbPrimaryJsonProjectionConfig` 并调用 `reconcile_db_vs_json`；严禁调用 `initialize_for_startup`、replay、append audit、export、apply 或任何 Tauri command。
- probe 只能打开副本 DB 为 read-only；前后比较副本及真实源 manifest，均必须 byte/hash 不变。
- 输出仅限：table name、DB/JSON/matched counts、db-leading/json-leading/hash-mismatch counts、每个非空 key group 的 sorted-set SHA-256 与短 tail、以及固定 `record_hash` shape 统计。不得输出 key、proposal/user 文本、`record_json`、raw stderr、路径、auth/token 或 private home。
- 在 test-only probe 之外不改 production source、schema、M5、H2 approval、read-only/sandbox、watchdog、invalid-resume、process cleanup 或消息运输路。若现有 test seam 不能做到此范围，停止并另包；不得为方便新增 production command/sidecar/MCP server。

## 4. 离线验证与止损

1. 先以可控 fixture red/green 证明 probe 对相同 key + hash divergence 的输出只含脱敏字段，且不调用 replay/audit。
2. 在离线 copy 上运行一次；源 store、copy inputs、fixed project 与 source hashes 均须前后不变。
3. 运行定向 reconcile/M5 tests 和 `cargo check --lib`；任一安全面、shape、DB-primary/CAS 或 existing test 退步即停止。
4. probe 结果若显示 DB-leading，只能另出现场恢复包；若 JSON-leading/hash mismatch，只能另出最小代码／数据修复包；若 green，则仍须另包从新 Gate 0 重做 R4。任何情况都不在本包启动 App、发送两句、重发、点卡、批准方案、启动 chain 或派 worker。

## 5. 禁止项

- 不写真实 store，不重 seed、迁移、恢复、rollback 或修改 storage config；不接触固定测试项目。
- 不输出或入仓私有副本、proposal/user 内容、完整 ID、record JSON、原始错误、token/auth/CODEX_HOME 内容。
- 不放宽/改动 H2 单工具预批准、approval/sandbox/read-only/path-lock、watchdog、invalid-resume、进程组清理或 M5 DB-primary/CAS/fallback。
- 不 stage、commit、push、reset、clean、stash；不启动真实 App 或发送消息。
