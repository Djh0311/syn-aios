# Station 3a Binding-ID Migration Evidence

- The frozen pre-migration workflow state contained 71 binding records but only 61 unique `binding_id` values.
- The full identity tuple `(workflow_id, node_id, work_item_id, native_thread_id)` was unique for all 71 records.
- The rebuilt Syn debug executable ran only its startup migration; it did not launch a supervisor or worker.
- The post-migration state contains 71 records, 71 unique IDs, and 71 `binding:sha256:<64 hex>` values.
- The migration audit reports `migrated_count=71` and points to the write-before backup frozen here as `workflow-state.migration-backup.json`.
- `workflow-state.before.json` and `workflow-state.migration-backup.json` both hash to `d577cc1d4bef4fe3e4b717df4cbc54a2b09902f73f4d1af40c8b5d54d59fde62`.
- `workflow-state.after.json` hashes to `8b33b79f90d0c9b6776fdf0a0a0c4629721687d40559845a998deede16d2dea6`.

An independent post-migration audit then found that the first migration had changed only the binding rows: all 350 dispatch rows that already carried a `binding_id` still referenced legacy IDs, and two early prepared rows had no `binding_id`. The repaired startup migration was therefore run once more, again without launching a supervisor or worker.

- `workflow-state.before-reference-repair.json` freezes that incomplete state: 71 unique SHA-256 bindings, 352 dispatch rows, 350 legacy references, and two missing references.
- `workflow-state.backup-reference-repair.json` is the automatic write-before backup from the successful repair. It is byte-identical to the frozen before state.
- `workflow-state.after-reference-repair.json` contains 71 unique bindings and 352 dispatch rows; every dispatch has a `binding_id`, all 352 references resolve to exactly one binding, and there are zero orphan references.
- The second migration audit reports `migrated_binding_count=0`, `migrated_dispatch_count=352`, and `migrated_count=352`.
- `reference-repair-verification.txt` records the before/after conservation checks. `SHA256SUMS` covers the three frozen reference-repair state files and that verification record.

This migration repairs ledger identity only. It does not alter the historical v7 business result or start Station 3b.
