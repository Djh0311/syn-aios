# 实现任务包:B1·全局主管读口供并反应(结果复核意见上交货脸)· 主导线 → 执行线 v1

日期:2026-07-07　性质:**轻档**(新只读 agent + 新 sidecar store + 前端复核区;单线双面·文件边界 §2.5;死线 0-diff)。Phase B 第一片,正本 `decisions/2026-07-07-phase-b-advisory-supervisor-and-secretary-v1.md`。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(单线双面:后端新模块 + 前端复核区)。**子线不 commit。** 全程中文。
- **背景**:Phase A 已把数据面铺好——worker 口供(契约 json)经链解析、由 `record_worker_structured_report_at`(`c4_c6_workflow_governance_entrypoints.rs:340`·登记机器·append-only)落库,step 级 `report_summary/report_warning/report_status` 已上交货脸(黄牌)。**但没人读口供并反应**。B1 = 全局主管 agent 读本轮口供+证据+所批方案 → 出复核意见 → 上交货脸。**意见不是闸**(定稿四条之一):建议动作配按钮、按钮用户点。
- **主导线已核的接缝(直接用)**:
  1. 复核 agent 的 LM 通道 = 现成 `readonly_codex_consult`(consultant 家族·只读 sandbox·06-25 guard 豁免先例);json 提取照 `consultant_extract_json_block` / `worker_report::parse_worker_report` 先例;
  2. 链状态只读口 = `workflow_chain_controller.rs:655 get_project_workflow_chain_status`(**只调不改**·manual_relay「只调未改」先例);
  3. 口供落库点 = `record_worker_structured_report_at`(c4_c6:340)。**开工先核查它写向哪个 store**(疑 `observation_store`,未定谳)并用**现成只读 loader** 读;没有现成只读口 → **停、报回**(别自造第二套读法、更别碰 c4_c6);
  4. sidecar store 家族先例一排(`plan_authorization_store.rs` 等):原子写/备份/损坏跳过的模式照抄;新模块声明照 `worker_report.rs`/`store_hygiene.rs` 先例(它们都没碰 lib.rs);
  5. 供给类失败分类 = 现成 `classify_codex_provider_failure`(fix8·只调)。

## 1. 拍板摘要

- **要做的事**:交货后自动出一份全局主管复核意见(每任务点评·黄牌必评·总判+建议动作),async 上脸不挡交货;按轮幂等防重烧。
- **为什么**:中间版 §0.6 第 9 步(终局复核)advisory 形态落地;上周口供逮到的假完成(「无法启动浏览器·未完成手动验收」)从「躺在库里」变「有人读了、给了说法」。
- **代价**:一轮。后端一个新 agent 模块 + 一个新 store 模块 + 一条命令;前端交货脸一个复核区。

## 一句话判据

**「是不是只:新增只读复核 agent(读盘→consult→意见落自己的新 store+审计)+ 交货脸复核区(意见+建议按钮·按钮走现成动作)——而链/闸/判决体/口供登记机器/工作流状态全 0-diff、复核不驱动任何状态?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 后端·新模块 `global_supervisor_agent.rs`(新文件)

- **档案(prompt)常量**:角色=全局主管(复核最终结果·不是审批);输入=用户批的方案(goal/steps/scope)+ 本轮任务清单与执行态 + 每任务口供(did/outputs/status/evidence)+ warnings;要求:**每任务一句点评、黄牌(status≠done/没交口供)必评**、保守判(拿不准 → needs_human_check,不装确定)、全中文人话;契约段照 `WORKER_REPORT_CONTRACT_TEXT` 风格:最后输出**且仅输出**一个 ```json 块;
- **输出 schema**(serde 全 default 软着陆):`{ overall: "pass"|"needs_rework"|"needs_human_check", tasks: [{title, verdict: "ok"|"issue", comment}], summary, suggested_action: "none"|"replan"|"human_verify", human_note }`;
- **命令 `run_global_supervisor_review`**(注册进 `command_registry.rs`,+1 处):入参 project_root(+workflow_state_path 侧车定位照现有命令惯例)/workflow_id/chain_started_at;
  - **幂等**:该 (workflow_id, chain_started_at) 已有复核记录 → **直接返回,不再 consult**(成本护栏);入参加 `force: bool` 供[重新复核];
  - 流程:盘上读方案(proposal store)+ 链记录/状态(现成只读口)+ 本轮口供(§0.3 核查后的现成 loader)→ 组 prompt → `readonly_codex_consult` → 提取 json → 落新 store + 审计事件 `global_supervisor_review_recorded`;
  - **输入全从盘读,不收前端转述**(事实核心落盘·principles §4);
  - 失败三分(照 fix8 先例):供给类 → `classify_codex_provider_failure` 人话;解析失败/consult 失败 → 落「复核不可用:原因」记录(可重试);**任何失败不 Err 断面板**——返回结构里带 status;
  - **记录里写 model/档案版本字段**(定稿·§10-1 零成本半边)。
- **新 store 模块 `global_supervisor_review_store.rs`**(新文件·sidecar `global-supervisor-reviews.v1.json`):按 (workflow_id, chain_started_at) 存取;原子写/写前备份/损坏跳过照家族先例;**append/upsert 只此一店**——复核不写任何其他 store。
- 测试进两个新模块**自己的 `#[cfg(test)] mod`**(照 worker_report 先例·不进 lib.rs):见 §4。

### 2.2 前端·交货脸复核区(`ProjectJiaobanPanel.tsx`)

- 交货翻脸(thisRoundChainStatus 判完成)→ **自动** invoke(fire-and-forget + 结果态缓存进 JiaobanRunCache,重挂载先读缓存/再按幂等键补拉):
  - loading:「全局主管复核中…(约 2-7 分钟,不影响交货)」;
  - 意见到:区块「全局主管意见」——总判一行(通过=绿一行/建议打回/建议亲验)+ 每任务点评列表 + 建议动作按钮:`replan` → **[按建议打回重拆]** 调**现成** rePlan 动作;`human_verify` → 显 human_note(人话「建议你亲验:…」);`none` → 无按钮;
  - 失败:「复核不可用(人话原因)」+ **[重试]**(force);**绝不零出路**;
- **词表**:「全局主管意见/复核意见」;禁「审批/通过审批」措辞(意见不是闸);不露 thread_id/store 黑话;
- 无本轮链或旧数据 → 整区不渲染,零回退。
- invoke 封装进 `lib/tauri.ts`,类型进 `lib/types/`(加法)。

### 2.3 明确不做(§7 同)

自动打回/自动重拆;复核不过拦交货;批前边界意见(B2);秘书面(B3);卡住脸复核(首版只交货态);复核结果喂记忆/喂下一单(另议)。

### 2.4 触发与成本(定稿第 3 条,照做)

每单自动、async、不挡交货;幂等防重烧;[重新复核]才强制重跑。

### 2.5 文件边界(越界即停)

- 允许:**新** `global_supervisor_agent.rs` / **新** `global_supervisor_review_store.rs` / `command_registry.rs`(+1 注册)/ 模块声明(照 worker_report 先例的那处,最小加法)/ `ProjectJiaobanPanel.tsx` / `projectWorkflowSidePanel.css` / `lib/tauri.ts` / `lib/types/*`(加法)/ `tests/` 新离线 DOM 文件 + 跑器 1 行;
- **0-diff**:`c4_c6_workflow_governance_entrypoints.rs`(读它写的数据≠碰它)/ `workflow_chain_controller.rs`(只调 655)/ `commands.rs` / `codex_local_runner.rs`(readonly_codex_consult 只调)/ `control_core.rs` / `director_agent.rs` / `consultant_agent.rs` / `worker_report.rs` / `manual_relay.rs` / 两执行 store / `lib.rs` / 其余一切。

## 3. 安全死线

- 复核 agent **结构性只读**(readonly consult·写盘根空);它唯一的写 = 自己的新 store + 审计;
- **意见不驱动**:后端不因 verdict 改任何工作流/链/工作项状态;前端按钮全走现成用户动作;
- 高危清单 0 接触;真跑只打固定测试项目;渲染类**必须真机过**;fmt 只本包文件(`--config skip_children=true`)。

## 4. 验收

- **单测**(两新模块 mod):schema 三态(合法/缺字段 default 容忍/坏 json→不可用记录);store 往返+损坏跳过;**幂等命中不重跑**(stub consult 计调用次数=1);force 重跑;供给类失败→人话分类;
- **真跑**(`#[ignore]`·测试项目·额度在):对着最近一轮真链跑一次 → 意见 grounded(引用真口供内容,如浏览器验收那单)·记录落盘·审计在·`.codex` auth 未碰;
- **离线 DOM**:复核区四态(loading/意见含建议按钮/不可用+重试/pass 绿行)+ 词表断言(无「审批」);
- **真机**(用户):跑一单到交货 → 复核区自动出现 loading → 数分钟后意见上脸;造/遇一单黄牌 → 黄牌任务有点评、建议按钮可点([按建议打回重拆] 真起重拆);
- 三闸绿 + 0-diff 自证(§2.5 全名单)+ 计数不降。

## 5. 回交

- §4 证据 + §0.3 口供落库点核查实答(哪个 store/哪个 loader)+ 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 复核驱动状态(自动打回/拦交货)/ 碰 c4_c6 或 controller 本体 / 自造口供第二读法 / 前端转述当 LM 输入 / 不幂等每次翻脸都烧一次 / 词表写成「审批」/ 失败 Err 断交货脸。
