# 主导线交接 · 工作流自动连环 P1 在飞（2026-06-23）

**先读**：`CURRENT.md`（活正本）+ `AGENTS.md`（规矩，尤其高危#4）+ `decisions/2026-06-23-test-project-auto-chain-light-tier-v1.md`（本次第①闸）+ 本文。

## 这是什么（别误解成"中间·半自动"）
设计文档 `docs/plans/2026-06-23-workflow-mid-tier-semi-auto-chained-execution-v1.md` 标题叫"中间·半自动"，但顶上 2026-06-23 决策更新把"逐步审批"删了 → **实质是「乙·自动连环」，圈死在固定测试项目**。这是**跨高危#4**的下放，不是真正的"中间"。设计文档 §2/§3/§5/§6 仍写着作废的逐步审批描述（split-brain），**待对齐**。

## 关键拍板（别重新纠结）
用户在听完"这是高危#4·需两道闸"的完整解释后，明确"直接干" = **两道闸都给了**：
- **第①闸**：固定测试项目内"自动连环"下放轻档（`decisions/2026-06-23-test-project-auto-chain-light-tier-v1.md`，已按"直接干"标拍定）。
- **第②闸**：开工实现 P1。
- 仍锁：自动连环进**非测试真实项目** = 高危#1+#4 锁；**多项目接力**锁；auto-approve「乙」全量锁。

## 现在到哪（P1 后端+前端做完、机器绿、未真机、未 commit）
- **后端** 新模块 `src-tauri/src/workflow_chain_controller.rs`：拓扑序逐节点 → 每节点调**已 gated 的** `execute_project_workflow_node_at`（不旁路、不新开闸）→ 链状态进 workflow-state 新数组 `workflow_chain_runs`。
- **四条硬护栏 + path-lock 闸都有 cargo 测试坐实**：runaway 上限 / 可中断(节点边界 stop 标志) / 审计 / 可回滚(backup) / 失败即停 / 断点续(跳已完成) / 非测试项目被闸拒。
- **前端** `ProjectWorkflowCanvasView.tsx` HUD 加「▶▶ 开始链 / ■ 停链」+ 回执；`tauri.ts` 两封装；类型进 `types/canvas.ts` + barrel。
- **机器四闸**：cargo **578/0**（+11 测试、零回归）· tsc 干净 · offline 15+r4 · build OK。
- **核实物（git diff --stat）**：8 文件、+511 行；`codex_local_runner`(沙箱 `command_plan_for`)/`manual_relay`/闸函数 **字节未碰**（`commands.rs` 不在改动列表，只是被调用）。高危#3 守住。

## 立刻要接的
1. **真机跑一条链（必做，机器绿 ≠ 真机；computer-use 抓不到 dev 二进制，只能你做）**：
   测试项目画多节点工作流 → 选中 → **▶▶ 开始链** → 看 codex **逐节点连跑、沙箱只动测试目录** → 跑中点 **■ 停链** 验中断 → 整一个会失败的节点验"失败即停"。**重点核：沙箱真没外溢、停链真停。**
2. **看 UI**：那两个 HUD 按钮我没法亲眼看，过一眼位置/可用。
3. **主导线 commit（问一次）+ CURRENT 回写**——执行线没 commit。

## 待回写清单（决策文档「影响面」同款）
- `CURRENT.md` ③（阶梯推进到"自动连环·圈测试项目 在做"）+ ④c（注"圈测试项目的自动连环已下放轻档"）。
- `AGENTS.md` 高危#4 + §五（细化：固定测试项目 auto-chain = 轻档，类比高危#1 的 2026-06-22）。
- 设计文档 §2/§3/§5/§6 对齐（删作废的逐步审批描述）。

## 诚实边界（先泼冷水）
- **"可中断"是节点边界级**——停不了正在跑的那个 codex 节点，只能它跑完后、下个开始前停。快节点链窗口小。真正挡灾的是 runaway 上限 + 回滚 + path-lock。
- **链真跑每个节点**（含 director 型）——某节点没会话就在那失败即停；绑会话后起链=断点续接上。P1 最简语义，"某类节点不跑"留 P2。
- **起链同步阻塞**：一次 invoke 跑完整条链（UI 不冻，停链是另一条命令置标志）。慢链可能跑几分钟。
- **path-lock + 沙箱现在是无人值守链的写安全几乎全部**（删了逐步闸后）——一松就回高危#4，门槛只升不降。

## 协作模型（别犯过的错）
- 执行线**不 commit**；commit 收口**必带 CURRENT 回写**；commit **问一次**。
- **核实物**：读真 diff、亲跑四闸、不信自报。碰闸/真跑那刀逐字核。
- 改安全闸 / 真跑进非测试真实项目 / 扩大自动连环范围 = **用户明确授权那一下，不可省**。

## 指针
- 活正本 `CURRENT.md`。决策 `decisions/2026-06-23-test-project-auto-chain-light-tier-v1.md` / `...2026-06-22-p3-test-project-real-run-light-tier-v1.md`。设计 `docs/plans/2026-06-23-workflow-mid-tier-semi-auto-chained-execution-v1.md`。记忆 `no-flattery-rules` / `dont-implement-without-explicit-go` / `ux-render-bugs-measure-before-guessing`。
- 锁着的编排引擎 `workflow_execution_entrypoints.rs`（4 角色轮次机、stub 测、无真 runner）= **没用**：P1 是新写薄 controller 走通用 DAG，不解封那个引擎（§5 择一的结果）。
