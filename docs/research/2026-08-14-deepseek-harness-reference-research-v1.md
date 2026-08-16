# DeepSeek Harness 参考研究 v1

日期：2026-08-14

状态：**历史初步研究来源。** 2026-08-16 的官方一手资料复核、AI OPC 映射和 Syn 原生核心结论见 [`2026-08-16-deepseek-harness-ai-opc-reference-research-v1.md`](2026-08-16-deepseek-harness-ai-opc-reference-research-v1.md)。本文保留第一次判断的来路，不再单独充当当前结论，也不授权实现。

2026-08-16 校准：本文当时主要基于发布日公开材料，且把“没有用户批准链”写得过于绝对。当前官方版本已有失败封闭的一次性 Approval、Checkpoint 与文件系统 Sandbox 等积木，但仍没有 Syn 所需的持久身份、作用域、授权、撤销和公司级治理。Cordis 的可逆注册也只适用于受管理的运行时 effect，不能外推为邮件、付款、网络发送等现实副作用可逆。Syn 当前决定是不以 DSH 为核心依赖，而是原生实现治理与执行核心，并把 DSH 作为方法来源和可选 `AgentRuntime` 适配对象。

## 0. 先说薄弱点

- DeepSeek Harness 于 2026-08-13 进入 developer preview（v0.1），MIT 开源。本文基于发布当日公开文档、官网、新闻报道和技术分析文章，未本地运行或审计源码。
- v0.1 处于 developer preview，文档明确说明会有 breaking changes，技术细节仍在快速变化。
- 本文只代表 2026-08-14 公开资料状态，后续版本可能有重大调整。
- DeepSeek Harness 是 plugin-first agent runtime，不是我们的项目主管制个人 AI 工作台，不能照搬为最终架构。

## 1. 本轮结论

DeepSeek Harness 值得作为 M5（执行闭环）和 M8（adapter / connector）阶段的参考，但不应进入当前执行计划。现阶段 Syn 处于 M1（合同与安全基础），Harness 的工程细节暂不需要。

最有参考价值的是：

- session log 作为 agent 唯一真源的设计（append-only event log，可重建上下文）
- step / turn 分离的 agent loop 结构
- 工具注册的 schema + policy enforcement 前置
- Cordis 插件可逆卸载机制（reversible registration）
- 多模型 provider adapter 抽象（与模型无关的运行时）

明确不吸收：

- 整个 Harness 不能作为 Syn 的 agent runtime 底座：它没有项目隔离、作用域、正式事实治理、用户批准链
- Cordis「Everything is a Plugin」不能替代 Syn 的 identity/scope kernel：权限边界可插拔在 Syn 是风险
- Git-hosted plugin `prepare` 脚本在沙箱外运行（官方已标注 supply chain 风险），与 Syn 安全原则不兼容
- 不能因为 Harness 已接入某 provider 就自动认为该能力在 Syn 已授权可用

## 2. DeepSeek Harness 是什么

2026-08-13 随 V4 Pro 模型同步发布，MIT 开源，GitHub 地址 `deepseek-ai/deepseek-harness`。

核心理念：**Everything is a Plugin**。基于 [Cordis](https://github.com/cordiverse/cordis) 插件元框架，模型、工具、session 状态、沙箱、agent loop、调度、UI 全是可替换插件，没有不可拔出的核心。

不是 DeepSeek 专属：已文档化支持 Anthropic、OpenAI、Bedrock、Vertex、Azure、Codex 和自定义 OpenAI-compatible endpoint。

快速启动：

```bash
npx @deepseek-ai/dsh web
```

## 3. 技术架构

### 3.1 Cordis 插件系统

五个基础机制：

- 插件挂载到共享 context，使用稳定 key（`ctx.llm`、`ctx.tools`、`ctx.sessions`）
- 依赖通过 `inject` 声明，控制激活顺序
- typed events 处理观察、包装、并行工作和有序决策
- **注册可逆**：卸载插件会干净移除其 listener、工具和 prompt section
- 无特权核心：整个运行时是挂载服务的树结构

### 3.2 Agent Loop 设计

区分两个层次：

- **Step**：一次模型请求 + 工具调用（含结果）
- **Turn**：多个 step 直到没有后续工作

Session log 存储每个 turn 的持久化事件、模型消息、工具调用和工具结果。架构不变量：**任何展示给模型的内容都必须可从 log 单独重建**，支持 resume / fork / replay / prompt 派生。

### 3.3 四种运行模式

| 模式 | 工具集 | 用途 |
|---|---|---|
| Standard | 文件编辑、Shell、搜索、规划、子 agent | 通用开发 |
| Code（PTC） | TypeScript SDK，模型生成代码协调多轮工具链 | 程序化工具调用 |
| Minimal | Shell + 文件编辑 | 基准测试 |
| Creator | Standard + 运行时状态 / 记忆调试插件 + 自定义模式创建 | 插件开发 |

### 3.4 工具系统

- 工具存在于作用域化的注册表，有 schema、pre-execution policy enforcement 和结果处理
- 通过 `ctx.tools` 访问
- 注册表本身是可替换插件，不只是工具可替换

### 3.5 Session / 存储

- Append-only event log，持久化 + replay + fork + resume 作为一等操作
- 凭据存为 write-only secrets（`$DSH_HOME/.credentials.yaml`）
- 存储行为是部署级关注点，由插件决定

### 3.6 沙箱与执行隔离

- 文件系统、shell、子进程、终端、沙箱 provider 全可替换
- 审批流由「活跃权限策略」管理
- **已知供应链风险**：Git-hosted plugin 的 `prepare` 脚本在沙箱外运行（官方文档明确标注）

## 4. 与 Syn 目标架构的对位

### 4.1 可吸收为蓝图约束（对应 M5 / M8）

| Harness 设计 | 对应 Syn 目标 | 参考阶段 |
|---|---|---|
| Append-only session log 作为唯一真源 | M2 事件账本、readback 结构；「任何展示给模型的内容必须可从 log 重建」可作为 agent context 重建的验证标准 | M2 / M5 |
| Step / Turn 分离 | `Turn` / `WorkerHandoff` / `ExecutionAttempt` 的边界划分参考 | M5 |
| 工具注册 schema + policy enforcement 前置 | 控制核心 adapter capability 校验位置；工具调用结果全文不进事件账本 | M5 / M8 |
| Cordis reversible registration | M9 旧路退役时的 adapter 干净卸载机制参考 | M9 |
| 多 provider adapter 抽象 | M8 AgentAdapter 多 provider 接入合同参考 | M8 |
| Trajectory view（inspect / resume / fork） | Agent run 的 runtime session viewer 设计参考（对应 Paseo §5.9 会话中心升级） | M10 |

### 4.2 明确不吸收

- **不用 Harness 作为 Syn 的 agent runtime 底座**：无项目隔离、作用域、正式事实治理、用户批准链；塞入会绕过整个控制核心
- **不用 Cordis 插件系统替代 identity/scope kernel**：「Everything is a Plugin」意味着权限边界也可插拔，这是 Syn 的安全红线
- **不允许 Harness plugin `prepare` 脚本在 Syn 沙箱外运行**：违反路径逃逸防线和 CredentialRef 规则
- **不把 Harness session log 等同于 Syn 事件账本**：Harness session 是 agent-centric，Syn 事件账本是 scope + project 隔离的权威状态层，两者性质不同
- **不因 Harness 已支持某模型 provider 就认为该 provider 在 Syn 已授权接入**：每个 provider 仍需独立凭据、合同和 M8 任务激活

## 5. 与同类参考文档的关系

| 参考 | 主要价值 | 对应 Syn 阶段 |
|---|---|---|
| Paseo（§5.9） | daemon 作为 control plane、agent lifecycle、worktree 隔离、schedule/loop | M3 / M5 / M10 |
| Odysseus（§5.8） | workspace confinement、prompt injection 防线、Deep Research 建模 | M5 / M8 / M10 |
| **DeepSeek Harness（本文）** | session log 唯一真源、step/turn 分离、工具注册 policy 前置、可逆 adapter 卸载 | M2 / M5 / M8 / M9 |

三份参考都不替代 Syn 的控制核心、项目主管制和正式记忆治理；都只能在对应阶段经 Harness 任务激活后吸收具体能力。

## 6. 后续研究路线建议

```
HARNESS-0 Session Log 格式对比设计
  → 对比 Harness append-only log 与 Syn WorkbenchEventEnvelope 结构差异
  → 输出：M2 事件 payload 设计参考文档

HARNESS-1 Tool Registry Policy 机制研究
  → 研究 Harness 工具注册 schema + pre-execution policy 的具体实现
  → 输出：M5 AgentAdapter 工具调用前置校验的设计参考

HARNESS-2 Provider Adapter 抽象研究
  → 研究多 provider 切换时 session / tool / approval 层保持不变的实现
  → 输出：M8 AgentAdapter 多 provider 合同参考
```

以上路线只是建议，不属于当前 Stage 1，也不授权实现。后续吸收具体能力必须先形成合同并经 Harness 任务激活；全局主管可给 advisory，不能代替用户授权。
