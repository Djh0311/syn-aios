# 实现任务包:交办地基·第一刀(一键合流 + 档位写范围 + 自动绑会话 + flaky retry)· 主导线 → 执行线 v1

日期:2026-07-02　性质:**轻档**(命令编排/装配逻辑;不碰执行闸/沙箱/path-lock 本体;真跑仍圈固定测试项目)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(后端·新开干净上下文)。**子线不 `git add`/`commit`。** 全程中文。
- **决策正本**:`decisions/2026-07-02-project-jiaoban-tab-final-design-v1.md`(交办定稿)。本包 = 其中「地基六件」的 1/2/3/5(部分)/6;**批前预拆 + 图存档 + 任务级节点 = 刀2,本包不做**。
- **工作树现状(重要)**:main 上有**未提交**的 `consultant_agent.rs` + `lib.rs`——那是「咨询自带执行范围」包(`tasks/2026-07-01-consultant-propose-execution-scope-v1.md`)已执行、已验(622/0/36、死线 0-diff、真跑 proof)的成果。**本包以它为底座在其上重塑,不许 revert**:定稿把 write_roots/tools/roles 的来源从「咨询提」改成「档位装配」,但咨询的 `target_files`(会改的文件)/`checks`(怎么验)/解析向后兼容/纯咨询只读分流**全部保留**。
- **先读**:① `consultant_agent.rs`(未提交版全文:`ConsultationExecutionScope`/`map_consultation_to_c1_input` 分流/prompt)② `director_agent.rs`(`auto_advance_authorized_role_loop` + 内层 / `run_director_task_chain` 的 retry 落点 / `start_project_director_chain`)③ 方案确认/边界复核/授权生效的现成命令路径(方案 store + plan_authorization store + C3 record_global_boundary_review)④ 会话创建/绑定现成路径(session_policy「新建会话真跑时建」已有决策与机制,找到并复用)⑤ 记忆 `real-codex-run-flaky-verify-by-artifact`、`tier1-codex-exec-no-ondemand-read-inject`。
- **一句话**:把「允许并开始」做成一个后端动作(确认方案→边界复核记录→授权生效→prepare→自动绑会话→起链),写范围由**档位**(固定测试项目)装配,worker 偶发早退**自动重试一次**,起链前**复查授权仍 active**。

## 1. 拍板摘要

- **要做的事**:用户在授权卡点一下 [允许并开始],后端把批后的所有步骤一口气自动做完(全复用现成步骤);「写范围为空」「needs_binding」「随机早退看着像坏了」三类扫兴从 happy path 根除。
- **为什么**:好用五柱的「少动手」(7 步→2 下)+「等得安心」;用户原始痛点「步骤太多」。
- **代价**:一轮·后端。一个合流命令(编排现成件)+ map 重塑(档位装配)+ 自动绑会话 + retry + 授权复查。**执行闸/沙箱/path-lock/链判决体不碰。**

## 一句话判据

判改动在不在本包——问:**「是不是只在『用户已点允许』这个人闸之后,把现成步骤串成一个命令、把 scope 来源换成写死的测试项目档位、把绑会话自动化、给 worker 早退加一次 retry、起链前复查授权——而没有新开闸/自动确认方案/把档位参数化成任意路径/碰任何死线本体?」** 是 → 做;否 → **停、回主导线。**

## 2. 建什么

### 2.1 档位装配(重塑未提交的 `map_consultation_to_c1_input`)
- 定义常量档位(如 `PROFILE_EDIT_TEST_PROJECT`):`write_roots = [固定测试项目根]`、`tools = 现有 allowed_tools 词表中的读+写能力`、`role_ids = ["codex-dev","project_director"]`。**写死,不可参数化。**
- `execution_scope = Some`(要改东西)时:`scope_draft` 的 write/tools/roles 从**档位**填;`checks` 仍用咨询提的;`target_files` 仍进 proposed_steps(喂授权卡「会改的文件」)。`None`(纯咨询)分流照旧只读。
- 咨询 prompt 相应改:只要求报 `target_files`/`checks`/是否需要改东西,**不再要求报 write_roots/tools**(老输出仍能解析=向后兼容,多报的字段忽略)。原「write_roots 越界拒」护栏对象消失,可删或改为编译期断言档位=测试项目根。

### 2.2 合流命令(核心)
新命令(名字执行线定,如 `confirm_and_start_authorized_run`):输入 `project_root` / `proposal_id` / `session_choice`(new | existing{session_id})/ `actor_id`。
- **前提 = 人闸**:本命令**只能**表达「用户刚在 UI 点了允许」;第一步校验方案状态 = PendingUserConfirmation,由本次调用记录用户确认。**无任何免用户路径**(不给定时器/链/别的命令调它的口子)。
- 步骤全复用现成逻辑,按序:确认方案 → 记录全局边界复核(Phase A 用户演全局主管,这一下同时是边界批准,actor=用户)→ 授权生效 → 调 `run_auto_advance_authorized_role_loop` 内层(LM 拆 → prepare → 链;**内层本体尽量 0-diff,只加 2.3/2.4/2.5 的缝**)。
- 入口 `require_test_project_path_lock`(与 auto_advance 同);async + spawn_blocking(真 codex 长耗时,别冻 UI——同 P2 范本)。
- 返回沿用/扩展 `AutoAdvanceRoleLoopOutcome`(stage/message/停因人话已建,别退化)。

### 2.3 自动绑会话
- needs_binding 分流处接上:`session_choice=new` → 走现成「新建会话」机制真建一条并绑(**复用** session_policy 已有的「新建会话真跑时建」路径,别造第二套);`existing` → 绑传入 session_id(校验存在)。
- 绑失败 → 停,停因人话(「会话没建起来/没找到:…」),**不静默**。绑成后链照跑,happy path 不再出现 needs_binding 停。

### 2.4 flaky 自动重试一次
- `run_director_task_chain` 的 worker 派发失败处:若特征为 tier-1 偶发早退(exit 1 且无输出——按记忆 `real-codex-run-flaky-verify-by-artifact` 的特征判),**同一任务原地重试一次**,chain warnings 记「任务 X 已自动重试」;重试仍败 → 照现状 fail-stop。
- **只 retry 早退**;越权被拒/闸拦/超时按原语义不 retry。**严禁写成循环重试。**

### 2.5 起链前复查授权
- 合流路径与 `start_project_director_chain`(C1)起链前,复查方案授权仍 active(批与跑之间可能被撤);失效 → 拒,停因人话。

## 3. 安全死线(0-diff / 不碰 / 不绕)

- **人闸不省**:合流命令 = 用户点击的直接效果;不新增 auto-approve、不给非用户路径调用口;方案确认/边界复核/授权生效走**现成状态机逻辑**(复用,不旁路)。
- **档位写死测试项目根**:不可由请求参数改写(防「能预览任意项目」滑成「能改任意项目」);真执行仍 path-lock + 沙箱 + 四护栏,本体 0-diff。
- **死线本体 0-diff**:`decide_real_execution_command` / `command_plan_for` 沙箱 / `execute_project_workflow_node_at` / `prepare_authorized_auto_dispatch` 判决体 / `workflow_chain_controller` 本体 / `codex_local_runner::readonly_codex_consult`。retry/复查授权是**调用处加缝**,不改判决体。
- 不放开非测试项目 / 多项目 / 方案授权自动(**永不**)。

## 4. 验收(两条线自己验·不丢给用户)

- **单测·合流**(stub 咨询/主管/链,不起 codex):Pending 方案 + 点允许 → 确认+复核+授权+prepare+链 stub 一气跑完;非 Pending 状态被拒;非测试 root 被拒。
- **单测·档位**:execution_scope=Some → scope_draft.write/tools/roles == 档位、checks == 咨询提的、target_files 进 steps;None → 只读照旧(2026-07-01 包的既有单测**按新口径改**,总数不降)。
- **单测·自动绑**:session_choice=new → 会话建且绑、链继续;绑失败 → 停因人话。
- **单测·retry**:注入一次假早退 → 重试成功、warnings 有记;注入两次 → fail-stop;注入「越权拒」→ 不 retry。
- **单测·授权复查**:授权撤销后起链 → 拒。
- **真跑**(`#[ignore]`·测试项目·用户不在场也由两条线做):目标 →(现成)咨询 → 合流一个命令 → proof 落测试项目、`.codex` 凭据没碰。核实物:读 proof + 链记录,别只信 exit code。
- **regression**:`cargo test --lib` 计数不降、fmt(只本包文件·防 rustfmt 递归·见记忆)、死线 0-diff 扫 diff 自证。

## 5. 本包不做(deferred)

- 批前预拆 / 批的图存档 / 所批即所跑闭环 / 任务级节点+依赖边落画布 =(**刀2 包**,本包落地后开)。
- UI(交办 tab)= UI 线包 `tasks/2026-07-02-jiaoban-tab-ui-v1.md`。
- 秘书/全局主管 agent、非测试项目、可编辑工序图。

## 6. 回交

- 跑 §4;回交列:合流命令签名与步骤、档位常量、map 重塑差异(对 2026-07-01 版:保留了什么/换源了什么)、绑会话复用的现成路径、retry 落点、每类单测证据 + 真跑 proof + 死线 0-diff 自证 + 计数 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 不接受为:出现任何**免用户确认**的自动路径 / 档位可参数化成任意路径 / retry 写成循环或 retry 被闸拒的 / revert 了未提交底座而不是重塑 / 碰死线本体 / 绑会话造了第二套机制。
- 不接受为交办整体完成(UI 与工序图刀2另包;本包只到「点一下允许 → 测试项目里自动跑完出 proof」)。
