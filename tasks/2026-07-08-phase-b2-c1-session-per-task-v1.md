# 实现任务包:B2·C1 每任务独立会话(会话跟任务走)· 主导线 → 执行线 v1.2

日期:2026-07-08(v1.1 曾拍死线两旗;**v1.2 修订 2026-07-09:用户复核影响账后改拍「观察模式」——两旗收回、runner 回全 0-diff,v1.1 两旗从未派出实现**;§2.2/§8 为准)　性质:**轻档**(后端为主·测试项目圈内;文件边界 §2.4)。正本:`decisions/2026-07-08-phase-b2-execution-loop-final-v1.md` + 七拍 `decisions/2026-07-08-b2-transfer-protocol-gap-final-v1.md`(第 3 拍收口两轮见其修订记录)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **canon**:「会话跟任务走」(车间模型旧句已修订·定稿 §canon 演化)——每个任务一个全新 codex 会话,以任务命名;worker 知道的**只有**工作台发给它的任务包(上下文隔离从"顺带记得"变"制度保证")。
- **主导线已核的接缝(直接用)**:
  1. **先生后绑机器现成**(方案a·014c254/3495790):合流 new 分支经 `manual_relay` new_session **单次路径**真建会话 → 回执取 thread_id → 绑定 → 链照旧 resume;`codex_db::find_thread_by_id` 存在性校验在;**C1 = 把这台机器从「每单一次」搬到「每任务一次」**(链派每任务前调用);
  2. 链驱动 = `run_director_task_chain`(director_agent.rs);现状每任务 resume 同一个角色节点绑定会话(拐杖);
  3. `TaskPackage.target_session_id`(types.rs·C0 差量 §5.1):struct 有字段无生产者——C1 落真实赋值(新会话 thread_id 物化进任务包 artifact);
  4. **C0 实测事实**:codex-cli 0.134.0;`codex exec` 自动化面只有 resume(fork 够不到);`multi_agent: stable/true` 与 `memories: experimental/true` **默认开启**(features list 亲测);
  5. **主导线 07-08/09 加验**:memories 注入**实锤**——`~/.codex/memories/memory_summary.md` 全局摘要(用户其它项目内容)以 developer 消息进每个新会话上下文;`codex exec --disable memories` 实测关死(探测会话 019f424f 外部标记 0 命中 vs 基线 1 命中)。

## 1. 拍板摘要

- **要做的事**:链派每个任务前新建专属会话(任务名命名·智能体页可见),worker 在干净上下文里只吃任务包;共用会话拐杖退役。
- **为什么**:蓝图 §5 一会话一任务;上下文可控是 C2 转发/C3 求助的地基;换会话三招之三的机器同源。
- **代价**:一轮;**每任务 +约 1 分钟建会话(知情代价·定稿已记)**。

## 一句话判据

**「是不是只:链派每任务前经现成先生后绑建新会话并绑定(任务名命名·target_session_id 真实赋值·失败留档停任务不崩链)——而 manual_relay/runner/闸/判决体本体 0-diff、人闸与四护栏不动?」** 是 → 做;否 → 停、回主导线。(v1.2:两旗已收回,runner 恢复全 0-diff 死线。)

## 2. 建什么

### 2.1 链内每任务建会话

- `run_director_task_chain` 派发每任务前:经现成先生后绑单次路径建新会话(cwd=测试项目·初始化消息含任务名)→ 回执 thread_id → 绑定到本任务的执行目标(节点/工作项语义按现状机器,**别为绑定新造第二套**)→ 该任务 dispatch 用**新 thread** resume;
- **会话命名 = 任务名**(截断安全),智能体页列表可辨;审计记每次建会话(现成 rebind/new_session 事件族,不新开);
- **失败处置**:建会话失败(额度/断供/超时)→ 该任务按现有失败语义留档停(人话含「新建会话失败」),**链不崩、不静默回落共用会话**(回落=拐杖复活,§7 禁);供给类分类照 fix8 复用;
- `target_session_id` 物化进任务包 artifact(C0 差量 §5.1 的 C1 项);
- **拐杖退役**:角色节点旧绑定语义保留为**只读兼容**(历史数据/手动挡照读),链主路径不再消费它派新任务;注释记「C1 退役·手动挡 override 仍走旧绑定」。

### 2.2 第 3 拍·运行时防护(v1.2 终拍:观察模式)

- **实测账(主导线 07-08/09 亲验+执行线真跑佐证)**:memories 注入侧**实锤**——全局摘要以 developer 消息进每个新会话(窗口 07-07 21:38 起,7/98 会话中招,+3346 input tok/次;测试项目文件/工作台 store/记忆池反向 三面**零渗出**;「任务间搬运」未成品);`multi_agent` 开但 `enable_fanout` 关 ⇒ exec **不自发子 agent**(执行线真跑零子 agent 事件);`codex exec --disable memories` 可一旗关死(探测 019f424f 已验,备用)。
- **终拍(2026-07-09 用户)**:实害零、内容多为用户行为习惯层 → **暂不加旗、不动 runner(回全 0-diff 死线)、不写 config.toml;跑一段观察再定**。「工作台内置记忆开关(per-run 可配的产品功能)」记为**将来功能候选,未拍**。
- **known-gap 记档(不吹全隔离)**:C1 隔离 = **会话级**;codex 记忆层跨会话仍通。对外表述一律「每任务新对话+记忆层观察中」。
- **观察巡检(主导线,每切片收口顺手)**:重跑渗出三查(测试项目/工作台 store/记忆池反向)+ 记忆池内工作台条目计数;发现渗出成品或池内出现工作台条目 → 回到加旗/开关议题重拍。

### 2.3 明确不做(§7 同)

C2 字段扩容/C3 求助/C4 终标(各归各包);fork(自动化面够不到·C0 实锤);dispatch cancelled 终态(七拍已收窄到 C3);每任务会话的复用池/预热(过早优化)。

### 2.4 文件边界(越界即停)

- 允许:`director_agent.rs`(链派发前建会话+绑定+物化 target_session_id+自测 mod)/ `c4_c6_...entrypoints.rs` **仅当**物化 artifact 加 target_session_id 一键(加法一处·判决体/guard 函数 0 命中——碰到别处即停)/ `tests/` + 跑器 1 行(若有前端断言需要;预期纯后端);
- **0-diff**(v1.2 恢复全名单):`manual_relay.rs`(只调)/ `codex_local_runner.rs` / `commands.rs` / `control_core.rs` / `workflow_chain_controller.rs` 本体 / `worker_report.rs` / consultant / 两意见 agent / secretary / run_history / 前端全部 / lib.rs。

## 3. 安全死线

- 新会话全部圈**固定测试项目**(先生后绑 cwd 写死不变);人闸/授权复查/prepare guard/四护栏一字不动;`.codex` 凭据不碰(会话文件是 codex 自家写入,非我们写);memories 观察模式(07-09 拍):**不加旗、不动 runner、不写** `~/.codex/config.toml`;
- 真跑属测试项目轻档;**每任务 +1 分钟的真实代价在回交里给实测数**(一条 3 任务链的端到端耗时前后对比)。

## 4. 验收

- **单测**(director 自 mod):3 任务链 → stub 会话工厂被调 3 次、各任务 dispatch 用各自 thread、target_session_id 三个互异且物化;建会话失败 → 该任务失败留档人话、链按 fail-stop 语义、**无共用会话回落**(桩证);手动挡旧绑定路径回归不动;
- **真跑**(`#[ignore]`·额度在):一条 2-3 任务链端到端——codex home 新增 N 个以任务命名的会话、各任务口供照常落库、耗时对比记录;
- 三闸绿 + §2.4 0-diff 自证 + 计数不降 + fmt 净。

## 5. 回交

- §4 证据 + 耗时实数 + 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 静默回落共用会话 / 为绑定新造第二套语义 / 碰 manual_relay·runner 本体(v1.2:两旗收回,runner 全死线) / 改 `~/.codex/config.toml` 关特性(高危#2) / fork / 提前做 C2-C4 的活 / 会话不以任务命名(智能体页认不出=白建)。

## 8. v1.2 收尾轮(2026-07-09 回交核后余项——本轮范围)

首轮回交已核收:每任务先生后绑建会话+失败即停不回落+target_session_id 物化+直起链切 C1+3 单测,725/0/42 绿、死线全 0-diff(主导线 07-09 亲验:c4_c6 单 hunk、生产接线、测试重跑)。余三项:

1. **链级 3× 集成测**:一次链调用直证「creator 被调 3 次 + 各任务 dispatch 用各自 thread + target_session_id 三个互异」;若 prepared-chain 全流程夹具过重且 lib.rs 死线够不到,允许以 director 自 mod 的链级 stub 测形态实现(判据是「一次链调用覆盖三断言」,不是夹具形式);
2. **真跑耗时实测**(§4 原硬项):2-3 任务链端到端,codex home 见任务命名会话、口供照常落库、每任务 +1 分钟给实数;
3. **auto_advance 接 C1**:「[接着跑]/授权后自动推进」路径同款切到 with_session_creator——先盘 auto_advance 生产码位;**若必须动 lib.rs 或任何 0-diff 文件 → 停手报回**,不许为它松死线。两条生产路径(直起/自动推进)不一致的状态不能过夜到 C2。

边界/死线/回交同 §2.4/§3/§5;观察模式下**不加旗**(§2.2 v1.2)。
