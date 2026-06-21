# 工作流 · 会话与作用域模型 · 方案 v1（2026-06-21）

> 状态：待开发（用户已拍板模型方向）。在现有 `CanvasView` / `WorkflowTemplate` / `execute_workflow_node_dispatch` 上扩，**不换 React Flow 底座、不改双闸逻辑本身**。
> 来源：用户拍板 + 市面成熟范式调研（Temporal / LangGraph / n8n / Dify·Flowise / GitHub Actions）。

## 0. 一句话
把「工作流图（定义）」和「跑一次（运行）」分两层；**会话（执行上下文）绑在运行层、模板只存策略**；作用域分**实验 / 项目**两档；节点会话来源 **「新建 / 已有」平级**（无默认偏向，用户按真实使用再调）。

## 1. 调研依据（成熟范式 → 映射）
- **定义 vs 运行分层**（Temporal）：定义静态、无运行态；输入在**执行启动时**给；一份定义多次执行，各有独立 id / 输入 / 状态。
- **定义声明接口、调用方绑值**（GitHub Actions reusable workflows）：定义 `workflow_call` 声明它需要的 inputs/secrets，**调用时才绑**具体值，定义与来源解耦。→ 模板声明「这节点要一条会话」，运行时才绑真会话。
- **会话 = 线程**（LangGraph）：新 `thread_id` = 开新会话；同 id = resume 续上；状态按线程隔离。→ 我们的 codex 会话同理：新建 = mint 新会话；已有 = resume thread_id。
- **作用域 = 项目，实验↔项目是成熟度阶梯**（n8n 个人/团队项目；Dify vs Flowise）：资源按项目作用域；连接「选已有 or 新建」；先实验、成熟了再进项目。

## 2. 概念模型（三个对象）
| 对象 | 是什么 | 现状落点 | 关键约束 |
|---|---|---|---|
| **定义 Definition** | 画布图：nodes + edges + 节点数据 | `CanvasDefinition` | **不含具体会话 id**（改造点） |
| **成熟模式 Template** | 可复用的定义快照 + 元数据 | `WorkflowTemplate` | 节点存**会话策略**，不存解析后的会话 |
| **运行 Run** | 对定义跑一次：run_id + 每节点解析出的真实会话 + 状态/结果 | 接 C 的 `execute_workflow_node_dispatch` | 会话在此层解析、绑定、真跑 |

## 3. 作用域两档（正交第一层）
画布带 `scope: "experiment" | "project"` + `project_root?: string`。
- **实验画布**：不绑项目。画/试编排。真跑只能打**固定测试项目**（= 现 C 双闸的靶子）。
- **项目画布**：绑一个真实项目。真跑落该项目（高危 #1，逐次授权）。
- **升级**：实验→「绑定到项目」可把草稿升级为项目画布（类比 Flowise→Dify、n8n 个人→团队）。
- **闸不变**：后端 `execute_workflow_node_dispatch` 双闸仍是**唯一权威**；scope 只决定 UI 给不给真跑入口 + 默认新建会话建在哪。

## 4. 会话模型（正交第二层）— 核心
节点的「会话」= 它的执行上下文（一条 codex 会话，resume-based）。存**策略**，运行时解析。
- **会话策略**（两种**平级**，UI 对等单选/分段，无默认偏向）：
  - `{ mode: "new" }` — 运行时在（实验=测试项目 / 项目=该项目）内 **mint 一条新 codex 会话**绑上（走已验的 `new_session` 路径）。
  - `{ mode: "resume", thread_id }` — resume 指定的已有会话（走 resume 路径）。
- **运行时解析**：Run 启动时，每节点 policy → 真实 thread_id（new=新建 / resume=用给定）。= GitHub Actions「调用方绑值」+ LangGraph「thread_id 生命周期」合流。

## 5. 关键改造（从现状到模型）
**现状的 reuse bug（已坐实）**：`CanvasNode` 把 `session_id` 焊进节点（定义层）；`WorkflowTemplate` 存的 nodes 含 session_id；`instantiateTemplateGraph` 原样带过去 → **从成熟模式起新工作流会继承别人的旧会话 id**，一跑就 resume 了那条旧对话。这正是「会话不该进定义/模板」的活证据。

**改造**（轻档，纯前端 + 存储，零执行）：
1. **CanvasNodeData**（TS）：去掉 `session_id: string|null`，加 `session: SessionPolicy`，其中 `SessionPolicy = { mode: "new" } | { mode: "resume"; thread_id: string }`。
2. **持久化**：`session` 存进 `CanvasNode.data`（现成的 opaque passthrough，跟 `prompt`/`work_item_id` 同处），**不动 Rust `CanvasNode` 结构**。
   - 向后兼容：旧节点若 `data.session` 缺失 → 看顶层 `session_id`：有值 → `{ mode:"resume", thread_id }`，无 → `{ mode:"new" }`。顶层 `session_id` 保留供既有 sealed 逻辑读。
3. **模板**：`WorkflowTemplate` 节点照常存（policy 在 data 里就自动跟着走）。**实例化时把 resume 的 thread_id 处理掉**（见 §8 决策点）。
4. **运行层**（P3）：`buildNodeDispatchRequest` 前加一步 **policy → session 解析**：resume 用 thread_id；new 调 `new_session` 路径建会话再绑。

## 6. UX
- 节点编辑器「接执行」折叠区里，**会话**改成两个**平级**选项：`[ 新建会话 | 续已有会话 ]`（分段控件）；选「续已有」才出可搜的会话选择器（即前面那个 datalist/搜索框，仅在此分支出现）。
- 画布顶部：**scope 指示**（实验 / 项目 + project_root）+（实验态）「绑定到项目」按钮。
- 真跑入口按 scope 给：实验 → 打测试项目；项目 → 打该项目（均走双闸）。

## 7. 边界 / 档位
- **轻档**（默认）：数据模型（session_policy、scope 字段）+ 向后兼容迁移 + UI 选择器/指示 + 模板存 policy + 实例化。纯前端 + 存储，**零执行**。
- **重档**（高危 #1）：运行层 `policy → session` 解析接 C 派发（new 建会话 / resume）+ 真跑。复用现双闸、不新开闸、逐次授权。「new 在真实项目建会话」也算真跑准备，仍过闸。
- 不换 React Flow；不改双闸逻辑本身；**执行子线不 commit**；机器绿 ≠ 真机，真机由用户统一验。

## 8. 决策（已拍板 2026-06-21）
1. **模板里的 resume{thread_id}**：模板只存「续已有」意图、**实例化时 thread_id 清空**（落地为「续已有但未指定」，用户在新实例上重选），不继承旧会话。✅
2. **新建会话 mint 时机**：**真跑时才建**（定义层无副作用，合 Temporal）；编辑时不产生真会话。✅
3. **一次运行的会话粒度**：**每节点各自一条会话**（codex resume 是单会话）。✅

## 9. 建议分期
- **P1（轻档）**：数据模型 `session_policy` + `scope` 字段 + 向后兼容迁移 + 离线测（策略往返 / 旧 session_id 迁移 / 模板实例化清 thread_id）。不碰运行。
- **P2（轻档）**：UX —— 会话平级分段控件、scope 指示 + 「绑定到项目」、模板存 policy 并按 §8.1 实例化。
- **P3（重档）**：运行层 `policy → session` 解析接 C 派发（new=`new_session` / resume），真跑逐次授权。

## 验证
- 机器：typecheck + offline（policy 往返、向后兼容迁移、实例化清 thread_id、buildNodeDispatchRequest 按 policy 出请求的断言）。
- 真机：用户统一验（建节点设两种会话策略 → 存 → 重载不丢；模板起新工作流不继承旧会话；P3 授权下真跑 new/resume 各一次）。

## 同步
- 落地后回写 `CURRENT.md`：会话/作用域模型从「在做」挪入对应状态；标注 P1/P2 轻档、P3 重档待授权。
- 关联：`docs/plans/2026-06-21-free-canvas-node-authoring-and-mature-pattern-plan-v1.md`（A/B/C 落地）、`decisions/2026-06-21-next-step-unseal-workflow-engine-for-test-project-v1.md`（双闸/派发）。
