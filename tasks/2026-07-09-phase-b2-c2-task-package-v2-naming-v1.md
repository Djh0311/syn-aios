# 实现任务包:B2·C2 任务包 v2·三层命名统一 + 新字段 + 可配· 主导线 → 执行线 v1

日期:2026-07-09　性质:**较重**(跨层字段改名·消费面 ~10 文件·含一个沙箱攸关键 `allowed_write`)。主导线已 measure-first 亲读三层+消费方(见 §0)。正本:任务包设计 `docs/workflow-task-package-design-v1.md` §3.4 + C0 差量 §5.1。

## 0. 接手须知(冷启即读·前提已核到底)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **三层现状(主导线亲读·当前代码)**:
  1. **planned_task**(`ProjectDirectorPlannedTask`·types.rs:2451):`objective`(≠正本 task_goal)、`report_format`(=正本 ✓)、scope.`allowed_read_scope`/`allowed_write_scope`(=正本 ✓·types.rs:2438);
  2. **物化 artifact**(c4_c6:2385-2426·json 字面量):`"brief": task.objective`(2390)、`"required_return": task.report_format`(2413)、`"allowed_read": …allowed_read_scope`(2404)、`"allowed_write": …allowed_write_scope`(2405)——**四个键 diverge**;`"forbidden_actions"` 硬编码 4 串(2406-2411)、`"model_id": "codex-local-prepared"` 硬编码(2422);
  3. **TaskPackage struct**(types.rs:4649·**活的**·构造点 workflow_read_model_entrypoints.rs:835):用正本名 task_goal/report_format/allowed_read_scope。
- **分叉本质**:真正 diverge 在**物化那一步**(把正本名翻成 brief/required_return/allowed_read/allowed_write);planned_task 只有 `objective` 一处偏。
- **消费方(主导线 grep·改键要同步这些)**:
  - `"brief"` 读方:workflow_state_lifecycle / project_workflow_automation / workflow_execution / workflow_read_model / lib;
  - `"required_return"` 读方:workflow_state_lifecycle / workflow_execution / workflow_read_model / lib;
  - `"allowed_read"` 读方:workflow_state_lifecycle / workflow_run_dispatch / workflow_read_model / lib;
  - **`"allowed_write"` 读方(最广·7 文件·含敏感)**:workflow_state_lifecycle / h5_project_dispatch_bridge / workflow_run_dispatch / **`real_execution_command.rs`(喂沙箱写权限)** / session_continuation_store / workflow_read_model / lib。

## 1. 拍板摘要

- **做什么**:①三层命名统一到正本(objective→task_goal·物化键 brief/required_return/allowed_read→task_goal/report_format/allowed_read_scope);②新字段 timeout_policy/failure_policy/available_skills/available_knowledge_refs;③forbidden_actions/model_id 硬编码→按任务可配。
- **`allowed_write` 键单独隔离**(§2.5·沙箱攸关·先验失配模式再动)。
- **为什么**:新字段落进统一命名体系(否则再添第四套混乱·C0 §5.1)。

## 一句话判据

**「是不是只:objective→task_goal(编译强校验)+ 物化三键改名(brief/required_return/allowed_read→正本)且所有读方同步(grep 证零旧键残留)+ 新字段加(serde default·additive)+ forbidden_actions/model_id 改可配——而沙箱/path-lock/审批/execute 本体 0-diff、`allowed_write` 键按 §2.5 隔离处置?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 objective → task_goal(编译强校验·先做·最安全)

- `ProjectDirectorPlannedTask.objective`(types.rs:2454)→ `task_goal`;全仓 `.objective` 读写点(director_agent 拆任务产出、c4_c6:2390 物化、workflow_read_model 投影、lib 测…)同步改名。**这是 struct 字段改名·编译器会逼你改全**——编译过=没漏。

### 2.2 物化三键改名 + 读方同步(字符串键·无编译校验·最易漏)

- c4_c6 物化:`"brief"`→`"task_goal"`(2390)、`"required_return"`→`"report_format"`(2413)、`"allowed_read"`→`"allowed_read_scope"`(2404);
- **§0 列的每个读方逐个改**(读旧键的 `.get("brief")` 之类改新键);
- **验收硬条**:改完 `grep -rn '"brief"\|"required_return"\|"allowed_read"' --include=*.rs`(排除本包新写的注释/测试历史)**零业务读方命中**——字符串键漏一个=运行时静默取 None,必须 grep 自证清零。

### 2.3 新字段(additive·serde default·不破旧数据)

- planned_task/scope 加 `timeout_policy`/`failure_policy`/`available_skills`/`available_knowledge_refs`(类型按正本 §3.4·`#[serde(default)]`);物化 artifact 落这些键;TaskPackage struct 同步加(若正本已有则对齐);**旧数据反序列化不破**(default 兜底)。

### 2.4 forbidden_actions / model_id 可配

- 物化的 `forbidden_actions`(c4_c6:2406 硬编码 4 串)→ 从 planned_task/scope 取(缺省仍给那 4 串兜底·别丢现有保护);`model_id`(2422 硬编码)→ 从任务/harness 配置取(缺省 `codex-local-prepared`)。

### 2.5 🛑 allowed_write 键·隔离处置(沙箱攸关·别跟大流)

- `"allowed_write"`(c4_c6:2405)读方含 `real_execution_command.rs`(喂沙箱写权限)。**先做一件事**:读 `real_execution_command.rs` 里读 `allowed_write` 的地方,确认**键缺失时的失配模式**——是 fail-closed(缺→不给写权限·安全)还是 fail-open(缺→放开·危险);
- **失配是 fail-closed 且你能证 7 个读方全同步**:才可改名 `allowed_write`→`allowed_write_scope`;
- **失配是 fail-open、或拿不准、或读方多到没把握全改**:**停手报回**——`allowed_write` 键这一项归单独一步(不阻塞 2.1-2.4),别为凑「统一」冒沙箱失配风险。

## 3. 安全死线

- 沙箱 / path-lock / 授权 active+边界复核 / prepare 的 needs_binding 就绪逻辑(C1 刚立)/ execute 本体 — **全 0-diff**;
- `real_execution_command.rs` **只读不改**(除非 §2.5 fail-closed 确证后改 allowed_write 键读处·且逐处 justify);
- C2 是改名+加字段·**不改任何派发/授权/执行行为**——语义等价,只动名字与新增可选字段;
- 真跑圈测试项目;memories 观察模式不加旗。

## 4. 验收(改名类·grep 自证为命)

- **编译**:objective→task_goal 全改(编译过=struct 侧没漏);
- **grep 清零**:2.2 三键(+ 若做了 allowed_write 则四键)业务读方零旧键残留(命令+输出贴回交);
- **单测**:一条 planned_task→物化→读回 全链,新键读得到、旧键读不到(证改彻底);新字段 default 兜底(旧 json 无该字段不崩);forbidden_actions/model_id 可配且缺省兜底;
- **真跑**(`#[ignore]`·测试项目):一条链端到端·worker 真读到新键名的任务包·派发/授权行为无变化(语义等价自证);
- **§2.5 实答**:allowed_write 的失配模式结论 + 处置(改了/隔离报回);
- 三闸绿 + 沙箱/execute 本体 0-diff 自证 + 计数不降 + fmt **自己真跑 `rustfmt --check` 别自报**(前科·注意 commands.rs 有 35 块历史漂移·只看你的新增块净不净)。

## 5. 回交

- §4 全证据(尤其 grep 清零命令+输出、§2.5 失配实答)+ 沙箱 0-diff 自证 + 落点清单 → 主导线核实物(**我重点核:字符串键零残留、沙箱失配没恶化、语义真等价**)。**子线不 commit。**

## 7. 不接受为

- 字符串键改名漏读方(运行时静默 None·grep 必清零)/ 为凑「统一」硬改 allowed_write 而没验沙箱失配模式(§2.5 停手)/ 改派发/授权/执行语义(C2 只改名+加字段)/ 碰沙箱/path-lock/审批/execute 本体 / 丢 forbidden_actions 现有 4 串保护 / 新字段无 default 破旧数据 / 自报 fmt 不真跑 / 顺手动 real_execution_command 本体。

## 8. 主导线说明(scope 决策·2026-07-09)

C2 measure-first 发现比路线图标题「三层命名统一」更宽(3 活层·4 字符串键·~10 读方)且含沙箱攸关键。故本包:2.1-2.4 是核心统一(可一轮做),**2.5 allowed_write 隔离**(fail-open/拿不准就归单独一步)。若你回交时 §2.5 停手报回,主导线单独拆 allowed_write 一步(沙箱失配专项验)。这是吸取 S0 教训——宁隔离不一锅端。
