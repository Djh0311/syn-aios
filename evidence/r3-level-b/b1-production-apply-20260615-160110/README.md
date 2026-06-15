# R3 Level B B1 Production Apply Retry

日期：2026-06-15

状态：`completed`

本窗口是 B1 production apply retry。与上次 abort 的差异只有 expected source root hash 改为 B0 hash 校准后的 canonical 值：

```text
31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801
```

## 创建了什么

本窗口创建了真实 workbench-owned production DB：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/workbench-state.v1.sqlite
```

DB SHA-256：

```text
12d65f21ae383b72afd1b23347548974502ba60ca6a4143ca6b6fc94270f03ba
```

本窗口还创建了 backup / manifest / rollback / report：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/backup/production-apply-backup-manifest.json
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/backup/production-apply-manifest.json
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/backup/production-apply-export-manifest.json
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/rollback/rollback-manifest.json
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/reports/production-apply-report.json
```

这些 JSON 证据的副本已保存到 `artifacts/`。

## Source 未动

source root 仍只有两个文件：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/plan-authorizations.v1.json
```

单文件 hash 前后一致：

```text
workflow-state.v0.json
4bd5434fdca9e82c8fafc42989e1a267ed7d677bfe2972273fb3afaa26829972

plan-authorizations.v1.json
6962e4781f49246525d4cde37d3133924a66faa12b8aab90db106c3c9f401b0e
```

post-apply preflight：

```text
R3_B0_CANONICAL_SOURCE_ROOT_HASH=31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801
```

## Apply 输出

B1 runner 输出：

```text
R3_B1_PRODUCTION_APPLY_REPORT_PATH=/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/reports/production-apply-report.json
R3_B1_PRODUCTION_DB_PATH=/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/workbench-state.v1.sqlite
R3_B1_BACKUP_MANIFEST_PATH=/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/backup/production-apply-backup-manifest.json
R3_B1_ROLLBACK_MANIFEST_PATH=/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b1-production-apply-20260615/rollback/rollback-manifest.json
R3_B1_SOURCE_ROOT_HASH=31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801
R3_B1_EXPORT_HASH=d3fcca756c1d663df2b46495708e18c61a503a34c10bf36b0d46fc1c92a40a3d
test result: ok. 1 passed; 0 failed
```

apply report summary：

```text
status=completed
level=level_b_workbench_owned_state
db_schema_version=workbench_sqlite_schema_v0
projects=5
workflows=5
source_records=627
import_sources=2
export_status=verified
```

## 验证

apply 前：

```text
cargo test --lib sqlite_apply
test result: ok. 6 passed; 0 failed

cargo test --lib sqlite_export
test result: ok. 3 passed; 0 failed

cargo test --lib sqlite_production
test result: ok. 29 passed; 0 failed; 1 ignored

cargo fmt -- --check
empty output

node scripts/harness/workbench-shape-gate.js --mode check
Status: pass
Errors: 0
Warnings: 0

git diff --check
empty output
```

apply 后：

```text
cargo test --lib
test result: ok. 492 passed; 0 failed; 18 ignored

node scripts/harness/workbench-shape-gate.js --mode check
Status: pass
Errors: 0
Warnings: 0

git diff --check
empty output
```

## 本窗口未做

- 未执行 read-cut。
- 未执行 stop-write。
- 未改 product global read path。
- 未改 product global write path。
- 未写 source JSON / sidecar。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未读取或写入 `/Users/yoyi/.codex`。
- 未读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout / prompt body。

## 独立复核

复核线 Beauvoir (`019eca53-354e-7072-89ea-997304f506fb`) 只读核验结论为 `CLEAR_WITH_P2`，无 P0 / P1。

P2：`execution-record.json` 的 `do_not_claim` 数组包含 forbidden claim 短语，可能被机械 grep 误报。该字段按本窗口审计要求保留，它是禁止声称清单，不是实际声称。
