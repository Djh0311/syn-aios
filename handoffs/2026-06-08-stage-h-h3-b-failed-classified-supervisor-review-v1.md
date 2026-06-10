# Handoff：Stage H / H3-B Failed Classified Supervisor Review v1

日期：2026-06-08

## 结论

全局主管复核接受 H3-B 为：

- 一次授权范围内隔离 fixture 真实 `codex exec` new-session probe 已执行。
- 本次 probe 结果为 `failed_classified`，failure / readback / runtime log / audit / evidence / handoff 可追溯。
- 产品路径已补 `new_session` 的 `--skip-git-repo-check`，可作为下一次授权 retry 的前置修补。

不接受为：

- H3-B 成功。
- 真实 Codex session 已成功创建。
- H3 通用真实 send / 新会话产品化完成。
- H4 / H5 或阶段 H 完成。
- 自动重试、planned adapters、provider credential / model verification 完成。

## 复核依据

- `evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`
- `handoffs/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1-result.md`
- `tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/session-continuations.v1.json`
- `tmp/h3-new-session-fixture/.workbench/h3-b-runs/run-1780858797592229000/runtime-logs.v1.json`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`

## 复核发现

- Sidecar 显示 `attempt.status = failed`、`prompt_sent = true`、`real_codex_executed = true`、`writes_codex_home = true`。
- Readback 显示 `readback_failed`，`result_count = null`，没有把失败读回伪装成 0 条结果。
- Runtime log 只记录脱敏摘要和 refs，没有保存 prompt body、raw transcript 或 secret。
- 失败原因与 evidence 一致：当次 command plan 缺少 `--skip-git-repo-check`。
- 当前源码已经补上 H3-B / `new_session` 的 `--skip-git-repo-check` command plan。

## 本轮主管修补

修补了两个残留旧口径：

- `tasks/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`

同时主管线已补过：

- `CURRENT.md`

修补目的只是同步事实：H3-B 已执行一次但失败分类，后续 retry 仍需重新授权；没有改变产品代码，没有再次执行真实 Codex。

## 主管线过程偏差

主管线复扫旧口径时有一条 `rg` 搜索命令误把 Markdown 反引号放进 shell 双引号，触发了 shell command substitution。命令输出显示 `codex exec` 被空 stdin 调起，并在尝试初始化 `/Users/yoyi/.codex/state_5.sqlite` 时因 readonly database 失败退出。

本偏差不属于产品代码路径，不是 H3-B retry，没有发送 prompt，没有执行任务，也没有读取 transcript / secret / credential；但它确实触碰了 Codex CLI 初始化路径，因此不能声称主管线本轮“完全没有触碰 Codex 命令”。后续 shell 搜索必须避免在双引号内包含 Markdown 反引号，或改用单引号 / 固定文件读取。

## 下一步边界

- H3-B retry 可以准备，但必须重新取得执行点授权，并验证新 command plan 生效。
- H4 Level A 可以并行推进，因为它不依赖 H3-B 成功，且默认不允许真实 Codex 执行。
- H5-Level-B 真实项目工作流派发不能跳过 H3-B 成功回收和 H4 安全链路。
