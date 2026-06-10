# Evidence: Stage I / I0 Codex Multi-thread Collaboration Reference Mapping And Neutral Protocol Boundary v1

日期：2026-06-08

## 结论

I0 已完成，结论为：

```text
accepted
```

I0 接受为 Codex 多线程协作参考复核和工作台中立协作协议边界冻结完成。

I0 不接受为 I1 / I2 产品代码完成，不接受为真实多 agent 编排完成，不接受为 planned adapters 真实接入，不接受为 provider / credential / model 验证完成，也不授权新的真实 Codex 执行。

## 核对范围

已核对：

- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `tasks/2026-06-08-stage-h-h7-stage-h-final-acceptance-and-freeze-v1.md`
- `evidence/2026-06-08-stage-h-h7-stage-h-final-acceptance-and-freeze-v1.md`
- `handoffs/2026-06-08-stage-h-h7-stage-h-final-acceptance-and-freeze-v1-result.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- 长期开发线 I0 只读复核进展。

## Reference Mapping Summary

| Codex 能力 | 工作台采用方式 |
| --- | --- |
| thread / thread id | 映射为 adapter runtime 的 opaque persistence handle，不作为项目事实 |
| send message to thread | 映射为 `DispatchRequest`，必须经过控制核心和权限 |
| delegation prompt | 映射为 `WorkerHandoff` / `PermissionEnvelope` / `TaskMemoryPacketRef`，不绑定 XML |
| thread reuse | 映射为 `WorkLaneReusePolicy`，用于降低上下文维护成本 |
| final answer / handoff | 映射为 `WorkerHandoff` 和 `RunEvidence`，不能直接写正式事实或正式记忆 |
| supervisor review | 映射为 `ReviewGate` / `ReviewDecision` |
| active / idle / failed | 映射为 `RunLifecycleStatus` / `RunAttention` |
| transcript / readback | 映射为 `ReadbackResult` / `RunEvidenceRef`，默认不读 full transcript |

## Adopt / Reject / Defer Summary

Adopt：

- 主管线派发、开发线执行、验证线复核、回交后主管接受 / 返工 / 阻断。
- 长期线程复用，减少重复上下文。
- checkpoint 同步入口文档，避免碎片任务反复维护权威入口。
- 明确职责、边界、输入、禁止项和回交格式。
- 状态和注意力可见化。

Reject：

- 不把 Codex thread / delegation / handoff 硬编码为工作台事实模型。
- 不让 Codex 自带协作能力绕过控制核心、任务包、权限、记忆、runtime log 和 audit。
- 不开放自由 Codex 控制台替代项目工作流。
- 不把 worker report / readback / verifier report 直接写成正式事实或正式记忆。

Defer：

- planned adapters 真实执行。
- provider credential / model verification。
- 自动 spawn / retry / stop / restart。
- 多 provider 真实成本、数据出境和凭据治理。
- GEPA / Paseo / Odysseus 的具体能力融合。

## I 阶段协议草案

I0 冻结以下对象方向，后续由 `I1-I2` 合并 checkpoint 实现：

- `WorkerAdapterDescriptor`
- `WorkerCapabilityDescriptor`
- `WorkThread`
- `RunUnit`
- `RunLifecycleStatus`
- `RunAttention`
- `RunPersistenceHandle`
- `DispatchRequest`
- `DispatchGuardResult`
- `PermissionEnvelope`
- `TaskMemoryPacketRef`
- `WorkerHandoff`
- `ReviewGate`
- `ReadbackResult`
- `RunEvidence`

## 推进节奏调整

按用户要求，后续阶段 I 不再过细拆任务。

当前冻结节奏：

- `I1-I2` 合并为一个实现 checkpoint。
- `I3-I4` 视依赖合并推进。
- `I5` 独立。
- `I6` 独立。

入口文档只在 checkpoint、阻断或阶段边界变化时同步。

## 验证记录

I0 不改产品代码，不跑 `npm` / `cargo`。

本轮验证方式：

- 只读核对 H-I plan 的 I0-I6 范围。
- 只读核对 H7 evidence / handoff 和当前入口。
- 精确扫描确认旧“下一步 H7”口径无命中。
- 正向扫描确认入口文档已指向 I0。
- 复用长期开发线做 I0 只读交叉复核；截至 I0 主管线落文档时，该线程未出现越界信号。
- 长期开发线随后完成 I0 回交，结论与本 evidence 一致：建议 I0 接受为完成，并建议 I1-I2 合并推进；该线程未改代码、未同步入口、未执行真实 Codex、未读写 `/Users/yoyi/.codex`。

## 边界确认

I0 产品路径没有：

- 修改产品代码。
- 发送 prompt。
- 启动 Tauri / GUI / 截图。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 新建额外开发线程。
- 接入 planned adapters。

过程偏差：

- 收尾扫描命令误把 Markdown 反引号放进 shell 双引号，触发了 command substitution，导致 `codex exec` / `codex exec resume` 被空 stdin 调起。
- 输出显示 `No prompt provided via stdin`；同时尝试打开 `/Users/yoyi/.codex/state_5.sqlite`，但因 readonly database 失败。
- 该偏差未发送 prompt，未读取 full transcript / secret / token / provider credential，未成功写入工作台产品数据；但本轮不能再严格声称“完全没有触发 Codex 命令 / 完全没有触碰 `.codex`”。

## 结论

I0 可以回收为完成。

下一步进入 `I1-I2` 合并 checkpoint，但 `I1-I2` 仍默认不授权真实 Codex 执行、不读写 `/Users/yoyi/.codex`、不接 planned adapters、不读取凭据。
