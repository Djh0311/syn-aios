# Handoff: Root Treatment / P2-1 R3 Level B Window Plan Document v1

日期：2026-06-13

状态：已完成；只写窗口计划，不执行 Level B。

## 1. 完成内容

新增窗口计划：

- `docs/plans/2026-06-13-root-treatment-r3-level-b-execution-window-plan-v1.md`

新增 evidence：

- `evidence/2026-06-13-root-treatment-p2-1-r3-level-b-window-plan-document-v1.md`

本计划把 R3 Level B 拆为 B0-B5：

- B0：只读 preflight。
- B1：production DB apply，不 read-cut。
- B2：`workflow_state_summary` limited read-cut。
- B3：observation / export verification。
- B4：stop-write decision，默认不实际 stop-write。
- B5：final matrix。

## 2. 关键边界

本文档明确：

- 第一次真实窗口建议只做 B0。
- B1-B5 都必须另写 execution record。
- actual stop-write、rollback/recovery 写 source JSON / sidecar、产品读写路径切换、多 agent 解锁都必须再次用户拍板。
- 本轮不读取真实 state root，不创建 production DB，不执行 Level B。

## 3. 同步文件

已同步：

- `tasks/2026-06-13-root-treatment-p2-1-r3-level-b-window-plan-document-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`

## 4. 验证

已通过：

- 旧口径扫描无命中。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。

未跑 `cargo test` / `npm`：本轮只改文档和任务包状态，不改产品代码。

## 5. 边界确认

未执行真实 `codex exec` / `codex exec resume`，未发送 prompt，未读写 `/Users/yoyi/.codex`，未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，未启动 Tauri / Browser / Chrome / Vite / screenshot，未读取真实 workbench state root，未创建 production DB，未切 read/write path，未停写 JSON / sidecar。

## 6. 下一步建议

当前仍等待用户最终车道裁决：

1. 确认 R2 后段是否按 `decisions/2026-06-13-root-treatment-r2-late-stage-closure-track-v1.md` 收口。
2. 若确认转 R4 硬目标，优先执行 `tasks/2026-06-13-root-treatment-r4-h1-frontend-types-domain-split-hard-target-v1.md`。
3. 再执行 `tasks/2026-06-13-root-treatment-r4-h2-workbench-snapshot-page-query-first-slice-v1.md`。
4. R3 Level B 仍不得执行；若未来要执行，先从 B0 preflight 单独任务包开始。

## 7. 不接受为

本轮不接受为 R3 Level B 已执行、R3 已完成、production DB 已创建、read-cut / stop-write 已发生、rollback 已验证于真实数据、多 agent 并行真实执行已解锁、真实 Codex 执行或 `.codex` 接触已授权。
