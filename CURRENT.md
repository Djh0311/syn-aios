# Current Authority（精简版 · 2026-06-21）

> **本文是唯一「每次工作完必更」的活正本**（四块：能用 / 在做 / 下一步 / 锁着）。per-task 状态以本文为准；`docs/plans/2026-06-18-master-roadmap-phased-v1.md` 只在阶段切换时动。规则见 `AGENTS.md`。完整历史进 `archive/` + git。

## 一、现在真能用什么（验过的）

- **甲·中转 relay（A 线 ③b 已收口）**：GUI 在 Syn 里**真发 codex 成功**（里程碑 `9b7360a`）；绑会话 Enter 直发、`codex exec` 沙箱限项目 `--sandbox workspace-write` + `--add-dir` + 拒审批绕过、在场 env 闸、stop 真杀、回执。后端 `manual_relay.rs`。**唯一真能指挥 codex 的路径。**
- **⭐ 智能体页 UI = 整体完成**（真机验过、已入库）：codex 布局；信息呈现收口；会话流拆固定虚拟化 + 消抖 + 按轮分组 + 过程折叠；会话列表过滤 subagent（604→136）+ 无项目统一 + 标题截断 + 拖拽改宽 + 侧栏收窄。
- **运行工作流画布 = 画布优先重画 P1–P4 完成**（typecheck 0 + offline 15+r4 亲验）：任务包页改成执行画布（状态带 + 只读 React Flow 节点 + 连线着色 + 阶段段带 + 右栏详情）；纯前端零后端、点节点不执行。**唯一没验**：真机对图。
- **统一记忆层 + 真攒记忆**：存储 = JSON sidecar（App Support）、命令全接通、线上实存 **3 条真实正式记忆**（已建已用，非「没做」；被冻结的部分见 ④b）。
- **前端其余（B 线已收口）**：拆瘦 App 1104→695 / 记忆页 1340→676 / 工作流侧栏 953→340；全 views 渐进披露。
- 后端基线：`cargo test --lib` = **555 passed / 0 failed**（2026-06-21 本机）。

## 二、在做什么

- **自由节点画布 = 真机已通**（左侧栏「运行中工作流」入口已重指向实验画布 `CanvasView`）：空白双击 / 调色板建**任意种类**节点（主管/子agent/审查/工具/便签/自定义）+ 右栏编辑 + 拖连 + 触控板平移缩放；React Flow 会话态、纯前端零后端。**真机修过的坑**：高度链塌 0（#004 —— `canvas-view` 改视口定高、`canvas-flow` / `running-canvas-stage-wrap .project-flow-stage` `height:100%`）、双击被缩放截走（`zoomOnDoubleClick={false}` + 显式 `nodesDraggable/Connectable`）、入口重指（`ActiveWorkbenchView` 的 `runningWorkflows` 分支改渲染 `CanvasViewWithProvider`；RunningWorkflowsView 暂留代码不从此进）。验：typecheck0 / offline0 + 用户真机确认能建/编/连/平移。方案 `docs/plans/2026-06-21-free-canvas-node-authoring-and-mature-pattern-plan-v1.md`。
- **画布下一步**：① 操作手感打磨（用户："还有点不舒服"，待细化哪里别扭）；② 入口标签「运行中工作流」该改名（它现在是实验画布）；③ **A4 落盘 + B 成熟模式保留 = 已建 + 主导线核实物 + 本次 commit**（B store **折进 `mcp/storage.rs` 非新文件**；原子写/校 schema/list 跳损坏/delete 幂等、**独立于记忆 `mature_pattern_store`**；4 命令前后端接线 + CanvasView「成熟模式」面板）；验 cargo555 / typecheck0 / offline 真断言（多行 prompt+自定义字段往返无损 / B3 实例化 id 重置+边重映射+悬空边丢弃）/ 后端在跑的二进制里（mtime 对）——**真机 UI 仍待用户统一验收**（computer-use 抓不到 Tauri dev 二进制 CFBundleId=NULL：建节点设 payload→存→重载不丢 / 存模板→起新工作流，这步用户做）；④ **C 接执行 = 已建 + 主导线独立核实物（安全轴全过）+ 本次 commit**：画布节点「▶ 运行此节点」接已走通双闸命令 `execute_workflow_node_dispatch`（前端 `buildNodeDispatchRequest`/`nodeRunReadiness`/`executeWorkflowNodeDispatch` 只造请求+发、不判闸）。**复核**：闸函数 + execute 体字节未改（diff 仅 +1 闸测试 `workflow_engine_gate_seals_non_test_project_regardless_of_env`）、`&&` 在 path 短路、前端设不了后端 env 钥匙→按钮单独开不了闸、sandbox 收紧（read-only 0 写根 / workspace-write 限 project_root）、**零自动执行未真跑**（测试项目无新 proof）；验 cargo556/0 / typecheck0 / offline 真断言（C1 请求映射 + C2 readiness）/ build277。落差：节点↔workflow-state work_item 无自动对应、靠手填「工作项 ID」+ 真跑前需备测试项目 workflow-state/会话。**真跑一个节点 = 高危#1，仍逐次授权**（设 env `WORKFLOW_ENGINE_TEST_CONFIRM=CONFIRMED_TEST_PROJECT_REAL_RUN` + project_root=测试项目 + 绑真会话 + work_item_id → 点运行）。
- **成熟模式 bug 修 + 右栏精简 = 真机已验、本次 commit**：B 的存/删原靠 `window.prompt`/`window.confirm`（Tauri webview 不弹）→ 静默失败；改成页面内标题输入 + 两步确认，**用户真机验过：存/删通**。右栏修横向溢出（input/select `width:100%`+`box-sizing`+`min-width:0`）、预设精简（种类 6→4 留 director/subagent/reviewer/custom、状态建议 5→3 但 tones 仍全）、把沙箱/会话/工作项/运行折进「接执行」折叠区。**注**：会话选择器现为临时版（select→datalist），P2 按会话模型方案重做平级分段、届时超越；A4 重载不丢 / C default-safe 仍待真机。
- **工作流画布架构 = 两面一引擎、方案已出 + 功能定义已拍**：两个画布——**实验=沙盒**（不碰真实项目、只模拟/测试）/ **项目=项目内真实工作**（同样自由编排，但每个影响真实项目的动作受控制核心/权限/审计）；**底层一套可编辑引擎、两种调用配置**（扩现 `canvasSurfaceBoundaries`，已是两面边界种子）。核心轴 = **沙盒 vs 真实受控**（非功能多少）；成熟模式两面都有，**项目真实跑出来的才稳定可复用**（§26.4）。现状两套分叉引擎（实验可编辑 / 项目只读）且**拧反**（蓝图 §11 要项目可编辑）。分期 P0 抽 `WorkflowCanvasEngine` → P1 项目面接引擎并开可编辑 → P2 规则条/双视图/运行性检查 → P3 重档真跑分面接闸。会话/作用域模型并入（scope = 所在面）。方案 `docs/plans/2026-06-21-workflow-canvas-two-surfaces-one-engine-v1.md`。
- **会话与作用域模型 = 方案已出 + 决策已拍、待开发**：定义/运行分层 + 会话绑运行层（模板只存策略·新建/已有**平级**）+ 作用域**实验/项目**两档；治本现「模板实例化继承旧会话」reuse bug（`instantiateTemplateGraph` 带走 session_id）。分期 P1/P2 轻档（数据模型 `session_policy`+`scope`、向后兼容迁移、UX 平级分段控件）、P3 重档（运行层 policy→会话解析接 C 真跑·逐次授权）。决策：模板 resume 实例化清 thread_id / 新建会话真跑时建 / 每节点各自一条会话。调研依据 Temporal·LangGraph·n8n·Dify·GitHub Actions。方案 `docs/plans/2026-06-21-workflow-session-and-scope-model-v1.md`。
- **教训（反复踩、已坐实）**：A1-A3 当初 typecheck/offline 绿就报「已核」，真机整块不可用（高度塌 + 交互被截）。**画布/UX 类必须真机过才算完成**，机器绿 ≠ 真机能用。
- **工作流引擎解封·第一刀 = 适配器真跑已验**（高危#1 用户授权下，2026-06-21）：`codex_local_runner.rs` 的 `RealWorkflowNodeCodexRunner`（复用 `command_plan_for`+`run_real_codex_process`）+ `commands.rs` 双闸（真实项目零变化）。**实物核过**：直接调适配器在测试项目 `/Users/yoyi/codex-workflow-mario-test` 真起一次 codex，codex 建了 `workflow-real-run-proof.txt`（内容对）、沙箱只动测试目录没外溢、exit 0；new_session 路径通。`cargo test --lib` 555 + #[ignore] 真跑测试。决策 `decisions/2026-06-21-next-step-unseal-workflow-engine-for-test-project-v1.md`。
- **工作流引擎解封·走通已验（全派发路径真跑）**：bootstrap 工作流 → 绑真会话 `019ed9f7` → `execute_workflow_node_dispatch_for_index_at`（= 双闸命令过闸后的真实现）→ 适配器 resume → 真 codex 建 `workflow-fulldispatch-proof.txt`（内容对）、沙箱只动测试目录、dispatch `state=completed` exit0。`#[ignore]` 集成测试 `real_run_full_dispatch_resume`。**即:工作流 worker 节点已能经生产派发路径真跑 codex。**（注:派发是 resume 已绑真会话那套——节点需先绑一个真 codex 会话。）
- 运行工作流画布·真机对图打磨：代码 P1–P4 完成，剩起 Tauri 微调视觉（未做）。

## 三、下一步

1. **引擎解封已走通**，往下可选:① 起 Tauri 过真双闸命令端到端（GUI 层，含 AppState）；② 把闭环其余生成节点（咨询出方案 / 主管拆任务 / 候选提取——现都只存/预览、无真 AI）也接 codex，让 §0.6 闭环更多节点活；③ 回到积压（文档归档 / 画布真机对图）。高危#1 真跑仍逐次授权。
2. 据核实矩阵（`handoffs/2026-06-21-full-project-fact-reconciliation-result-v1.md`）做文档归档 + 死码清理——注：`workbench_sqlite_*` 孤岛虽 ~480 dead warning，但是 R3 已拍板的**未来迁移机制，保留勿删**，只压噪声。
3. 画布真机对图打磨（纯视觉）。
4. 再往后 relay GUI 尾巴 / 中间·半自动。

## 四、锁着的 / 没接（区分三种「不在线上默认」，别压成「deferred」一个词——那是上一版误报之源）

**a) 故意锁（impl 完整、可解条件明确）**
- 工作流多节点编排引擎：前后端双锁（`App.tsx:547` / `commands.rs:1789`→legacy blocked）；四角色真环实现完整（`workflow_execution_entrypoints.rs:1489`），但**只 stub 测过、无真 runner**。⚠️ 主导线正解封到固定测试项目（见 ②）。
- product-command Phase B / 受控 real resume：对真实项目封，探针跑过。
- 真跑 codex 进真实项目（非 temp）：用户在场授权那一下，不可省。

**b) 已建 + 已演 + 用户拍板「先不翻闸」（不是没做）**
- R3 真库切换（JSON→sqlite）：迁移机制全建、fixture+Level-B 演完、`ac7813b` 拍板，结论 `ready_but_not_executed`。**线上仍走 JSON**（`workflow-state.v0.json`）；翻闸被拍板暂缓，机制 0 命令、0 生产调用方。
- 统一记忆层 + 真攒记忆：见 ①。冻结的只有「切 DB」+「多 agent 专属门」，不是整块 deferred。

**c) 没建（终局）**
- 乙·自动连环 / 多项目接力：风险到这才真大，没开。

---

*阶梯：甲·手动中转（已收口）→ 中间·半自动（下下步）→ 乙·自动连环（终局）。**本文每次 commit 必回写**（AGENTS.md §五）。证据正本：`handoffs/2026-06-21-full-project-fact-reconciliation-result-v1.md`。*
