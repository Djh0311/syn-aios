# Kickoff · UI 专线 · 角色循环半自动编排「上界面」（让后端引擎真能点）v1

日期：2026-06-27 · 主导线（后端线）→ UI 专线。
> **这是什么**：UI 专线的启动简报。后端「说目标 → AI 出方案 → 你确认 → 自动推进 → 看结果」的引擎已建好且真跑验过（命令层），**就差接上界面让用户真能点**。本文告诉你：调哪些命令、什么顺序、守哪几道闸、挂起的前端怎么接手。
> **先读**：`CURRENT.md`（① 现在能用 + 🔜）、决策 `decisions/2026-06-27-role-loop-semi-auto-orchestration-light-tier-v1.md`、设计 `docs/plans/2026-06-27-role-loop-auto-orchestration-spec-v1.md`、`principles.md` §4、`docs/middleware-version-development-plan-v1.md` §0.3（方案授权制）。

## 0. 一句话目标

把后端这条引擎接成 app 里能点的流程：**用户说目标 → 看 AI 出的方案卡 → 审/准（方案授权·人闸）→（边界复核）→ 一键自动推进（拆任务→prepare→worker 链跑）→ 看结果**。手点从原来 7 步塌成 **说目标 + 审一张卡**。

> ⚠️ **workflow 锚点（2026-06-27 更新·真机暴露 e2e 卡点后定）**：角色循环跑在**项目里选中的工作流**上（不是隐藏的「默认」）。后端走 **(b)**——放开 C4「只认默认」死校验（`tasks/2026-06-27-role-loop-run-on-project-workflows-v1.md`·后端线做）。**你这边**：P3 锚 `projectWorkflow`（画布选中那条）即可；**你之前那个「给方案打 selected 标签」的修回退**（方向反了）。等后端 (b) 落 + 你锚选中，两条对上号就通。

## 1. 后端给了你什么（已建·真跑验过·命令层可调）

| 阶段 | 命令（后端·已 committed） | 入 / 出 | 前端 wrapper |
|---|---|---|---|
| **出方案** | `run_project_consultation`（async·P2·`5adbd77`） | 入 `{project_root, workflow_id, goal, actor_id}` → 出 新方案（`proposal_id`/目标/范围摘要/`status=PendingUserConfirmation`） | **❌ 要你加**（`tauri.ts`，照 `startProjectDirectorChain` 范本） |
| **确认方案**（方案授权·人闸） | `record_project_consultation_proposal_decision`（confirm/request_changes/reject） | 现成 | ✅ 已接（治理面板用） |
| **边界复核**（全局主管·现你演） | `record_global_boundary_review`（approved 让授权 active） | 现成 | ✅ 已接 |
| **自动推进** | `auto_advance_authorized_role_loop`（async·P1·`bd503a3`） | 入 `{project_root, workflow_id, actor_id}` → 出 `{stage(ran/needs_binding/blocked/no_dispatchable), planned_task_count, prepared_count, needs_binding_count, blocked_count, chain_outcome, stop_reason, message}` | **❌ 要你加** |
| **链进度/停**（自动推进内部跑链·这俩可选复用展示） | `get_project_workflow_chain_status` / `stop_project_workflow_chain` | 现成 | ✅ 已接 |
| **绑会话**（撞 needs_binding 时） | 现成绑定 UI（执行面板「节点会话绑定」） | — | ✅ 已在（但散在执行面板·见 §3 挂起件 #3） |

**你主要要加 2 个前端 wrapper**：`runProjectConsultation`、`autoAdvanceAuthorizedRoleLoop`（都照现成 `startProjectDirectorChain`：`invoke<Out>("命令名",{request})`）。

## 2. UI 要建什么（P3 = 件 D）

1. **触发口**：「说目标」输入 + 「AI 出方案」按钮 → 调 `run_project_consultation` → 拿回方案。**取代现有手填模板**（面板里那个「创建方案草案」可保留作手动挡）。
2. **方案授权卡（人话·核心）**：把方案讲清——**这次会改你哪几个文件 / 干啥 / 几个 worker / 范围 / 风险 / 必停点**，给 `[确认运行]/[要求修改]/[拒绝]` → 调现成 `record_..._decision`。这就是 S2「方案授权制 UI」那项。
3. **边界复核**（现你演全局主管）：approved → 授权 active。可跟方案卡合一张展示、但**是两道不同角色的审**（用户 vs 全局主管），别糊成一道（见 §4）。
4. **一键自动推进按钮**：授权 active 后出现 →（无参/项目+工作流）调 `auto_advance_authorized_role_loop` → 据返回 `stage` 展示：
   - `ran` → 链跑了，展示 `chain_outcome`（completed/进度），可复用 `get_project_workflow_chain_status` 轮询 + `stop_project_workflow_chain` 停。
   - `needs_binding` → 提示「去绑会话」（引到执行面板绑定·或顺手做 §3 #3）。
   - `blocked`/`no_dispatchable` → 展示 `message`、停、等用户。
5. **看结果**：proof / 链完成态 / 审计。

## 3. 挂起的前端要接手（工作树未提交·机器侧过·真机待你验）

主导线这边**挂起了 3 个前端文件**（`lib/tauri.ts` / `lib/types/workflow.ts` / `views/projects/ProjectWorkflowGovernancePanels.tsx`）+ 包 `tasks/2026-06-27-s2-director-chain-ui-button-slice-v1.md`，**typecheck0·`src-tauri` 0-diff·但没真机**。内容（你接手·真机·commit）：
1. **C1「按计划开干（整条主管链）」按钮** + 进度/停（在 `ProjectDirectorTaskPlanCard`）——这是手动挡，自动推进上线后它降为 override。
2. **3 处真机暴露的 bug 修**：① 第四步「生成拆任务草案」死按钮加可见原因（需先确认方案+边界复核）② 方案 rejected/changes_requested 加「重新创建方案草案」出口 ③ needs_binding 提示改成可操作（指向绑定 UI）。
3. **绑会话散在执行面板**：撞 needs_binding 时用户不知道去哪绑——你 P3 顺手把绑定入口接进角色循环流（或在 needs_binding 提示里直接给绑定口）会顺很多。

> ⚠️ 这些挂起改动**未 commit**（主导线没真机不敢 commit·画布/UX 类必真机过才算完成）。**怎么把它们交到你手上见 §6**。

## 4. 死线 / 口径（必守·别破）

- **方案授权这道人闸不省**（principles §4 / §0.3「方案授权制·用户确认方案=确认一段自动执行范围」）：UI **绝不自动确认方案 / 自动跑**——必须用户点「确认运行」。`auto_advance` 后端已强制「无 active 授权即拒」，前端别想绕。
- **几道审**：用户确认方案（1 道·人·不省）+ 全局主管复核边界/结果（角色·现你演·未来 agent 自动）。别把这两道随手合并成一道（canon 是不同角色）。
- **前端只造请求 + 发**：圈测试项目的闸在后端（path-lock）；非测试项目前端造不了钥匙、按钮单独开不了闸。UI 不判闸、不自建更松路径。
- **真执行提示**：点「自动推进」= 测试项目真起 codex 链（真执行·非预览），UI 要写清。
- **机器绿 ≠ 真机能用**：你这条线必真机过才算完成（computer-use 抓不到 Tauri dev 窗口，所以归你·tauri dev 改前端记得 app 里 Cmd+R 强刷·热更不可靠）。

## 5. 流程图（接线对照）

```
[说目标]──run_project_consultation──▶[方案(Pending)]
                                          │
                                [方案授权卡:确认/改/拒]──record_..._decision──▶[user_confirmed]
                                          │
                                [边界复核(你演全局主管)]──record_global_boundary_review(approved)──▶[授权 active]
                                          │
                                [一键自动推进]──auto_advance_authorized_role_loop──▶ stage:
                                          ├ ran ─────────▶ 链跑(get_chain_status轮询/stop可停)──▶[proof/结果]
                                          ├ needs_binding ▶ 提示绑会话
                                          └ blocked/no_dispatchable ▶ 提示+停
```

## 6. 协作 / 怎么拿到挂起件 + 边界

- **后端我继续在主导线做**（非 UI：C-2 自动绑 / 全局主管 agent / NL 等·按 CURRENT 🔜 或你需要的优先）。**前端 + 真机全归你 UI 专线。**
- **挂起的 3 前端文件怎么交你**：① 若你 UI 专线在**同一工作树/分支** → 直接在工作树里接着改（它们就在）；② 若你是**独立 worktree/分支** → 跟主导线说一声，我把这 3 个挂起件 + 包**作 WIP commit**（明标「机器侧过·真机待 UI 专线验」·非「做好了」）推上去给你接。**默认我先不 commit**（怕替你 UI 决定），你说要哪种。
- **你回交**：P3 + 挂起件真机过 → typecheck/build + 真机录屏/截图 → 主导线扫 diff（碰没碰后端/闸）→ commit（带 CURRENT 回写）。
- **撞后端不够用**（命令缺字段 / outcome 不够展示）→ 跟主导线说，我补后端。

## 7. 参考

- 后端引擎：P2 `tasks/2026-06-27-p2-consultant-llm-wire-v1.md`(`5adbd77`)、P1 `tasks/2026-06-27-p1-role-loop-auto-advance-v1.md`(`bd503a3`)、C1 `tasks/2026-06-26-s3-director-chain-c1-start-command-v1.md`(`9ebd090`)。
- 决策/方案：`decisions/2026-06-27-role-loop-semi-auto-orchestration-light-tier-v1.md`、`docs/plans/2026-06-27-role-loop-auto-orchestration-spec-v1.md`。
- 当前事实：`CURRENT.md`（① + 🔜）。
