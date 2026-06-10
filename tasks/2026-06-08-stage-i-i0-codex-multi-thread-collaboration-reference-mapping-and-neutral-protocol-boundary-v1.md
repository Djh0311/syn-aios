# Stage I / I0 Codex Multi-thread Collaboration Reference Mapping And Neutral Protocol Boundary v1

日期：2026-06-08

状态：已完成，结论为 `accepted`。

## 目的

I0 是阶段 I 的参考复核和抽象边界 checkpoint，不是产品代码任务。

本任务从当前 Codex 多线程协作能力中提取可借鉴的架构模式：主管线派发、开发线执行、验证线复核、回交、接受、返工、线程复用和边界约束。

I0 的核心原则：

- 学习模式，不照搬实现。
- 工作台必须拥有自己的中立事实模型。
- Codex thread / delegation / handoff 只能作为 adapter runtime 的一种外部参考，不能硬编码为工作台事实模型。
- 后续阶段 I 要支持多 agent、多模型、多 provider，不以 Codex 为中心。

## 范围依据

- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `tasks/2026-06-08-stage-h-h7-stage-h-final-acceptance-and-freeze-v1.md`
- `evidence/2026-06-08-stage-h-h7-stage-h-final-acceptance-and-freeze-v1.md`
- `handoffs/2026-06-08-stage-h-h7-stage-h-final-acceptance-and-freeze-v1-result.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`

## Reference Mapping

| Codex 协作概念 | 可参考模式 | 工作台中立概念 | I0 判断 |
| --- | --- | --- | --- |
| Codex thread | 一个可持续工作的执行上下文，可被主管线继续派发 | `WorkThread` + `RunPersistenceHandle` | 参考，不照搬 |
| Thread id | 外部 runtime 的定位句柄 | `ExternalRunHandle` / `AdapterPersistenceHandle` | 只能做 opaque handle |
| 主管线 send message | 主管对执行线发起下一步工作 | `DispatchRequest` | 必须经过控制核心 |
| delegation XML / prompt | 派发意图、职责边界、输入材料和禁止项 | `WorkerHandoff` + `PermissionEnvelope` + `TaskMemoryPacketRef` | 参考结构，不绑定 XML |
| 开发线执行 | adapter 承接任务并产出变更或结论 | `RunUnit` | 中立化 |
| 验证线只读复核 | 独立 review / verifier lane | `ReviewGate` / `VerifierRun` | 应采用 |
| 回交 final answer | worker 结构化汇报 | `WorkerHandoff` / `WorkerReportCandidate` | 应采用，但不能直接写事实 |
| 主管复核 | 接受、返工、阻断、deferred freeze | `ReviewDecision` | 应采用 |
| 线程复用 | 降低上下文维护成本，保留角色记忆 | `WorkLaneReusePolicy` | 应采用，但要避免耦合 |
| thread status | idle / active / failed 等运行状态 | `RunLifecycleStatus` / `RunAttention` | 应采用为派生状态 |
| thread transcript | 外部执行上下文正文 | `ReadbackResult` / `RunEvidenceRef` | 默认不读 full transcript |
| handoff 文档 | 工程证据和交接材料 | `RunEvidence` / `AuditEvent` / `HandoffRef` | 保留为引用，不铺进主 UI |

## Adopt / Reject / Defer

### Adopt

- 采用“主管线 -> 开发线 / 验证线 -> 回交 -> 主管复核”的组织模式。
- 采用长期线程 / 长期工作线复用，减少重复上下文和文档维护成本。
- 采用明确的职责边界、输入材料、禁止项和回交格式。
- 采用独立验证线或 review gate，避免开发线自证完成。
- 采用 checkpoint 式入口同步：只在 checkpoint、阻断或阶段边界变化时同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 和阶段计划。
- 采用状态可见化：active / idle / blocked / completed / deferred 都要能被工作台读模型表达。

### Reject

- 不把 Codex thread id 当成工作台项目事实。
- 不把 Codex delegation / handoff XML 当成产品协议格式。
- 不把 Codex 的线程层级直接映射成项目主管 / worker / 验证线关系。
- 不让 Codex 自带协作能力绕过控制核心、权限、任务包、记忆包、runtime log、audit 和用户确认。
- 不开放自由 Codex 控制台来替代项目工作流。
- 不把 readback、worker report、验证线结论或 tool output 直接写成正式事实或正式记忆。

### Defer

- 多 provider / 多模型真实接入延后到 I3 之后的独立 checkpoint。
- planned adapters 真实执行延后；I 阶段内默认只能 planned / unavailable / credential missing / model unverified。
- 自动 spawn / stop / restart / retry / archive worker 延后，必须先有中立权限和审计模型。
- 全量真实 Tauri 协作 UI 验收延后到后续 UI checkpoint。
- GEPA / Paseo / Odysseus 的具体能力融合延后，只保留蓝图参考。

## I 阶段协议对象草案

I0 只冻结草案，不实现代码。

| 对象 | 责任 | 关键边界 |
| --- | --- | --- |
| `WorkerAdapterDescriptor` | 描述 adapter 类型、状态和执行边界 | `codex-local` 只是第一个实现映射 |
| `WorkerCapabilityDescriptor` | 描述 adapter 能做什么、风险等级和不可用原因 | capability 不等于授权可执行 |
| `WorkThread` | 工作台自有的长期工作线 / 执行上下文抽象 | 不等于 Codex thread |
| `RunUnit` | 一次具体派发 / 执行 / 验证 / 回收单元 | 事实源来自工作台，不来自 adapter UI |
| `RunLifecycleStatus` | pending / authorized / running / readback / completed / failed / blocked / deferred | 状态必须可审计 |
| `RunAttention` | 是否卡住、是否等权限、是否等 readback、是否需要用户 | 进入运行中 / 通知 / 待办的派生源 |
| `RunPersistenceHandle` | 外部 runtime 持久化句柄 | opaque，不暴露 secret，不作为业务事实 |
| `DispatchRequest` | 主管发起派发的中立请求 | 必须来自项目任务包和控制核心 |
| `DispatchGuardResult` | 派发前 guard 结果 | blocked 不能调用 adapter |
| `PermissionEnvelope` | 用户确认、写入范围、sandbox、prompt 摘要、风险 | 高风险动作必须用户确认 |
| `TaskMemoryPacketRef` | 派发时引用的冻结任务记忆包 | candidate / observation 不能冒充正式记忆 |
| `WorkerHandoff` | worker 回交摘要、证据引用、结果声明 | 只能成为 observation / candidate / process fact 来源 |
| `ReviewGate` | 主管 / 验证线复核门 | 决定 accepted / needs_changes / blocked / deferred |
| `ReadbackResult` | 从 adapter/runtime 读回的结果分类 | unknown 不能显示为真实 0 条 |
| `RunEvidence` | evidence / handoff / audit / runtime log 引用 | 普通 UI 不铺 raw log 和全文 |

## 后续推进节奏

为了减少碎片文档维护，阶段 I 后续按 checkpoint 合并推进：

- `I1-I2` 合并为一个实现 checkpoint：WorkerAdapter / WorkThread / RunUnit 中立模型 + DispatchRequest / PermissionEnvelope / WorkerHandoff 协议。
- `I3-I4` 视依赖合并推进：capability / provider / credential 风险 envelope + 多 worker 编排和项目工作流集成。
- `I5` 保持独立：Adapter SDK / CLI parity / diagnostics 预留。
- `I6` 保持独立：阶段 I 最终验收和后续 adapter 路线冻结。

入口文档只在上述 checkpoint 完成、阻断或阶段边界变化时同步。

## 边界确认

I0 产品路径没有：

- 修改产品代码。
- 发送真实 prompt。
- 启动 Tauri / GUI / 截图。
- 读取 auth / token / secret / `.env` / keychain / OAuth / provider credential / full transcript / rollout。
- 接入 planned adapters。
- 验证 provider / model / credential。

过程偏差：

- 收尾扫描时有一条 shell 命令误把 Markdown 反引号放进双引号，触发了 shell command substitution，导致 `codex exec` / `codex exec resume` 被空 stdin 调起。
- 命令输出显示 `No prompt provided via stdin`，并且打开 `/Users/yoyi/.codex/state_5.sqlite` 时因 readonly database 失败。
- 该偏差不是产品代码路径，没有发送 prompt，没有读取 full transcript / secret / provider credential，也没有成功写入工作台产品数据；但 I0 不能再严格声称“本轮完全没有触发 Codex 命令 / 完全没有触碰 `.codex`”。

## 验收

I0 接受为：

- Codex 多线程协作参考映射完成。
- adopt / reject / defer 冻结完成。
- 阶段 I 中立协议对象草案完成。
- I1-I2 合并 checkpoint 推进建议完成。

I0 不接受为：

- I1 / I2 产品代码完成。
- WorkerAdapter / RunUnit 类型实现完成。
- 真实多 agent 编排完成。
- planned adapters 真实接入。
- provider / credential / model 验证完成。
- 新的真实 Codex 执行授权。

## 下一步

下一步进入 `I1-I2` 合并 checkpoint：WorkerAdapter / WorkThread / RunUnit 中立模型 + DispatchRequest / PermissionEnvelope / WorkerHandoff 协议。

`I1-I2` 默认不授权真实 Codex 执行、不读写 `/Users/yoyi/.codex`、不接 planned adapters、不读取凭据。
