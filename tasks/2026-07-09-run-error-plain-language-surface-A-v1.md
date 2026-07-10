# 实现任务包:A·运行错误上脸(人话诊断层)· 主导线 → 执行线 v1(**可派·2026-07-10 核实物定稿**)

日期:2026-07-09 起草 / **2026-07-10 主导线核实物定稿可派**　性质:**轻档**(后端翻译层+读模型+前端呈现;冻结核/安全闸 0-diff·但收编 fix8 报告层要动 runner 报告区·见 §3)。落位:**B2 尾片 C6(观测补强·用户 07-09「顺手做」)**。状态:**可派**(排期约束「C1 收尾轮清后派」已满足——B2 于 2026-07-10 整个收口·C1 已落定)。上承:提案 `docs/plans/2026-07-09-run-error-surface-plain-language-proposal-v1.md`(方向已拍·反馈必人话)。**两派前决定已拍(§8)。**

## 00. 主导线核实物修订记录(2026-07-10·派前重核磁盘·纠 07-09 草案漂移)

草案 §0 接缝坐标是 07-09 勘的;B2/C1 收口后重核磁盘,以下**以本节为准、正文已改**:

1. **worker_report 范式接点漂了**:草案「`worker_report.rs:57-60 report_warning/report_status`」已过期——C3 在 :57 插了 `help_signal_from_raw`,把范式挤到 **字段 `report_warning:75`/`report_status:78` + `fn report_status_field:94`**。正文 §0.4 已改。
2. **resume 分类器多两个调用点**:`classify_codex_resume_failure`(`workflow_execution_entrypoints.rs:217`)另被 **`workflow_run_dispatch_entrypoints.rs:872` / `:883`** 调用——改它内部路由时这两处随之受益、签名别变。
3. **新增接缝:已有 state-error 探测器(草案漏·别另造)**:`classify_phase_b_stderr_for_codex_state_error`(`codex_local_runner.rs:1320`·bool)+ `phase_b_mentions_codex_state_error`(`:1019`·bool)+ `classify_phase_b_status`(`:1001`·产 `"codex_state_error"` 状态)**已探测「state db 只读/permission denied」**——正是 A 族④(沙箱/只读)+族⑥(codex 子系统)重叠区。族④/⑥ 判据**复用/一致于这些现成探测器,不许另造矛盾探测**(见 §2.1)。
4. **新增硬约束:`codex_provider_unavailable:` 前缀是跨模块 retry 承重标记(草案误判为纯呈现 hack)**:生产消费者 **`director_agent.rs:1384`**(dispatch flaky 判定·排除供给类不 retry)+ **`:1396` `is_director_plan_flaky_early_exit`**(供给类→不 retry·白等一分钟)。**裸删前缀会断 retry 逻辑。**(前缀生产点 `director_agent.rs:4691`/`:5302` 在 `#[cfg(test)]`@4461 之后 = 测试脚手架·非生产 emit。)这坐实并收紧草案 §2.1.2 的对冲:见 §2.1 收编 scope 二分 + §3。
5. 其余坐标(`classify_codex_provider_failure` def:386/call:364,373/resume:226、`append_stderr_tail`:409、`humanize_consult_error` secretary:185+global:410、run-history 停因位 ~:315)**逐个复核仍吻合**。前端呈现文件确认在 **`src/views/projects/ProjectJiaobanPanel.tsx`**。
6. 防重造复核:能力地图 v2 §概念1「错误人话翻译」列的 4 处 = 草案 4 处一致,**无隐藏第五套翻译器**;主导线源码全扫(`fn humanize/classify/translate` + 所有 `codex_provider_unavailable` 串)确认到第 3/4 条为止就是全部相关面。

## 0. 接手须知(冷启即读·本包自包含)

- 你是**执行线**(后端为主+一处前端呈现)。**子线不 commit。** 全程中文。
- **A 是什么 / 不是什么(硬约束·用户 07-09 拍)**:A = 捕获 codex/worker 运行的原始错误 → **翻译成人话** → 按 run 上脸。**A 不是**把 `--json` 裸错误/stderr 灌进面板。默认脸给人话,原文只在下钻时看。
- **主导线已勘的接缝(2026-07-10 核过·直接用):**
  1. **现成翻译层就一处**:`codex_local_runner.rs:386 classify_codex_provider_failure`(fix8 加·**非** `command_plan_for` 冻结核)——只认供给类(`subscription_not_found`/`usage limit`/`quota`/`unauthorized`/`403`/`401`/`reconnecting 5/5`),命中→人话前缀 `codex_provider_unavailable:`;调用点 `:364`/`:373`,另 `workflow_execution_entrypoints.rs:226` 也调;
  2. **未命中就吞真相**:`codex_local_runner.rs:409 append_stderr_tail`(用 :370/:376)把裸 stderr 截 200 贴在原错误后,**不翻译**——非供给类错误(codex 子系统报错/沙箱拒绝/命令 exit≠0/超时/`consult_last_message_read_failed`)全落这条,**这是 A 要补的洞**;
  3. **run-history 停因是半包接**:`run_history_read_model.rs` `state`=机器键、`state_note`=人话一句(doc ~:28);~:315 `"failed" => "跑挂了(去工作流看详情)"` 注释「只给人话状态尾巴;具体停因在『工作流』详情看(UI 半包接)」——**A 要把翻译后的错误喂到这个「详情」位**;
  4. **黄牌范式(呈现不驱动·照抄哲学)**:`worker_report.rs` 字段 `report_warning:75`/`report_status:78` + `fn report_status_field:94`(**注:草案说的 :57-60 已被 C3 `help_signal_from_raw` 占**),前端据此判黄牌、不改链态。A 的错误呈现同哲学:**呈现不阻断·不是闸**;
  5. **state-error 现成探测器(别另造)**:`classify_phase_b_stderr_for_codex_state_error:1320` / `phase_b_mentions_codex_state_error:1019` / `classify_phase_b_status:1001`(产 `codex_state_error`)已判 state-db 只读——族④/⑥ 与之一致,别造矛盾探测;
  6. **`codex_provider_unavailable:` 前缀 = retry 承重标记(不是纯呈现)**:`director_agent.rs:1384`/`:1396` 生产消费它判「供给类不 retry」——**动前缀前先读 §2.1 收编 scope + §3**;
  7. **活证据**:2026-07-08 worker transcript 里 `codex_memories_write::phase2::job: failed to claim job (no such table: jobs)`——当前界面零呈现,A 完成后应翻成「codex 记忆子系统写入失败(本地缺表)·不影响本次任务结果」这类人话。

## 1. 拍板摘要

- **做什么**:把 fix8 翻译层从「只认供给类」推广成**错误族全谱分类器**(结构化返回`{人话摘要, 原文, 错误族}`),接到 run-history 详情位,前端两层脸(默认人话·下钻原文)。
- **为什么**:主线是「让编排可观测」;现在 worker 挂了只看得到语义层(节点 failed/停因一句),原始诊断层零呈现,调试要手扒 transcript。
- **不做**:B(开发者工具/devtools·用户方式未定);把裸错误直接上脸(违 07-09 硬约束);改成败判定(A 只影响呈现·延续 fix8「不改成败」);**为消 humanize×2 而重构 director retry 契约**(见 §2.1·倾向归 §8)。

## 一句话判据

**「是不是只:加一个错误族分类器(结构化`{人话/原文/族}`·unknown 保守兜底)+ 喂到 run-history 详情位 + 前端两层脸呈现——而 runner `command_plan_for`/`run_real_codex_process`/沙箱/安全闸 0-diff、不改任何成败判定、不断 director retry 对供给类的识别、呈现不阻断?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 错误族全谱分类器(翻译层核心)

- **新增独立模块** `run_error_translation.rs`:`classify_run_error(raw: &str) -> RunErrorHuman { family, human, raw_snippet }`;
- **收编 scope 二分(2026-07-10 核实物收紧·关键)**——把「加结构化翻译」和「删前缀契约」分开,别混:
  - **(a) 本片 DO(真增益·低危):** 供给类判据**整段搬进新模块**作为族①(单一真源的**判据**);runner 两调用点(:364/:373)+ resume 分类器(:226)改调新模块;runner 现被 `append_stderr_tail` 吞掉的非供给类错误**路由进新模块翻译**。**硬约束:新模块对供给类仍产出稳定信号**(`family=provider_unavailable`·或保留 `codex_provider_unavailable:` 前缀映射),**让 director retry 消费者(1384/1396)照旧能判「供给类不 retry」**——不许裸删前缀导致 retry 读不到供给类信号。
  - **(b) 谨慎/倾向缓到 §8:** `humanize_consult_error`×2 消除 + 前缀 hack **彻底删除** + director retry 改读结构化 family——这是跨 secretary/supervisor/director-retry 的**契约重构**,不只是呈现。**评估边界后**:若确认只需「secretary:185 + global_supervisor:410 两处 `humanize_consult_error` 改调新模块取 `human` 字段」而**不动 director retry 读法** → 可本片顺手消这两处 copy-paste;若要动 director 1384/1396 对前缀的读法 → **停手、归 §8 单独一步**,别为消重造 retry 回归。
- **收编一整家(2026-07-09 能力普查①逼出·别只收 fix8 变第五套)**:现有错误人话/标记散在四处,C6 要么收编要么路由,**不许再加第五套并行**:
  1. `classify_codex_provider_failure`(runner:386)= 底·判据搬进新模块(见 (a));
  2. **`humanize_consult_error`×2(`secretary_agent.rs:185` + `global_supervisor_agent.rs:410`·逐字节真重复)**= 剥 `codex_provider_unavailable:` 前缀取人话——按 (b) 评估边界后定本片消/缓;
  3. `classify_codex_resume_failure`(`workflow_execution_entrypoints.rs:217`·:226 调 fix8·被 `workflow_run_dispatch_entrypoints.rs:872`/`:883` 调)= 复用非重复→ **改路由到新 `classify_run_error`**(老函数删/改后它才编得过·签名别变、两 dispatch 调用点随之受益);
  - **验收加一条**:改完 `grep -rn 'classify_codex_provider_failure\|humanize_consult_error' 生产码` = 零残留、或只剩「路由到新模块 / 取新模块 human 字段」的调用,证「收编成一套·没留散的两套翻译器」(humanize×2 若按 (b) 缓,则本条只验前缀判据单一真源、humanize 归 §8 明列);
- **错误族(保守表·大小写不敏感·延续 fix8「拿不准不装」)**:① 供给类(收编老判据:`subscription_not_found`/`usage limit`/`quota`/`unauthorized`/`403`/`401`/`reconnecting 5/5`) ② 断供/网络(reconnecting/stream disconnected) ③ 超时(timed_out·与既有超时打回主管一致) ④ 沙箱/权限拒绝(sandbox/permission denied/read-only·**判据一致于现成 `classify_phase_b_stderr_for_codex_state_error`/`phase_b_mentions_codex_state_error`**) ⑤ 命令失败(exit code≠0·带命令上下文) ⑥ codex 内部子系统(memories_write/no such table 类·翻「codex 自身某子系统报错·一般不影响本次任务」) ⑦ 口供读取失败(`consult_last_message_read_failed`);
- **unknown 兜底(保守归一化纪律)**:未命中任何族 → `family=unknown`、`human="未识别错误(附原文供排查)"`、`raw_snippet` 带原文——**不硬编假人话**;
- **结构化不吞真相**:`raw_snippet` 永远保留原文(截断安全),替代现在「人话」与「原文」二选一挤进一个 error string 的做法。

### 2.2 接到 run-history 详情位

- 把 `classify_run_error` 的结构化结果落进 step/run 记录的错误位(现状 `state_note` 只有人话尾巴·停因半包接),让 run-history 读模型能投影出`{人话摘要 + 可下钻原文 + 错误族}`;
- **不改成败判定**:A 只加呈现字段·节点 failed/completed 判定一字不动(延续 fix8「只影响报告」)。

### 2.3 前端两层脸(一处呈现·就近工作历史左栏)

- 就近现成 run-history 呈现位(交办页工作历史左栏/旧单详情卡·`src/views/projects/ProjectJiaobanPanel.tsx`:`JiaobanHistoryColumn`/`JiaobanHistoryDetail`/`classifyBlocked`):挂单失败时默认显**人话摘要+错误族标**,一个「查看原文」下钻展原始 stderr/错误;
- 呈现不阻断·不是闸(同黄牌哲学);死配对绝不零按钮(延续 classifyBlocked 纪律·下钻是增益不是必点)。

### 2.4 与 C5 边界(别抢活)

- C5 = 链 `event_type` 向 13 词表对齐 + `entry_type` 枚举校验(审计账本层)。A 的错误事件**天然是 audit 事件一类**,但 A 只做「翻译 + 呈现」;**审计词表/枚举归 C5**——A 落错误呈现字段时用现成事件写入口,不新造 event_type 命名(碰到要新造 → 停手·归 C5 一起拍)。

## 3. 安全死线(收编改了原「runner 全 0-diff」·精确重划)

- **冻结核 0-diff(绝不碰)**:`command_plan_for`(codex_local_runner.rs:1586)/ `run_real_codex_process` / `RealCodexLocalPhaseBProcessRunner::run_phase_b` 进程本体 / 沙箱 / 任何安全闸 / 人闸 / prepare guard / 四护栏;
- **收编可动区(仅此·用户 07-09 授权)**:runner **报告层**——`classify_codex_provider_failure`(:386·判据搬走)+ `append_stderr_tail`(:409·并入或保留)+ 消费 run_phase_b 结果做翻译的那段(:354-381 两调用点改调新模块)。**判据:改的是"拿到结果后怎么翻译呈现",不是"怎么起进程/怎么沙箱/怎么判成败"**;
- **retry 契约不许断(2026-07-10 补)**:动 `codex_provider_unavailable:` 前缀时,`director_agent.rs:1384`/`:1396` 对供给类的识别**必须照旧成立**(供给类仍不 retry);做不到 → 前缀留着、按 §2.1(b) 把 director 改读归 §8,别硬删;
- **红线**:收编若发现必须动冻结核(command_plan_for/run_phase_b 本体/沙箱)才能完成 → **停手报回**,不许为收编松冻结核;同理为消重必须重构 director retry 契约 → 停手归 §8;
- 不改任何节点成败判定/链态驱动(A 纯增呈现·翻译层"只影响报告不改成败"是 fix8 原纪律·收编后照守);
- `.codex` 凭据不碰;真跑属测试项目轻档。

## 4. 验收

- **单测**:分类器七族各一命中样本 + unknown 兜底(装人话骗人=不通过·必带原文)+ 大小写不敏感 + 现成供给类判据搬家后不回归;结构化`{人话/原文/族}`三段齐;**族④判据与 `phase_b_mentions_codex_state_error` 对同一 state-db 只读样本判定一致**(证没造矛盾探测);
- **retry 不回归**:供给类错误经新模块后,`is_director_plan_flaky_early_exit` 仍判「不 retry」(现成 fix8 测试 `lib.rs:8745` 一带的供给类不 retry 断言逐条绿);
- **接线测**:一条失败 run → run-history 读模型投影出人话摘要+可下钻原文+族,成败判定字段逐字节不变(呈现纯增·不驱动);
- **真跑**(`#[ignore]`·测试项目):故意造一个非供给类失败(如超时/命令 exit≠0),核界面出人话而非裸 stderr、下钻能看原文;顺带核 07-08 那条 memories 子系统错现在翻成人话;
- **前端**:失败单默认人话脸+下钻原文·死配对不零按钮(真机一眼);
- 三闸绿 + 0-diff 自证(runner 冻结核 `git diff` 零命中)+ 计数不降(现基线 `cargo test --lib` = 753/0/43·2026-07-10)+ fmt 净(**用权威 `cargo fmt --check`·别 ad-hoc rustfmt**)。

## 5. 回交

- §4 证据 + 分类器覆盖清单(七族+unknown) + 0-diff 自证 + 收编落点清单(改了哪几处调用/删没删 humanize) + `grep` 残留自证 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 裸错误/stderr 直接上脸(违 07-09 硬约束·必人话) / unknown 硬编假人话(必保守兜底带原文) / 碰冻结核 `command_plan_for`·`run_phase_b` 进程本体·沙箱·安全闸(收编只动报告层·见 §3) / **裸删 `codex_provider_unavailable:` 前缀致 director retry 断供给类识别**(§2.1(a)硬约束) / **为消 humanize×2 而重构 director retry 契约**(归 §8) / 留两套翻译器不收编(前缀判据必单一真源) / 另造与现成 state-error 探测器矛盾的判据 / 改成败判定或链态 / 新造 audit event_type(归 C5) / 做 B(开发者工具·未拍) / 呈现变成闸(黄牌哲学:呈现不阻断)。

## 8. 派前决定(用户 2026-07-09 已拍·闭)+ 本片内边界(2026-07-10 主导线定)

- ✅ **A 落位 = B2 尾片 C6**(观测补强·「顺手做」·不等 Phase C)——roadmap B2 行已记 C6;
- ✅ **fix8 分类器 = 收编**(不是复用):供给类**判据**搬进新模块、runner 两调用点 + resume 分类器改调、单一真源。死线精确重划见 §3;
- ✅ **排期约束已满足**:草案「C1 收尾轮清后才派」——B2 于 2026-07-10 整个收口(C1-C5 done),此约束解除,**本包可派**;
- ⚖️ **本片 vs §8 后续的边界(2026-07-10 核实物定)**:
  - **本片做**:新模块 + 七族分类器 + 供给类判据单一真源 + runner 非供给类错误路由翻译 + run-history 详情位 + 前端两层脸 + 供给类稳定信号保 director retry。
  - **评估边界后定(倾向本片顺手,除非动 retry 读法)**:`humanize_consult_error`×2 两处改调新模块取 `human` 字段消 copy-paste。
  - **明确归后续单独一步(别本片硬做)**:彻底删 `codex_provider_unavailable:` 前缀 + director 1384/1396 改读结构化 family 的 retry 契约重构——单独评估、单独一步,不为消重造 retry 回归。
