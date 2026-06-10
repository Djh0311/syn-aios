# Evidence: Stage H / H7 Stage H Final Acceptance And Freeze v1

日期：2026-06-08

## 结论

阶段 H / H7 已完成，最终冻结为：

```text
accepted_with_deferred_items
```

H7 接受为阶段 H 的最终验收和冻结完成：H0-H6 的可接受项、deferred 项、禁止冒领项和 H-to-I handoff 已明确。

H7 不接受为：

- H3-B 真实 new-session 成功。
- H4-Level-B 真实失败 / 超时探针完成。
- H6 真实 Tauri 关键截图清单完整完成。
- 任意项目 / 任意 session / 任意写入范围的自由 Codex 控制台完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 自动重试、自动恢复、stop / kill / restart 产品化完成。
- 最终蓝图完整工作台完成。

## 核对范围

已核对：

- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- H0-H6 任务包、evidence、handoff。
- H6 开发线 / 验证线回交。
- 现有长期验证线 H7 只读复核的中间回交。

## Acceptance Matrix

| 阶段 | 结论 | 核心证据 | H7 判断 |
| --- | --- | --- | --- |
| H0 | 已完成 | `evidence/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md` | accepted |
| H1 | 已完成 | `evidence/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md` | accepted |
| H2 | Phase B `mario test` 真实 resume 探针已完成 | `evidence/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1.md` | accepted with limited fixture proof |
| H3-A / H3.1 | 非执行授权冻结 / no-op 产品路径完成 | `evidence/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1.md`、`evidence/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md` | accepted |
| H3-B | 真实 new-session fixture run 失败分类完成 | `evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md` | deferred retry |
| H4 | Level A 非真实产品化完成 | `evidence/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md` | accepted Level A |
| H5 | product command / bridge checkpoint 完成，B1/B2 有真实 probe | `evidence/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md` | accepted with limited mario test proof |
| H6 | 真实执行 UI 产品化完成，Tauri 截图清单 deferred | `evidence/2026-06-08-stage-h-h6-real-execution-ui-productization-and-tauri-acceptance-v1.md` | accepted with deferred Tauri checklist |

## 三项核心验收回答

### 通用真实 resume 产品化

H7 结论：阶段 H 已完成 `codex-local` 受控真实 resume 的最小产品路径，并有 `mario test` 真实探针证据；但不接受为自由无限制执行。

证据摘要：

- H1 完成 `CodexLocalRunner` / guard / command plan / prompt ref/hash / readback / runtime / audit 契约。
- H2 Phase B 成功执行真实 `codex exec resume`，写入 `/Users/yoyi/.codex`，readback 返回 `H2_PHASE_B_MARIO_TEST_REAL_RESUME_OK_2026_06_08`。
- H4 / H5 对 duplicate、unknown-result、diagnostics blocked、user rejected 等边界有测试覆盖。

### 通用真实 send / 新会话产品化

H7 结论：未完成。H3-B 仅接受为一次真实 `new_session` fixture run 已执行且失败分类完成，不能当作 H3 成功。

证据摘要：

- H3-A / H3.1 已完成授权冻结和 no-op 产品路径。
- H3-B 真实 `codex exec` new-session probe 失败，`readback_failed`，`result_count=null`。
- 产品路径已补 `--skip-git-repo-check`，但未二次真实执行。

### 项目工作流真实派发闭环

H7 结论：接受为 H5 product command / bridge 和 `mario test` B1/B2 受控真实派发探针完成；不接受为任意项目通用派发闭环完成。

证据摘要：

- H5 Level A 完成 preview / readiness / permission envelope / request / guard / readback boundary。
- H5-Level-B1 read-only 真实 probe 成功，readback 返回固定 marker，核心文件 hash 一致。
- H5-Level-B2 workspace-write 真实 probe 成功，只写 `.workbench/h5-b2/real-dispatch-write-probe.md`，核心文件 hash 一致。

## H-to-I Handoff

I0 可以开始，原因：

- H 阶段已经提供 `codex-local` 真实执行链路的安全边界、事实证据和失败分类。
- H 阶段已经证明工作台不能把 Codex 内部线程机制直接当事实模型，需要抽象为工作台自有协议。
- I0 的任务是参考复核和抽象映射，不要求 H3-B retry、H4-Level-B 或 H6 全量截图先完成。

I0 / I1 限制：

- 不执行真实 Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不把 Codex thread / delegation / handoff 硬编码为工作台事实模型。
- 不显示 planned adapters 为可执行。
- 不处理 provider credential / model verification。

## 验证记录

本轮 H7 没有改产品代码，因此未重跑 `npm` / `cargo`。

本轮执行的验证类型：

- H1-H6 evidence / handoff 只读复核。
- 当前入口旧口径扫描。
- H-I 计划 H7 / I0 / I1 范围核对。

沿用证据中的底层验证：

- H6 evidence 记录 `npm run typecheck`、`npm run test:offline-interaction`、`npm run build` 均通过。
- H6 evidence 记录 Rust 定向边界验证通过：`h5_project_dispatch_bridge`、`session_continuation`、`codex_local_runner`、`runtime_log`、`diagnostics`、`workflow_authorization` 和 `rustfmt --check`。
- H5 evidence 记录 `cargo test --lib` 为 `258 passed / 0 failed / 5 ignored`。
- H2 / H3-B / H5-B1 / H5-B2 的真实执行事实来自各自 evidence，不在 H7 重放。

## 入口文档复核

本轮发现并处理的入口要求：

- H6 后续不再拆小 probe。
- H7 是阶段 H 总验收和冻结。
- 入口文档只在 checkpoint 完成、阻断或阶段边界变化时同步。

H7 完成后入口应统一为：

- 阶段 H 已完成，结论为 `accepted_with_deferred_items`。
- 下一步进入 I0。
- I0 是 Codex 多线程协作参考复核和抽象映射，不是产品代码实现，不授权真实 Codex。

## 边界确认

H7 本轮没有：

- 修改产品代码。
- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送 prompt。
- 启动 Tauri / GUI / 截图。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 创建新线程；仅复用长期验证线做只读交叉复核。

## 结论

阶段 H 可以进入 I0。

进入 I0 不表示 H 阶段 deferred 项消失；H3-B retry、H4-Level-B、H6 全量 Tauri 截图、planned adapters 真实接入、provider/model verification、自动重试和最终蓝图能力仍应保留为后续独立任务。
