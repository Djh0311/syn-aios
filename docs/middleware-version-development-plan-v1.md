# 中间版本开发方案

版本：v1.0  
日期：2026-06-03  
状态：已确认中间版本权威口径；原始阶段草案保留为历史素材，执行必须按本文件第 0 节和后续任务包解释。

审查结论（2026-06-03）：

- 本文的目标方向可作为“骨架之后的中间版本目标草案”：自动化工作流、角色协作、候选到正式记忆、任务包召回和想法箱方向都与最终蓝图一致。
- 但本文下方原始阶段草案不能按原文直接作为执行任务包。当前权威架构要求控制核心掌握事实、权限、状态机、审计和候选转事实；原始草案多处把 `codex exec` 自动派发、正式记忆写入、SQLite 主存储和角色会话能力放得过早。
- 后续应按第 0 节改写为小批次执行计划：先做 schema / 端口 / 读模型 / 方案授权闭环，再做受控真实派发、正式记忆写入和任务包召回；任何真实 `codex exec`、正式 `MemoryRecord` 写入、workflow state 结构变更或 `~/.codex-workbench/` 数据目录落地都必须单独任务包和用户确认。
- `readback` 原生 parser 迁移已完成，但“读取失败可见化”后置，不能把读取失败显示成真实 0 条结果；该后续项记录在 `backlog.md`。

用户补充确认（2026-06-03）：

- 中间版本最终必须做完“自动化工作流”和“记忆层”两个核心闭环，不能只停在骨架、候选治理或只读模型。
- SQLite 是否替换现有 JSON / sidecar 不是产品目标，只是实现选择。后续任务不能把“建库建表”当成完成记忆层。
- 本版本最重要的完成标准是把 `docs/memory-layer-design-v1.md` 规划的记忆层落实到位：观察、候选、正式记忆、来源、版本、权限、冲突、审计、召回和任务包注入都要形成可验证闭环。

---

## 确认后的中间版本权威口径

本节是当前解释权最高的中间版本口径。下方原始阶段规划保留为历史草案和素材，不能绕过本节直接执行。

后续任务包如果和本节冲突，以本节为准；如果需要调整本节，必须先更新本文和 `CURRENT.md` / `tasks/README.md`，不能只在对话里临时口头改。

### 0.1 一句话

中间版本要做成：

用户确认方案后，工作台能在授权范围内自动推进一轮项目工作；项目主管管理过程，全局主管复核方案边界和最终结果，用户主要看方案、结果和必须拍板的事项；秘书只整理、提醒和解释，不当工作成果裁判。

### 0.2 角色分工

用户负责：

- 确认方案。
- 确认最终结果是否接受。
- 确认用户偏好、跨项目影响、全局蓝图和高风险越界。
- 不负责盯每个 worker 的日常过程。

全局主管负责：

- 方案开始前复核方案边界、整体架构和跨项目影响。
- 结果结束后复核最终成果、全局影响和是否需要沉淀。
- 中途只在重大异常时介入，例如越界、架构冲突、跨项目影响、严重失败、时间或成本失控。
- 不逐个确认 worker 的日常汇报。

项目主管负责：

- 在用户已确认的方案范围内拆任务、派 worker、看汇报、确认过程事实。
- 决定继续、返工、暂停或上报。
- 生成项目记忆候选，确认低风险本项目记忆。
- 生成子智能体任务记忆包。

worker 负责：

- 执行具体任务。
- 输出结构化汇报、证据、文件变化、失败原因和需要更多权限的请求。
- worker 汇报不是正式事实，必须由项目主管结合证据确认。

秘书负责：

- 整理信息、解释状态、汇总待确认项、提醒风险、收纳想法。
- 不确认 worker 的话。
- 不判断项目过程事实。
- 不直接派活。
- 不直接写正式记忆。

### 0.3 方案授权制

中间版本采用方案授权制，不采用每一步确认制。

用户确认方案时，等于确认一段自动执行范围：

- 目标是什么。
- 允许哪些角色和 agent 参与。
- 允许读取、写入和执行的范围。
- 允许哪些测试、检查或工具。
- 哪些动作必须停下来请用户确认。
- 最终如何验收。

方案确认后，项目主管可以在授权范围内自动派 worker 和推进过程，不需要每个 worker 启动前都问用户。

必须停下来问用户的情况：

- 超出方案范围。
- 需要读取敏感内容。
- 需要删除、大范围改动或不可逆操作。
- 涉及跨项目影响。
- 和当前架构或最终蓝图冲突。
- 需要确认用户偏好、全局蓝图、跨项目记忆或成熟模式。
- 失败、耗时或成本超出方案预期。

### 0.4 AI 汇报和事实确认

- worker 的话只是汇报。
- 项目主管结合证据确认项目内过程事实。
- 全局主管确认方案边界和最终结果，不逐条确认 worker 汇报。
- 用户确认高层方案、最终接受、重大方向和高风险记忆。
- 秘书不做事实确认。

worker A 的汇报可以作为后续 worker 的材料，但要区分层级：

- worker 自己声称的结果：只能作为低可信过程材料。
- 项目主管确认的过程事实：可以进入后续 worker 的任务包。
- 全局主管复核的最终结果：可以进入结果总结和记忆候选。
- 正式长期记忆：必须满足来源、版本、权限、冲突和审计规则。

### 0.5 记忆层完成标准

中间版本最终必须做完自动化工作流和记忆层两个核心闭环，不能只停在骨架、候选治理、只读模型或建库建表。

SQLite 是否替换现有 JSON / sidecar 不是产品目标，只是实现选择。验收只看记忆层能力是否符合 `docs/memory-layer-design-v1.md`。

必须跑通：

- `ObservationStore`：记录 worker 汇报、项目主管确认、全局主管复核、方案和结果。
- `MemoryCandidate`：从观察和主管总结中生成候选，标明来源、作用域、类型、权限和状态。
- `MemoryRecord` / `MemoryPage`：正式记忆写入，包含当前结论、正文、来源引用、版本、生命周期状态、权限和审计链接。
- `MemoryVersion`：正式记忆变更保留版本，不直接覆盖旧记忆。
- `MemoryConflict` / `MemoryLintFinding`：发现冲突、重复、过期、缺来源时进入提示或阻断。
- `MemoryAuditEvent`：采纳、修改、废弃、冻结、召回进入任务包都要可追踪。
- `TaskMemoryPacket`：后续 worker 启动前，能从正式记忆和项目主管确认的过程事实中召回相关内容，并记录纳入和排除原因。
- 用户偏好记忆必须作为高优先级记忆类型处理；普通聊天不能自动进入正式长期记忆。

补充确认：

- 上面这组能力只是“记忆层能跑通”的最低闭环，不是中间版本记忆系统的完整终点。
- 中间版本最终目标是把 `docs/memory-layer-design-v1.md` 的工作台记忆系统做成可运行产品能力，而不是只做一条“候选变正式记忆再召回”的 demo。
- 因此最终还必须覆盖正式记忆生命周期：新增、编辑、废弃、冻结、归档、合并、拆分、上升为全局记忆、下沉为项目记忆，并且所有变化都有版本和审计。
- 最终还必须覆盖关系和实体治理：实体去重、别名、关系候选、正式关系、因果关系确认、重复 / 漂移 / 放错层级检查。
- 最终还必须覆盖维护任务：过期、冲突、缺来源、重复、私密内容、权限撤回、索引状态和成熟模式检查。维护任务只能生成提醒、候选、冲突或隔离，不能自动改正式记忆。
- 最终还必须覆盖任务包召回治理：候选、观察、知识库命中、LLM 摘要、派生图谱不能伪装成正式记忆；正式记忆进入或排除任务包都要有理由和审计。
- 最终还必须覆盖知识库边界：Obsidian-compatible 知识库是材料和思考空间，不是记忆层权威；知识库材料可以生成候选，但不能绕过状态机。

### 0.6 自动化工作流完成标准

必须能跑通：

1. 项目咨询把用户目标整理成方案。
2. 用户确认方案。
3. 全局主管复核方案边界。
4. 项目主管拆任务并在授权范围内派 worker。
5. worker 执行并结构化汇报。
6. 项目主管确认过程事实，决定继续、返工、暂停或上报。
7. 工作台记录过程事实、权限、readback、失败和审计。
8. 后续 worker 可以使用已确认过程事实或正式记忆继续执行。
9. 全局主管复核最终结果。
10. 用户查看最终结果和必须拍板的事项。

## 修订后的执行边界

1. Shell + 结构化协议只能通过 AgentAdapter / 控制核心执行，不能由 UI 或项目主管输出直接触发真实命令。
2. 工作台解析结构化指令后，在已确认方案授权范围内可以由控制核心自动派发；超出授权范围时必须停下来确认。
3. 候选记忆确认不能被偷换成正式记忆写入；正式记忆写入必须满足来源、版本、权限、冲突、审计和回滚规则。
4. SQLite 可以成为中间版本主存储方向，但不能把“建库建表”当成记忆层完成；如果替代现有 `workflow-state.v0.json` 或候选 sidecar，必须有迁移 / 双写 / 回滚方案。
5. 角色会话必须映射到现有架构：秘书是核心协作层，项目主管 / 咨询 / worker 是项目单元内的角色和适配器会话，所有执行动作由控制核心落账和审计。
6. 任务包召回只能使用已确认、可审计、权限允许的正式记忆；候选、黑板内容、知识库命中和 LLM 摘要不能伪装成正式记忆。
7. 自动化闭环必须保留方案授权边界、失败可见化、审计记录和手动恢复入口。

## 一、目标定义

### 1.1 核心目标

中间版本必须实现：
1. **自动化工作流**：项目咨询 → 项目主管 → 子智能体完整闭环
2. **记忆层**：观察 → 候选 → 正式记忆 → 任务包召回
3. **5个角色**：秘书、全局咨询、项目咨询、项目主管、子智能体
4. **想法箱**：自然对话收纳想法

### 1.2 核心验证场景

注意：下面是原始验证场景草案。执行时必须按 `## 确认后的中间版本权威口径` 修订：用户确认的是方案授权范围；项目主管在授权范围内自动推进；全局主管只复核方案边界和最终结果；秘书不当裁判；正式记忆必须满足来源、版本、权限、冲突和审计规则。

```
用户 → 项目咨询："我想加用户注册功能"
  ↓
项目咨询细化需求 → 生成方案（Markdown）
  ↓
用户确认方案
  ↓
项目主管读方案 → 拆解任务 → 输出结构化指令
  ↓
工作台解析指令 → 创建2个子会话（后端接口 + 前端表单）
  ↓
子会话A执行 → 输出结构化汇报
  ↓
项目主管读汇报 → 提取事实 → 生成候选记忆
  ↓
用户确认候选 → 正式记忆
  ↓
子会话B启动 → 任务包自动包含"后端接口已完成"记忆
  ↓
子会话B执行 → 汇报
  ↓
验证：会话B能看到会话A的成果
```

### 1.3 明确不做的功能

- ❌ 向量检索（推迟到后续）
- ❌ GraphRAG 社区摘要（推迟到后续）
- ❌ 代码图谱（推迟到后续）
- ❌ 知识库（明确推迟）
- ❌ semantic/causal/entity 关系（只做 temporal）
- ❌ 内置 LLM（中间版本依赖 Codex/Claude Code）

---

## 二、架构决策

### 2.1 会话创建方式

注意：本节是原始草案，需要按方案授权制重写。不得把“项目主管输出 JSON”直接解释为“无条件立即执行真实 `codex exec`”。真实派发只能在已确认方案授权范围内，经 AgentAdapter / 控制核心落账和审计；超出授权范围必须停下来确认。

**问题**：MCP 现有功能不支持自动创建会话，Codex/Claude Code 没有 HTTP API

**决策**：Shell + 结构化协议

- 项目主管输出 JSON 格式的结构化指令（不是自然语言）
- 工作台解析 JSON → 调用 `codex exec` 创建子会话
- 子会话按约定格式输出结构化汇报
- 这是临时方案，最终版本会内置 LLM

**结构化指令格式**：
```json
{
  "action": "create_workers",
  "workers": [
    {
      "worker_id": "worker_001",
      "agent": "codex",
      "task_package": {
        "goal": "实现后端注册接口",
        "memory_refs": ["mem_001", "mem_002"],
        "constraints": ["不能删除文件", "必须写单元测试"]
      }
    }
  ]
}
```

**结构化汇报格式**：
```json
{
  "completed": "实现了后端注册接口 POST /api/auth/register",
  "facts": [
    "API 路由：POST /api/auth/register",
    "使用 JWT 认证",
    "密码用 bcrypt 加密"
  ],
  "files_changed": ["src/api/auth.ts", "tests/auth.test.ts"]
}
```

### 2.2 记忆层存储架构

**决策**：SQLite 为主，JSON 为辅

**目录结构**：
```
~/.codex-workbench/
├── workbench.db (SQLite，11张表)
└── projects/
    └── {project_id}/
        ├── ideas.json (想法箱)
        ├── memory/ (Markdown 展示页，可选)
        └── workflows/ (工作流状态)
```

**为什么用 SQLite**：
- 查询性能（WHERE status='active' AND project_id=?）
- 事务支持（候选确认是原子操作）
- 审计不可删除（只追加）

### 2.3 召回策略

**决策**：关键词匹配 + 时间排序 + 任务意图过滤

**召回流程**：
1. 查询所有 `status='active'` 的项目记忆
2. 按任务类型过滤（implement/review/research/debug）
3. 提取任务目标的关键词
4. 计算每条记忆的关键词匹配分数
5. 按分数排序，分数相同按时间排序（最新优先）
6. 裁剪到 token 限制（10K tokens）
7. 记录被排除的记忆和排除原因

**任务意图过滤规则**（参考文档第19.3节）：
- `implement` 任务 → 只召回"当前代码状态、项目规则、相关决策"
- `review` 任务 → 只召回"验收标准、历史问题"
- `research` 任务 → 只召回"最终蓝图、已确认设计"
- `debug` 任务 → 只召回"最近变化、失败记录、相似历史问题"

**不做向量检索的原因**：
- 实现成本高（需要 embedding 模型）
- 关键词匹配已能覆盖 80% 场景
- 文档第9节明确说"早期可以先用关键词、标签、来源引用和简单关系查询"

---

## 三、数据模型

### 3.1 核心表结构（11张表）

**3.1.1 projects**
- 存储项目信息
- 字段：project_id, name, root_path, status, created_at

**3.1.2 role_sessions**
- 存储角色会话（秘书、咨询、主管、子智能体）
- 字段：session_id, project_id, role, agent_type, status, created_at, last_active_at
- role 枚举：secretary, global_consultant, project_consultant, project_director, worker

**3.1.3 workflows**
- 存储工作流
- 字段：workflow_id, project_id, status, created_at

**3.1.4 workflow_nodes**
- 存储工作流节点
- 字段：node_id, workflow_id, node_type, session_id, status, task_package_json, report_json, created_at
- node_type 枚举：consultant, director, worker, review

**3.1.5 memory_candidates**
- 存储候选记忆（等待用户确认）
- 字段：candidate_id, project_id, memory_type, claim, body, source_refs_json, generated_by_role, requires_user_confirmation, status, created_at
- status 枚举：draft, needs_review, approved, rejected

**3.1.6 memory_pages**
- 存储正式记忆
- 字段：memory_id, project_id, memory_type, title, current_summary, body, status, version_id, source_refs_json, created_at, updated_at
- status 枚举：active, conflicted, deprecated, frozen

**3.1.7 memory_versions**
- 存储记忆版本历史
- 字段：version_id, memory_id, version_number, change_type, change_summary, body_snapshot, created_at

**3.1.8 memory_relations**
- 存储记忆之间的关系
- 字段：relation_id, from_memory_id, to_memory_id, relation_type, confidence, status, created_at
- relation_type：中间版本只做 temporal（时间顺序）
- confidence 枚举：user_confirmed, director_confirmed, static_extracted, llm_inferred

**3.1.9 memory_audit_events**
- 存储审计事件（不可删除）
- 字段：audit_event_id, event_type, actor_role, target_ref, before_state, after_state, reason, created_at

**3.1.10 observation_store**
- 存储观察层数据（临时）
- 字段：observation_id, project_id, session_id, workflow_node_id, observation_type, summary, source_refs_json, created_at

**3.1.11 task_memory_packets**
- 存储任务包（审计用）
- 字段：packet_id, workflow_node_id, included_memory_refs_json, excluded_memory_refs_json, retrieval_reason, generated_at

### 3.2 核心数据流

```
观察层（临时）
  ↓
候选层（等待确认）
  ↓
正式记忆层（长期）
  ↓
任务包（按需召回）
```

---

## 四、开发阶段（12周 = 3个月）

### 阶段 0：基础设施（Week 1-2）

#### 目标
搭建数据层和基础 UI

#### 交付物
1. SQLite 数据库和 11张表
2. 基础 CRUD Tauri commands
3. 项目列表 UI
4. 记忆候选列表 UI
5. 候选详情卡片 UI

#### 功能需求

**数据库**
- 创建 11张表
- 创建必要的索引（project_id, status, created_at）
- 支持事务

**Tauri Commands**
- `create_project(name, root_path)` → project_id
- `list_projects()` → Project[]
- `create_memory_candidate(candidate_data)` → candidate_id
- `list_memory_candidates(project_id?, status?)` → MemoryCandidate[]
- `load_memory_candidate(candidate_id)` → MemoryCandidate
- `approve_memory_candidate(candidate_id)` → memory_id
- `reject_memory_candidate(candidate_id)` → void

**UI 组件**
- 项目列表：显示所有项目，支持创建新项目
- 记忆候选列表：显示待审查的候选，按项目过滤
- 候选详情卡片：显示候选内容、来源、确认/拒绝按钮

#### 验收标准
- ✅ 能创建项目，显示在列表中
- ✅ 能手动创建候选记忆
- ✅ 能在 UI 查看候选列表和详情
- ✅ 能确认候选 → 自动创建正式记忆、版本记录、审计事件
- ✅ 能拒绝候选 → 更新状态、写审计事件

---

### 阶段 1：工作流基础（Week 3-4）

#### 目标
实现工作流创建和节点手动执行

#### 交付物
1. 工作流创建 UI
2. 工作流节点列表视图（不做复杂图形）
3. 节点执行引擎（支持 worker 节点）
4. Shell 调用 `codex exec`
5. 汇报解析

#### 功能需求

**工作流管理**
- `create_workflow(project_id, name)` → workflow_id
- `add_workflow_node(workflow_id, node_type, label, task_package?)` → node_id
- `list_workflow_nodes(workflow_id)` → WorkflowNode[]
- `load_workflow(workflow_id)` → Workflow

**节点执行**
- `execute_workflow_node(node_id)` → WorkerReport
  - 读取节点的 task_package
  - 构建任务包 prompt（包含目标、记忆、约束、汇报格式）
  - 调用 `codex exec --project=<root_path>`，通过 stdin 传入 prompt
  - 等待执行完成（最长1小时）
  - 从 stdout 解析汇报（提取 `===TASK_REPORT===` JSON 块）
  - 保存汇报到 workflow_nodes 表
  - 更新节点状态为 completed 或 failed

**任务包 Prompt 格式**
```
你是项目主管派发的子智能体。

【任务目标】
<task_package.goal>

【项目记忆】
<从 memory_pages 加载 task_package.memory_refs>
- 记忆1的 current_summary
- 记忆2的 current_summary

【约束】
<task_package.constraints>
- 约束1
- 约束2

【汇报要求】
完成后，请输出以下格式的汇报：
===TASK_REPORT===
{
  "completed": "任务完成情况",
  "facts": ["关键事实1", "关键事实2"],
  "files_changed": ["文件列表"]
}
===END_REPORT===
```

**汇报解析规则**
- 从 stdout 查找 `===TASK_REPORT===` 和 `===END_REPORT===`
- 提取中间的 JSON
- 反序列化为 WorkerReport 对象
- 如果解析失败，节点状态标记为 failed，错误信息保存到 report_json

**UI 组件**
- 工作流创建表单：输入名称，选择项目
- 工作流节点列表：显示所有节点，每个节点显示类型、标签、状态
- 节点执行按钮：点击后调用 `execute_workflow_node`，显示执行中状态
- 节点汇报显示：执行完成后，显示汇报内容

#### 验收标准
- ✅ 能创建工作流
- ✅ 能手动添加节点（worker 类型）
- ✅ 能为节点配置 task_package（目标、约束）
- ✅ 能执行节点，调用 `codex exec`
- ✅ 能在 UI 看到"执行中"状态
- ✅ 执行完成后，能看到汇报内容（completed, facts, files_changed）
- ✅ 如果汇报格式错误，节点状态为 failed

---

### 阶段 2：记忆生成流程（Week 5-6）

#### 目标
从工作流汇报自动生成候选记忆

#### 交付物
1. 汇报 → ObservationStore
2. 项目主管读 ObservationStore → 生成候选记忆
3. 候选确认 → 正式记忆（阶段0已实现，本阶段集成）

#### 功能需求

**观察层**
- `save_observation(project_id, session_id?, workflow_node_id?, observation_type, summary, source_refs)` → observation_id
  - 自动调用：节点执行完成后，汇报自动保存为 observation
  - observation_type = "worker_report"
  - summary = report.completed
  - source_refs 包含 node_id 和完整 report JSON

**候选生成**
- `generate_memory_candidates_from_observation(observation_id)` → MemoryCandidate[]
  - 读取 observation
  - 调用项目主管会话（通过 `codex exec` 模拟）
  - 项目主管 prompt 包含：
    - 角色定义："你是项目主管"
    - 任务："从以下工作汇报中提取长期有效的事实"
    - 汇报内容
    - 输出格式要求（JSON 数组，每项包含 claim, body, memory_type）
  - 解析项目主管输出的 JSON
  - 为每个候选创建 MemoryCandidate 记录
  - status = "needs_review"
  - requires_user_confirmation = true
  - source_refs 指向 observation_id

**项目主管输出格式**
```
===MEMORY_CANDIDATES===
[
  {
    "claim": "用户注册接口实现完成",
    "body": "API 路由：POST /api/auth/register，使用 JWT 认证，密码用 bcrypt 加密（salt rounds = 10）",
    "memory_type": "project_memory"
  }
]
===END_MEMORY_CANDIDATES===
```

**自动化流程集成**
- 节点执行完成 → 自动保存 observation
- 自动调用 `generate_memory_candidates_from_observation`
- UI 显示"生成了 N 个候选记忆"通知

**UI 增强**
- 工作流节点完成后，显示"生成了 N 个候选记忆"
- 候选列表显示来源（从哪个 observation 生成）
- 点击来源，能跳转到对应的工作流节点

#### 验收标准
- ✅ 节点执行完成后，汇报自动进入 observation_store
- ✅ 自动调用项目主管生成候选记忆
- ✅ 项目主管能正确解析汇报，提取事实
- ✅ 候选记忆的 source_refs 正确指向 observation
- ✅ 用户能在候选列表看到新生成的候选
- ✅ 确认候选后，正式记忆包含完整的来源链

---

### 阶段 3：任务包生成和召回（Week 7-8）

#### 目标
新节点启动时，自动加载相关记忆到任务包

#### 交付物
1. TaskMemoryPacketBuilder
2. 关键词匹配算法
3. 任务意图过滤
4. 任务包注入到节点 prompt

#### 功能需求

**任务包生成**
- `build_task_memory_packet(project_id, task_goal, task_type)` → TaskMemoryPacket
  - 输入：project_id, task_goal（字符串）, task_type（implement/review/research/debug）
  - 输出：
    - included_memory_refs：入选的记忆列表，每项包含 memory_id 和 reason
    - excluded_memory_refs：被排除的记忆列表，每项包含 memory_id 和 reason
    - retrieval_reason：总体召回理由

**召回算法**
1. 查询：`SELECT * FROM memory_pages WHERE project_id = ? AND status = 'active'`
2. 任务意图过滤
3. 关键词提取和相关性打分
4. 排序：按分数降序，分数相同按 updated_at 降序
5. Token 裁剪：累加到 10K tokens
6. 记录排除原因

**集成到节点执行**
- 修改 `execute_workflow_node`：
  - 如果 task_package.memory_refs 为空，自动调用 `build_task_memory_packet`
  - 生成 TaskMemoryPacket
  - 保存到 task_memory_packets 表
  - 加载 included 的记忆
  - 注入到 prompt

**UI 组件**
- 任务包预览：节点执行前，显示"将加载 N 条记忆"
- 点击展开，显示入选的记忆列表和理由
- 显示被排除的记忆数量和主要排除原因

#### 验收标准
- ✅ 节点执行前，自动生成任务包
- ✅ 任务包正确召回相关记忆
- ✅ 任务包记录了排除原因
- ✅ 节点 prompt 包含召回的记忆
- ✅ 验证场景通过：会话A的记忆被会话B召回

---

### 阶段 4：角色会话管理（Week 9-10）

#### 目标
实现5个角色会话的创建和管理

#### 交付物
1. 角色会话创建 UI
2. 秘书会话（全局）
3. 全局咨询会话
4. 项目咨询会话（每项目一个）
5. 项目主管会话（每项目一个）

#### 功能需求

**角色会话创建**
- `create_role_session(project_id?, role, agent_type)` → session_id
  - project_id：null → 全局角色，非null → 项目角色
  - role：secretary, global_consultant, project_consultant, project_director
  - 自动检查：同一项目的同一角色只能有一个 active 会话

**会话对话功能**
- `send_message_to_role_session(session_id, message)` → response
  - 通过 shell 调用对应 agent
  - 保存对话历史（observation_store）

**角色系统提示词注入**
- 秘书：自动加载所有 memory_type = 'user_preference' 的记忆
- 全局咨询：自动加载所有 memory_type = 'global_blueprint' 的记忆
- 项目咨询：自动加载该项目的 ideas.json，有工具 `add_idea_to_backlog`
- 项目主管：自动加载该项目的所有 project_memory

**UI 组件**
- 角色会话列表：显示所有角色会话，按项目分组
- 角色会话创建表单：选择项目、角色、agent
- 角色会话对话界面：简单的聊天窗口
- 秘书快捷入口：悬浮图标

#### 验收标准
- ✅ 能创建秘书会话（全局）
- ✅ 能创建项目咨询会话（每项目一个）
- ✅ 能创建项目主管会话（每项目一个）
- ✅ 同一项目的同一角色不能重复创建
- ✅ 角色会话能正确加载对应的系统提示词和记忆
- ✅ 项目咨询能调用工具添加想法到 ideas.json
- ✅ 项目主管能输出结构化指令

---

### 阶段 5：想法箱和完整工作流（Week 11-12）

#### 目标
实现想法箱和端到端自动化工作流

#### 交付物
1. 想法箱 UI
2. 项目咨询 → 想法箱自动收纳
3. 完整工作流：咨询 → 主管 → 子智能体
4. 端到端验证

#### 功能需求

**想法箱**
- 数据结构：`projects/{project_id}/ideas.json`
- `add_idea(project_id, title, summary)` → idea_id
- `list_ideas(project_id, status?)` → Idea[]
- `update_idea_status(idea_id, status)` → void
  - status：idea, discussing, confirmed, abandoned

**项目咨询工具集成**
- 项目咨询会话有工具：`add_idea_to_backlog`
- 对话中识别想法 → 调用工具 → 自动添加到 ideas.json

**方案生成功能**
- 项目咨询能生成方案（Markdown 格式）
- 方案包含：背景、目标、建议、影响范围、风险
- `save_proposal(project_id, content)` → proposal_id
- `load_proposal(proposal_id)` → Proposal

**完整工作流集成**
- `create_workflow_from_proposal(project_id, proposal_id)` → workflow_id
  - 创建工作流
  - 添加咨询节点（已完成）
  - 添加主管节点（待执行）
  - 自动执行主管节点 → 拆解任务 → 创建 worker 节点

**UI 组件**
- 想法箱列表：显示所有想法，按状态分组
- 想法详情卡片：显示标题、摘要、状态
- 想法状态转换按钮
- 方案查看器：Markdown 渲染
- 方案确认按钮

#### 端到端验证流程
1. 用户和项目咨询聊："我想加用户注册功能"
2. 项目咨询识别想法 → 自动添加到 ideas.json
3. 项目咨询生成方案 → 保存为 Markdown
4. 用户确认方案 → 调用 `create_workflow_from_proposal`
5. 项目主管自动拆解任务 → 创建2个 worker 节点
6. Worker A 执行 → 汇报 → 生成候选记忆
7. 用户确认候选 → 正式记忆
8. Worker B 执行 → 任务包自动包含 Worker A 的记忆
9. Worker B 完成 → 汇报
10. 验证完成

#### 验收标准
- ✅ 项目咨询能自动收纳想法
- ✅ 项目咨询能生成结构化方案
- ✅ 能从方案自动创建工作流
- ✅ 项目主管能自动拆解任务并创建 worker 节点
- ✅ 端到端流程完整跑通
- ✅ 核心验证场景通过

---

## 五、技术栈

### 5.1 前端
- React 18
- TypeScript
- Tauri 2.x
- 复用现有 Codex Workbench 样式

### 5.2 后端
- Rust（Tauri commands）
- SQLite 3
- serde_json（JSON 序列化）

### 5.3 外部依赖
- Codex CLI
- Claude Code CLI
- Shell 命令执行

---

## 六、风险和缓解

### 6.1 风险1：Codex/Claude Code 输出格式不稳定

**风险**：LLM 可能不按约定格式输出结构化指令和汇报

**缓解**：
- 在 prompt 中强调格式要求
- 添加格式校验，失败时重试
- 提供格式示例
- 最多重试3次，失败后人工介入

### 6.2 风险2：记忆召回不准确

**风险**：关键词匹配可能召回不相关的记忆

**缓解**：
- 任务包预览，用户能看到召回了什么
- 记录排除原因，便于调试
- 后续迭代加入向量检索

### 6.3 风险3：Shell 调用性能问题

**风险**：`codex exec` 每次启动新进程，可能较慢

**缓解**：
- 显示执行中状态，用户知道正在等待
- 设置超时（1小时）
- 后续迭代改为内置 LLM

### 6.4 风险4：时间估算不准确

**风险**：12周可能不够

**缓解**：
- 每个阶段结束时评估进度
- 如果延期，优先完成阶段0-3（核心闭环）
- 阶段4-5可以推迟

---

## 七、成功标准

### 7.1 必须达到
- ✅ 核心验证场景完整跑通
- ✅ 记忆能正确生成和召回
- ✅ 工作流能自动执行
- ✅ 5个角色能正常工作

### 7.2 期望达到
- ✅ 召回准确率 > 80%
- ✅ 节点执行成功率 > 90%
- ✅ 候选记忆质量满足要求（用户确认率 > 70%）

### 7.3 可选
- ⚪ 性能优化（节点执行 < 5分钟）
- ⚪ UI 美化
- ⚪ 错误恢复机制

---

## 八、下一步

1. ✅ 用户确认本方案
2. ✅ 创建项目仓库和初始结构
3. ✅ 开始阶段0开发（Week 1-2）
