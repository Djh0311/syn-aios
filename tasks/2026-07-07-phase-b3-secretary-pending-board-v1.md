# 实现任务包:B3·秘书「待你拍板」汇总面(确定性秒出 + 按需 AI 解释)· 主导线 → 执行线 v1

日期:2026-07-07　性质:**轻档**(前端读模型扩展为主 + 一个薄 agent 模块;单线双面·文件边界 §2.5;死线 0-diff)。Phase B 第三片(收官片),正本 `decisions/2026-07-07-phase-b-advisory-supervisor-and-secretary-v1.md` 第 2 条。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(单线双面)。**子线不 commit。** 全程中文。
- **架构正本(必读·B3 的宪法)**:`docs/workbench-system-architecture-v1.md` §7 秘书核心协作层——秘书=核心协作角色非某界面;职责=汇总状态/整理待确认/提醒风险;**禁区照抄进档案**:不绕确认改事实、不绕主管操作项目、不写长期记忆、不替代审计中心、不当裁判。
- **秘书骨架已存在,B3 是接骨架不是新起**(主导线已核):
  1. 前端派生读模型 `lib/secretaryReadModel.ts::deriveSecretaryContext`(输入 snapshot/workflowState/blackboard/memoryCapture/memoryCandidate 五店·纯前端确定性·已算 pending_permission/blackboard/memory_candidate 计数与 risk_signals);
  2. 呈现 = `components/SecretaryBrief.tsx`(63 行·挂在 `RightDetailPanel.tsx:67` 秘书面板);shell 有「打开秘书」钮;
  3. **它不知道 Phase A/B 长出来的新事**:待批方案(proposal store)、全局主管两类意见(review store)——B3 = 把这两路灌进去;
  4. proposal store 前端已在(App.tsx 作用域内 `projectConsultationProposalStore`);**review store 无前端整店只读口**(现只有 run_* 两命令)→ 需加一条只读 load 命令(照 `loadFormalMemoryStore` 家族先例·加法);
  5. derive 调用点 = `App.tsx:619`(喂新输入就在这一处)。

## 1. 拍板摘要

- **要做的事**:秘书面变成一张「待你拍板的事」清单(全部确定性读盘·零 LM·秒出),旁置一个按需[让 AI 解释现状]按钮(唯一烧额度处·你点才花)。
- **为什么**:定稿第 2 条;5 角色就差它;「少懵」——所有等你的事一处看全。
- **代价**:一轮。读模型扩展 + Brief 扩块 + 一条只读 load 命令 + 一个薄 explain 命令。

## 一句话判据

**「是不是只:derive 加两路输入(方案店/复核店)算『待拍板』清单 + Brief 呈现 + 只读 load 命令 + 按需 explain(readonly consult·零持久)——而秘书全程零写入、零判断、零驱动,现有 secretaryContext 字段语义 0-diff?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 读模型扩展(`lib/secretaryReadModel.ts`·确定性·主体)

- `deriveSecretaryContext` 输入**加法**两项(可选参数·旧调用不炸):`proposalStore` / `supervisorReviewStore`;
- 新增派生块 `pending_board`(待你拍板):
  1. **待批方案**:status=pending_user_confirmation 的方案(计数 + 标题 + 生成时间;非今天生成标「旧」——口径照批卡 stale 判据);
  2. **全局主管提醒**:结果复核 overall=needs_human_check 或 suggested_action=human_verify(带 human_note 首句)+ 批前边界 verdict=mismatch(带 summary 首句)——各计数 + 条目;
  3. **记忆候选待确认**:沿用现有计数,进同一张单(别重复算·引用既有字段);
  4. 每条带**去处提示**(人话:「在交办页批」「记忆中心处理」——不做跳转接线,B3 不碰导航,提示文字即可);
- 现有字段(risk_signals/suggestions/global_summary)**语义 0-diff**——只加不改。

### 2.2 呈现(`components/SecretaryBrief.tsx` + `RightDetailPanel.tsx` 秘书面板)

- Brief 顶部「需要你确认」计数并入 pending_board 总数;新增「待你拍板」列表区(方案/主管提醒/记忆候选三组·空组不渲染·全空显「桌面干净,没有等你的事」);
- 词表:人话,不露 proposal_id/verdict 枚举原文(mismatch→「主管说这方案对不上目标」类人话);「这些是提醒,不是命令」一句边界话保留现有风格;
- **[让 AI 解释现状]** 按钮(放秘书面板·一个就够):loading「秘书正在整理解释…(约 1-2 分钟)」→ 解释文本(前端会话内缓存·再点才重跑)→ 失败一行人话+[重试];**不挡任何东西**。

### 2.3 后端(薄·两条命令)

- `global_supervisor_review_store.rs` **只加**一条整店只读 load 命令(照家族先例·store 本体语义 0-diff);
- **新模块 `secretary_agent.rs`**:秘书档案常量(§7 职责+禁区原文·「你整理和解释,不判断不裁决不派活」)+ 命令 `run_secretary_explain`——**输入全后端盘读**(pending 方案/候选计数/需留意的主管意见——读现成 store,不收前端转述文本),组 prompt → `readonly_codex_consult` → 返回纯文本解释;**零持久化**(解释是即抛的帮助,不是记录——秘书不写为原则;前端缓存足够);失败照 fix8 分类人话;
- registry +2 注册;模块声明照 worker_report 借道先例(lib.rs 0-diff)。

### 2.4 明确不做(§7 同)

秘书跳转接线(导航收编——Phase A 用户否过的雷区,只给文字去处);机会简报(B3+ 另议);解释持久化/入审计;任何写入/确认/派发能力;把 pending_board 喂给其他 agent。

### 2.5 文件边界(越界即停)

- 允许:`lib/secretaryReadModel.ts` / `components/SecretaryBrief.tsx` / `components/RightDetailPanel.tsx` / **`App.tsx` 窄口**(仅 derive 调用点喂新输入 + 新 store 装载,±20 行内·别的地方一行不碰)/ 相关 css / **新** `secretary_agent.rs` / `global_supervisor_review_store.rs`(仅加只读 load)/ `command_registry.rs`(+2)/ `lib/tauri.ts` / `lib/types/*`(加法)/ `tests/` 新文件 + 跑器 1 行;
- **0-diff**:global_supervisor_agent.rs(B1/B2 本体)/ director / consultant / c4_c6 / controller / commands / runner / control_core / worker_report / manual_relay / lib.rs / ProjectJiaobanPanel(交办面与秘书无涉)。

## 3. 安全死线

- 秘书**结构性只读**(explain 走 readonly consult·写盘根空);全程零写入零状态迁移零驱动——比 B1/B2 更严(它连自己的 store 都没有);
- 架构 §7 禁区进档案原文;渲染类真机过;fmt skip_children。

## 4. 验收

- **单测**(derive 纯函数·前端离线):pending_board 三组各自的入选/排除判据(pending vs confirmed 方案·needs_human_check 入/pass 不入·mismatch 入/caution 不入[caution 是提醒过的·别把秘书面堆成噪音]);空店/缺参零炸(向后兼容);现有字段回归断言;
- **后端单测**(secretary mod):explain 输入装配 grounded(stub consult 收到 pending 事实)/ 供给类失败人话;load 命令往返;
- **离线 DOM**:待拍板三组渲染/全空文案/词表(无枚举原文·无「审批」);
- **真机(用户)**:打开秘书面板 → 待拍板单秒出(与交办页实况对得上)→ 点[让 AI 解释现状] → 1-2 分钟人话解释;
- 三闸绿 + §2.5 0-diff 自证 + 计数不降。

## 5. 回交

- §4 证据 + App.tsx 窄口 diff 行数自证 + 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 秘书有任何写入/确认/派发口 / 改 secretaryContext 现有字段语义 / App.tsx 超窄口乱动 / 解释持久化 / caution 也堆进待拍板(噪音) / 导航跳转接线 / 词表露黑话。
