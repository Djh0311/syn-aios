# 方案 · 角色循环「半自动编排」(canon-faithful · 说目标→咨询出方案→你确认→自动拆+派+链跑) v1

日期：2026-06-27 · 出自：主导线（Claude）+ 用户对话拍定方向 + **核权威文档对齐**。
> 这是**设计方案**（交用户审），不是实现任务包。审过后另拆实现包。
> 上游核过的正本：`principles.md` §4、`docs/middleware-version-development-plan-v1.md` §0.3 方案授权制 + §142-161 闭环、`docs/plans/2026-06-23-workflow-mid-tier-semi-auto-chained-execution-v1.md`、`CURRENT.md` ④c 北极星。

## 0. 痛点 + 解的核

**痛点（用户 2026-06-27）**：现角色循环要手点 **7 步**（建方案→确认→边界复核→生成拆任务→绑会话→准备派发→开干），太繁琐。**用户要的是自动化——但「方案要确认、不是直接自动跑」。**

**解（canon-faithful）**：自动化的是「**造计划 + 执行**」的体力；**保住「方案授权」这道人闸**（principles §4「LM 不得绕过权限确认执行」+ §0.3「方案授权制，不采用每一步确认制；用户确认方案=确认一段自动执行范围」）。

```
[你：说目标]
  → 咨询 agent 自动出方案
[你：确认方案 = 方案授权]            ← 人闸·不省（§0.3）
[你（演全局主管）：复核方案边界]      ← 角色审·现你演（§142-150；未来全局主管 agent 自动）
  → 主管 agent 在授权范围内 自动：拆任务 → 绑会话 → prepare → worker 链跑（直接全跑·不逐步审）
[你（演全局主管）：复核最终结果]      ← 角色审
[看结果]
  · 超授权范围 → 必停下确认（§156）
```

**手点从 7 步 → 2~3 步（说目标 + 确认方案 [+ 边界复核/结果复核：你演全局主管]）。** 中间「造计划+执行」自动。

## 1. 现状（核实物·零件齐，散成手动）

| 环节 | 实物 | 状态 |
|---|---|---|
| 咨询出方案 | `CliConsultantAgent`/`consult` trait/`map_consultation_to_c1_input`（→ `CreateProjectConsultationProposalInput`） | ✅ 建了·**但没接 Tauri 命令**（现 UI 手填模板·价值已证仅 `#[ignore]`） |
| 方案授权 | `create_project_consultation_proposal` / `record_..._decision` / `record_global_boundary_review` 命令 + 前端 | ✅ 已接 |
| 主管拆任务 | `preview_project_director_task_plan`（真 director LM） | ✅ 已接·独立命令 |
| prepare | `prepare_authorized_auto_dispatch` | ✅ 已接·独立命令 |
| worker 链跑 | `run_director_task_chain` + `start_project_director_chain`（C1·本会话建·可中断·真跑验过） | ✅ 已接·独立命令（前端按钮挂起待真机） |

**两个真缺口**：① 咨询 LM 没接命令（出方案是手填模板，非真咨询）；② plan/prepare/chain 是**三个独立命令**——没有「授权后自动推进」把它们串成一下（=手动点 3 次）。

## 2. 要建什么（3 件）

### 件 A · 接咨询 LM（出方案自动化）
新 async 命令 `run_project_consultation`（spawn_blocking·真 codex 咨询长耗时，同链）：建 `ProjectContext` → `CliConsultantAgent.consult(goal)` → `ConsultationProposal` → `map_consultation_to_c1_input` → `create_proposal`。**出方案从"手填模板"变"AI 自动出"。** 复用现成零件、咨询结构性只读（不碰执行闸·安全死线不动）。

### 件 B · 授权后自动推进（**核心·步骤塌缩在这**）
新 async 命令 `auto_advance_authorized_role_loop`（spawn_blocking）：**前提 = 该工作流已有 active 授权**（方案授权 + 边界复核都过）。一下自动串：
1. `preview_project_director_task_plan`（主管 LM 拆任务·授权范围内）
2. （件 C 绑会话）
3. `prepare_authorized_auto_dispatch`（→ prepared）
4. `run_director_task_chain`（worker 链真跑·四护栏在）
- **复用现成命令本体**（plan/prepare/chain 不改逻辑），只新增"串起来"的编排 + 审计每步。
- **canon 依据**：§156「已确认方案授权**范围内**可由控制核心**自动派发**」；§161「自动闭环必须保留 方案授权边界 / 失败可见 / 审计 / 手动恢复入口」。
- **超授权范围 / 任一步失败 → 停 + 可见 + 等你定**（§156/§3 失败即停·不自动重试）。

### 件 C · 绑会话处理（auto-advance 的前置坑）
auto-advance 要 codex-dev 节点绑了真会话否则 prepared 出不来（= 用户真机撞到的 needs_binding）。**两种处理，待你拍（§4 决策）**：
- **C-1（简单·安全）**：没绑 → auto-advance **停在"需绑会话"**、提示去绑，不自动绑。绑是 setup、不是 scope。
- **C-2（更自动）**：自动绑到一条可用 Codex 会话（需定"可用"判据 + 绑定是否要你点头一次）。

### 件 D · 触发 + 方案授权 UI（你审的那张卡）
「说目标」入口 + 「方案授权卡」（人话：**这次会改你哪几个文件、干啥、几个 worker、范围、风险，[确认运行]/[改]/[拒]**——§0.3 + S2「方案授权制 UI」）+ 「按授权自动推进」按钮 + 进度/结果。**这块是 UX·要你真机**（机器绿≠真机能用）。

## 3. 分期（建议·先拿步骤塌缩的价值）

- **P1 = 件 B（授权后自动推进）+ 件 C-1（没绑就停提示）**：**步骤塌缩的核心价值在这**——授权后一键自动 拆+prepare+链跑。复用现成命令、机器侧可测（stub 全链）。真跑验证用户在场。**先做这个。**
- **P2 = 件 A（接咨询 LM）**：出方案自动化（手填模板 → AI 出）。自包含·低危。
- **P3 = 件 D（触发 + 方案授权 UI）**：把 P1/P2 接成"说目标→审卡→自动跑"的界面。要真机迭代（S2 可用性）。
- **后续**：全局主管 agent（边界/结果复核自动）；真·NL 主对话启动（北极星）；非测试项目（高危#1·锁）。

## 4. 安全死线（本方案·必守）

- **方案授权这道人闸不省**（§0.3 + principles §4「LM 不得绕权限确认执行」）。auto-advance 的**前提是 active 授权已存在**——它只在授权**之后**、**范围内**自动推进，不碰授权本身。
- **圈固定测试项目**（高危#1 轻档·path-lock）；非测试真实项目仍锁死。auto-chain 四护栏（runaway/可中断/审计/可回滚）在。
- **超授权范围必停 + 失败即停 + 手动恢复入口**（§156/§161/§3）。
- **0-diff**：plan/prepare/chain/闸/沙箱本体不改逻辑，只新增编排命令 + 接咨询命令（复用现成）。
- **全局主管复核（边界/结果）先由你演**（角色）；做成 agent 自动 = 后续，不在本方案。

## 5. 开放决策（要你拍）

1. **件 C 绑会话**：C-1（没绑就停提示·安全简单）还是 C-2（自动绑·更顺但要定可用判据）？**我倾向 C-1 起步。**
2. **几道审**：用户确认方案（1 道·人·不省）+ 全局主管复核边界/结果（角色·现你演）——确认这个口径（= 核文档得出的）。
3. **分期**：先 P1（授权后自动推进·拿步骤塌缩）对不对？

## 6. 本方案不做（deferred）

- 真·NL 主对话「按计划开干」启动（北极星·留乙）；全局主管/秘书 agent；非测试真实项目（高危#1）；auto-approve 把方案授权也自动（**违 §0.3·永不做**）；多项目接力。
