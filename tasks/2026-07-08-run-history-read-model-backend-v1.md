# 实现任务包:工作历史·后端读模型(按单列史·纯只读)· 主导线 → 执行线 v1

日期:2026-07-08　性质:**轻档**(新只读读模型模块 + registry 一行 + 薄前端封装;**零 UI**——UI 半包等整台原型 M1 拍板后另出)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **需求背景(用户 2026-07-08 已拍)**:交办页要有**工作历史**——「中断的、没做的可以直观看到」。UI 长相由整台原型验证中;**后端读模型与 UI 形态无关,先行落地**:把散在各店的「一单交办的一生」拼成一条历史记录。
- **同树另一线在作业**:主导线正在 `manual_relay.rs`(测试区)动手——该文件对本包 **0-diff**,别碰。
- **主导线已核的数据事实(设计以此为准)**:
  1. 方案 = proposals store(`project-proposals.v1.json`):proposal_id / status(pending_user_confirmation·user_confirmed…)/ 目标文本 / created_at_ms / scope_draft(**写根空 = 纯建议**·前端同判据);
  2. 授权 = plan-authorizations store:按 workflow 多条·created_at_ms·**没有 proposal_id 字段**;
  3. 链 = workflow state 内链记录(controller 家·有现成只读口):started_at / 状态 / 节点进度;**链也没存 proposal_id**;
  4. 两道意见 = `global_supervisor_review_store`:结果复核按 (workflow_id, chain_started_at)、边界意见按 proposal_id——**唯一能精确挂回方案的店**;
  5. **跨店关联没有外键,只能按 workflow_id + 时间窗近似**(B2「所批方案取最新 UserConfirmed」同族已知近似·B1 圈轮先例)。**红线:不许为了好关联去改任何写入路径加字段**——那是另一包另拍;本包纯只读,近似就老实近似并在字段里注明。

## 1. 拍板摘要

- **要做的事**:一条命令,给定项目返回按时间倒序的「单」列表——每单一个状态(五态+「批了没跑」),中断/没做一眼可见。
- **为什么**:已拍需求的 UI 无关半边;原型 M1 回来 UI 直接接,不空等。
- **代价**:一轮。一个新模块 + registry 一行 + 薄封装。

## 一句话判据

**「是不是只:新只读模块把现成各店读出来、按 workflow+时间窗拼成单列表返回——零写入、零 LM、零 UI、任何店本体与写入路径 0-diff?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 新模块 `run_history_read_model.rs`

- **命令 `list_project_run_history`**(registry +1):入参 project_root(+workflow_state_path 侧车惯例)/ 可选 workflow_id / 可选 limit(默认 50·返回 total 计数);出参 `Vec<RunHistoryEntry>` 按 created_at_ms 倒序;
- **`RunHistoryEntry`(serde·字段全加法思维设计)**:
  `{ proposal_id, workflow_id, goal_text(首行截断), created_at_ms, state, state_note(人话一句·如停因尾巴/「N 项要看」), advice_only: bool, chain: Option<{started_at, done_count, total_count}>, review_flags: {result_verdict?: String, boundary_verdict?: String}, correlation: "exact"|"time_window" }`;
- **状态推导(确定性规则·一单恰一态)**:
  1. `pending_user_confirmation` → **待批**(非今天生成 → note 标「旧方案」·口径同批卡 stale);
  2. 纯建议(写根空)且无链 → **纯建议**(批没批照 status 注明);
  3. `user_confirmed` + 关联不到任何链 → **批了没跑**(用户点名要看见的「没做」);
  4. 关联链最新态 = 跑着 → **跑着**(带 done/total);
  5. 关联链最新态 = fail-stop/blocked → **卡住**(state_note = 停因人话尾巴);
  6. 关联链最新态 = 完成 → **交货**(有结果复核且含 issue/needs_human_check → note「有 N 项要看」;没有复核记录就不硬造);
- **关联算法(诚实近似)**:同 workflow 内,链按 started_at 归属「其之前最近的已确认方案」(时间窗);边界意见按 proposal_id 精确挂;结果复核按 (workflow_id, chain_started_at) 精确挂链再随链归单;`correlation` 字段如实标 exact/time_window;归属歧义(两方案确认时间同毫秒级)→ 归最近者并在 state_note 加「(归属按时间近似)」;
- **软着陆**:任一店缺失/损坏 → 该店数据缺席、其余照拼、命令不 Err(增益不是闸);空项目 → 空列表。

### 2.2 薄前端封装(零 UI)

- `lib/tauri.ts` invoke 封装 + `lib/types/*` 类型(加法)——**不接任何组件**;UI 半包等 M1 拍板。

### 2.3 明确不做(§7 同)

改任何写入路径/加外键字段;每单详情下钻(口供全文等——列表级先行,详情随 UI 半包);跨项目聚合(首页/秘书那层另说);缓存层(50 条直读足够快,量出来慢再说)。

### 2.4 文件边界(越界即停)

- 允许:**新** `run_history_read_model.rs`(含自测 mod·不进 lib.rs·模块声明照 worker_report 借道先例)/ `command_registry.rs`(+1)/ `lib/tauri.ts` / `lib/types/*`(加法);
- **0-diff**:全部 store 本体(proposals/plan_auth/review/workflow state·**只调现成只读 loader**)/ controller(只调现成只读口)/ director / consultant / global_supervisor / secretary / c4_c6 / commands / runner / **manual_relay(另一线在作业)** / lib.rs / 前端组件全部。

## 3. 安全死线

- **纯只读**:零写入、零状态迁移、零 LM、零审计事件(读模型不留痕是对的);
- fmt skip_children;人话词表(state_note 不露黑话)。

## 4. 验收

- **单测**(模块自 mod·夹具照 director 测试家族造店):六种状态各至少一例(含**批了没跑**与**纯建议**);时间窗归属(两方案两链归对)+ 歧义标注;stale 旧方案 note;交货+复核 issue →「有 N 项要看」/无复核不硬造;店损坏软着陆;倒序+limit+total;
- 三闸绿(cargo/typecheck/build——typecheck 证薄封装类型通)+ §2.4 0-diff 自证(`git status` 全列)+ 计数不降;
- 真机不需要(零 UI);数据正确性以单测夹具 + 主导线对线上店抽查为准。

## 5. 回交

- §4 证据 + 关联近似的实测说明(线上真店抽 3 单核归属对不对)+ 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 任何写入路径被碰(含「顺手加个 proposal_id 字段」——明令另拍)/ 关联装 exact 不标近似 / 店损坏 Err 断面板 / 接了 UI / 动 manual_relay / 词表黑话。
