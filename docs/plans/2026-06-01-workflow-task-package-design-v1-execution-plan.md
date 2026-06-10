# Workflow Task Package Design v1 Execution Plan

> **For agents:** Required skill when executing this plan: `executing-plans`.

**Goal:** 把 `docs/workflow-task-package-design-v1.md` 的草案目标落成可操作的工作台能力：项目主管通过工作流组织节点、生成并派发任务包、接收汇报、审查、处理异常、记录账本，并让用户看到进度、风险、待确认和最终汇报。

**Architecture:** 先修当前原型的真实短板，再逐层扩展工作流事实层。前端以当前 Tauri + React + Vite 原型为承载，先把已写好的完整项目工作流界面接到主路径；后端以当前 `workflow-state.v0.json` 为过渡事实层，新增兼容读模型和命令，再决定是否迁移到 SQLite。画布先接草案要求的节点详情、运行检查、账本和任务包，不提前做通用节点执行器。

   **Tech Stack:** Tauri 2、Rust、React、TypeScript、Vite、React Flow、JSON workflow state v0/v1 过渡层，长期 SQLite + FTS 后置。

   **Path:** Strict

   **Read Scope:** `/Users/yoyi/workspace/product-line/CURRENT.md`、`/Users/yoyi/workspace/product-line/tasks/README.md`、`/Users/yoyi/workspace/product-line/decisions/**`、`/Users/yoyi/workspace/product-line/docs/workflow-task-package-design-v1.md`、`/Users/yoyi/workspace/product-line/docs/memory-layer-design-v1.md`、`/Users/yoyi/workspace/product-line/docs/tooling-and-mcp-registry.md`、`/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/**`，不读 `/Users/yoyi/.codex`、auth、token、`.env`、完整 transcript。

   **Write Scope:** `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/**`、`/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/**`、`/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/**`、`/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/scripts/**`、`/Users/yoyi/workspace/product-line/docs/plans/**`、必要的 `evidence/**` 与 `handoffs/**`。执行真实 `codex exec`、写 `/Users/yoyi/.codex`、写真实业务项目目录，必须另行获得用户明确批准。

---

## 依据

- 草案依据：`/Users/yoyi/workspace/product-line/docs/workflow-task-package-design-v1.md`。
- 当前主线依据：`/Users/yoyi/workspace/product-line/CURRENT.md` 明确主线是 Codex 会话管理 + Codex 工作流编排，不是任务包管理器。
- 当前任务队列依据：`/Users/yoyi/workspace/product-line/tasks/README.md` 明确任务包只作为内部协议、审计、导出和交接物。
- 当前画布策略依据：`/Users/yoyi/workspace/product-line/decisions/2026-05-31-editable-canvas-codex-as-director-v1.md` 已把 ComfyUI、n8n、Langflow、React Flow、Storybook 设为后置研究对象，不作为当前 v1 产品路线。
- 当前 app 进度依据：`ProjectsView.tsx` 已有 `ProjectDetail` 完整项目工作台组件，但当前 `ProjectsView` 主路径仍渲染简化的 `ProjectStage`。
- 当前测试风险依据：`npm run test:offline-interaction` 当前会触发 React hook invalid call；`cargo test` 当前有 1 个测试因读到真实会话数失败。

## 已知问题先列出来

- 项目页主路径没有接入完整项目工作流界面。
- 前端离线测试直接调用 hook 组件，测试方式不成立。
- Rust `snapshot_keeps_metadata_without_session_body` 没隔离真实 Codex sqlite / 本机会话，断言不可靠。
- 当前 schema 还偏 `workflow_state_v0` 事实账本，没有完整覆盖草案里的 `WorkflowRunCheck`、`WorkflowLedgerEntry`、`SubagentReport`、`ReviewResult`、`WorkflowException`。
- 当前画布 `CanvasView` 是独立 canvas / run 文件层，尚未与项目 workflow state 明确合一或分工。
- 默认画布为空，尚未提供四角色车间模板。
- harness、知识库、记忆层、工具注册中心、模型池还没有稳定接口契约。

## 不做

- 不把产品改成任务包管理器。
- 不让任务包成为产品主界面。
- 不让子智能体直接找用户决策。
- 不让子智能体自己标记完成。
- 不把工具调用全文写进工作流账本。
- 不把 harness 当普通画布节点。
- 不做 ComfyUI 式插件节点生态。
- 不做通用自动化平台或任意节点执行器。
- 不在本计划默认执行真实 `codex exec resume`。

## 画布研究闸门

当前不需要在开始前深入研究 ComfyUI。理由：本计划第一阶段阻塞是已写 UI 未接主路径和测试不可靠，不是外部画布参考不足。

必须先深入研究画布并同步修改本计划的触发条件：

- 开始实现复杂画布编辑器、节点模板库、运行队列、历史记录、节点参数面板大改。
- 准备把 canvas / run 文件层和项目 workflow state 合一。
- 准备引入类似 ComfyUI 的模板、节点类型系统、执行历史或队列模型。

研究输出要求：

- 新增 `docs/plans/YYYY-MM-DD-canvas-reference-research.md` 或对应 decision。
- 明确 ComfyUI、n8n、Langflow、React Flow、Storybook 分别借鉴什么、不借鉴什么。
- 把研究结论转译为工作台自己的 workflow state、权限确认、audit、outbox、节点详情，不得直接照搬外部工具模型。
- 回写本执行计划，标注新增任务、调整顺序和新增验证。

## 阶段总览

1. 先稳住当前 app：接入完整项目工作流界面、修前端离线测试、修 Rust 测试隔离。
2. 建立草案对象到当前状态文件的兼容读模型。
3. 补 `WorkflowRunCheck`，让运行前检查成为 UI 和后端都能用的对象。
4. 强化任务包生成算法，落实权限白名单、模型、工具、知识库、记忆引用和 harness 要求。
5. 补工作流账本、子智能体汇报、审查结果、异常通知。
6. 补界面：方案视图、运行状态视图、节点详情、任务包预览、账本、审查、异常。
7. 明确和建议方案、记忆层、知识库、审计、harness、工具注册中心的接口。
8. 做端到端验收，不把代码路径说成真实业务自动编排。

## Task 4-12 开发前确认闸门

`Task 0-3` 可以先执行。`Task 4-12` 已经开始写产品规则和跨模块接口，不能让执行者按自己的理解直接实现。

执行 `Task 4-12` 前，必须先完成本节确认。确认结果可以写入本计划、`docs/open-questions.md`、新的 decision，或用户在当前对话中明确确认。没有确认的项只能做只读调研、列风险、写问题，不能写入后端状态机、存储结构、任务包生成规则或画布运行逻辑。

当前已确认范围：

- `Task 4` 到 `Task 12` 均采用本节对应确认项的已确认策略开发。
- `Task 7-12` 已确认采用保守闭环策略；不得把子智能体、审查智能体、harness、画布或知识库扩展成未确认的新产品能力。
- `Task 4-6` 开发完成后，执行者必须给用户一份手动测试清单，说明在应用里点哪里、看什么、哪些结果算通过、哪些结果算失败。
- `Task 7-12` 开发完成后，也必须给用户一份手动测试清单，说明通知中心、待办中心、运行中工作流、工作流画布、任务包、会话页分别怎么验证。

每个确认项必须记录：

- 结论：选了什么。
- 依据：来自哪份文档、哪段用户确认、哪段代码现状。
- 影响：会影响哪些 Task 和哪些文件。
- 未定：还不能判断的地方，不得补编。

### 确认 1: 状态事实源和迁移边界

影响：`Task 4`、`Task 7`、`Task 8`、`Task 11`。

确认状态：已确认用于 `Task 4`。对 `Task 7`、`Task 8`、`Task 11` 的影响已由确认 4、确认 5、确认 6、确认 9 补齐。

需要确认：

- 当前阶段是否继续以 `workflow-state.v0.json` 作为事实源。
- 是否只做兼容读模型，不做 SQLite 迁移。
- 是否允许新增 v1 派生字段；如果允许，字段写在哪里。
- v0 缺字段时是返回 `null`、空数组、warning，还是 blocked。
- 真实本机 workflow state 是否允许写入；默认不允许。

已确认策略：

- 先做 v0 兼容读模型。
- 不迁移数据库。
- 不写真实用户 workflow state，只用 fixture 和测试状态文件。
- 缺业务事实只返回缺失，不自动生成。
- 新的 `Workflow`、`WorkflowNode`、`WorkflowRunCheck`、`TaskPackage` 等对象先作为派生读模型，不直接写回真实状态文件。
- v0 缺字段时返回 `null`、空数组、warning 或 missing，不补编业务内容。

强阻塞点：

- 如果要迁移 SQLite，必须另开迁移计划，不能在本计划里顺手做。

### 确认 2: 运行前检查规则

影响：`Task 5`、`Task 6`、`Task 8`、`Task 11`。

确认状态：已确认用于 `Task 5` 和 `Task 6`。对 `Task 8`、`Task 11` 的影响已由确认 6 和确认 9 补齐。

需要确认每类问题是 blocked、warning，还是可运行：

- 没有项目主管。
- 没有绑定会话。
- 没有模型。
- 没有读范围。
- 没有写范围。
- 没有工具白名单。
- 没有验收标准。
- harness 必填但没有配置。
- 有未解决方向风险。
- 有权限冲突。
- 有知识库引用但知识库不可用。
- 有记忆引用但记忆未确认。

已确认策略：

- 缺写范围、缺验收标准、权限冲突、未解决方向风险：blocked。
- 缺模型：blocked，不自动选择模型。
- 缺工具白名单：如果节点需要工具则 blocked；如果节点不需要工具则 warning 或 empty。
- 缺 harness：如果节点要求 harness 则 blocked；如果节点不要求 harness 则 warning 或 empty。
- 缺知识库或记忆引用：不自动补；如果任务包声明需要则 blocked，否则为空。
- 没有工作流：blocked。
- 没有项目主管：blocked。
- 要派发的节点没有绑定会话：blocked。
- 没有读范围：blocked。
- 会写文件但没有写范围：blocked。
- 没有验收标准：blocked。
- 有未解决方向风险：blocked，并进入等待决策。
- 记忆引用未确认：如果作为任务依据使用则 blocked；不作为依据则为空。
- 运行前检查只阻止运行、派发和标记准备完成，不阻止用户查看或编辑草稿。

强阻塞点：

- 如果 blocked / warning 表没有确认，不得实现 `inspect_workflow_run_check` 的最终规则，只能先写草稿 helper 或测试草案。

### 确认 3: 任务包版本、编辑和失效规则

影响：`Task 6`、`Task 7`、`Task 8`、`Task 11`。

确认状态：已确认用于 `Task 6`。对 `Task 7`、`Task 8`、`Task 11` 的影响已由确认 4、确认 5、确认 6、确认 9 补齐。

需要确认：

- 任务包是否有版本号。
- 用户或项目主管是否可以手工编辑任务包。
- 手工编辑后是否必须重新运行检查。
- 工作流节点、权限、模型、知识库、记忆引用变化后，旧任务包是否自动失效。
- 任务包是否可以导出，导出内容是否包含内部审计引用。
- "没写允许就是不允许" 是否作为全局硬规则写入代码和文档。

已确认策略：

- 任务包必须有版本号。
- 允许人工编辑草稿，但编辑后必须重新检查。
- 节点目标、范围、模型、工具、知识库、记忆、harness、验收标准任一变化，都让旧任务包变为 stale。
- "没写允许就是不允许" 作为硬规则。
- 任务包可以导出，但导出内容默认只包含执行所需内容、摘要和引用，不包含内部审计全文。
- 派发前必须检查任务包不是 stale，且所有必需字段完整。
- 系统不得自动补模型、权限、知识库、记忆或业务事实。

强阻塞点：

- 如果不确认失效规则，不能做派发按钮和后端 dispatch readiness。

### 确认 4: 账本、审计和工具输出保留边界

影响：`Task 7`、`Task 9`、`Task 11`。

确认状态：已确认用于 `Task 7`、`Task 9`、`Task 11`。

需要确认：

- 哪些事件必须进入 workflow ledger。
- 哪些事件只保留 audit refs。
- 工具调用是否只存摘要、风险标记和引用。
- tool output 全文是否永远不进 workflow ledger。
- ledger entry 是否允许用户编辑；如果允许，是否保留修订记录。
- evidence、handoff、audit、ledger 的关系怎么显示。

已确认策略：

- ledger 记录事件摘要、来源引用、风险标记、审查结果和用户决定。
- 工具调用全文不进 ledger。
- ledger entry 不直接编辑，只追加更正事件。
- 工作流账本只追加，不覆盖、不改旧记录。
- evidence、handoff、audit、ledger 在 UI 中以引用关系显示，不把正文全部铺进工作流画布。
- 异常、待处理确认、运行中状态进入右侧通知中心、待办中心、运行中工作流入口，不堆在工作流画布主界面。

强阻塞点：

- 如果工具输出保留边界未确认，不能实现账本写入。

### 确认 5: 子智能体汇报和审查权力边界

影响：`Task 7`、`Task 8`、`Task 11`。

确认状态：已确认用于 `Task 7`、`Task 8`、`Task 11`。

需要确认：

- 子智能体汇报必须包含哪些字段。
- 子智能体能不能请求权限；如果能，请求到谁。
- 子智能体能不能把节点标为完成；默认不能。
- 审查智能体通过后，节点是否仍需项目主管确认；默认需要。
- 审查退回几次后变成异常。
- 方向风险出现时，是暂停节点、暂停整个工作流，还是只提示项目主管。

已确认策略：

- 子智能体只能提交汇报、风险和权限请求。
- 审查只给结果，不拥有最终完成权。
- 项目主管是节点完成的唯一确认者。
- 方向风险进入 `waiting_decision`，不自动继续。
- 子智能体汇报必须包含：做了什么、改了什么、证据引用、未解决问题、权限请求、方向风险、后续建议、验收状态。
- 审查智能体通过后，节点仍需项目主管确认。
- 审查退回达到阈值时生成异常；阈值可先用配置或常量，不能让审查智能体自行结束节点。

强阻塞点：

- 如果权力边界没确认，不能实现状态机中的完成判定。

### 确认 6: 状态机最终规则

影响：`Task 8`、`Task 11`。

确认状态：已确认用于 `Task 8`、`Task 11`。

需要确认：

- workflow 状态流转表是否接受。
- node 状态流转表是否接受。
- `failed` 后是否允许重试；如果允许，走 retry、reopen，还是复制新节点。
- `returned` 后是否回到原智能体继续，还是回到项目主管重新派发。
- `waiting_decision` 是否必须用户或项目主管明确确认。
- `passed` 是否只表示审查通过，不表示任务完成。

已确认策略：

- `waiting_decision` 必须人工确认。
- `passed` 不等于 completed。
- `failed` 不能直接回到 running，必须有显式 retry/reopen 事件。
- `returned` 回到项目主管决定重派还是继续原会话。
- 采用本计划 `Task 8` 中列出的 workflow 和 node 状态流转表。
- 节点完成必须满足：目标完成、验收标准满足、有 evidence refs、需要 review 或 harness 时已通过、无未解决风险、记忆候选是否处理或跳过已记录。
- 子智能体和审查智能体都不能直接把节点改成完成。

强阻塞点：

- 状态机一旦进后端测试，就会变成产品规则；没确认前不能写死。

### 确认 7: 记忆层、知识库、工具、模型、harness 接口边界

影响：`Task 6`、`Task 9`、`Task 10`、`Task 11`。

确认状态：已确认用于 `Task 9`、`Task 10`、`Task 11`。对 `Task 6` 的影响沿用确认 2 和确认 3。

需要确认：

- 记忆层只读确认记忆，还是允许生成记忆候选。
- 记忆候选能不能自动写入正式记忆；默认不能。
- 知识库当前只是显式资料引用，还是要接近 Obsidian 原生能力。
- 工具能力来自工具注册中心，还是先写静态白名单。
- 模型池是否允许自动选择模型；默认不自动选。
- harness 是运行检查和完成闸门，还是可以成为普通节点；默认不是普通节点。

已确认策略：

- 任务包可以引用确认记忆和显式知识资料。
- 工作流可以生成记忆候选，但不能自动写正式记忆。
- 知识库先只做显式引用，不做自动扫描。
- 工具和模型都必须白名单声明。
- harness 只影响检查、任务包模板和完成判定。
- 模型必须显式指定，不自动选择。
- 工具必须来自工具注册中心或静态白名单；没有白名单就不可调用。
- Obsidian 原生能力后面单独研究；没研究前不把知识库写成正式系统能力。

强阻塞点：

- 如果知识库要内置 Obsidian 原生功能，必须先补知识库专项设计，再改 `Task 9-10`。

### 确认 8: 画布职责

影响：`Task 10`、`Task 11`。

确认状态：已确认用于 `Task 10`、`Task 11`。

需要确认：

- 画布只是工作流的可视化和编辑入口，还是也是执行队列。
- canvas / run 文件层是否和 workflow state 合一。
- 是否需要节点模板库。
- 是否需要运行历史。
- 是否需要节点参数面板。
- 是否需要借鉴 ComfyUI 式队列和执行记录。

已确认策略：

- 当前先把画布作为工作流可视化、节点详情和状态查看入口。
- 不做通用节点执行器。
- 不做 ComfyUI 式插件节点生态。
- 不合并 canvas / run 文件层和 workflow state，除非先完成画布研究。
- 工作流页只显示工作流画布。
- 节点详情可以右侧滑出或在画布内展开，不在画布外堆面板。
- 项目内会话、任务包、交接、技能、验证、设置都走各自标签页。
- 当前 UI 方向是：项目列表 + 工作流画布；项目页像内嵌一个 Codex 子界面，有统一边框，但内部不做多层卡片堆叠。

强阻塞点：

- 一旦要做模板库、执行队列、运行历史、复杂参数面板，必须先执行 "画布研究闸门" 并回写本计划。

### 确认 9: 端到端验收范围

影响：`Task 11`、`Task 12`。

确认状态：已确认用于 `Task 11`、`Task 12`。

需要确认：

- 验收是否只验证代码路径、测试 fixture 和 UI 展示。
- 是否要执行真实 `codex exec resume`。
- 是否允许写 `/Users/yoyi/.codex` 或真实 Codex 状态数据库。
- 是否允许写真实业务项目目录。
- 是否需要 Tauri 窗口截图作为 UI 完成证据。

已确认策略：

- 不执行真实 `codex exec resume`。
- 不写 `/Users/yoyi/.codex`。
- 不写真实业务项目目录。
- UI 完成声明必须有 Tauri 或浏览器截图证据。
- 默认只验证代码路径、测试 fixture 和 UI 展示。
- 开发完成后必须给用户手动测试清单，由用户按步骤点应用验证。
- 真实 Codex 执行、真实项目写入、真实 workflow state 写入都必须另行获得用户明确批准。

强阻塞点：

- 如果要做真实 Codex 执行，必须另行获得用户明确批准，并新增安全计划。

### 确认 10: 当前权威文档更新规则

影响：`Task 12`。

确认状态：已确认用于 `Task 12`。

需要确认：

- 哪些实现完成后要更新 `CURRENT.md`。
- 哪些实现完成后要更新 `tasks/README.md`。
- 草案是否仍保留为草案，还是升为当前权威。
- 哪些内容需要新增 decision。
- 哪些内容只放 evidence / handoff。

已确认策略：

- 只有经过测试和 evidence 支撑的内容才能写入 `CURRENT.md` 的已完成状态。
- `docs/workflow-task-package-design-v1.md` 仍是草案，除非用户明确升格。
- 产品规则变化写 decision，不藏在代码里。
- `tasks/README.md` 只更新当前任务状态、限制和下一步。
- 具体测试结果写入 evidence 和 handoff。
- 不把代码路径、fixture 验证或普通浏览器壳验证说成真实 Tauri 数据页完整验证。

强阻塞点：

- 如果当前权威入口没确认，不能在 `Task 12` 里宣称功能已进入最终形态。

---

### Task 0: 执行前基线确认

**Files:**

- Read: `/Users/yoyi/workspace/product-line/CURRENT.md`
- Read: `/Users/yoyi/workspace/product-line/tasks/README.md`
- Read: `/Users/yoyi/workspace/product-line/docs/workflow-task-package-design-v1.md`
- Read: `/Users/yoyi/workspace/product-line/decisions/2026-05-31-editable-canvas-codex-as-director-v1.md`

**Step 1: 确认当前工作树和测试基线**

Run:

```bash
npm run build
```

Working directory:

```text
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
```

Expected: PASS。

Run:

```bash
npm run test:offline-interaction
```

Expected: 当前可能 FAIL，记录错误文本。

Run:

```bash
cargo test --lib
```

Working directory:

```text
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri
```

Expected: 当前可能 1 FAIL，记录失败测试名。

**Step 2: 记录 evidence**

- Create: `/Users/yoyi/workspace/product-line/evidence/YYYY-MM-DD-workflow-task-package-plan-baseline.md`
- 必须记录 build、offline interaction test、cargo test 的结果。

---

### Task 1: 修前端离线测试，避免直接调用 hook 组件

**Files:**

- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` if needed
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx` only if pure selectors/action builders must be exported
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/AgentView.tsx` only if pure selectors/action builders must be exported

**Step 1: 写失败用例或标注现有失败**

Run:

```bash
npm run test:offline-interaction
```

Expected before fix: FAIL with invalid hook call.

**Step 2: 改测试策略**

Do:

- 不再用自制 `visibleText` 直接调用 hook 组件函数。
- 对只看静态文本的断言，改用 `react-dom/server` 的 `renderToStaticMarkup`。
- 对需要验证按钮动作的场景，优先抽出纯 action builder 或 selector 测试，不强行模拟 DOM 点击。
- 如果某个组件必须交互测试而没有 DOM 测试环境，先降级为纯函数测试 + server render smoke，不引入新依赖，除非用户确认新增测试依赖。

**Step 3: 验证**

Run:

```bash
npm run test:offline-interaction
npm run build
```

Expected: PASS。

**Step 4: 记录**

- Evidence: 离线测试从 invalid hook call 变为 PASS。
- Risk: 该测试仍不是浏览器点击测试；后续 UI 完成声明仍需要 Tauri / Browser 验证。

---

### Task 2: 修 Rust snapshot 测试隔离

**Files:**

- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

**Step 1: 复现失败**

Run:

```bash
cargo test --lib snapshot_keeps_metadata_without_session_body -- --nocapture
```

Expected before fix: FAIL，实际 session_count 受本机真实会话污染。

**Step 2: 加隔离入口**

Do:

- 把 `build_snapshot` 的会话来源拆成可注入读法。
- 测试 `snapshot_keeps_metadata_without_session_body` 使用 index fixture 的 `threads[]`，不能读取真实 Codex sqlite。
- 真实 app 路径仍可保留 sqlite overlay，但测试必须显式走 fixture。

Suggested shape:

```rust
enum SessionSourceMode {
    RealWithSqliteFallback,
    IndexOnly,
}
```

或等价的 helper 函数。实际实现以现有代码最小改动为准。

**Step 3: 验证**

Run:

```bash
cargo test --lib snapshot_keeps_metadata_without_session_body -- --nocapture
cargo test --lib
```

Expected: PASS，ignored 测试仍 ignored。

---

### Task 3: 把完整项目工作流界面接入当前应用

**Files:**

- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/App.tsx` if props need adjustment
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/styles.css` only if layout breaks

**Step 1: 写失败测试**

Add test assertion:

- Rendering `ProjectsView` with a project and workflow state should expose the full project tool rail labels:
  - `工作流`
  - `Agent 会话`
  - `任务包`
  - `Handoff / Evidence`
  - `Skills`
  - `Harness`
  - `设置`
- It should expose workflow sections:
  - `工作流编排`
  - `节点会话绑定`
  - `派发指令`
  - `工作流机器`
  - `项目工作流草稿`

Run:

```bash
npm run test:offline-interaction
```

Expected before fix: FAIL because current `ProjectsView` renders `ProjectStage` instead of `ProjectDetail`.

**Step 2: 接主路径**

Do:

- In `ProjectsView`, replace the current `ProjectStage` main render path with `ProjectDetail`.
- Keep selected project list on the left.
- Add `selectedTool` state in `ProjectsView`.
- Add `focusedAgentThreadId` state if needed.
- Pass through existing props:
  - `workflowState`
  - `onRequestAction`
  - `onLoadTranscript`
  - `onRenderTaskPreview`
  - `onInspectDispatchReadiness`
- Preserve current session transcript read behavior through `ProjectAgentSessionsPanel`.

**Step 3: Remove dead import if needed**

Do:

- If `WorkflowStatePanel` remains unused, remove the import or explicitly mount it in the full project detail if it belongs there.
- Do not leave unused imports that rely only on TypeScript ignoring them.

**Step 4: 验证**

Run:

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

Expected: PASS。

**Step 5: Browser/Tauri 验证**

If a Tauri window can be launched with user-approved workflow:

```bash
npm run tauri:dev
```

Manual checks:

- 项目页显示完整项目工具栏。
- 工作流页显示节点绑定、派发指令、工作流机器、任务包草稿。
- 旧的首页、Agent、Skill、Harness、画布导航仍能进入。

If Tauri cannot be launched, record as unverified and only claim code/test verification.

---

### Task 4: 建立草案对象到当前状态文件的兼容读模型

**Files:**

- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- Test: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- Test: Rust tests in `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

**Step 1: Define frontend types**

Add or extend types for:

- `Workflow`
- `WorkflowNode`
- `WorkflowRunCheck`
- `TaskPackage`
- `WorkflowLedgerEntry`
- `SubagentReport`
- `ReviewResult`
- `WorkflowException`

Conservative default:

- Keep existing `WorkflowStateSnapshot` stable.
- Add derived summaries rather than breaking current frontend props.

**Step 2: Backend derived read model**

Do:

- Add Rust parser/summary helpers that can derive draft objects from existing `workflow_state_v0`.
- Do not migrate storage yet.
- Expose missing fields as `null`, empty arrays, or warnings, not guessed facts.

**Step 3: Tests**

Add Rust fixture test:

- Given a v0 workflow with work item, artifact, dispatch, review.
- Read model returns:
  - one Workflow
  - expected WorkflowNodes
  - TaskPackage summary
  - ledger summary
  - warnings for unsupported fields

Run:

```bash
cargo test --lib workflow_task_package_read_model
npm run typecheck
```

Expected: PASS。

---

### Task 5: 实现 WorkflowRunCheck

**Files:**

- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- Test: Rust tests in `src-tauri/src/lib.rs`
- Test: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

**Step 1: Backend command**

Add command:

```text
inspect_workflow_run_check
```

Input:

- `project_root`
- optional `workflow_id`

Output:

- `status`: `runnable` | `warning` | `blocked`
- checks:
  - missing owner
  - missing session
  - missing model
  - missing permission scope
  - missing harness requirement
  - missing acceptance criteria
  - policy violation
  - unresolved conflict

**Step 2: UI**

Add project rule status strip to workflow page:

- 当前 harness
- 运行性检查状态
- 违反规则数量
- 证据完整度
- blocked reasons

**Step 3: Tests**

Rust tests:

- no workflow -> blocked
- workflow with no binding -> blocked
- workflow with ready work item but missing acceptance criteria -> warning or blocked according to rule
- fully prepared fixture -> runnable

Frontend tests:

- blocked result renders blocking reason
- runnable result enables next step display but does not auto-run

Run:

```bash
cargo test --lib workflow_run_check
npm run test:offline-interaction
npm run build
```

Expected: PASS。

---

### Task 6: 强化任务包生成算法

**Files:**

- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- Test: Rust tests in `src-tauri/src/lib.rs`
- Test: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

**Precondition:**

- Use the confirmed strategies from "确认 1"、"确认 2"、"确认 3" only.
- If executing only `Task 6`, do not also implement `Task 7-12` in the same pass.
- Do not write real user workflow state.

**Step 1: Inputs**

TaskPackage generation must read:

- workflow node
- target role/session
- allowed read/write scope
- available skills
- available knowledge refs
- available memory refs
- callable tool capabilities
- model id
- harness requirements
- acceptance criteria
- forbidden actions
- report format
- timeout / failure policy

**Step 2: Permission whitelist**

Rules:

- 没写允许，就是不允许。
- 子智能体只能看到任务包列出的资料、工具、技能、模型。
- 如果字段缺失，不自动补编业务内容，只返回 blocked / missing fields。

**Step 3: UI**

Task package preview must display:

- allowed read
- allowed write
- knowledge refs
- memory refs
- tools
- skills
- model
- harness requirements
- acceptance criteria
- report format
- forbidden actions

**Step 4: Tests**

Rust tests:

- missing allowed write -> not ready
- missing report format -> not ready
- forbidden action conflict -> blocked
- harness template applied -> includes required return format
- no memory refs -> empty, not guessed
- no knowledge refs -> empty, not guessed

Frontend tests:

- preview shows missing fields as missing, not generated prose
- confirmation dialog shows write boundary before generate/dispatch

Run:

```bash
cargo test --lib task_package
npm run test:offline-interaction
npm run build
```

Expected: PASS。

**Step 5: 给用户手动测试清单**

Create or update handoff:

- `/Users/yoyi/workspace/product-line/handoffs/YYYY-MM-DD-workflow-task-package-task4-6-user-test.md`

The handoff must use plain Chinese and include:

- 打开哪个应用页面。
- 选择哪个项目或测试数据。
- 怎么看到运行前检查结果。
- 怎么看到任务包预览。
- 哪些缺失字段应该显示为 blocked。
- 哪些非必需字段应该显示为空或 warning。
- 怎么验证系统没有自动补模型、权限、知识库、记忆或业务事实。
- 怎么验证任务包编辑后变为 stale，并且必须重新检查。
- 哪些结果算通过。
- 哪些结果算失败，需要截图或回报。

---

### Task 7: 实现工作流账本、子汇报、审查结果、异常通知

**Files:**

- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- Test: Rust tests in `src-tauri/src/lib.rs`
- Test: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

**Step 1: Ledger**

Implement append/read summaries for:

- `task_package_created`
- `subagent_started`
- `permission_requested`
- `permission_granted`
- `permission_denied`
- `tool_call_summary`
- `subagent_report`
- `review_result`
- `node_returned`
- `node_failed`
- `node_passed`
- `director_summary`
- `user_decision`

Rule:

- Do not store tool call full output in workflow ledger.
- Store `summary`, `source_refs`, `tool_call_refs`, `audit_refs`, `risk_flags`.

**Step 2: SubagentReport**

Require fields:

- executed what
- changed what
- evidence refs
- open issues
- permission requests
- direction risk
- follow up suggestions
- acceptance status

**Step 3: ReviewResult**

Implement:

- passed
- returned
- failed
- not_required

Rule:

- ReviewResult cannot directly complete node.
- Project director still marks final node result.

**Step 4: WorkflowException**

Implement exception generation for:

- subagent timeout
- subagent failed
- repeated review return
- long permission wait
- unresolved direction risk
- harness blocked

**Step 5: UI**

Add panels in full project workflow page:

- ledger panel
- subagent report panel
- review result panel
- exception notification panel

**Step 6: Tests**

Run:

```bash
cargo test --lib workflow_ledger
cargo test --lib subagent_report
cargo test --lib review_result
cargo test --lib workflow_exception
npm run test:offline-interaction
```

Expected: PASS。

---

### Task 8: 实现状态机和完成判定

**Files:**

- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- Test: Rust tests in `src-tauri/src/lib.rs`

**Step 1: Workflow 状态机**

Allow:

- `draft -> ready`
- `ready -> running`
- `running -> paused`
- `paused -> running`
- `running -> waiting_decision`
- `waiting_decision -> running`
- `running -> completed`
- `running -> failed`
- `completed -> archived`
- `failed -> archived`

Reject:

- direct `draft -> running`
- direct `waiting_decision -> completed`
- direct `failed -> running` without explicit retry/reopen action

**Step 2: WorkflowNode 状态机**

Allow:

- `not_started -> waiting`
- `waiting -> running`
- `running -> waiting_permission`
- `waiting_permission -> running`
- `running -> waiting_decision`
- `waiting_decision -> running`
- `running -> reviewing`
- `reviewing -> passed`
- `reviewing -> returned`
- `returned -> running`
- `running -> failed`
- `running -> paused`
- `paused -> running`
- `waiting -> skipped`

Rules:

- `passed` 不等于 workflow completed。
- 子智能体不能改成 `passed`。
- `waiting_decision` 不能自动恢复。
- `failed` 后由项目主管选择重试、退回、换会话或结束。

**Step 3: Completion gate**

Project director can mark node complete only if:

- task goal completed
- acceptance criteria met
- evidence refs exist
- review or harness passed when required
- no unresolved risk
- memory candidate step considered
- final user report needed/not needed recorded

**Step 4: Tests**

Run:

```bash
cargo test --lib workflow_state_transition
cargo test --lib workflow_node_state_transition
cargo test --lib director_completion_gate
```

Expected: PASS。

---

### Task 9: 补权限、工具、harness、知识库、记忆层接口边界

**Files:**

- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- Modify: `/Users/yoyi/workspace/product-line/docs/open-questions.md` if unresolved questions must be tracked
- Modify: `/Users/yoyi/workspace/product-line/docs/decisions.md` or `decisions/**` only if a new decision is accepted

**Step 1: Define interface stubs**

Add backend/domain interfaces or equivalent pure helper boundaries for:

- proposal interface
- memory candidate interface
- knowledge refs interface
- tool capability registry
- model pool selector
- harness requirement provider
- audit refs

**Step 2: Conservative defaults**

- Memory: task package may reference confirmed memory; workflow ledger can produce memory candidates but cannot write formal memory.
- Knowledge: knowledge refs are explicit material refs only; no auto-scan.
- Tools: tool capabilities are whitelist entries.
- Harness: affects run check and completion gate, not a normal node.
- Models: if missing, run check returns blocked or warning; do not pick a model silently.

**Step 3: Open questions**

If still unresolved, record:

- multiple harness conflict handling
- harness failure policy
- harness output UI detail
- tool call summary retention
- task package manual edit policy
- task package invalidation after workflow modification
- ledger to audit reference granularity

**Step 4: Tests**

Run:

```bash
cargo test --lib workflow_interfaces
npm run typecheck
```

Expected: PASS。

---

### Task 10: 画布和界面补齐草案形态

**Files:**

- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/CanvasView.tsx`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/styles.css`
- Modify: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- Test: `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

**Step 1: Research gate**

Before this task, decide whether it only uses existing React Flow patterns or requires deeper node-tool research.

If implementing any of these:

- template library
- execution queue UI
- run history UI
- advanced node parameter panel
- canvas / workflow state merge

Then first execute the "画布研究闸门" section and update this plan.

**Step 2: Views**

Implement:

- proposal view
- running state view
- ability to switch back
- run check status strip
- evidence completeness
- rule violation count

**Step 3: Nodes**

Main nodes:

- consultation
- director
- subagent
- review
- report

Do not make these main nodes by default:

- manual confirmation
- knowledge read
- tool call
- ordinary permission read

**Step 4: Node detail panel**

Right panel shows:

- knowledge permission
- tool permission
- model
- skills
- acceptance criteria
- review requirements
- harness requirements
- task package
- report
- ledger records
- audit links

**Step 5: Four-role template**

Add initial template or load action for:

- director
- codex-dev
- validation
- review
- report

Do not create Codex sessions from canvas without a separate accepted task.

**Step 6: Tests and verification**

Run:

```bash
npm run test:offline-interaction
npm run build
```

If UI completion is claimed, also run Tauri/browser visual verification and save screenshots/evidence.

---

### Task 11: End-to-end acceptance scenarios

**Files:**

- Modify tests as needed under `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/**`
- Modify Rust tests under `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- Create evidence under `/Users/yoyi/workspace/product-line/evidence/**`
- Create handoff under `/Users/yoyi/workspace/product-line/handoffs/**`

**Step 1: Acceptance 10.1**

Scenario: subagent finds direction risk.

Expected:

- subagent report writes direction risk to project director
- node enters `waiting_decision`
- project director creates proposal / decision request
- no direct user request from subagent
- no direct workflow direction change by subagent

**Step 2: Acceptance 10.2**

Scenario: task package limits context.

Expected:

- package includes explicit memory refs, knowledge refs, tool capabilities, skills, model, read/write scopes
- missing allowed scope means not allowed
- dispatch blocked when package incomplete

**Step 3: Acceptance 10.3**

Scenario: subagent completes and reports.

Expected:

- report enters workflow ledger
- memory candidate can be generated after director summary
- no formal memory write happens automatically

**Step 4: Acceptance 10.4**

Scenario: review agent passes.

Expected:

- review result stored
- project director still must mark node passed/completed

**Step 5: Acceptance 10.5**

Scenario: harness enabled.

Expected:

- harness affects run check, task package template, completion gate
- harness is not shown as ordinary main workflow node

**Step 6: Verification commands**

Run:

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib
```

Expected: PASS except explicitly ignored confirmation-only tests.

**Step 7: Evidence and handoff**

Create:

- `/Users/yoyi/workspace/product-line/evidence/YYYY-MM-DD-workflow-task-package-design-v1-execution.md`
- `/Users/yoyi/workspace/product-line/handoffs/YYYY-MM-DD-workflow-task-package-design-v1-execution-result.md`

Must state:

- what is implemented
- what remains unverified
- whether real `codex exec resume` was run
- whether `/Users/yoyi/.codex` was touched
- whether workflow state was written
- tests run and exact results

---

### Task 12: Update current authority after implementation

**Files:**

- Modify: `/Users/yoyi/workspace/product-line/CURRENT.md`
- Modify: `/Users/yoyi/workspace/product-line/tasks/README.md`
- Modify: `/Users/yoyi/workspace/product-line/docs/workflow-task-package-design-v1.md` only if the草案 itself changes
- Modify: `/Users/yoyi/workspace/product-line/decisions/**` only for accepted product/architecture decisions

**Step 1: Update current state**

Do:

- Move completed items from "未完成" to "已完成" only with evidence.
- Keep limitations explicit.
- Do not claim "真实业务自动编排" unless a real approved run proves it.

**Step 2: Update task queue**

Do:

- Add next dispatchable task if implementation is incomplete.
- Mark superseded/paused items.

**Step 3: Verification**

Run:

```bash
rg -n "workflow-task-package-design-v1|工作流|任务包|画布" /Users/yoyi/workspace/product-line/CURRENT.md /Users/yoyi/workspace/product-line/tasks/README.md
```

Expected: current authority points to real evidence and does not overclaim.

---

## Completion Definition

The plan is complete only when:

- Full project workflow interface is reachable from the current app Projects page.
- Frontend offline interaction test passes without invalid hook call.
- Rust tests pass without leaking real local Codex session counts into fixture tests.
- WorkflowRunCheck exists and blocks unsafe/incomplete workflows.
- TaskPackage generation enforces whitelist permissions and does not invent missing business facts.
- Workflow ledger records summaries and refs, not full tool outputs.
- SubagentReport, ReviewResult, WorkflowException are represented in backend and UI.
- Project director remains the final completion authority.
- Harness affects run check and completion gate but is not treated as ordinary node.
- Memory and knowledge boundaries match `docs/memory-layer-design-v1.md`.
- All accepted scenarios from section 10 of the草案 have tests or explicit evidence.
- `CURRENT.md` and `tasks/README.md` are updated with evidence-backed status.

## Residual Risks

- `workflow_state_v0` may become too large or awkward before SQLite migration. Mitigation: keep repository/read-model boundary stable.
- Canvas file layer and workflow state file layer may conflict. Mitigation: decide merge/separation before advanced canvas work.
- Adding too many panels may make UI hard to operate. Mitigation: keep default view focused on progress, risk, waiting decisions, and final report.
- Tests may pass without real Tauri UI validation. Mitigation: browser/Tauri screenshot verification is required before UI completion claims.
- External reference tools may pull scope toward generic automation. Mitigation: use the画布研究闸门 and update plan only after translating findings to Codex workflow objects.
