---
contract_id: syn-dat-001b-live-manifest-preflight-v2
version: 2
status: EVIDENCED_HOLD
evidence_level: LIVE_MANIFEST_READ_ONLY
supersedes: syn-dat-001b-live-manifest-preflight-v1 (physical-root assertions only)
reference_slice_id: workflow-state-sidecar
---

# SYN-DAT-001B：真实 Workbench 只读 Manifest Preflight v2

本合同只校正 v1 中虚构的 `$HOME/.syn/**` 物理路径；不授权真实数据迁移、删除、复制或切换，也不把本次读证据写成 DAT-007 通过。

## 真实受控根

- Root：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench`
- JSON sidecar：`workflow-state/workflow-state.v0.json`
- storage-mode：`runtime-artifacts/storage-mode.v1.json`
- SQLite primary：`production-db/workbench-state.v1.sqlite`
- 明确排除：`/Users/yoyi/.codex`、凭据目录、任何非上述 Workbench-owned 路径。

每个候选须为 root 内普通文件、无符号链接、owner/mode 可记录。storage-mode 的两组 JSON/DB 路径字段必须分别精确指向上述两份数据文件；否则本 preflight fail-closed。

## 允许的只读动作与输出

- `stat` / `lstat`：记录 canonical path、type、owner、mode、mtime、size、symlink 状态。
- SHA-256：读取字节仅用于 hash，不输出或保存原始内容。
- JSON parse：仅统计固定顶层数组、对象/数组数和受限键形态计数；不输出字段值。
- SQLite `-readonly` + `PRAGMA query_only=ON`：只允许 `integrity_check` 与固定表 count。

“程序为了 hash/parse 而读取字节”不等于人工查看或输出值；普通项目证据不得保存 JSON、SQLite、prompt、transcript、凭据或受限字段值。

## 受限对象的当前处置

| 对象 | owner | sensitivity | migration state | retention / quarantine / rollback / export |
| --- | --- | --- | --- | --- |
| restricted key shapes | `UNRESOLVED_DATA_OWNER` | `RESTRICTED_SHAPE_ONLY` | `HOLD_NO_ORDINARY_STORE` | 保留在当前 live source；不复制值；人工分类前不得迁移；rollback 为 no-op；export 未获授权。 |
| `execution_attempts` | `UNRESOLVED_LEGACY_WORKFLOW_RUNTIME_OWNER` | `RESTRICTED_LEGACY_RUNTIME_STATE` | `HOLD_NO_CUTOVER` | 保留当前 source、不得删除；需显式 quarantine/migration decision；rollback/export 均未验证。 |

因此，本合同的正确输出只有 `LIVE_MANIFEST_READ_ONLY / HOLD`，直到上述 owner 与 disposition 获得正式、无值的分类结论。

## 零 mutation 保证

本合同禁止 migration、seed、VACUUM、写 SQLite、写 JSON、创建 lock、复制真实数据、删除真实数据和任何 provider 调用。若未来需要真实数据写/删，必须先由新任务包定义系统临时目录外部备份、精确 write surface 和恢复路径。
