# Stage J / J1-B Mario Test Codex Control Real Resume Execution Point v1

日期：2026-06-09

状态：已完成并通过长期只读复核线复核，结论为 `accepted_with_deferred_items`。任务包已通过长期只读复核线审查，结论为“带 P2 通过”；P2 已修补；J1-B 真实执行点已在 2026-06-09 启动并完成一次 read-only `codex_control` -> 统一 Product Command Phase B -> `codex-local resume` marker probe。回收记录见 `evidence/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md` 与 `handoffs/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1-result.md`；主管复核记录见 `evidence/2026-06-09-stage-j-j1-b-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-09-stage-j-j1-b-supervisor-acceptance-review-v1-result.md`。

全局主管任务。本文是 Stage J / J1-B 的真实执行点任务包，用于在 J1-A 已验收的 Codex Control Plane 产品入口之后，对指定 `mario test` 项目和指定 `codex-local` session 做一次最小 read-only 真实 `resume` 探针。本文不授权裸 CLI、不授权 H5 / legacy / direct CLI 冒充产品路径、不授权 planned adapters、不授权读取 secret / full transcript / rollout。

## 0. 先说薄弱点

- J1-A 已完成的是非真实产品入口和 Phase A no-op trace，不等于真实 Codex 已执行。
- 如果 J1-B 继续只做文档或 preview，Stage J 仍无法证明“自由操控 Codex”在产品链路内可用。
- 如果 J1-B 直接执行 `codex exec resume`，会绕过 J1-A、PCR0-PCR10、权限、runtime log、audit、readback 和记忆捕获边界。
- 本轮只做最小 read-only marker 探针，不做项目文件改写，不做自动编排，不做记忆正式化。

## 1. 权威依据

必须服从：

- `tasks/2026-06-09-stage-j-j0-permission-product-scope-and-acceptance-matrix-freeze-v1.md`
- `tasks/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md`
- `evidence/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md`
- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`
- `tasks/2026-06-09-unified-product-command-routing-pcr10-final-review-and-checkpoint-closure-v1.md`
- `docs/plans/2026-06-09-unified-product-command-routing-development-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `CURRENT.md`
- `tasks/README.md`

## 2. 执行点冻结

本执行点只允许以下 fixture：

- 项目：`/Users/yoyi/Documents/mario test`
- adapter：`codex-local`
- operation：`resume`
- target session：`019e798a-6ce5-76c3-b8ee-33bd0fda841f`
- session 口径：历史 E5 / H2 使用过的 `mario test` 总指导 session；J1-B 执行记录必须重新写明，不得只引用历史授权。
- sandbox：`read-only`
- allowed write roots：`["/Users/yoyi/Documents/mario test"]`；仅作为执行边界根说明，不等于项目写授权；`read-only` 下不得写项目文件。
- allowed native Codex write：仅允许 Codex CLI 自身为本次 `resume` 写入 `/Users/yoyi/.codex` 原生运行状态。
- product writes：只允许工作台自有 product command sidecar、session continuation sidecar、runtime log、audit/readback refs、J1-B evidence/handoff。
- denied paths：secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- prompt body：只作为 runtime stdin 发送给 Codex；不得写入 product command sidecar、continuation sidecar、runtime log、audit、memory、evidence 或 handoff。
- prompt summary/ref/hash：必须写入受控 product command 链路。

## 3. Canonical Prompt

prompt summary：

```text
J1-B mario test Codex Control real resume marker probe
```

prompt ref：

```text
workbench-runtime-prompt:j1-b:mario-test:2547d65c4e86
```

prompt sha256：

```text
2547d65c4e86e6357906a7a55b5923f806f719b952658606e2a6ff9d3797755b
```

hash 口径：按下方 canonical prompt source 代码块内文本计算，包含 marker 行后的单个换行，不包含代码块后的额外空行。

canonical prompt source：

```text
你正在通过 product-line Stage J / J1-B Codex Control Plane 真实 resume 探针被工作台调用。
请只回复以下 marker，不修改项目文件，不读取 secret/token/.env/keychain/OAuth/provider credential/full transcript/rollout：
J1_B_MARIO_TEST_CODEX_CONTROL_RESUME_OK_2026_06_09
```

## 4. 必须走的产品链路

J1-B 成功路径必须满足：

1. 使用 J1-A `codex_control` source 构造统一 Product Command。
2. Product Command `command_family` 继续为 `real_execution_product_command`。
3. 经过 preview / prepare / user confirmation / Phase B。
4. 真实执行只能来自 `run_real_execution_product_command_phase_b` 或其受控 wrapper。
5. 不得使用 H5 dispatch、legacy dispatch、direct CLI、测试 helper、MCP canvas run 冒充产品路径。
6. 执行前必须记录 expected store revision / record version；revision conflict 必须阻断。
7. 成功 flags 必须为：
   - `prompt_sent=true`
   - `real_codex_executed=true`
   - `writes_codex_home=true`
   - `writes_project_files=false`
8. readback 必须只读取本次 attempt 的必要摘要或 last message，不读取完整 transcript。
9. readback 成功必须包含 marker：`J1_B_MARIO_TEST_CODEX_CONTROL_RESUME_OK_2026_06_09`。
10. `result_count` 必须按真实读回结果记录；unavailable / failed / timed out 仍为 `null`，不得伪装成 0。

## 5. 执行前检查

执行前必须完成：

- 重新确认 J1-A checkpoint 已完成，且当前入口不是 `implemented_pending_review`。
- 重新确认任务包、evidence、handoff 的 prompt hash 自洽。
- 记录 `mario test` 核心文件 baseline hash，至少覆盖：
  - `/Users/yoyi/Documents/mario test/index.html`
  - `/Users/yoyi/Documents/mario test/styles.css`
  - `/Users/yoyi/Documents/mario test/game.js`
  - `/Users/yoyi/Documents/mario test/README.md`
- 确认本轮不需要读取 full transcript；如果需要读取 full transcript 才能确认 session，必须停止。
- 确认不需要读取 secret/token/`.env`/keychain/OAuth/provider credential/rollout。
- 确认没有 duplicate running attempt；如有 active attempt，必须按 H4 duplicate guard 阻断。
- 确认真实执行权限弹层 / user confirmation 记录为 `confirmed_by: "user"`。

## 6. 失败分类

以下情况必须失败或阻断：

- 未走统一 Product Command。
- 使用 legacy / H5 / direct CLI / test helper 冒充。
- prompt hash 与 canonical source 不一致。
- 缺用户确认或 `confirmed_by != "user"`。
- target session 缺失或不是冻结 session。
- sandbox 不是 `read-only`，除非另开 J1-B write probe 任务包。
- 试图写项目核心文件。
- 试图读取 secret / full transcript / rollout。
- readback unavailable / failed / timed out。
- result marker 不匹配。
- product command sidecar / continuation sidecar / runtime log JSON 损坏。
- store revision / record version 冲突。

## 7. 验收标准

J1-B 可接受为：

- 指定 `mario test` / 指定 session 的一次 J1 Codex Control Plane 真实 `resume` 探针完成。
- 真实 prompt 已通过统一 Product Command Phase B 发送。
- Codex 原生状态发生本次执行所需的最小写入。
- 工作台 product command attempt、continuation attempt、runtime log、audit/readback refs 可追溯。
- readback marker 成功读回。
- 项目核心文件 hash 前后一致。

J1-B 不接受为：

- J1 最终完成。
- `new_session` 真实成功。
- 任意项目自由执行完成。
- 自动化工作流编排 J2 完成。
- 记忆捕获 J3 完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 自动 retry / stop / restart 完成。
- 真实 Tauri 全量验收完成。

## 8. 验证矩阵

执行后必须记录：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib real_execution_command`
- `cargo test --lib session_continuation`
- `cargo test --lib runtime_log`
- `cargo test --lib codex_local_runner`
- `cargo test --lib`
- `cargo fmt -- --check`
- 真实执行 / 敏感路径扫描分类。
- 项目核心文件 hash 前后对比。

如只创建任务包、未执行真实 Codex，则不得把上述执行后验证写成已通过。

## 9. 分线职责

主管线：

- 创建本任务包。
- 交给长期只读复核线审查。
- 若复核通过，再决定是否启动真实执行点。
- 真实执行后回收 evidence / handoff 并同步 checkpoint。

复核线：

- 只读复核本任务包是否足够安全。
- 不执行真实 Codex，不发送 prompt，不读写 `/Users/yoyi/.codex`。
- 重点核对 prompt hash、session、sandbox、allowed/denied paths、J1-A / PCR10 继承边界。

执行线：

- 只有在主管线明确启动 J1-B 执行点后才运行。
- 必须走产品链路，不得直接 CLI。
- 必须记录所有成功 / 失败证据。

## 10. 回交产物

本任务包：

- `tasks/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`

若执行并回收，应新增：

- `evidence/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`
- `handoffs/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1-result.md`

主管复核如单独记录，可新增：

- `evidence/2026-06-09-stage-j-j1-b-supervisor-acceptance-review-v1.md`
- `handoffs/2026-06-09-stage-j-j1-b-supervisor-acceptance-review-v1-result.md`
