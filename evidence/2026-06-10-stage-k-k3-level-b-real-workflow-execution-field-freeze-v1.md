# Stage K / K3-Level-B Real Workflow Execution Field Freeze Evidence v1

日期：2026-06-10

状态：正式字段冻结已完成，结论原为 `accepted_with_pre_execution_blocker`；后续已由 K3-B1.0 prompt freeze repair 修补为 `accepted_with_prompt_freeze_repair`。K3-B1 原 hash `e963aa00f7ba0cb94d973996794e98db9d019bf8d9e568c330eb272d7ddd9fbf` 因正文不可复原已 superseded；当前 B1 runtime prompt hash 为 `ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039`。

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。

## 已完成

- 创建 `tasks/2026-06-10-stage-k-k3-level-b-real-workflow-execution-field-freeze-v1.md`。
- 冻结 K3-B1 mario test read-only workflow loop 草案字段。
- 冻结 K3-B2 isolated project workspace-write workflow loop 草案字段。
- 记录 B1 当前核心文件 baseline hash。
- 记录 B2 当前 fixture 文件清单、baseline hash 和 allowed file 不存在事实。
- 向既有长期开发线和验证线派发只读复核任务，要求回交字段建议和 P0/P1 验收矩阵。
- 开发线已回交：字段冻结可推进，但真实执行前必须补 K3-B 专用 bridge / harness；不能直接复用 J2-B bridge 或 K2 探针。
- 验证线已回交：不建议直接执行 B1/B2；必须先冻结字段、P0/P1 gate、hash/readback/runtime/audit/run unit refs 验收矩阵。
- 主管线已将字段冻结升级为正式 freeze，并新增前置任务包 `tasks/2026-06-10-stage-k-k3-b0-real-workflow-execution-bridge-and-harness-v1.md`。

## Baseline

B1 mario test：

```text
f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf  /Users/yoyi/Documents/mario test/index.html
6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f  /Users/yoyi/Documents/mario test/styles.css
814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd  /Users/yoyi/Documents/mario test/game.js
02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5  /Users/yoyi/Documents/mario test/README.md
```

B2 isolated fixture：

```text
cf1289518849fc1a6947c2c034717f5c4e5afaa0726d56b5de9c733bdd1c201c  test-fixtures/stage-k-isolated-project/README.md
603b54aac32b919db4f2b19758c8e0e361c75dc1802cbc9bc33b549dc89d0a07  test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/new-session-write-probe.md
```

B2 allowed file currently absent:

```text
test-fixtures/stage-k-isolated-project/.workbench/stage-k/k3/k3-b2-workspace-write-probe.md
```

## Prompt Hashes

Prompt 正文只用于本地一次性 hash 计算，不写入 sidecar、runtime log、audit 或 memory。

```text
B1 prompt_hash: ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039
B1 superseded_prompt_hash: e963aa00f7ba0cb94d973996794e98db9d019bf8d9e568c330eb272d7ddd9fbf
B2 prompt_hash: 9057c04f1bbd9ef5ff28b55e4f041fdc1a924c8ba5eeae18c564079ea80226c3
```

## 当前不能执行 B1 / B2 的原因

- K3-B 专用 bridge / harness 尚未完成。
- B1/B2 产品路径是否已有 K3 专用 env-gated ignored real execution entry 仍需 K3-B0 确认。
- B2 采用 `new_session`，但新 session / readback / run unit refs 的稳定回链必须由 K3-B0 证明；如不能证明，必须修订 B2 字段表。
- 执行前仍需重新确认 permission envelope、hash、duplicate guard、diagnostics、memory packet 和用户确认。
- 历史 K2 / H5 / J2-B 授权、prompt、marker、execution point id 和完成证据均不能复用为 K3-B 完成证据。

## 下一步

进入 K3-B0 real workflow execution bridge / harness。K3-B0 只做产品路径和 fake/no-op / ignored env-gated harness，不执行真实 Codex。K3-B0 通过后，才可单独进入 K3-B1；B1 通过主管复核后再进入 K3-B2。本 evidence 不接受为任何真实执行完成。
