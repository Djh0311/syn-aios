# 实现任务包:B2·C1 每任务独立会话(会话跟任务走)· 主导线 → 执行线 v1

日期:2026-07-08　性质:**轻档**(后端为主·测试项目圈内;文件边界 §2.4)。正本:`decisions/2026-07-08-phase-b2-execution-loop-final-v1.md` + 七拍 `decisions/2026-07-08-b2-transfer-protocol-gap-final-v1.md`(第 3 拍在本包有硬闸)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **canon**:「会话跟任务走」(车间模型旧句已修订·定稿 §canon 演化)——每个任务一个全新 codex 会话,以任务命名;worker 知道的**只有**工作台发给它的任务包(上下文隔离从"顺带记得"变"制度保证")。
- **主导线已核的接缝(直接用)**:
  1. **先生后绑机器现成**(方案a·014c254/3495790):合流 new 分支经 `manual_relay` new_session **单次路径**真建会话 → 回执取 thread_id → 绑定 → 链照旧 resume;`codex_db::find_thread_by_id` 存在性校验在;**C1 = 把这台机器从「每单一次」搬到「每任务一次」**(链派每任务前调用);
  2. 链驱动 = `run_director_task_chain`(director_agent.rs);现状每任务 resume 同一个角色节点绑定会话(拐杖);
  3. `TaskPackage.target_session_id`(types.rs·C0 差量 §5.1):struct 有字段无生产者——C1 落真实赋值(新会话 thread_id 物化进任务包 artifact);
  4. **C0 实测事实**:codex-cli 0.134.0;`codex exec` 自动化面只有 resume(fork 够不到);`multi_agent: stable/true` 与 `memories: experimental/true` **默认开启**(features list 亲测)。

## 1. 拍板摘要

- **要做的事**:链派每个任务前新建专属会话(任务名命名·智能体页可见),worker 在干净上下文里只吃任务包;共用会话拐杖退役。
- **为什么**:蓝图 §5 一会话一任务;上下文可控是 C2 转发/C3 求助的地基;换会话三招之三的机器同源。
- **代价**:一轮;**每任务 +约 1 分钟建会话(知情代价·定稿已记)**。

## 一句话判据

**「是不是只:链派每任务前经现成先生后绑建新会话并绑定(任务名命名·target_session_id 真实赋值·失败留档停任务不崩链)+ 第 3 拍运行时防护实测——而 manual_relay/runner/闸/判决体本体 0-diff、人闸与四护栏不动?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 链内每任务建会话

- `run_director_task_chain` 派发每任务前:经现成先生后绑单次路径建新会话(cwd=测试项目·初始化消息含任务名)→ 回执 thread_id → 绑定到本任务的执行目标(节点/工作项语义按现状机器,**别为绑定新造第二套**)→ 该任务 dispatch 用**新 thread** resume;
- **会话命名 = 任务名**(截断安全),智能体页列表可辨;审计记每次建会话(现成 rebind/new_session 事件族,不新开);
- **失败处置**:建会话失败(额度/断供/超时)→ 该任务按现有失败语义留档停(人话含「新建会话失败」),**链不崩、不静默回落共用会话**(回落=拐杖复活,§7 禁);供给类分类照 fix8 复用;
- `target_session_id` 物化进任务包 artifact(C0 差量 §5.1 的 C1 项);
- **拐杖退役**:角色节点旧绑定语义保留为**只读兼容**(历史数据/手动挡照读),链主路径不再消费它派新任务;注释记「C1 退役·手动挡 override 仍走旧绑定」。

### 2.2 第 3 拍·运行时防护实测(硬闸)

- 实测 exec/resume 路径下 `multi_agent`/`memories` 行为:① 有无 per-exec 关闭口(`-c features.multi_agent=false` 类 config flag·查 CLI 文档+实测);② worker 会话在我们的 prompt 形态下会不会自发子 agent(观察 `--json` 事件流有无子 agent 事件);
- **处置阶梯**:能关 → runner 调用层加关闭参数(**注意:runner `command_plan_for` 是死线**——若加参必须停手报回主导线拍,不许自改;优先找不碰死线的注入点,如 exec 参数在非保护层拼装处);关不了 → 实证 exec 面不触发并把证据+风险+缓解写进回交;**两者都做不到 → 停手报回**(定稿硬闸原文)。

### 2.3 明确不做(§7 同)

C2 字段扩容/C3 求助/C4 终标(各归各包);fork(自动化面够不到·C0 实锤);dispatch cancelled 终态(七拍已收窄到 C3);每任务会话的复用池/预热(过早优化)。

### 2.4 文件边界(越界即停)

- 允许:`director_agent.rs`(链派发前建会话+绑定+物化 target_session_id+自测 mod)/ `c4_c6_...entrypoints.rs` **仅当**物化 artifact 加 target_session_id 一键(加法一处·判决体/guard 函数 0 命中——碰到别处即停)/ `tests/` + 跑器 1 行(若有前端断言需要;预期纯后端);
- **0-diff**:`manual_relay.rs`(只调)/ `codex_local_runner.rs` / `commands.rs` / `control_core.rs` / `workflow_chain_controller.rs` 本体 / `worker_report.rs` / consultant / 两意见 agent / secretary / run_history / 前端全部 / lib.rs。

## 3. 安全死线

- 新会话全部圈**固定测试项目**(先生后绑 cwd 写死不变);人闸/授权复查/prepare guard/四护栏一字不动;`.codex` 凭据不碰(会话文件是 codex 自家写入,非我们写);
- 真跑属测试项目轻档;**每任务 +1 分钟的真实代价在回交里给实测数**(一条 3 任务链的端到端耗时前后对比)。

## 4. 验收

- **单测**(director 自 mod):3 任务链 → stub 会话工厂被调 3 次、各任务 dispatch 用各自 thread、target_session_id 三个互异且物化;建会话失败 → 该任务失败留档人话、链按 fail-stop 语义、**无共用会话回落**(桩证);手动挡旧绑定路径回归不动;
- **真跑**(`#[ignore]`·额度在):一条 2-3 任务链端到端——codex home 新增 N 个以任务命名的会话、各任务口供照常落库、耗时对比记录;
- **第 3 拍实测**:处置阶梯三选一的实答+证据;
- 三闸绿 + §2.4 0-diff 自证 + 计数不降 + fmt 净。

## 5. 回交

- §4 证据 + 耗时实数 + 第 3 拍实答 + 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 静默回落共用会话 / 为绑定新造第二套语义 / 碰 manual_relay·runner 本体(含为关特性自改 `command_plan_for`——那要停手报回) / fork / 提前做 C2-C4 的活 / 会话不以任务命名(智能体页认不出=白建)。
