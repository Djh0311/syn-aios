# Gate 3 apply 命令生成记录（2026-07-20 00:39 +0800）

## source-root hash 现场重算（既有 preflight/apply 同口径）

- 方法 = 07-16 第一跳先例：占位 `R3_B1_EXPECTED_SOURCE_ROOT_HASH` 跑同一 ignored 测试，hash 闸在任何写入前拦停并吐真值。
- 探针日志：`gate3-hash-probe.log`（exit=101，`1 failed` 为预期闸火，21.72s）。
- 第一次探针因 `--exact` 裸名未匹配跑 0 测试（无写入），已用全模块路径重跑。
- **现场 source-root hash = `c3038dc407fa9decf1323fed21909b6a72beb50f13aaf3dd30524c31326540f2`**
  （与 07-16 的 `b47603a7…` 不同属正常：JSON 现场已演进；按包要求现场重算，未沿用旧值。）
- 探针后核验：apply-backup / report / rollback-manifest 均未生成，production-db 目录仍空。

## 待用户亲手执行的命令（Gate 4，仅此一次）

cwd：`/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri`

```bash
env R3_B1_APPLY_CONFIRM=CONFIRMED_USER_PRESENT_2026_06_15 \
    R3_B1_EXPECTED_SOURCE_ROOT_HASH=c3038dc407fa9decf1323fed21909b6a72beb50f13aaf3dd30524c31326540f2 \
    R3_B1_SOURCE_STATE_ROOT="/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state" \
    R3_B1_PRODUCTION_DB_PATH="/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/production-db/workbench-state.v1.sqlite" \
    R3_B1_BACKUP_ROOT="/Users/yoyi/workspace/product-line/evidence/raw/2026-07-19-m5-live-reseed-h2/apply-backup" \
    R3_B1_REPORT_PATH="/Users/yoyi/workspace/product-line/evidence/raw/2026-07-19-m5-live-reseed-h2/production-apply-report.json" \
    R3_B1_ROLLBACK_MANIFEST_PATH="/Users/yoyi/workspace/product-line/evidence/raw/2026-07-19-m5-live-reseed-h2/rollback-manifest.json" \
    cargo test --lib workbench_sqlite_production_apply::tests::r3_b1_production_apply_confirmed_paths_requires_env_authorization -- --ignored --exact --nocapture
```

预期：exit=0、`test result: ok. 1 passed`、report `status=completed`。任何非零/非 completed 即停。
