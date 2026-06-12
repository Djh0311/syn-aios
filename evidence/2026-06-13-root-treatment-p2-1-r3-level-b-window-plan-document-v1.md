# Evidence: Root Treatment / P2-1 R3 Level B Window Plan Document v1

日期：2026-06-13

状态：已完成，文档计划已写；未执行 R3 Level B。

## 1. 范围

本轮执行 `tasks/2026-06-13-root-treatment-p2-1-r3-level-b-window-plan-document-v1.md`。

产出：

- `docs/plans/2026-06-13-root-treatment-r3-level-b-execution-window-plan-v1.md`

同步：

- `tasks/2026-06-13-root-treatment-p2-1-r3-level-b-window-plan-document-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`

## 2. 输入依据

已读并引用：

- `handoffs/2026-06-12-root-treatment-execution-strategy-review-claude-to-codex-v1.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a11-production-observation-export-verification-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a12-stop-write-json-decision-and-rollback-drill-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a13-transaction-acceptance-and-cutover-gap-matrix-v1.md`

## 3. 计划内容覆盖

窗口计划包含：

- Level B 前置清单。
- 用户需在场步骤。
- B0-B5 分窗策略和预计时长。
- allowed roots / denied paths。
- production DB path / backup path / report path / rollback manifest path 占位模板。
- before / after source hashes。
- execution record JSON 格式。
- fresh verify 清单。
- 中止条件。
- rollback / recovery 序列。
- 必须再次用户拍板的动作。

关键裁决：

- R3 Level B 不作为单个大窗口执行。
- 第一次真实窗口建议只做 B0 preflight。
- B1 production apply、B2 limited read-cut、B3 observation、B4 stop-write decision、B5 final matrix 必须拆分。
- `workflow_state_summary` 是 B2/B3 的默认低风险 read model scope。
- actual stop-write、rollback/recovery 写 source JSON / sidecar、产品读写路径切换、多 agent 解锁都必须另行拍板。

## 4. 验证

已执行：

```text
rg -n "等待用户与 Codex 回归|Codex 回归后按 R2-T0|代班主管线不预定方向" CURRENT.md README.md AUTHORITY.md STAGE_PLAN.md tasks/README.md docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md
```

结果：无命中。

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：pass，0 errors，0 warnings。

```text
git diff --check
```

结果：通过。

本轮未跑 `cargo test` / `npm`，原因：P2-1 是文档窗口计划任务，不改产品代码、不改 Rust/TS/TSX/CSS、不改测试。

## 5. 边界确认

本轮没有：

- 执行 R3 Level B。
- 读取真实 workbench state root。
- 创建真实 production DB。
- 切 app startup / Tauri command / UI / 产品全局读写路径。
- 停写 JSON / sidecar。
- 执行 rollback 或 recovery。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite / screenshot。
- 解冻 backlog 功能。

## 6. 不接受为

本轮不接受为：

- R3 Level B 已执行。
- R3 已完成。
- production DB 已创建。
- production apply / read-cut / observation / stop-write 已发生。
- rollback 已验证于真实数据。
- 多 agent 并行真实执行已解锁。
- 真实 Codex 执行或 `.codex` 接触已授权。
