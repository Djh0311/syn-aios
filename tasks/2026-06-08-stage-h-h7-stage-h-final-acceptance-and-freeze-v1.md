# Stage H / H7 Stage H Final Acceptance And Freeze v1

日期：2026-06-08

状态：已完成，结论为 `accepted_with_deferred_items`。

## 目的

对 H0-H6 做全局主管最终复核，冻结阶段 H 的可接受项、deferred 项、禁止冒领项和进入阶段 I 的前置条件。

H7 是 checkpoint，不是新开发任务包。本任务不新增产品代码，不执行真实 Codex，不启动 Tauri，不读写 `/Users/yoyi/.codex`。

## 范围依据

- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `tasks/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md`
- `tasks/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md`
- `tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- `tasks/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1.md`
- `tasks/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md`
- `tasks/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`
- `tasks/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md`
- `tasks/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md`
- `tasks/2026-06-08-stage-h-h6-real-execution-ui-productization-and-tauri-acceptance-v1.md`

## H Acceptance Matrix

| 项目 | H7 结论 | 证据 |
| --- | --- | --- |
| H0 安全边界和任务包冻结 | accepted | `evidence/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md` |
| H1 CodexLocalRunner 架构和数据契约 | accepted | `evidence/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md` |
| H2 受控真实 resume 产品路径 | accepted with limited fixture proof | `evidence/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1.md` |
| H3-A / H3.1 新会话授权冻结和 no-op 产品路径 | accepted | `evidence/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1.md`、`evidence/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md` |
| H3-B 真实 new-session fixture run | failed classified / deferred retry | `evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md` |
| H4 readback / failure / timeout / duplicate guard | accepted Level A | `evidence/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md` |
| H5 项目工作流真实派发产品 command / bridge | accepted with limited mario test B1/B2 proof | `evidence/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md` |
| H6 真实执行 UI 产品化 | accepted with deferred Tauri checklist | `evidence/2026-06-08-stage-h-h6-real-execution-ui-productization-and-tauri-acceptance-v1.md` |

## H7 回答的三项核心问题

### 1. 通用真实 resume 产品化完成与否

结论：接受为 `codex-local` 受控真实 resume 最小产品路径完成，并已有 `mario test` 真实探针证明；不接受为任意项目 / 任意 session / 任意写入范围的自由执行开放。

证据：

- H1 已提供 `CodexLocalRunner`、guard、结构化 argv、prompt stdin ref/hash、runtime log / audit / readback 分离边界。
- H2 Phase B 对 `/Users/yoyi/Documents/mario test` session `019e798a-6ce5-76c3-b8ee-33bd0fda841f` 成功执行真实 `codex exec resume`，`prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`，readback 返回 `H2_PHASE_B_MARIO_TEST_REAL_RESUME_OK_2026_06_08`。
- H4 / H5 evidence 覆盖 guard blocked、duplicate blocked、readback unavailable / failed / timed out、user rejected、diagnostics blocked 等非成功路径。

### 2. 通用真实 send / 新会话产品化完成与否

结论：不完成。H3-A / H3.1 已完成非执行授权冻结和 no-op 产品路径；H3-B 已执行一次真实 `codex exec` new-session fixture run，但结果为 `failed_classified`，未成功创建可接受的新会话。

冻结状态：

- 产品路径已补 `--skip-git-repo-check`。
- 任何 H3-B retry 必须再次取得执行点授权。
- 不能把 H2 resume 成功或 H3-B 失败分类解释为通用真实 send / 新会话产品化完成。

### 3. 项目工作流真实派发闭环完成与否

结论：接受为 H5 product command / bridge、preview / readiness / permission envelope、B1 read-only 真实 probe、B2 workspace-write 真实 probe 和 H5 acceptance matrix 收束完成；不接受为任意项目通用真实派发产品化完成。

证据：

- H5 Level A 完成 C4 prepared dispatch、M6 task memory packet、H1/H2/H3 request / guard、H4 readback 边界、G1/G2 runtime / diagnostics、C5/C6 handoff 的非真实预览 / 校验链路。
- H5-Level-B1 对 `/Users/yoyi/Documents/mario test` 既有开发线 worker session 执行 read-only 真实 `resume` probe，readback 返回 `H5_LEVEL_B_MARIO_TEST_CODEX_DEV_REAL_DISPATCH_OK_2026_06_08`，核心项目文件 hash 一致。
- H5-Level-B2 对同一测试项目执行 workspace-write 真实 `resume` probe，只写 `.workbench/h5-b2/real-dispatch-write-probe.md`，readback 返回 `H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08`，核心项目文件 hash 一致。

## 可接受项

- 阶段 H 已完成 `codex-local` 真实自动化工作流的安全边界、runner 契约、受控 resume 路径、项目工作流 dispatch bridge、runtime log / audit / readback / duplicate guard 基础和真实执行状态 UI 产品化。
- H2 / H5 的真实执行都被限制在用户授权的测试项目 / 指定 session / 指定写入范围内，并保留 evidence / handoff。
- `readback unavailable / failed / timed out` 等 unknown-result 不写成真实 0 条结果。
- 进入阶段 I 的前置条件成立：可以基于 H 阶段的 `codex-local` 事实，把协作模型抽象为中立 `WorkerAdapter` / `RunUnit` / `DispatchRequest` / `WorkerHandoff` 等协议。

## Deferred 项

- H3-B retry：真实 new-session 成功仍未完成。后续若继续，必须重新授权 fixture、work item / workflow / node、allowed write roots、`.codex` 最小范围、prompt summary/ref/hash、readback、runtime log、audit、evidence 和 rollback。
- H4-Level-B：真实失败 / 超时探针未完成。后续必须独立授权，不得混入普通 H7 或 I0。
- H6 真实 Tauri 关键截图清单未完成，只接受窗口探针和导航探针；不能声明 H6 Tauri acceptance 完整通过。
- 通用自由 send / resume 控制台不在阶段 H 接受范围内。
- planned adapters、provider credential / model verification、自动重试、stop / kill / restart、自动恢复仍未完成。

## I 阶段前置条件

允许进入 I0 / I1，但必须遵守：

- I0 只做 Codex 多线程协作参考复核和抽象映射，不写产品代码，不执行真实 Codex，不读写 `/Users/yoyi/.codex`。
- I1 可以建立 adapter-neutral 的 `WorkerAdapter` / `RunUnit` 中立模型，但 `codex-local` 只能作为第一个实现映射，不能成为事实模型中心。
- planned adapters 只能声明 planned / unavailable / no credential / model unverified，不得显示为可执行。
- 阶段 I 不继承任何新的真实执行授权；真实执行点仍需单独任务包和执行点确认。

## 边界确认

H7 本轮没有：

- 修改产品代码。
- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送真实 prompt。
- 启动 Tauri / GUI / 截图。
- 读写 `/Users/yoyi/.codex`。
- 读取 auth / token / secret / `.env` / keychain / OAuth / provider credential / full transcript / rollout。
- 同步阶段 I 的产品实现。

## 验收

H7 验收方式：

- 只读核对 H1-H6 任务包、evidence、handoff 和当前入口。
- 扫描旧口径和冒领口径。
- 输出 H acceptance matrix、H evidence / handoff、H-to-I handoff 和入口同步。

H7 不要求重跑 `npm` / `cargo`，因为本任务不改产品代码；底层验证沿用 H1-H6 evidence 中已记录的测试和真实运行证据。

## 下一步

进入 I0：Codex 多线程协作参考复核和抽象映射。

I0 不得直接照搬 Codex 当前多线程能力，也不得硬编码 Codex thread / delegation / handoff 为工作台事实模型。它只能提取架构模式，映射到工作台自有的中立多 agent / 多模型协作协议。
