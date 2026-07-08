# 实现任务包:A·运行错误上脸(人话诊断层)· 主导线 → 执行线 v1(草案·待派)

日期:2026-07-09　性质:**轻档**(后端翻译层+读模型+前端呈现;冻结核/安全闸 0-diff·但收编 fix8 报告层要动 runner 报告区·见 §3)。落位:**B2 尾片 C6(观测补强·用户 07-09「顺手做」)**。状态:**待派·C1 收尾轮清后派**(避免与 C1 执行线撞车·并让 A 吸收 auto_advance 接线后的最终形态);**两派前决定已拍(§8)**。上承:提案 `docs/plans/2026-07-09-run-error-surface-plain-language-proposal-v1.md`(方向已拍·反馈必人话)。

## 0. 接手须知(冷启即读·本包自包含)

- 你是**执行线**(后端为主+一处前端呈现)。**子线不 commit。** 全程中文。
- **A 是什么 / 不是什么(硬约束·用户 07-09 拍)**:A = 捕获 codex/worker 运行的原始错误 → **翻译成人话** → 按 run 上脸。**A 不是**把 `--json` 裸错误/stderr 灌进面板。默认脸给人话,原文只在下钻时看。
- **主导线已勘的接缝(直接用·省你重找)**:
  1. **现成翻译层就一处**:`codex_local_runner.rs:386 classify_codex_provider_failure`(fix8 加的·**非** `command_plan_for` 冻结核)——只认供给类(`subscription_not_found`/`usage limit`/`quota`/`unauthorized`/`403`/`401`/`reconnecting 5/5`),命中→人话前缀 `codex_provider_unavailable:`;
  2. **未命中就吞真相**:`codex_local_runner.rs:409 append_stderr_tail` 把裸 stderr 截 200 贴在原错误后,**不翻译**——非供给类错误(codex 子系统报错/沙箱拒绝/命令 exit≠0/超时/`consult_last_message_read_failed`)全落这条,这是 A 要补的洞;
  3. **run-history 停因是半包接**:`run_history_read_model.rs:28` `state`=机器键、`state_note`=人话一句;:315 注释「只给人话状态尾巴;具体停因在『工作流』详情看(UI 半包接)」——A 要把翻译后的错误喂到这个「详情」位;
  4. **黄牌范式(呈现不驱动·照抄哲学)**:`worker_report.rs:57-60` `report_warning`/`report_status`,前端据此判黄牌、不改链态。A 的错误呈现同哲学:**呈现不阻断·不是闸**;
  5. **活证据**:2026-07-08 worker transcript 里 `codex_memories_write::phase2::job: failed to claim job (no such table: jobs)`——当前界面零呈现,A 完成后应翻成「codex 记忆子系统写入失败(本地缺表)·不影响本次任务结果」这类人话。

## 1. 拍板摘要

- **做什么**:把 fix8 翻译层从「只认供给类」推广成**错误族全谱分类器**(结构化返回`{人话摘要, 原文, 错误族}`),接到 run-history 详情位,前端两层脸(默认人话·下钻原文)。
- **为什么**:主线是「让编排可观测」;现在 worker 挂了只看得到语义层(节点 failed/停因一句),原始诊断层零呈现,调试要手扒 transcript。
- **不做**:B(开发者工具/devtools·用户方式未定);把裸错误直接上脸(违 07-09 硬约束);改成败判定(A 只影响呈现·延续 fix8「不改成败」)。

## 一句话判据

**「是不是只:加一个错误族分类器(结构化`{人话/原文/族}`·unknown 保守兜底)+ 喂到 run-history 详情位 + 前端两层脸呈现——而 runner `command_plan_for`/`run_real_codex_process`/沙箱/安全闸 0-diff、不改任何成败判定、呈现不阻断?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 错误族全谱分类器(翻译层核心)

- **新增独立模块** `run_error_translation.rs`:`classify_run_error(raw: &str) -> RunErrorHuman { family, human, raw_snippet }`;
- **收编 fix8(用户 07-09 拍)**:把现成 `classify_codex_provider_failure`(codex_local_runner.rs:386)的**供给类判据整段搬进**新模块作为族①,**删掉 runner 里的老函数**,把两个调用点(:364/:373 `if let Some(human) = classify_codex_provider_failure(...)`)改调新模块 `classify_run_error`——单一真源,不留两套(理由:老函数仅 ~20 行、留着两套翻译器会漂移)。**注意:这动的是 runner 报告层不是冻结核**,边界见 §3;
- **错误族(保守表·大小写不敏感·延续 fix8「拿不准不装」)**:① 供给类(收编老判据:`subscription_not_found`/`usage limit`/`quota`/`unauthorized`/`403`/`401`/`reconnecting 5/5`) ② 断供/网络(reconnecting/stream disconnected) ③ 超时(timed_out·与既有超时打回主管一致) ④ 沙箱/权限拒绝(sandbox/permission denied/read-only) ⑤ 命令失败(exit code≠0·带命令上下文) ⑥ codex 内部子系统(memories_write/no such table 类·翻「codex 自身某子系统报错·一般不影响本次任务」) ⑦ 口供读取失败(`consult_last_message_read_failed`);
- **unknown 兜底(保守归一化纪律)**:未命中任何族 → `family=unknown`、`human="未识别错误(附原文供排查)"`、`raw_snippet` 带原文——**不硬编假人话**;
- **结构化不吞真相**:`raw_snippet` 永远保留原文(截断安全),替代现在「人话」与「原文」二选一挤进一个 error string 的做法。

### 2.2 接到 run-history 详情位

- 把 `classify_run_error` 的结构化结果落进 step/run 记录的错误位(现状 `state_note` 只有人话尾巴·停因半包接),让 run-history 读模型能投影出`{人话摘要 + 可下钻原文 + 错误族}`;
- **不改成败判定**:A 只加呈现字段·节点 failed/completed 判定一字不动(延续 fix8「只影响报告」)。

### 2.3 前端两层脸(一处呈现·就近工作历史左栏)

- 就近现成 run-history 呈现位(交办页工作历史左栏/旧单详情卡·`ProjectJiaobanPanel.tsx`):挂单失败时默认显**人话摘要+错误族标**,一个「查看原文」下钻展原始 stderr/错误;
- 呈现不阻断·不是闸(同黄牌哲学);死配对绝不零按钮(延续 classifyBlocked 纪律·下钻是增益不是必点)。

### 2.4 与 C5 边界(别抢活)

- C5 = 链 `event_type` 向 13 词表对齐 + `entry_type` 枚举校验(审计账本层)。A 的错误事件**天然是 audit 事件一类**,但 A 只做「翻译 + 呈现」;**审计词表/枚举归 C5**——A 落错误呈现字段时用现成事件写入口,不新造 event_type 命名(碰到要新造 → 停手·归 C5 一起拍)。

## 3. 安全死线(收编改了原「runner 全 0-diff」·精确重划)

- **冻结核 0-diff(绝不碰)**:`command_plan_for`(codex_local_runner.rs:1586)/ `run_real_codex_process` / `RealCodexLocalPhaseBProcessRunner::run_phase_b` 进程本体 / 沙箱 / 任何安全闸 / 人闸 / prepare guard / 四护栏;
- **收编可动区(仅此·用户 07-09 授权)**:runner **报告层**——`classify_codex_provider_failure`(:386·删)+ `append_stderr_tail`(:409·并入或保留)+ 消费 run_phase_b 结果做翻译的那段(:354-381 两调用点改调新模块)。**判据:改的是"拿到结果后怎么翻译呈现",不是"怎么起进程/怎么沙箱/怎么判成败"**;
- **红线**:收编若发现必须动冻结核(command_plan_for/run_phase_b 本体/沙箱)才能完成 → **停手报回**,不许为收编松冻结核;
- 不改任何节点成败判定/链态驱动(A 纯增呈现·翻译层"只影响报告不改成败"是 fix8 原纪律·收编后照守);
- `.codex` 凭据不碰;真跑属测试项目轻档。

## 4. 验收

- **单测**:分类器七族各一命中样本 + unknown 兜底(装人话骗人=不通过·必带原文)+ 大小写不敏感 + 现成供给类判据复用不回归;结构化`{人话/原文/族}`三段齐;
- **接线测**:一条失败 run → run-history 读模型投影出人话摘要+可下钻原文+族,成败判定字段逐字节不变(呈现纯增·不驱动);
- **真跑**(`#[ignore]`·测试项目):故意造一个非供给类失败(如超时/命令 exit≠0),核界面出人话而非裸 stderr、下钻能看原文;顺带核 07-08 那条 memories 子系统错现在翻成人话;
- **前端**:失败单默认人话脸+下钻原文·死配对不零按钮(真机一眼);
- 三闸绿 + 0-diff 自证(runner 冻结核 git diff 零命中)+ 计数不降 + fmt 净。

## 5. 回交

- §4 证据 + 分类器覆盖清单(七族+unknown) + 0-diff 自证 + 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 裸错误/stderr 直接上脸(违 07-09 硬约束·必人话) / unknown 硬编假人话(必保守兜底带原文) / 碰冻结核 `command_plan_for`·`run_phase_b` 进程本体·沙箱·安全闸(收编只动报告层·见 §3) / 留两套翻译器不删老的(用户拍收编=单一真源) / 改成败判定或链态 / 新造 audit event_type(归 C5) / 做 B(开发者工具·未拍) / 呈现变成闸(黄牌哲学:呈现不阻断)。

## 8. 派前决定(用户 2026-07-09 已拍·闭)

- ✅ **A 落位 = B2 尾片 C6**(观测补强·「顺手做」·不等 Phase C)——roadmap B2 行加 C6;
- ✅ **fix8 分类器 = 收编**(不是复用):供给类判据整段搬进新模块、删 runner 老函数、改两调用点——单一真源(理由:老函数 ~20 行·两套会漂移)。死线精确重划见 §3;
- 唯一剩的排期约束:**C1 收尾轮清后才派**(不与 C1 执行线撞 runner·且吸收 auto_advance 最终形态)。
