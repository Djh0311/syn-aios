# 五站快车道:到「真实 AI 角色干真活」的排布正本 v1

日期:2026-07-11 · 用户已拍(同日,连同 `decisions/2026-07-11-machine-ruling-dichotomy-v1.md`)。
上承:主管编排模式提案(`docs/plans/2026-07-11-supervisor-orchestrator-mode-proposal-v1.md`,方向已拍·本文是其排布落地)+ 完整路线图(`2026-06-27-complete-workbench-phased-roadmap-v1.md`,Phase 映射见 §末)。**取代 `CURRENT.md` §三.1 旧工作序列**(历史轨迹在 git)。

## 重心声明(先于一切排序)

**站 3b 是工作台第一次在真实项目完成受控交付，是低风险真实控制通路验收；它不是多 worker 编排价值验收。** 站 1-2 是通往这次真实交付的最短路径。编排与治理后续如何选路，以 `decisions/2026-07-13-orchestration-and-governance-two-axis-routing-v1.md` 为准。

## 治理姿势(全程)

两分法(见决策):确定性越权=拦 · LM 意见=留证上脸 · 否决权只在人。人闸/path-lock/高危清单/审计一寸不动;账本反而变细(主管每次工具调用=一条审计事件)。

编排和治理分轴判断：是否启用主管看协调复杂度，护栏强度看后果风险；简单任务默认单 agent，不能因“只读”一词自动判定既不需编排也无治理风险。

## 主干五站

| 站 | 交付 | 前置 | 进下一站的证据 |
|---|---|---|---|
| **1 工具房+契约**(现在·两线并行) | 执行线:MCP 工具面包(七工具=现成命令的带闸包装·调用即落账);总指导:主管契约 v1 草案(**一页纸死线**)+ 试点验收数字预写死 | 无(前置已清) | 工具面机器验过;契约用户核字 |
| **2 主管上岗** | 契约+按单开关;固定测试项目、只读单先行;新旧模式对照跑 | 站1 | 账本对照:越权 0 / 卡住率 ≤ 老管线 / 追问在预算内 / 每单可回放 |
| **3 账本说话·两扩** | **3a 写单试点已完成**（fail-closed、三方职责分层、控制核心桥、全新 worker 会话与 v7 真跑闭环）；**3b 已在 `/Users/yoyi/Documents/mario test` 完成只读真实项目闭环**（沙箱 read-only·零写根·单 worker·按项目一次授权） | 站2 数字达标 | 3a：`PASS__RISK_CLEANED__READY_FOR_3B`；3b：`PASS__SINGLE_WORKER__ZERO_WRITE__NOT_COMMITTED` |
| **4 Phase D 按痛点拉动** | 记忆生命周期/模板/R3 真库翻闸——真项目疼一件做一件,**不整块推** | 3b 有真实使用 | 每件各自验收 |
| **5 Phase E 乙·生产档** | 写真实项目+自动连环+重护栏(照旧重档) | 1-4 账本攒够信任 | 用户授权那一下 |

## 支线(不挡主干,空窗插)

- manual_relay 首发抽风定点修(小包/卡片);
- **备份补齐**(记忆库+store 146M 零副本+push 未答——留证哲学自己的保险,后续空窗做);
- 浏览器无头 MCP 小件(将来即 worker 的一件工具);
- 回放视图小件(决策条款 5,试点后排);
- **明确不做**:旧管线毒条款不修(试点即替身);Phase C 重定义继续挂(主管编排落地后再看)。

## 当前关口（2026-07-13）

- 站 3a 已由固定测试项目 v7 真跑完成：全新 authorization、work item、supervisor run 与 native worker thread；唯一 worker、一次执行、零追问；动作顺序为 `dispatch_worker -> inspect_worker -> finalize(pass) -> report_user`。权威证据：`evidence/2026-07-12-orchestrator-station3a-control-core-bridge-v1.md`。
- v7 后的三轮独立复核先发现旧 `binding_id` 截断碰撞，再发现首轮迁移漏同步 dispatch 引用，最后排除历史引用被误改绑的算法边界；已在 3b 前完成 SHA-256 身份、重复校验、仅真实 legacy 候选迁移、歧义拒写、SQLite 双边守恒回归和两次写前备份迁移。当前 71 条 binding ID 全部唯一，352/352 条 dispatch 引用均存在且可解析，零孤儿。授权快照/任务包指纹、fresh provenance 和 worker 回程账本一致性也已收紧。
- 站 3b 已在 `/Users/yoyi/Documents/mario test` 完成一次单 worker、零写根、read-only 的真实 UI 闭环：`dispatch_worker → inspect_worker → finalize(pass) → report_user`，项目 7 个内容文件前后 SHA-256 一致。状态为 `PASS__SINGLE_WORKER__ZERO_WRITE__NOT_COMMITTED`，权威证据：`evidence/2026-07-13-orchestrator-station3b-mario-test-readonly-real-run-v1.md`。
- 用户在 3b 完成后明确要求先停。当前不自动进入站 4，不启动 SQLite M0-M6，也不把本次授权外推到写单、其它真实项目、多 worker 或自动连环。新对话交接：`handoffs/2026-07-13-station3b-real-loop-pass-and-architecture-pause-handoff-v1.md`。

## 架构评审接入(2026-07-13·正本 `docs/2026-07-13-architecture-review-v1.md`)

评审发现按「卡哪站」接住（报告是正本，此处只挂门）：

- **主管编排真跑后仍未覆盖的能力**：follow-up 代码已补“新报告读回 + 旧 inspect 失效”，但 3b PASS 单 `follow_up_count=0`；真实“派工→追问→读回追问结果”仍需单独验证，不能借 3b PASS 冒领。
- **主 store 并发止血已落 WIP**：revision CAS 已把静默丢写改成显式 `workflow_state_revision_conflict`；业务级重放、stale lock 与整本改写仍未解决。
- **卡「Phase D · R3 翻闸」的门**：新 launcher 材料已移入 `runtime-artifacts/`，但 91 个历史 txt 尚未迁移；更高优先级的阻断是当前 SQLite 迁移合同漏五组主状态数组、多个 sidecar 与主管账本。未补齐前禁止翻闸。
- **贯穿债(未排期)**:god file(director 7470/Jiaoban 3845)侵蚀「核实物」、安全谓词人肉同步——按职责重切、收单源,待用户拍先动哪件。

## 提速三点(不碰治理强度)

① 站1 两线并行(代码归执行线、契约归总指导+用户核);② 验收数字前置(到站翻账本,不再吵);③ 3b 用"只读=物理不可写"换时间,把干真活从 Phase E 提前到站 3。

## 与路线图 Phase 映射

站1-2 = 提案 §6 的两包;站3b+4 ≈ 路线图 d·真项目试点 + Phase D(改为拉动式);站5 = Phase E。roadmap 仍是阶段语义正本,本文是当前排布正本;per-task 状态以 `CURRENT.md` 为准。
