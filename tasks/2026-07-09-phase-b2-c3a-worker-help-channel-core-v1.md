# 实现任务包:B2·C3a worker 求助通道核心(实现A成唯一真源)· 主导线 → 执行线 v1

日期:2026-07-09　性质:**较重**(改 worker 回程契约+消费分支+链行为·核心数据流)。主导线已 measure-first 亲读 worker_report 契约链(见 §0)。正本:七拍 `decisions/2026-07-08-b2-transfer-protocol-gap-final-v1.md` 拍①③ + C0 §5.3 + 任务包设计 §3.6。**C3 拆两片·本片=核心;C3b(收敛启发式+cancelled 终态)另包后置。**

## 0. 接手须知(冷启即读·前提已核到底)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **现状死角(主导线亲读)**:`consume_worker_report_after_completion`(worker_report.rs:79)**只有「完成」一条路**——parse 到契约块就归档(88-118)、parse=None 就软着陆(119+),**任务恒算完成**。**没有任何「求助/blocked」分支**。这就是 C3a 要开的口。
- **四求助字段恒空**:`build_report_input`(worker_report.rs:155)把 `open_issues/permission_requests/direction_risks/follow_up_suggestions` 硬编码 `Vec::new()`(212-215)——因为源 struct `WorkerReport`(19-28)只有 4 字段(did/outputs/status/evidence),没有求助源字段。
- **blocked 是死值**:`acceptance_status` match(187-192)只产 done/partial/failed→白名单前三值;白名单第四值 `blocked`(185 注释)**这个 match 永不产出**。
- **契约文本**(31)只让 worker 输出 did/outputs/status(done|partial|failed)/evidence,**没有求助路径**。
- **链调用点**:`director_agent.rs:1285` 调 consume(在链的完成处理里)——**求助→等待分支要插这**。

## 1. 拍板摘要

- **做什么**:让 worker 能经回程契约表达求助(缺权限/缺资料/方向可能错)→ 契约加 blocked 路径 + WorkerReport 加求助字段 + consume 加求助分支(→等待·不当完成)+ 链停该任务待主管(不崩)+ 激活 blocked + 四字段填真源。
- **canon(拍①③)**:完成汇报**软着陆不变**;**求助=强信号不可软着陆**——status=blocked/求助特征→任务停+主管必见;**疑似求助但 json 坏→保守升级「疑似求助·主管必看」+任务停,不许降成普通 warning**。
- **拍②真源**:本片让实现A(worker_report 契约链)成为求助的**唯一真源**(四字段有真值);实现B 启发式退役、独立 bool 接真源 = **C3b**(不在本片)。

## 一句话判据

**「是不是只:契约加 blocked 求助路径 + WorkerReport 加求助字段(serde default·旧报文不破)+ consume 加求助分支(blocked/疑似求助→等待+主管必见·完成路软着陆不变)+ 链(director:1285)求助→停该任务待主管不崩 + 激活 blocked + build_report_input 填真源——而沙箱/path-lock/授权/execute/runner/relay/commands 0-diff、完成汇报软着陆语义不动?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 契约加求助路径(worker_report.rs:31)

- `WORKER_REPORT_CONTRACT_TEXT` 追加求助说明:**若被阻塞/需要更多权限或资料/认为方向可能错** → status 填 `"blocked"`,并在 json 里给求助字段(`permission_requests`:缺什么权限/资料·`open_issues`:卡在哪·`direction_risks`:为什么觉得方向不对)。完成路(done|partial|failed)措辞**不动**。确定性文本·不经 LM(安全死线:契约段不给 LM 发挥)。

### 2.2 WorkerReport 加求助字段(worker_report.rs:19)

- 加 `#[serde(default)]` 的 `permission_requests: Vec<String>`/`open_issues: Vec<String>`/`direction_risks: Vec<String>`/`follow_up_suggestions: Vec<String>`(字段名对齐 `WorkerStructuredReportInput`·正本 §3.6);**serde default=旧报文(无这些字段)照常解析不破**。

### 2.3 consume 加求助分支(worker_report.rs:79·核心)

- parse=Some 且(status=="blocked" **或** 任一求助字段非空)→ **求助分支**:返回一个新 outcome 形态(`WorkerReportConsumeOutcome` 加 `help_signal: Option<...>` 或等价·**呈现求助内容**),**不返回「完成」语义**;
- **疑似求助保守升级**:parse=None **但** 原文含求助特征(blocked/求助/卡住/需要权限 等保守词表)→ **不软着陆成普通 warning**,升级为 `help_signal`=「疑似求助·主管必看(原文尾…)」;
- 完成路(status=done/partial/failed·无求助字段)→ **软着陆语义逐字不变**(88-128 现状行为保持)。

### 2.4 激活 blocked + 填真源(worker_report.rs:187/212)

- acceptance_status match(187)加 `"blocked" => "blocked"`;
- build_report_input(212-215)四字段从 `report.permission_requests` 等填(**不再 Vec::new()**)——实现A 成唯一真源。

### 2.5 链:求助→停该任务待主管(director_agent.rs:1285)

- consume 返回 `help_signal` 时:该任务**进等待态**(用现成 `waiting_decision` 语义·workflow_read_model 已有该态机器)、**链停该任务不崩**(同 fail-stop 的「停」但**语义是"等主管决策"非"失败"**·审计人话「worker 求助·待主管」)、**主管必见**(求助内容呈现·不吞);
- **不自动完成**(现状完成分支恒 completed·求助分支绝不落 completed);
- 完成/失败/超时等现有分支**不动**。

### 2.6 明确不做(归 C3b)

实现B 启发式退役(workflow_read_model:922 `contains("direction")`)/ 读模型只投影真源 / unresolved_direction_risk bool 接真源 / dispatch cancelled 终态 —— **全归 C3b**。本片只立求助真源+通道,不碰读模型收敛。

## 3. 安全死线

- 沙箱/path-lock/授权 active+边界复核/prepare needs_binding(C1/C2 刚立)/execute 本体/runner/relay/commands/c4_c6 判决体 — **全 0-diff**;
- **完成汇报软着陆语义一字不动**(拍①:只有求助是强信号·完成照旧不阻断);求助分支**只加不改**完成路;
- 契约文本改动**不经 LM**(确定性拼接);真跑圈测试项目;memories 观察模式不加旗。

## 4. 验收

- **单测**:① worker 报 status=blocked+permission_requests → consume 返 help_signal(非完成)·四字段有真值·acceptance_status=blocked;② 疑似求助(原文"我卡住了需要权限"但 json 坏)→ 保守升级 help_signal 非普通 warning;③ 完成报文(done)→ 软着陆行为逐字不变(现有完成测全绿·证没碰完成路);④ 旧报文(无求助字段)→ serde default 解析不破;
- **链测**:一条链某任务 worker 求助 → 该任务进 waiting_decision·链停不崩·**不落 completed**·求助内容可见·主管必见;完成任务照常 completed;
- **真跑**(`#[ignore]`·测试项目):worker prompt 引导出一次 blocked 求助 → 端到端停在待主管;
- 三闸绿 + 死线 0-diff 自证 + 计数不降 + fmt **`cargo fmt --check` 自己真跑**(非 ad-hoc rustfmt·会假报)。

## 5. 回交

- §4 证据(尤其「完成路软着陆没变」+「求助不落 completed」两侧)+ 死线 0-diff 自证 + 落点清单 → 主导线核实物(**我重点核:完成汇报语义真没动、求助真进等待不当完成、疑似求助没被软着陆掉**)。**子线不 commit。**

## 7. 不接受为

- 求助被当完成软着陆掉(违拍①强信号)/ 疑似求助降成普通 warning(违拍①保守升级)/ 动完成汇报软着陆语义 / 契约段经 LM / 碰沙箱/授权/execute/runner/relay/commands/c4_c6 判决体 / 提前做 C3b(启发式退役/读模型收敛/cancelled)/ 自报 fmt 或用 ad-hoc rustfmt 核 / worker 求助字段无 serde default 破旧数据。
