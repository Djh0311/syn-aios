# Stage K Architecture Calibration Plan v2

日期：2026-06-10

状态：Stage K 架构校准补充计划。本文承接 `2026-06-10-stage-k-architecture-calibration-plan-v1.md`，不改变 Stage K 原目标，仍以“自由操控 Codex + 自动化工作流 + 记忆层记录”为交付方向。v2 的作用是处理 K3-B1 retry 被安全审查再次拒绝后的推进策略：真实 K3-B1 / K3-B2 执行线暂停，非真实执行的架构校准、UI 信息层级、memory consistency、运行队列和验收 gate 可以继续。

本文不是执行任务包，不授权新的真实 `codex exec` / `codex exec resume`，不授权直接读写 `/Users/yoyi/.codex`，不替代 `CURRENT.md`、`STAGE_PLAN.md`、`tasks/README.md` 或 evidence / handoff 的完成事实。

## 1. 当前事实

- K0、K1、K2、K2.5、K3-Level-A、K3-Level-B 字段冻结、K3-B0、K3-B1.0 和 K3-B1.1 已完成。
- K3-B1 已执行过一次并失败分类；K3-B1 retry 申请再次被安全审查拒绝。
- K3-B2 必须继续冻结，不能绕过 K3-B1。
- Stage K 原目标不变：日常可用工作台仍要做到自由操控 Codex、自动化工作流和记忆层记录。
- 当前可推进的工作必须是不触发真实 Codex、不发送 prompt、不读写 `/Users/yoyi/.codex` 的产品架构校准和体验收敛。

## 2. 总体判断

不建议把原 Stage K 全面暂停。真正应该暂停的是：

- K3-B1 真实 retry。
- K3-B2 workspace-write / new-session 真实执行点。
- 任何需要真实 `codex exec` / `codex exec resume` 的 dogfood。

可以继续的是：

- 架构扫描 gate 产品化。
- Product Command / workflow automation / memory capture 的代码层边界收敛。
- UI 信息层级继续去控制中心化。
- K4 memory capture / candidate / task memory packet 体验的非真实执行部分。
- K5 failure / todo / run queue / retry proposal 的非真实执行部分。

这样做的理由是：真实执行被安全审查挡住，不等于工作台架构不能继续变稳。相反，现在更需要把 probe、legacy、fixture、product path 的边界写进可重复检查，避免后续继续打补丁式扩散。

## 3. 暂停与继续矩阵

| 区域 | 当前处理 | 原因 |
| --- | --- | --- |
| K3-B1 real resume retry | 暂停 | 安全审查拒绝非沙箱真实 resume，禁止 workaround。 |
| K3-B2 real workspace-write / new-session | 暂停 | 依赖 K3-B1 成功和复核。 |
| Product Command Phase A / preview / prepare / decision | 继续 | 不触发真实 Codex，可继续稳定产品主路径。 |
| Project workflow run unit read model | 继续 | 可用 fixture / no-op / existing sidecar 证明链路。 |
| Memory capture / observation / candidate UX | 继续 | 不需要真实 Codex；只要不自动写 FormalMemory。 |
| Runtime queue / todo / failure control | 继续 | 可基于现有 attempt / readback / failure 分类。 |
| Architecture gate script | 继续 | 只读源码和文档，不触发执行。 |
| Tauri screenshot dogfood | 暂缓 | 可在 K6 或单独授权后做。 |

## 4. v2 校准目标

### 4.1 架构扫描 gate

新增可重复运行的 Stage K architecture gate，默认只读源码：

- 扫描裸 `Command::new("codex")` 是否只存在于批准 runner。
- 扫描普通前端是否调用 legacy workflow / canvas real-run wrapper。
- 扫描 `prompt_body` 是否只在 Phase B runtime input 和测试附近出现。
- 扫描 readback unknown / failed / timed_out 是否仍保持 `result_count=null` 语义。
- 扫描 candidate / observation / knowledge hit 是否被误写成 FormalMemory。
- 扫描 K2/J2/K3 fixture 常量是否继续泄漏到普通产品入口。

### 4.2 Product Command 主路径校准

产品主路径必须保持：

```text
Workbench UI
-> structured intent
-> Product Command preview / prepare / decision
-> Phase A preflight
-> Phase B real runner only when explicitly authorized
-> runtime log / audit / readback
-> run queue / memory capture
```

不得出现：

- 前端直接拼 CLI。
- 普通 UI 直接调用 legacy dispatch / workflow machine。
- canvas experimental runner 作为真实执行旁路。
- prompt body 持久化到普通 sidecar、runtime log、memory candidate 或 evidence。

### 4.3 K3-B1 阻塞后的 K3 策略

K3 继续保留两个层次：

- Level A：项目工作流自动编排、run unit、worker report、process fact、memory capture 的非真实产品链路，可以继续修补。
- Level B：真实 `codex-local` 执行点，必须等 K3-B1 retry 成功或用户手动执行 exact command 回交后再继续。

K3-B2 不得用 K2/J2 已成功的真实执行点替代。它需要自己的 K3 evidence、runtime log、readback 和文件 hash proof。

### 4.4 Memory consistency 校准

K4 之前需要确保：

- ProductCommand attempt 能追到 continuation attempt、runtime log、readback summary。
- MemoryCaptureEvent 能追到 product_command_id、attempt_id、run_unit_id 或 worker report。
- Observation 不是 FormalMemory。
- Candidate 不是 FormalMemory。
- FormalMemory 采纳必须能回到 candidate / user confirmation / audit。
- 缺链路时 UI 显示为“需要补证 / 待处理”，不能显示为完整成功。

### 4.5 UI 信息层级校准

普通 UI 必须继续按照人的操作方式组织：

- 智能体页：项目、对话、消息、输入、发送预览、确认状态。
- 项目页：目标、工作流、run units、当前状态、结果。
- 运行中工作流：正在做什么、卡在哪里、需要用户做什么。
- 记忆层：候选、正式记忆、来源、确认、影响。
- 设置：开发者详情、legacy、raw refs、diagnostics、adapter/provider 边界。

普通 UI 不应默认展示：

- `Product Command`
- `Phase A / Phase B`
- `runtime_log_ref`
- `audit_refs`
- `sidecar`
- `store_revision`
- `H/J/K/PCR` 阶段术语
- 长篇 adapter / provider / credential 边界文案

## 5. 分线职责

全局主管线：

- 维护 K3-B1 / K3-B2 冻结事实。
- 只在 checkpoint 完成、阻断或阶段边界变化时同步权威入口。
- 派发开发线时按写集拆分，不按过细概念拆分。
- 接收复核线报告后决定是否进入下一 checkpoint。

架构校准线：

- 实现和维护 Stage K architecture gate。
- 分类而不是直接删除 legacy / fixture。
- 不执行真实 Codex，不读写 `.codex`。

Execution 线：

- 继续收敛 Product Command 主路径和 permission envelope。
- 不做 K3-B1 retry，不做 K3-B2，除非出现新的任务级授权和安全审查通过。

Workflow 线：

- 继续做 run unit / worker report / process fact / final review 的非真实链路。
- K3 Level B 真实 worker 执行保持冻结。

Memory 线：

- 继续做 capture event、observation、candidate、task memory packet 和 consistency finding。
- 不自动写 FormalMemory。

UI 线：

- 继续降低内部术语在普通 UI 的可见度。
- 不改变真实执行语义。

复核线：

- 只读审查，不改代码。
- 输出 P0/P1/P2、证据行号和下一步建议。

## 6. 近期执行顺序

1. 写入 v2 架构校准计划。
2. 新增 Stage K architecture gate 脚本，先以 warn 模式运行。
3. 根据 gate 输出修一轮低风险分类问题，只处理不依赖真实 Codex 的项。
4. 等复核线回交，合并 P0/P1 结论。
5. 若没有 P0/P1，继续 K4/K5 的非真实产品化切片。
6. K3-B1 只有在用户手动回交成功结果或安全审查允许后恢复。

## 7. 验收口径

v2 可接受为：

- K3-B1 阻塞后的 Stage K 推进策略明确。
- 真实执行冻结项和可继续项明确。
- 架构扫描 gate 可以重复运行，并能指出 P0/P1/P2 风险。
- 后续开发不会再把 probe、legacy、fixture 或 canvas experiment 当普通产品主路径。

v2 不接受为：

- K3-B1 retry 成功。
- K3-Level-B 完成。
- K3-B2 可以开始。
- K4/K5/K6 完成。
- 任意项目无限制自由控制台。
- 自动 retry / stop / restart 已真实实现。
- planned adapters 或 provider credential 已接入。

## 8. 当前下一步

在 K3-B1 retry 未通过前，下一步不是 K3-B2，而是：

```text
执行 Stage K architecture gate，修补不依赖真实 Codex 的 P0/P1 架构问题，然后进入 K4/K5 非真实产品化切片。
```
