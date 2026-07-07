# 实现任务包:交办「看原始对话」桥·主路径入口(批卡/跑中/交货一键钻进智能体页)· 主导线 → 执行线 v1

日期:2026-07-06　性质:**轻档**(纯前端两文件·文件边界 §2.3;零后端、零新穿线)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(UI 面)。**子线不 commit。** 全程中文。
- **背景**:定稿决策(`decisions/2026-07-02-project-jiaoban-tab-final-design-v1.md`)承诺「每单活可一键『看原始对话』钻进智能体页;卡上『用哪个对话干』即桥的起点」——主路径一直没接,2026-07-06 用户拍板补做(决策文修订记录在)。
- **地基已全在,一根线都不用你穿(动了 = 越界)**:
  1. `ProjectJiaobanPanel` 已收 `onOpenAgentSession: (threadId: string) => void`(props 类型 ~38 行·解构 ~146 行);
  2. App 级实现已在:`setFocusedAgentThreadId(threadId) + setActiveView("agents")`(App.tsx ~676);`AgentView` 有 `focusedThreadId` 消费——跳过去该会话会被选中;
  3. 面板内已有先例调用:非测试项目兜底脸「去智能体直连」(~599 行 `latestSession && onOpenAgentSession(latestSession.thread_id)`)——**照这个抄**。
- **会话标识现状(设计依据·只读别改语义)**:`sessionChoice` 状态 = 真 thread_id(选了「接现有:X」)| `NEW_SESSION_CHOICE` 哨兵(选了新建)| null(未定);经 JiaobanRunCache 跨重挂载保留。**哨兵单的真会话 id 前端拿不到**(后端起跑才真建会话),别去猜、别去盘上反查。

## 1. 拍板摘要

- **要做的事**:交办主路径上,凡能确定对话的地方给一个「看原始对话」入口——一键跳智能体页并选中该会话。
- **为什么**:定稿正文承诺;「卡了有路」——活干歪时看原始对话是天然下钻路;Phase B 全局主管读口供后,用户抽查原始对话的需求只增。
- **代价**:一轮,两文件。

## 一句话判据

**「是不是只:面板里加入口、走已存在的 `onOpenAgentSession`——App / AgentView / ProjectsView / 穿线 / 后端全 0-diff?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 入口(全在 `ProjectJiaobanPanel.tsx`)

- **批卡·会话收纳行**(`JiaobanSessionPicker` 区):`sessionChoice` 为真 thread_id(=选了「接现有:X」)→ 行尾小链接/小按钮**「看原始对话」**→ `onOpenAgentSession(该 thread_id)`;选「开个新的」或未定 → **不显**(会话还没生,没得看——诚实);
- **干 / 交货 / 卡住脸**:同判据——`sessionChoice` 是真 thread_id 时显同款「看原始对话」;哨兵单(新会话)→ 用现成 `latestSession` 兜底,**词表改「看最近对话」**(最近≈本单但不保证,不许说成"本单对话"——诚实词表);`latestSession` 也没有 → 不显,零回退;
- **词表**:主词「看原始对话」(定稿原词)/ 兜底词「看最近对话」;不露 thread_id / 不露黑话;
- 样式进 `projectWorkflowSidePanel.css`,跟现有交办小按钮族一致,**别抢 [允许并开始] 等主按钮的视觉**。

### 2.2 明确不做

- 新会话单起跑后反查真 thread_id(后端回传 / 读盘——都不做,留以后有真需求再议);
- 智能体页内任何改动(`focusedThreadId` 选中机制现成);
- 入口出现在交办面之外的任何地方。

### 2.3 文件边界(越界即停)

- 允许:`ProjectJiaobanPanel.tsx` / `projectWorkflowSidePanel.css` / `tests/` 新离线 DOM 文件 + 跑器 `run-offline-interaction-test.mjs` 加 1 行;
- **0-diff**:`App.tsx` / `ActiveWorkbenchView.tsx` / `ProjectsView.tsx` / `AgentView.tsx` / `lib/**` / `src-tauri/**` 全部。

## 3. 安全死线

- 纯呈现 + 导航,零后端、零状态迁移;不碰人闸/合流/链;`sessionChoice` **只读**(选择语义是方案a哨兵修的果实,别搅);
- 渲染类**必须真机过**(前端改动 app 里 Cmd+R 即可)。

## 4. 验收

- **离线 DOM**(新文件入现有 harness):① 选「接现有」→ 收纳行入口在·点击后回调收到该 thread_id;② 选「开个新的」→ 批卡不显入口;③ 交货脸:existing 单显「看原始对话」/ 哨兵单+有 latestSession 显「看最近对话」/ 两者皆无零渲染;
- **真机**:批卡选现有会话 → 点「看原始对话」→ 跳智能体页**且该会话选中**;交货脸同验一次;
- 三闸(tsc / offline / build)绿 + 0-diff 自证(`git diff --name-only` 只两文件 + 测试 + 跑器)。

## 5. 回交

- §4 证据(真机截图:跳转后智能体页该会话选中态)+ 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 自己新穿一条回调 / 改 App 或 AgentView / 哨兵单硬造 thread_id / 词表吹大(「本单对话」)/ 入口挤进主按钮位 / 顺手改会话选择语义。
