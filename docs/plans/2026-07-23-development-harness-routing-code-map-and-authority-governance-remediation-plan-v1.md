# 开发 Harness 短路由、代码图谱与权威治理整改执行计划 v1

- 日期：2026-07-23
- 状态：**Phase 0～6、Phase 5-R1 与 Phase 6-R1 已完成并复核；Phase 7 等待用户单独派发**
- 类型：开发治理 / 防重复实现 / 防计划偏航 / 文档新鲜度
- 当前业务执行计划：`docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md`
- 当前开发规则：`AGENTS.md`
- 当前权威索引：`AUTHORITY.md`
- 当前事实正本：`CURRENT.md`
- 对照参考：`/Users/yoyi/Desktop/kt-erp/docs/harness-operating-model-and-lessons.md`

> 本计划只整改开发过程使用的 `scripts/harness/**` 与相关治理文档，不替代产品业务里的 Harness / 验收协议，也不改变当前对话优先业务主计划。
>
> Phase 0 的运行模型决策、基线与 consumer 审计，Phase 1 的只读、fail-open 短路由，Phase 2 的权威索引与 CURRENT 历史分离，Phase 3 的结构化 Code Map，Phase 4 的重要任务计划对齐，Phase 5 / R1 的 config、CLI 与 legacy consumer 收缩，以及 Phase 6 / R1 的只读 maintenance audit 均已完成并复核。本次状态对齐只回写已验收事实，不授权 Phase 7 的回放、观察期、Hook、业务代码或 stage / commit / push；后续阶段仍须逐阶段单独开工。

## 0. 一句话结论

当前 Harness 不需要继续增加总闸，而需要改成一套轻量导航系统：

```text
短路由
  → 先查现有能力
  → 绑定当前权威与计划
  → 只运行任务相关验证
  → 能力边界变化时同步代码图谱
  → 周期性只读查漂移
```

目标不是让每次开发多填表，而是让新会话先走对路、写代码前先看到已有实现、重要任务执行中不脱离当前方案，结束时只更新真正发生变化的文档。

## 1. 当前基线与已观察问题

以下是 2026-07-23 的计划基线。实施开工时必须重新冻结，不能把这些数字当永久事实。

### 1.1 仓库与工作树

- 当前 `HEAD=e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`，本地相对 `origin/main` ahead 18。
- 工作树已有大量已归属和未归属业务改动，包含 `AGENTS.md`、`AUTHORITY.md`、`CURRENT.md`、当前业务计划、Rust 源码、task / evidence / handoff。
- 实施时不得 reset、clean、stash、覆盖整文件或把无关改动一起 stage。
- 本计划必须与当前业务实施串行或使用明确隔离的写入面；不得在业务承重文件发生并行漂移时强行合并。

### 1.2 Harness 体量与默认语义

- `scripts/harness/` 当前共有 95 个文件：88 个 JavaScript、5 个 JSON、2 个 Swift。
- 根 CLI `scripts/harness/harness.js` 默认展示 35 个命令，仍包含 memory、task lifecycle、evidence lifecycle、capability scan 等已经停用或非当前默认路径的入口。
- `harness.config.json` 仍保留 `balanced`、hard/soft gate、pre-work、pre-completion、runtime docs、memory integration 等旧模型字段；其中 Hook 实际关闭、CI 非必需、AgentMemory 实际关闭。
- `.githooks/commit-msg` 是当前真实接线，机械要求 commit message 带 `catch:`；本计划不取消该线，也不新增 Code Map / 文档 Hook。
- `workbench-shape-gate.js` 被大量计划和证据引用，是当前承重的结构检查；`stage-k-architecture-gate.js` 是阶段性专项工具，不能继续扩大为日常总闸。

### 1.3 代码图谱现状

- 当前图谱正本候选是 `docs/2026-07-09-codebase-capability-map-v2.md`，253 行、约 104 KB。
- 该文件最后一次提交核验基于 2026-07-10 的代码；它是静态 Markdown 调查结果，不具备结构化 `query / overlay / check`。
- 它能帮助人工检索，但不能稳定回答“现有 canonical 是谁、调用方是谁、这次 staged rename 是否让入口悬空”。
- 当前 `scripts/harness/capability-map.js` 描述的是本机 Agent / 工具能力，不是仓库代码能力图谱；名称相近，容易误导。

### 1.4 文档与执行偏航

- `AUTHORITY.md` 已经被精简为权威入口，应继续作为唯一人工权威索引，不再新建第二份 registry。
- `CURRENT.md` 只有 80 行但约 56 KB，单行塞入大量历史覆盖说明，已经不再是可快速读取的当前视图。
- `docs/plans/README.md` 仍把旧 Stage K 计划称为当前计划，并要求更新已经停用的 requirements-matrix / task-queue / open-questions，和 `AGENTS.md` 当前规则冲突。
- 已有明确事故：交办页长期加固 resident/private-home 会话链，而没有优先复用智能体页已经存在的 existing/new session、事件转换、轮询、Stop 和 readback 底座。证据在 `handoffs/2026-07-22-jiaoban-conversation-module-reuse-and-syn-mcp-capability-guidance-v1.md`。
- 当前业务任务包已经纠正为复用共享 Conversation Transport，但这一纠偏主要依赖人工发现，尚未变成新任务开始前的固定导航动作。

## 2. 整改目标与不做事项

### 2.1 必须达成

1. 新会话在一分钟内找到当前规则、权威、执行包、下一步、阻塞和安全边界。
2. 新增模块、服务、公共 helper、共享类型、Tauri command、页面能力前，能显式查询已有能力与已知重复。
3. 重要任务开始时能证明它绑定了当前计划，并写清复用对象和禁止路线。
4. 能力边界、canonical、公共入口、关键 consumer、状态归属或合同发生变化时，代码图谱和代码在同一提交内同步。
5. 当前事实与历史记录分开；同一事实只保留一个正文权威，其余位置只放指针。
6. 长期 dirty 工作区不因为无关未提交文件被全局误阻塞。
7. 旧 Harness 入口按真实消费者逐步降级，不先删文件、不制造大爆炸迁移。

### 2.2 明确不做

- 不重写产品业务 Harness、任务验收协议或 Syn 产品功能。
- 不把 Code Map、文档治理或计划判断塞进 `workbench-shape-gate.js`。
- 不让 Code Map 自动改文件，不把它接进 pre-commit / pre-push。
- 不恢复“每个任务都建 task / evidence / reviewer / handoff”的旧生命周期。
- 不把计划、CURRENT 或历史文档里的状态当作代码已经完成的证据。
- 不追求第一版覆盖全仓；允许诚实的 partial seed。
- 不以 gate 数、脚本数、文档数或执行次数作为成功指标。
- 不在本整改中顺手清理业务源码、历史 shape 债或全部旧文档。

## 3. 术语与权威边界

### 3.1 两个 Harness 必须分开

| 名称 | 含义 | 本计划是否修改 |
| --- | --- | --- |
| 开发 Harness / dev-gate | `scripts/harness/**`，帮助开发者导航、检查、验证与留证 | 是 |
| 产品 Harness | 项目级验收协议、任务中的 `harness_requirements` 或产品能力 | 否 |

新文档和 CLI 默认使用“开发 Harness”或“dev-gate”。单独出现 `Harness` 时必须从上下文能看出属于哪一类。

### 3.2 三类权威不要混成一条

**授权与安全：**

```text
用户当次明确指令 > AGENTS.md 安全边界 > 任务包中的更窄限制
```

任务包和计划不能自行扩大用户授权，也不能放宽 `AGENTS.md` 的高危边界。

**执行路由：**

```text
AUTHORITY.md 当前指针
  > 被指向的当前决策 / 任务包
  > 当前业务总执行计划
  > 旧计划、旧 handoff、历史 evidence
```

**事实判断：**

```text
真实源码与新鲜直接验证
  > CURRENT.md 当前摘要
  > Code Map / 现状说明书
  > 计划状态、历史 evidence、聊天记忆
```

Code Map 是导航，不是源码之上的第二真相源；CURRENT 是当前摘要，不得覆盖新鲜的反向证据。

## 4. 目标架构

```text
常驻最小内核
├── AGENTS.md                 当前开发规则与高危边界
├── AUTHORITY.md              唯一人工权威索引
├── docs/project-context.json 当前短路由事实
└── 任务相关直接验证           代码完成声明的证据

显式按需工具
├── project-context --diagnostic
├── codebase-map query / overlay / check
├── checkpoint-audit
├── workbench-shape-gate
├── stage-k-architecture-gate
└── config / doctor / 专项检查

条件协议
├── 新能力 / 跨模块：existing-before-new
├── 重要任务：权威与计划对齐字段
├── 高危动作：AGENTS.md 重档授权
└── 提交：用户授权 + staged snapshot + catch: 标记

维护回路
└── 每周活跃开发期或阶段收口：只读漂移审计；有事实变化才改文档
```

默认路径必须短。Git、dirty tree、Hook、Code Map 全量状态、历史兼容和专项 gate 都不进入默认新会话输出。

## 5. 短路由合同

### 5.1 新文件与入口

计划新增：

- `docs/project-context.json`
- `scripts/harness/project-context.js`
- `scripts/harness/project-context.test.js`

默认命令：

```bash
node scripts/harness/project-context.js --target .
```

完整诊断：

```bash
node scripts/harness/project-context.js --target . --diagnostic
```

### 5.2 默认输出只回答六件事

1. 当前规则入口。
2. 当前权威索引。
3. 当前决策 / 任务包 / 计划指针。
4. 当前正在做什么。
5. 唯一下一步。
6. 当前 blocker 和安全提醒。

默认输出预算：不超过 25 行、不超过 4 KB。`--json` 默认也只输出 route 数据；完整诊断 JSON 必须显式加 `--diagnostic --json`。

### 5.3 默认禁止的动作

- 不运行 Git 命令。
- 不统计 dirty 数量。
- 不读取 Hook 状态。
- 不扫描 Code Map 或源码。
- 不读取历史 task / evidence / handoff 全文。
- 不写文件、不修文档、不生成 task。

### 5.4 失败语义

- 工具是 read-only、fail-open 的导航器，不是完成门。
- `project-context.json` 缺失、格式错误或有断链时，输出 `DEGRADED` 和最小人工入口 `AGENTS.md / AUTHORITY.md / CURRENT.md`。
- `DEGRADED` 只表示导航来源需要修复，不能自动阻塞一个已有明确当前任务包的普通开发任务。

## 6. 权威文档与更新合同

### 6.1 唯一权威索引

继续使用 `AUTHORITY.md`，不创建第二份“设计文档注册表”或“当前权威地图”。整改后每个登记项至少表达：

- `Path`
- `Category`
- `Current use`
- `Supersedes`
- `Caution`

`AUTHORITY.md` 只存指针和一句用途，不复制计划正文、当前状态或证据结论。

### 6.2 文档职责

| 文档 | 应存内容 | 更新触发 | 禁止存放 | 预算 |
| --- | --- | --- | --- | --- |
| `AGENTS.md` | 长期规则、高危边界、最短工作合同 | 规则或安全边界改变 | 当前任务状态、历史流水 | 保持可一次读完 |
| `AUTHORITY.md` | 当前权威指针与 supersede 关系 | 权威入口变化 | 正文复制、任务流水 | 约 80 行 / 16 KB 内 |
| `docs/project-context.json` | 当前路由、下一步、blocker、安全提示 | 路由事实变化 | 全量历史、诊断结果 | 默认 route 可在 25 行输出 |
| `CURRENT.md` | 已核实能力、当前工作、下一步、锁定项 | 当前事实变化 | 长篇覆盖日志、全部历史 | 约 30 行 / 12 KB 内 |
| 当前业务总计划 | 包顺序和计划归位 | 阶段切换或当前串行线变化 | per-task 流水 | 一处唯一入口 |
| 任务包 | 本次授权、范围、复用、验收、停止条件 | 包冻结或正式修订 | 全项目历史 | 只覆盖本任务 |
| Code Map | canonical、入口、consumer、状态归属、合同、重复线索 | 能力边界改变 | 运行 payload、计划占位、真实数据 | partial seed 可诚实扩展 |
| decision | 已拍板且需长期复用的结论 | 决策改变 | 当前进度 | 一题一决策 |
| evidence | 某次实际验证的条件与结果 | 产生新验证 | 未来任务派发 | 证据不能授权 |
| handoff | 跨会话恢复所需的短事实与指针 | 确有跨会话交接 | 第二份 CURRENT | 用后停止追加 |
| archive | 已冻结历史 | 迁移时一次性写入 | 新状态 | 冻结不再维护 |

### 6.3 立即更新触发器

发生以下任一事实时，在同一工作收口中更新对应正本：

1. 当前权威、唯一执行包、blocker 或 next action 改变。
2. 能力 canonical、公共入口、关键 consumer、状态 owner、跨模块合同改变。
3. 当前业务总计划的串行顺序发生改变。
4. 某决策正式替代旧决策。
5. 跨会话继续工作所需的事实发生变化。

未发生上述变化时，不为了“保持活跃”制造文档 diff。

### 6.4 历史与当前分离

`CURRENT.md` 的历史压缩必须单独实施：

1. 冻结当前文件 hash 和已有脏改归属。
2. 把 2026-07-22 以前的覆盖流水原样移入一个只读 archive 快照。
3. 用四块短视图重写 CURRENT：能用 / 在做 / 下一步 / 锁着。
4. `AUTHORITY.md` 登记 archive 仅为历史，不参与默认路由。
5. 用链接和关键事实抽样证明没有丢失当前状态。

不得在普通业务任务中顺手压缩 CURRENT，也不得另建并列 CURRENT 来躲避冲突。

## 7. Code Map 合同

### 7.1 目标结构

计划新增：

```text
docs/code-map/
├── README.md
├── index.json
└── domains/
    ├── conversation-transport.json
    ├── syn-mcp-supervision.json
    ├── workflow-execution-governance.json
    ├── persistence-canonical-state.json
    ├── ui-shared-foundation.json
    └── development-harness.json
```

首版只覆盖最容易重复和最承重的六个领域。已有 `docs/2026-07-09-codebase-capability-map-v2.md` 作为 seed 调查输入；新结构核验完成后把 v2 标为 historical/superseded，不删除。

### 7.2 每条能力字段

- `id`
- `domain`
- `name`
- `status`: `active | candidate | legacy | dead | needs-confirmation`
- `coverage`: `seed-partial | verified-partial | verified`
- `canonical`
- `entrypoints`
- `publicSymbols`
- `consumers`
- `stateOwners`
- `contracts`
- `tests`
- `related`
- `knownDuplicates`
- `keywords`
- `verifiedAtCommit`

所有路径必须是 Git tracked 的仓库相对路径。图谱不得记录环境变量值、凭据、完整本机私有路径、真实用户数据或运行 payload。

### 7.3 命令合同

```bash
# 写代码前查现有与相邻能力
node scripts/harness/codebase-map.js query --target . --query "conversation transport"

# 看未提交源码相对已提交图谱的结构覆盖层
node scripts/harness/codebase-map.js overlay --target .

# 显式检查 schema、路径、ID、引用与 staged rename/delete
node scripts/harness/codebase-map.js check --target . --staged --strict
```

行为要求：

- `query` 同时搜索 id、名称、中英文关键词、公共符号和路径。
- 未命中必须输出 `NO_MATCH_IN_PARTIAL_MAP`，不能说“仓库没有该能力”。
- `overlay` 只报告 live/uncommitted 结构线索，不自动写回 committed map。
- `check` 对 schema 错误、重复 ID、悬空 tracked 路径、已登记路径被 staged 删除或重命名负责。
- `check` 无法机械判断“这个新 helper 是否业务重复”，只能给出 `REVIEW_BOUNDARY` 提示，不能伪装语义权威。
- 源码事实高于图谱；发现冲突先修图谱或降为 `needs-confirmation`，不能为过闸编造 canonical。

### 7.4 更新触发器

只有以下变化要求同步 Code Map：

- 新增或删除一项可复用能力。
- canonical 移动、拆分或合并。
- 公共入口、公共符号或关键 consumer 改变。
- 状态 owner 或跨模块合同改变。
- 已知重复被确认、消除或改为 legacy。

内部算法、局部样式、私有变量、测试数据更新不要求修改图谱。

### 7.5 existing-before-new 判定

重要任务开工前必须把候选能力分为：

1. **直接复用**：现有 canonical 满足需求，不新增第二套。
2. **参数化 / 抽取**：能力存在但当前入口耦合，需要共享化。
3. **确实缺失**：Code Map 未命中且源码检索也未发现，才允许新增。

查询结果只是起点；partial map 未命中后仍必须做一次目标范围源码搜索。

## 8. 计划对齐合同

### 8.1 哪些任务需要

只有以下“重要任务”需要完整对齐块：

- 新模块、服务、共享 helper、共享类型、公共 command 或新页面能力。
- 跨前后端、跨存储、跨工作流或跨多个承重模块。
- 修改 canonical、状态 owner、公共合同或安全边界。
- 命中 `AGENTS.md` 高危清单。
- 用户明确要求任务包或阶段执行计划。

小范围文案、局部样式、单点 bug 修复不强制新建任务包，但仍要运行相关真实验证。

### 8.2 重要任务必备字段

当前 `tasks/*.md` 任务包增加以下标准块：

```markdown
## Authority and plan alignment

- authority_chain:
- plan_anchor:
- existing_before_new:
- capabilities_touched:
- forbidden_alternatives:
```

字段含义：

- `authority_chain`：这次工作实际服从的决策、任务包、计划和规则。
- `plan_anchor`：总计划中的具体阶段 / 包，不接受只写“按总计划”。
- `existing_before_new`：查询词、命中 canonical、复用 / 参数化 / 缺失结论和源码复核范围。
- `capabilities_touched`：Code Map capability id；无则明确 `none` 并解释。
- `forbidden_alternatives`：这次不得滑回的旧路线或第二真源。

不启用旧 `docs/task-packages/*.json` 生命周期，不把 `scripts/harness/task-package-new.js` 重新变成默认入口。对齐检查只针对 `docs/project-context.json` 指向的当前重要 Markdown 任务包。

### 8.3 开始与结束动作

开始重要任务：

1. 运行短路由。
2. 读取当前任务包与 plan anchor。
3. 执行 Code Map query；未命中再做目标源码检索。
4. 写清复用判断和禁止路线。
5. 冻结 staged / dirty / 承重文件 hash 与写入白名单。

结束重要任务：

1. 运行任务相关直接验证。
2. 运行显式 checkpoint audit，复核是否偏离 plan anchor。
3. 判断能力边界是否变化；有变化才更新 Code Map。
4. 判断当前路由和 CURRENT 是否变化；有变化才更新。
5. 区分代码完成、离线验证、真实 App 验收和用户确认，不互相外推。

checkpoint audit 只能核字段、diff 与证据是否一致，不能把“字段齐全”判成业务语义正确。

## 9. 现有 Harness 的去留分类

### 9.1 四类 active boundary

Phase 5 已在 `harness.config.json` 与 example 落地一个简单 `activeBoundary`，只声明当前真实入口：

| 分类 | 含义 | 代表入口 |
| --- | --- | --- |
| `mechanical` | 退出码能判断的窄安全 / 结构检查 | `commit-msg catch:`、config schema/check/policy、shape |
| `reportingOnly` | 只提供信息，不自动阻塞 | `context`、`checkpoint` |
| `explicitTool` | 任务或阶段需要时人工调用 | `context diagnostic`、Code Map、Stage K、doctor |
| `legacyIgnored` | 保留兼容但不再默认展示或推荐 | task/evidence lifecycle、AgentMemory、旧 capability scan、runtime-doc init 等 |

### 9.2 CLI 收缩

默认 `harness.js --help` 现只展示以下 9 个当前入口：

- `context`
- `context diagnostic`
- `map query`
- `map overlay`
- `map check`
- `checkpoint`
- `shape`
- `stage-k`
- `doctor`

`maintenance-audit` 已作为 Phase 6 的显式、只读工具落地，但仍不进入默认 CLI；不得用已退役的 `memory-maintenance.js` 冒充。Legacy 命令先通过 `--legacy` 或直接脚本路径访问；完成消费者审计后再决定物理删除。

### 9.3 consumer-first 退出

每个 legacy 入口按以下顺序处理：

1. 全仓搜索脚本、配置、模板、文档和 Hook consumer。
2. 把当前 consumer 迁到新入口或明确标为历史。
3. 默认 help、config 推荐项和模板不再展示 legacy。
4. 保留一轮兼容期并给出 deprecation 信息。
5. 零 consumer 且有回滚点后，另包决定删除。

不得先批量删除 88 个脚本再让引用报错，也不得用一份“已退休列表”代替真实 consumer 审计。

### 9.4 承重工具处理

- `workbench-shape-gate.js`：保留，继续区分历史基线债与本次净增；不接管 Code Map 或计划判断。
- `stage-k-architecture-gate.js`：保留为专项显式工具，不进入默认 route。
- `checkpoint-audit.js`：改为重要任务收口时的对齐报告，不生成生命周期状态。
- `config-check / config-policy / config-schema`：保留并增加 active boundary 一致性检查。
- `docs/harness-catch-log.md` 与 `commit-msg catch:`：保持现状；无实际 catch 不制造新条目。
- `capability-map.js / capability-scan.js`：改名或降为 legacy environment capability 工具，避免与 Code Map 混淆。

## 10. 分阶段执行

所有阶段默认不 stage、不 commit。每阶段完成后先回交真实 diff 与验证结果，由用户决定是否进入下一阶段和是否提交。

### Phase 0：基线与长期设计冻结

**目标：** 先把边界、术语、成功标准和现有 consumer 固定，避免边改边重新定义 Harness。

**写入面：**

- 新增 `decisions/2026-07-23-development-harness-operating-model-v1.md`
- 必要时最小更新 `AGENTS.md`
- 必要时最小更新 `AUTHORITY.md`
- 本计划状态区

**动作：**

1. 重新冻结 HEAD、staged、porcelain、Harness 文件清单、Hook、CLI 和 config。
2. 给 95 个 Harness 文件生成只读 consumer / 分类报告，不立即删除。
3. 冻结开发 Harness 与产品 Harness 的术语。
4. 冻结本文各预算、退出语义和非目标。

**验收：**

- 决策文档与 `AGENTS.md` 不冲突。
- `AUTHORITY.md` 只增加指针，不复制本文。
- consumer 报告能解释每个默认入口为何保留。

**完成线：** 仅文档和基线冻结完成；脚本行为未改变。

### Phase 1：短路由落地

**目标：** 新会话先进入正确权威和当前任务，不扫描整仓。

**写入面：**

- `docs/project-context.json`
- `scripts/harness/project-context.js`
- `scripts/harness/project-context.test.js`
- `AUTHORITY.md`
- `AGENTS.md`

**先红测试：**

- route 输出超过 25 行或 4 KB 时失败。
- 默认路径调用 Git、Hook 或 Code Map 时失败。
- 缺文件、坏 JSON、断链时输出 `DEGRADED` 但不非零阻塞。
- `--diagnostic` 才允许输出 workspace / Hook / map 诊断。

**验收：**

- 冷启动命令在普通本机环境快速返回。
- 当前 route 指向 07-22 shared Conversation Transport 决策、任务包和 07-16 唯一业务计划。
- route 明确 resident/private-home 主运输为历史，不将其派成当前路线。

**完成线：** 新会话无需先读 56 KB CURRENT 即可找到当前动作。

### Phase 2：权威索引与 CURRENT 历史分离

**目标：** 让权威索引、当前事实、历史材料各负其责。

**写入面：**

- `AUTHORITY.md`
- `CURRENT.md`
- 一个冻结的 `archive/*current-before-short-view*.md`
- `docs/plans/README.md`
- `docs/project-context.json`

**动作：**

1. 在不丢当前事实的前提下，把 CURRENT 历史覆盖流水转存 archive。
2. 把 CURRENT 收成四块短视图和约 30 行 / 12 KB。
3. 修正 plans README 的过期当前计划和已停用文档要求。
4. 让 AUTHORITY 明确 current / long-term / historical / superseded。

**验收：**

- `AUTHORITY.md` 是唯一人工索引。
- 当前计划入口仍只有一份业务总计划；本整改计划标为开发治理并行计划，不抢业务排期。
- 任意从 CURRENT 移出的当前事实都有 archive 来源和保留理由。
- 旧文档不再被写成当前执行入口。

**停止条件：** CURRENT 现有脏改归属不明、历史迁移会覆盖并行业务记录，立即停止为 `BLOCKED_DIRTY_OVERLAP`。

### Phase 3：结构化 Code Map seed 与查询工具

**目标：** 先覆盖最容易重复的承重能力，并让查询诚实可用。

**写入面：**

- `docs/code-map/**`
- `scripts/harness/codebase-map.js`
- `scripts/harness/codebase-map.test.js`
- `docs/2026-07-09-codebase-capability-map-v2.md` 仅加历史指针

**seed 顺序：**

1. 读取 v1 / v2 旧图，不从零画。
2. 从 tracked 源码核对 public Rust faces、Tauri commands、TS/TSX exports、页面入口、状态 owner 和测试。
3. 先登记六个高重复领域，不追求全仓覆盖。
4. 不确定项保留 `needs-confirmation`，不硬判合并。

**先红测试：**

- duplicate id、schema 错误和悬空 tracked 路径失败。
- partial map 未命中必须返回诚实提示。
- untracked / unstaged 变化只出现在 overlay，不写入 committed map。
- staged rename/delete 能定位受影响 capability。
- query 支持中英文关键词、symbol 和路径。

**关键场景验收：** 查询“交办会话 / conversation transport / Stop / poll / readback”必须优先返回现有 Agent conversation 底座，并把 resident/private-home 标为已知重复或 legacy 路线，而不是建议再建第三套。

**完成线：** seed、query、overlay、check 和测试全部存在；仍明确标记 partial。

### Phase 4：重要任务的计划对齐

**目标：** 让“复用什么、按哪个计划、禁止滑向哪里”进入重要任务包。

**写入面：**

- `AGENTS.md`
- `scripts/harness/checkpoint-audit.js`
- 对应 self-test
- 当前任务包仅在仍为 current 且用户授权时最小补齐
- `docs/project-context.json`

**动作：**

1. 增加五个标准字段，不恢复旧 JSON task lifecycle。
2. checkpoint 只检查当前重要任务包，不扫全部历史任务。
3. 对缺字段先 warning；只有用户明确把某包冻结为执行入口时才 strict。
4. 输出源码 / plan / map 三者冲突，不自动选择更宽路线。

**验收：**

- 小改动不被迫建任务包。
- 重要包缺 `existing_before_new` 或 `forbidden_alternatives` 时能被发现。
- 字段齐全不被误报为语义正确或实现完成。
- 当前共享 transport 包能明确指向现有底座和禁止 resident/private-home 扩张。

### Phase 5：CLI / config / legacy 消费者收缩

**目标：** 让默认展示与当前规则一致，同时保持可回滚。

**写入面：**

- `harness.config.json`
- `scripts/harness/harness.js`
- `scripts/harness/config-*.js`
- 对应 self-test
- `docs/harness-catalog.md`
- consumer 审计确认需要的 legacy 文件

**动作：**

1. 增加 `activeBoundary` 四分类。
2. 默认 CLI 收到不超过 10 个真实入口。
3. preWork / preCompletion 的旧推荐列表改成显式、任务相关验证说明，不再默认跑 lifecycle 工具。
4. memory、task/evidence lifecycle、runtime-doc init 等进入 legacy help。
5. 先迁 consumer，再决定是否删除零 consumer 脚本。

**验收：**

- `AGENTS.md`、CLI help、config 和 catalog 对默认语义没有互相打架。
- Hook 仍只有当前 `commit-msg catch:` 线；Code Map 和文档检查没有进入 Hook。
- 历史证据里的旧命令仍可直接运行或得到清楚 deprecation，不静默变义。
- shape gate 与 Stage K 结果和退出码未被无关修改。

**高危边界：** 若需要改 `.githooks/**`、放宽安全检查退出语义或改变 sandbox / approval，必须停止并另走 `AGENTS.md` 高危 #3 明确授权。本计划默认不需要这些改动。

#### 已验收实施与 R1（2026-07-23）

- `PHASE5_IMPLEMENTATION_PASS / PHASE5_R1_PASS / PHASE5_STATUS_ALIGNED` 只表示本阶段定向实现与治理回写通过；不表示聚合自测全绿、业务完成或产品验收。
- 首次 Phase 5 实现复核发现 `config-policy` 未把显式 `gates.hard` 与 non-mechanical boundary 交叉核验；修正后，`reportingOnly` / `explicitTool` 被写入 hard gate 会由 `config-check` 和 `config-policy` 以 `ACTIVE_BOUNDARY_NON_MECHANICAL_HARD_GATE` 拒绝。
- R1 首次复核又发现默认九项中的 `context diagnostic` 未进入两份配置的 `activeBoundary.explicitTool`，catalog 也未同步；R1 补齐三处声明，并以 9/9 定向测试锁定“默认九项全部且仅声明一次”、最长匹配与 hard-gate 拒绝。
- 当前默认 help 恰为 9 项，`--legacy` 隐藏 34 项；原 35 条路由仍保留直调参数与退出码。项目与 example 的 schema strict、config-policy strict、config-check strict 均退出 0；`autoRisk`、`verificationRunner`、`taskLifecycle` 仅作为兼容性可选字段。
- 未绿事实单列：聚合 `self-test.js` 仍 exit 1（177 pass / 9 fail）；shape baseline/check 仍为 exit 0 / 1、`16 error / 5 warning / 5 info`；Stage K 仍 exit 0、`0 error / 15 warning / 36 info`。JSON 捕获问题与其范围见 Phase 0 审计第 11 节，不能伪报为全绿。

### Phase 6：周期性维护审计

**目标：** 定期发现漂移，但不定期制造文档噪音。

**状态：** completed（`PHASE6_IMPLEMENTATION_PASS / PHASE6_R1_PASS / PHASE6_STATUS_ALIGNED`）；Phase 7 不因本阶段自动开启。

**写入面：**

- `scripts/harness/maintenance-audit.js`
- `scripts/harness/maintenance-audit.test.js`
- `AGENTS.md`
- `docs/code-map/README.md`

**只读检查：**

- 权威索引断链、重复 current、superseded 仍被当前 route 引用。
- project-context 行数 / 字节预算与指针有效性。
- CURRENT 行数 / 字节预算。
- Code Map schema、tracked 路径和 stale verified commit 提示。
- 默认 CLI / config / catalog active boundary 漂移。
- legacy consumer 是否归零。

**节奏：**

- 有活跃开发时每周一次，或每个业务阶段收口一次，取先到者。
- 同一周没有业务改动，不强制运行。
- 审计只报告；只有真实事实变化才修改文档或 Code Map。
- 阶段 evidence 已包含审计结果时，不另建重复报告文件。

**验收：** dirty 工作区存在大量无关改动时，审计仍能按目标文件和 staged snapshot 给出结果，不以 dirty 数量失败。

#### 已验收实施与 R1（2026-07-23）

- `PHASE6_IMPLEMENTATION_PASS / PHASE6_R1_PASS / PHASE6_STATUS_ALIGNED` 只表示本阶段定向实现与治理回写通过；maintenance audit 保持显式、只读、有界，不接 Hook、CI、cron 或默认 CLI，也不自动回写文档。
- 首次复核发现两处假绿：staged canonical rename/delete 没有报告受影响 capability；默认 CLI help 为空或不可解析时被当成零项而非 drift。R1 增加 `STAGED_RENAME_AFFECTS_CAPABILITY` / `STAGED_DELETE_AFFECTS_CAPABILITY` 与 capability ID，并让两类 help 均以 `DEFAULT_CLI_BOUNDARY_DRIFT`、exit 1 失败。
- `node --test scripts/harness/maintenance-audit.test.js`：R1 为 8/8；巨大 dirty / 超长 untracked fixture 仍断言 JSON 小于 64 KiB 且不泄漏路径。当前 `maintenance-audit --target . --json` 六项均 PASS，staged canonical impact 为 0。
- 未绿事实单列：shape baseline/check 仍为 exit 0 / 1、`17 error / 5 warning / 5 info`；Stage K 仍 exit 0、`0 error / 15 warning / 36 info`。这些数字不被包装成全绿、业务完成或产品验收。

### Phase 7：回放、观察期与收口

**目标：** 证明整改实际减少了错误路线，而不只是增加新脚本。

**至少回放六类任务：**

1. 交办会话 transport 复用。
2. Syn MCP / supervisor capability。
3. workflow / dispatch / authorization。
4. DB-primary / canonical / JSON 兼容投影。
5. UI shared primitives / display boundary。
6. 开发 Harness 与产品 Harness 术语区分。

**观察期：** 至少跨两个真实业务任务或一个完整阶段。记录：首次正确动作耗时、Code Map 命中与误导、发现的重复候选、发生的计划偏航、误阻塞次数。

**收口条件：**

- 所有关键验收场景通过。
- active boundary 与实际 consumer 一致。
- v2 静态图谱和 legacy 命令的历史状态已清楚。
- 没有新增默认 Hook、生命周期或第二权威索引。
- 观察数据支持保留；无发现价值的检查进入删减候选。

## 11. 验收矩阵

| 编号 | 场景 | 通过标准 |
| --- | --- | --- |
| A1 | 新对话恢复 | 默认 route ≤25 行、≤4 KB，一分钟内找到当前任务与下一步 |
| A2 | 当前路线 | route 指向 shared transport 决策 / 包 / 业务总计划，不派发 resident 历史线 |
| A3 | 防重复 | conversation 查询命中 existing/new、event mapping、poll、Stop、readback canonical |
| A4 | partial 诚实 | map 未命中写 `NO_MATCH_IN_PARTIAL_MAP`，不声称仓库不存在 |
| A5 | staged rename | 删除或移动已登记 canonical 时 `check --staged --strict` 精确指出受影响 capability |
| A6 | dirty 隔离 | 大量无关 dirty 不让 route、map query 或目标 check 失败 |
| A7 | 计划对齐 | 重要任务明确 plan anchor、复用对象、capability id 和禁止路线 |
| A8 | 文档权威 | AUTHORITY 只有一个，CURRENT / route / plan 不复制同一正文 |
| A9 | 文档预算 | CURRENT 约 30 行 / 12 KB，route 输出 ≤25 行 / 4 KB |
| A10 | 默认语义 | AGENTS、CLI、config、catalog 对 active / legacy 说法一致 |
| A11 | Hook 边界 | 未新增 Code Map / 文档 Hook；commit-msg catch 线保持 |
| A12 | 真实验证 | 业务完成仍由任务相关测试、build / check、UI / live 证据决定，Harness 状态不代替 |

## 12. 验证方案

每个阶段至少执行：

1. 新增脚本的 `node --test` 定向测试。
2. `node scripts/harness/config-schema.js --target . --strict --json`。
3. `node scripts/harness/config-policy.js --target . --strict --json`。
4. `node scripts/harness/config-check.js --target . --strict --json`。
5. `git diff --check`。
6. 仓根 `node scripts/harness/workbench-shape-gate.js --mode baseline` 与 `--mode check`，历史债单列，只接受整改零净增。
7. CLI help、route、query、overlay、check 的 snapshot / fixture 回归。
8. 复核 `git diff --cached --name-only`；未经用户授权必须为空。

Phase 5 还必须跑 legacy consumer 清单前后对比；Phase 7 必须完成六类任务回放。文档阶段不需要启动 Tauri App、Codex CLI、MCP server 或真实 store。

## 13. 成功指标

不看“新增多少 gate”，看以下结果：

- 新会话到第一次正确开发动作的时间。
- 新能力开工前 Code Map 命中或诚实未命中的比例。
- 因先查 existing capability 而取消第二套实现的次数。
- 重要任务中途偏离 plan anchor 的次数。
- 无关 dirty 导致误阻塞的次数。
- 当前文档断链、重复正文和过期指针数量。
- 每个普通改动产生的治理文档数量。
- 每条保留检查是否发现了新的证据维度。

建议目标：

- 冷启动小于一分钟。
- route 始终满足 25 行 / 4 KB。
- 六个高重复领域查询场景全部能导航到正确 canonical 或诚实 unknown。
- 普通低危小改不新建 task/evidence/handoff。
- Code Map 和文档维护不进入默认 Hook。
- 观察期内无新增“另建第二套 transport / state owner / canonical”而未被开始前发现的事故。

## 14. 风险、停止条件与回滚

### 14.1 主要风险

1. **把轻量整改做成第二套重型治理。** 对策：默认 route 有硬预算，工具按需，普通小改不建任务包。
2. **Code Map 过期反而误导。** 对策：partial 状态、源码优先、verified commit、诚实 unknown、周期只读审计。
3. **机械检查假装能判断架构语义。** 对策：check 只管 schema / path / staged 结构，复用判断保留源码证据。
4. **CURRENT 迁移覆盖正在进行的业务记录。** 对策：Phase 2 单独冻结 hash 和归属，冲突即停。
5. **Legacy 一次性删除破坏历史消费者。** 对策：consumer-first、兼容期、零 consumer 后另包删除。
6. **开发 Harness 与产品 Harness 再次混名。** 对策：文档、CLI 和 capability id 使用 dev-gate 前缀或完整中文名。
7. **维护节奏变成空更新。** 对策：周期运行只读，事实没变就零文档 diff。

### 14.2 全局停止条件

- `BLOCKED_DIRTY_OVERLAP`：承重文档或脚本出现无法归属的并行 hunk。
- `BLOCKED_AUTHORITY_CONFLICT`：出现两个 current plan / task / authority 入口且无法从用户指令消解。
- `BLOCKED_MAP_OVERCLAIM`：无法从 tracked 源码证明 canonical，却被要求登记为 active。
- `BLOCKED_FALSE_GATE`：新检查必须扫描全量 dirty 才能工作，或会阻塞无关 staged scope。
- `BLOCKED_SCOPE_EXPANSION`：整改需要修改业务源码、真实 store、sandbox、approval 或产品 Harness。
- `BLOCKED_LEGACY_CONSUMER`：准备删除的入口仍有当前 consumer。

停止时只报告最早 blocker 和最小下一步，不通过放宽规则换绿。

### 14.3 回滚原则

- 每个 Phase 单独可回滚，不跨阶段混合提交。
- 新 route / map 工具未稳定前保持旧直接脚本入口可用。
- CLI 先隐藏再删除；config 先兼容读旧字段再清理。
- Code Map seed 出错时可降级为 `needs-confirmation` 或回到历史 v2 指针，不影响源码运行。
- 文档迁移保留 archive 原文，CURRENT 可从 archive 和当前事实重建。

## 15. 执行状态表

| Phase | 状态 | 完成证据 | 下一门 |
| --- | --- | --- | --- |
| 0 基线与设计冻结 | completed（既有 config schema 漂移已登记） | `decisions/2026-07-23-development-harness-operating-model-v1.md`；`docs/2026-07-23-development-harness-phase0-baseline-and-consumer-audit-v1.md`；policy/check strict 通过，schema strict 因 legacy 字段缺失失败 | Phase 1 已完成 |
| 1 短路由 | completed（只读、fail-open、人工显式入口） | `docs/project-context.json`；`scripts/harness/project-context.js`；`scripts/harness/project-context.test.js`；Phase 0/1 审计第 7 节；5/5 定向测试通过 | Phase 2 已完成 |
| 2 文档当前 / 历史分离 | completed（合并授权后的稳定 CURRENT 基线已冻结） | `CURRENT.md`；`AUTHORITY.md`；`docs/plans/README.md`；`archive/2026-07-23-current-before-short-view-v1.md`；Phase 0/1/2 审计第 8 节 | Phase 3 已完成 |
| 3 结构化 Code Map | completed（六域 partial seed、真实 public symbol 校验、legacy / active 路由纠偏） | `docs/code-map/**`；`scripts/harness/codebase-map.js`；`scripts/harness/codebase-map.test.js`；旧 v2 历史指针；审计第 9 节；9/9 定向测试与十个关键查询通过 | Phase 4 已完成 |
| 4 计划对齐 | completed（显式 current important task；无绑定时 fail-open；不扫描历史包） | `AGENTS.md`；`docs/project-context.json`；`scripts/harness/checkpoint-audit.js`；`scripts/harness/checkpoint-audit.selftest.js`；审计第 10 节；45/45 self-test 通过 | Phase 5 已完成（见下一行） |
| 5 CLI / config / legacy 收缩 | completed（`PHASE5_IMPLEMENTATION_PASS / PHASE5_R1_PASS / PHASE5_STATUS_ALIGNED`） | `harness.config*.json`、`harness.js`、`config-{schema,check,policy}.js`、`harness-phase5.test.js`、catalog；R1 9/9，三项 config strict 通过，默认 9 项 / 隐藏兼容 34 项 | Phase 6 已完成（见下一行） |
| 6 周期维护审计 | completed（`PHASE6_IMPLEMENTATION_PASS / PHASE6_R1_PASS / PHASE6_STATUS_ALIGNED`） | `maintenance-audit.js` / test；R1 8/8、当前六项 PASS；两处假绿已修复，仍为显式只读工具 | Phase 7 等待用户单独派发 |
| 7 回放与收口 | pending（等待用户单独派发） | 未开始 | 用户单独派发并重新冻结目标边界 |

计划状态不是实施证据。每次状态更新必须附实际文件、命令和结果；没有验证就写“已实现，未验证”。

## 16. 后续唯一开工方式

Phase 0～6、Phase 5-R1 与 Phase 6-R1 已完成。当前不会自动进入 Phase 7：回放、观察期与收口必须由用户单独派发，并在开工前重新冻结目标边界与 dirty overlap；本次状态对齐不生成 Phase 7 开工授权。

不要把“Phase 5 / R1 或 Phase 6 / R1 完成”解释为 Phase 7 已获得实施授权，也不要把短路由的 `READY`、Code Map 命中、checkpoint 的 `FIELDS_PRESENT` 或 Harness 阶段状态当成业务完成或产品验收。
