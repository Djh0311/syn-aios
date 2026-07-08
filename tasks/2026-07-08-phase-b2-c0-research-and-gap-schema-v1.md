# 实现任务包:B2·C0 调研与差量对照(参考消化 + 现状↔任务包正本逐字段差量)· 主导线 → 执行线 v1

日期:2026-07-08　性质:**纯调研·零产线代码**(产出 = 一份研究报告 + 一张差量对照表;子线不 commit)。Phase B2 首包,正本 `decisions/2026-07-08-phase-b2-execution-loop-final-v1.md`。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**。全程中文。**红线:本包零代码、零产线改动**;唯一允许的"动手"是只读实测(见 §2.1,圈 temp/固定测试项目)。
- **背景**:Phase B2(执行闭环深化)已定稿——每任务独立会话/任务包 v2/求助通道/主管总结终标。动工前要两样底料:①外部设计参考消化;②现状与纸面正本的精确差量。这就是 C0。
- **设计正源与铁边界(用户三令五申 + 架构 §5.10 原文)**:Claude 与 Codex 的多 agent 协作是**参考**,消化后**转化为工作台自建能力**——不拿 codex thread id 当工作台业务主键、不把 parent/child/subagent 关系直接当角色关系、不把线程状态当 workflow state、**派发必须来自项目主管任务包 + 控制核心授权,不能来自 agent 自治 spawn**、不让原生协作绕过任务包/权限/审计/用户确认。报告里这个清单**原文照抄**作你的判据章。

## 1. 交付物(两件)

### 1.1 研究报告 `docs/research/2026-07-XX-agent-collab-transfer-reference-for-b2-v1.md`

体例照 odysseus/paseo 先例(可吸收为约束 / 明确不吸收 / 阶段边界):

- **Claude 多 agent 协同设计**(公开资料+库内先例):上下文隔离/单进单出信道/结构化契约(schema 强制+校验重试)/编排器中转数据/对抗式复核——**逐条标注哪些我们已落地**(worker 回程契约 d2dba24 即其子集)、哪些 B2 各切片要用;
- **Codex 当前 CLI 多线程/会话能力实测**(只读为主·真起会话限 temp 或固定测试项目):`codex exec`/`resume`/新线程行为/子线程(若有)/会话元数据——对照库内基线 `docs/plans/2026-06-18-codex-native-conversation-behavior-baseline-v1.md` 列**变化点**(CLI 一个月更过版);每条注明「工作台自建层怎么用/不用它」;
- **车间模型修订注记**:「会话跟节点走」→「会话跟任务走」(定稿已记)在报告中作为设计前提复述。

### 1.2 差量对照表(并入报告或独立附录)

拿**现状实物**逐字段对照**任务包设计正本** `docs/workflow-task-package-design-v1.md`:

- 现状盘点侧:`worker_report.rs`(契约/口供)/ director 的 planned_task(title/objective/depends_on/acceptance/report_format)/ prepare 物化 / 链步骤(DirectorChainStep 含 report_*)/ 审计事件族 / 授权 scope / M4 `TaskMemoryPacketBuilder`(A 线遗产·**核可用性**);
- 正本侧:§3.4 TaskPackage 十二项 / §3.5 WorkflowLedgerEntry(13 种 entry_type)/ §3.6 SubagentReport(含 direction_risk·permission_requests)/ §3.7 ReviewResult / §3.8 WorkflowException / §4.3–4.8 生命周期 / §5.2 节点状态机(waiting_permission·waiting_decision·reviewing·returned + 硬规则);
- 每行三态:**已有(在哪)/缺(C 几补)/语义偏(现状怎么偏·迁移建议)**;最后给**迁移次序建议**(哪些字段随 C1/C2/C3/C4 各自落)。

## 2. 边界与红线

### 2.1 实测边界
- codex 实测只许:读版本/帮助/元数据;真起会话仅 temp 目录或 `/Users/yoyi/codex-workflow-mario-test`(轻档);**不碰产线 store、不碰 `~/.codex` 写与凭据**;每次实测记录命令与输出摘要进报告。

### 2.2 文件边界
- 允许:`docs/research/` 新报告一份(+附录);
- **0-diff:其余一切**(产线代码/store/tasks/decisions/CURRENT——回写归主导线)。

## 3. 验收

- 报告体例三章齐(吸收/不吸收/阶段边界)+ §5.10 判据章原文在 + 基线变化点如实(没变也写没变);
- 差量表覆盖正本五对象+生命周期+状态机,每行三态标注,零"大概/可能"(不确定就标「未核·怎么核」);
- M4 遗产可用性实答(能接/不能接+因由);
- 全程零产线 diff(`git status` 自证,仅新增报告文件)。

## 4. 回交

报告+差量表路径 → 主导线核实物 → 主导线落**中转协议差量定稿决策** → C1 包。**子线不 commit。**

## 7. 不接受为

写任何产线代码 / 实测越出 temp·测试项目 / 拿"Codex 原生能力"当结论替代工作台自建层设计 / 差量表含糊(每行必三态) / 替主导线写定稿。
