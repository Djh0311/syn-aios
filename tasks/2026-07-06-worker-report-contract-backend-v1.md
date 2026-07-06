# 实现任务包:worker 回程契约(报文从"嘱咐"变"契约":强制 JSON 块 → 链解析 → 落库 → 出面)· 主导线 → 执行线 v1

日期:2026-07-06　性质:**轻档**(协议/解析/消费侧加缝;执行判决体/闸/runner 本体 0-diff)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **背景(用户拍板的方向)**:工作台派发信息结构对齐 Claude 多 agent 协同协议——**去程已齐**(objective 自包含+scope 注入+acceptance+report_format);**回程只是嘱咐**:director prompt 明写"report_format 好让链/主管 parse 了往下走",但链只看 `dispatch.state=="completed"`,worker 报文零解析;`record_worker_structured_report_at` 只挂手动命令,自动链 0 引用。本包把回程闭合成契约:**worker 被强制"最后输出且仅输出一个 ```json 块"(同 consultant/director 的成熟套路)→ 链解析 → 现成登记机器落库 → 链结果携带摘要**。
- **Claude 侧协议对照(设计基准)**:出口 schema 强制+校验重试;失败软着陆(报文缺失≠任务失败,槽位 null+警告);agent 间不直连、数据经编排者中转。codex 无工具层强制,等价物 = json 块+抠取+解析+兜底(仓里已熟)。
- **主导线已核的三接缝(直接用,省你查)**:
  1. 最后消息管道:runner 侧 `last_message_path` 全程在(`codex_local_runner.rs:37/71/137/165`——`CodexLocalExecutionRequest.last_message_path: Option<String>`);worker resume 的结果/派发记录里怎么暴露(全文?截断摘要?路径?)= **前置核查①**;
  2. 登记机器:`record_worker_structured_report_at(path, WorkerStructuredReportInput)`(c4_c6:340,带 `validate_worker_structured_report_input`)——**只调不改**;
  3. 物化点:c4_c6 ~2365 `"goals": [task.objective]` 一带(artifact 构造·带 forbidden_actions)——契约文案追加处。
- **一句话**:新模块定义报文契约与解析;prepare 物化时把契约段确定性追加进任务包 artifact;链每任务完成后读 worker 最后消息 → 解析 → best-effort 落库 → 链结果步骤带报文摘要;解析失败**不改成败**、出警告留原文尾巴。

## 前置核查(动手前做,两个都有 stop-gate)

1. **worker 最后消息的可读源**:从链侧拿到的 `WorkflowNodeDispatchResult`(或其 dispatch 记录)里,worker 最后消息是**全文字段、文件路径、还是截断摘要**?json 块要完整可解析——若唯一可得的是截断摘要且会切断 json:优先看结果里有没有 last_message 路径可读文件;**若两者都没有、补捕需改 `commands.rs`(execute)或 runner 本体 → 停、回主导线**(死线,不许动)。
2. **登记机器的副作用**:读 `WorkerStructuredReportInput` 全字段 + `record_worker_structured_report_at` 对 work_item **state 的影响**——链完成路径已管理状态,若登记会做与链冲突的状态跳转(如强推 ready_for_review 之外的迁移)→ **停、回主导线**(不许为落库改状态机或双写状态)。

## 1. 拍板摘要

- **要做的事**:回程从"顺序连跑"升到"结构化协同"的第一块砖——机器不再只看 exit,开始**读懂 worker 说了什么**并归档。
- **为什么**:用户拍板对齐 Claude 协同协议;这也是 Phase A 一直没做的"worker 汇报"半边。
- **代价**:一轮·后端。新模块 + 两处加缝(物化/链消费)。**"产物喂下一步"不在本包**(见 §5——那是下一块砖,别顺手做)。

## 一句话判据

**「是不是只:新模块定契约与解析 + prepare 物化追加契约段 + 链完成后读报文/解析/调现成登记/带摘要——而成败判定、重试语义、执行闸、runner 本体、登记机器本体、c4_c6 判决体全 0-diff?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么(代码边界逐文件写死)

### 2.1 新模块 `worker_report.rs`(**新文件**·照 `store_hygiene.rs` 先例经 `command_registry.rs` 挂载·测试放自己的 `#[cfg(test)] mod`,**不进 lib.rs**)
- `WorkerReport { did: String, outputs: Vec<String>, status: String, evidence: Vec<String> }`(serde,全 `#[serde(default)]`——契约字段:做了啥/产出在哪(路径列表)/成败(done|partial|failed)/怎么证明);
- `pub(crate) const WORKER_REPORT_CONTRACT_TEXT: &str`——追加给 worker 的契约段(人话,同 consultant prompt 风格):「干完后,最后输出**且仅输出**一个 ```json 代码块,严格 `{"did":"…","outputs":["…"],"status":"done|partial|failed","evidence":["…"]}`;outputs 写产出文件的完整路径;没有产出就空数组;别在块后再写任何字」;
- `pub(crate) fn parse_worker_report(raw: &str) -> Option<WorkerReport>`——复用 crate-root 现成 `consultant_extract_json_block` 抠块 + serde 解析;抠不到/解析失败 → None(**不 Err**——软着陆语义留给调用方);
- 本模块**零 IO、零状态写**(纯协议),落库调用在链侧。

### 2.2 `c4_c6_workflow_governance_entrypoints.rs`(**仅物化区**·刀2 同款纪律)
- artifact 构造处(~2365 `goals` 一带):`goals` 追加两条——task 的 `report_format` 各项(主管拆的·原有数据一直没人用,现在接上)+ `WORKER_REPORT_CONTRACT_TEXT`。**确定性追加,不经 LM**;
- **0-diff 圈界**:guard 评估/authorized-blocked 分流/`project_director_authorization_context`/`record_worker_structured_report_at` 本体/`validate_worker_structured_report_input`——**逐函数 0-diff,回交贴自证**(同刀2 的"唯一删行"级标准)。

### 2.3 `director_agent.rs`(链消费缝·**只在任务完成分支**)
- `run_director_task_chain` 的 `Ok(result) if state=="completed"` 分支(~538 区)加缝:按前置核查①的源取 worker 最后消息全文 → `parse_worker_report`:
  - **Some(report)** → 组 `WorkerStructuredReportInput`(work_item_id 从任务 annotate 字段取;字段映射按前置核查②的真实结构,老实填,缺的用报文原文兜)→ 调 `record_worker_structured_report_at`,**best-effort**:落库失败 → warning「任务 X 报文落库失败:…」,**不影响链继续**;`DirectorChainStep` 加 `report_summary: Option<String>`(did + status 一句话,serde 加法·前端渐进接);
  - **None** → warning「任务 X 完成但未按契约交报文(原文尾:{200 字}」,`report_summary=None`,**任务仍算完成**(报文缺失≠干活失败——tier-1 codex 服从率非 100%,软着陆);
- **不改**:成败判定/`is_tier1_early_exit` retry/停链/审计既有条目(报文事件是**新增** append)。

### 2.4 明确不碰
`commands.rs`(execute)/`codex_local_runner.rs` 本体/`control_core`/`workflow_chain_controller` 本体/两 store/`manual_relay`——**byte-0-diff**。前端(交货脸显示 report_summary)= 另开 UI 小包。

## 3. 安全死线

- 报文**只归档不驱动**:解析结果不改变任何执行决策(不重试/不跳步/不改状态迁移路径)——协同的"读"先落地,"据此行动"是下一阶段;
- 登记走**现成校验机器**(validate 在里面),不绕、不直写 store;副作用冲突 → 前置核查② stop-gate;
- worker 契约段是**确定性文本**追加,不给 LM 发挥空间;
- 全部死线 0-diff;fmt 只本包文件(skip_children·老规矩)。

## 4. 验收(执行线自己验)

- **单测·契约**(worker_report 自己的 mod):合法块 → 全字段解析;缺字段 default;无块/坏 json → None;块前后有废话 → 照抠(复用抠取器的行为)。
- **单测·物化**:prepare 后 artifact goals 含 report_format 各项 + 契约段(断言原 goals[objective] 仍在首位)。
- **单测·链消费**(stub):worker 最后消息带合法块 → store 里报文记录在(经登记机器·断言其校验真跑过)+ step.report_summary 对;无块 → completed 仍 completed + warning + summary None;登记失败注入 → warning 不断链。
- **真跑**(`#[ignore]`·测试项目·真 codex):真 worker 收到契约段后**真的交出 json 块**(这是契约对真模型服从率的验证)→ 报文落库、核实物(读 store 记录 + 最后消息原文);若真 codex 偶发不交块 → 软着陆路径真机生效也算过(如实报)。
- **regression**:计数不降;§2.2/§2.4 逐函数与整文件 0-diff 自证;fmt。

## 5. 本包不做(deferred·别顺手)

- **产物喂下一步**(把 task A 的 outputs 注入 task B 的 prompt——要动 artifact 的链间更新,单独一刀);审查/主管**消费**报文做复核(Phase B 角色活);报文驱动重试/分支;交货脸显示(UI 小包);报文进正式记忆。

## 6. 回交

- 跑 §4;回交列:前置核查①②的实答(最后消息源/登记副作用)、新模块 API、物化追加原文、链缝落点、软着陆三态证据、真跑服从率实况、0-diff 自证、计数 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 不接受为:报文影响了成败/重试/状态迁移 / 绕过登记校验直写 store / 动了 execute·runner·判决体 / 契约文本让 LM 生成 / 测试塞进 lib.rs / 把"喂下一步"顺手做了。
