# DAT-001B Workbench live-manifest value-free recheck

recorded-at: `2026-08-05T04:19:41+08:00`
evidence-level: `LIVE_MANIFEST_READ_ONLY`
result: `HOLD`

This is a read-only stability recheck of
`t3-dat-001b-live-manifest-value-free-2026-08-05.json`, not a migration,
cutover, export, backup, or data-disposition action.

## Scope and method

- Canonical root only: `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench`.
- Explicitly excluded: `/Users/yoyi/.codex`, credentials, non-Workbench data,
  and external providers.
- Read operations: `lstat`, SHA-256 over bytes, JSON parse only for aggregate
  shape checks, and `sqlite3 -readonly` with `PRAGMA query_only=ON` followed by
  `integrity_check` and fixed table counts.
- No Workbench file, SQLite row, JSON projection, lock, backup, or provider
  action was written. Reading bytes to hash/parse did occur; no values were
  included in this record.

## Stable source identity

The following stat/hash values equal the earlier value-free artifact exactly:

| Source | mode / uid:gid / mtime / size | SHA-256 |
| --- | --- | --- |
| root directory | `0755 / 501:20 / 1784823276 / 320` | n/a |
| `workflow-state/workflow-state.v0.json` | `0644 / 501:20 / 1785519338 / 7654539` | `4137836236f20fbc8390d7f1b73d2dcb19eb5e592c6de72ab633d4c70f52e89c` |
| `runtime-artifacts/storage-mode.v1.json` | `0644 / 501:20 / 1783961349 / 606` | `b35188a133852dc260f248c4af61e0cf186348698e5fb64742737691ef25c155` |
| `production-db/workbench-state.v1.sqlite` | `0644 / 501:20 / 1785517932 / 32235520` | `1c333ccdd98d852ad858e6fc1fb3f04a87e5625b8c90556a6a4c0238b6f32357` |

All inspected entries were ordinary non-symlink entries under the canonical
root. SQLite returned `integrity_check=ok`; value-free counts remained
`projects=5`, `workflows=8`, `work_items=58`,
`workflow_audit_events=1819`, and `execution_attempts=164`.

## Why the state remains HOLD

The unchanged bytes preserve, rather than resolve, the prior restricted-shape
and `execution_attempts` boundaries: owner, sensitivity classification,
migration disposition, quarantine decision, rollback, and export remain
unapproved. DAT-007 is therefore still `NOT_MIGRATED`; no real Workbench data
has been switched, deleted, or copied.
