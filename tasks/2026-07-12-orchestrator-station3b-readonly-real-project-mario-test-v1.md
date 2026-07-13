# 任务包:站 3b——只读单进真实项目「mario test」(v1)

日期:2026-07-12 · 档位:**重档**(高危#1 codex 进非测试真实项目 + 高危#3 改安全闸,均由用户本次拍板一并覆盖)· 状态:**✅ PASS（2026-07-13·单 worker·零写根·真实 UI 闭环）**

> 2026-07-13 真实复测已完成：新 run `supervisor:workflow-users-yoyi-documents-mario-test-default:1783918485705864000` 只派发一个 worker，完成 `dispatch → inspect → finalize(pass) → report_user`；写根为空，项目前后 7 个内容文件 SHA-256 全同。总证据：`evidence/2026-07-13-orchestrator-station3b-mario-test-readonly-real-run-v1.md`。

## 00 拍板面(等用户「可以」才动第一行代码)

1. **项目** = `/Users/yoyi/Documents/mario test`(**路径含空格**,全链注意)。勘察事实:5 月底 codex(vscode 线,threads 可查 4 条会话)所建 H5 小游戏;7 个内容文件共 140K(index.html / game.js 8.9K / styles.css / README.md);带 `.git` 可回滚;`.workbench/` 内存有 B2/stage-k 时代旧写探针(pcr9 / h5-b2 / stage-k/k2)——**历史遗物,不得当本轮证据**。
2. **只读目标**(单文)= §02。
3. **可读根** = 仅 `/Users/yoyi/Documents/mario test`,写根 = **空**。
4. **开闸方案** = §03(并列小闸;S1 原闸零字节)。

批准语义:只覆盖「该项目 + 只读 + 零写根」的一次授权;不外推其它项目、写单、自动连环;不复用站 3a 固定测试授权。

## 00.1 实施修订记录(2026-07-12 · 用户「直接开始推进」·总指导亲执行开闸段)

1. **§03-e 实勘结果**:worker 链**零改动**——冻结核 `command_plan_for` 完全参数化(`-C`/`--sandbox`/`--add-dir` 全来自数据、argv-only 无 shell),配合 commands 侧「缺或空 allowed_write 都是只读」既有纪律,3b worker 自然拿到 `-C "/Users/yoyi/Documents/mario test" --sandbox read-only`。
2. **包外新缺口(已修)**:只读授权段原**不物化任务包**(`pilot_task=None`)→ 主管无可派 work item,`dispatch_worker` 必失败——站 2 只读单主管独狼的根因。撤 None 分支,只读单一律物化(launcher :526);「主管写单任务包」话术改中性「主管任务包」(×7)。
3. **顺手收紧(案发测试逼出)**:`ensure_supervisor_pilot_write_scope` 自身 fail-closed——非测试非 3b 项目即使零写根也拒(旧实现空写根对任意项目 vacuous-Ok,全靠入口闸兜底)。
4. **基线修订**:mario test 的 `.git` 是**空壳**(无 commit、全文件 untracked)→ `git status` 对 untracked 内容改动不敏感,§05.2 零写铁证改为**全 7 文件 SHA-256 + 全树清单**比对(已存 `evidence/raw/2026-07-12-station3b-mario-test-readonly/pre-launch-baseline.txt`)。
5. **发射预检全绿**:项目已在应用索引(B2 时代注册);workflow-state 已有 `workflow:users-yoyi-documents-mario-test:default`(651 处引用);平台账本在 app-global 目录、不写项目内。
6. **开闸段验证**:`cargo test --lib` **870/0/43**(基线 867+新增 3 条案发测试);typecheck ✓;offline-interaction 22 组全过(pilot-switch 新增 12 条 3b 断言);`cargo fmt --check` 仅剩 3 个历史漂移文件;**commands.rs 纯增量**(`git diff` 零删行=S1 原闸/j2_b_b1 封条 0-diff 铁证)。
7. 改动面:5 文件 +284/−44(commands.rs / supervisor_session_launcher.rs / mcp/supervisor_orchestrator.rs / ProjectJiaobanPanel.tsx / jiaoban-supervisor-pilot-switch.test.tsx)。
8. **首发实拦复盘(attempt-1,2026-07-12 真机)**:S1 执行层合一闸 `authorization_complete` 原只认测试项目 path-lock → 主管授权 3b 只读派发被 `blocked_waiting_authorization` 安全拦下(安全面逐项正确:临时 HOME 建/清、C1 全新绑定、失败即弃绑定、主管停 waiting_user)。修复=新增 `real_execution_authorization_complete`(测试 path-lock ∨ 主管授权∧3b∧零写根),判决体一字不动,案发测试 6 断言;**实证收获**:guard 安全子集对 read-only+空写根首跑即过。复盘全文:`evidence/raw/2026-07-12-station3b-mario-test-readonly/attempt-1-blocked-safely.md`。验证更新:cargo test --lib **871/0/43**。
9. 雷区预扫(attempt-2 前):`has_inflight_dispatch` 只数 `running`(当前 0 条);attempt-1 残留 1 条 prepared 挂已耗授权段下、B2 旧账 45 条 prepared 均惰性(唯一性过滤按 plan_authorization_id 圈定)。
10. **attempt-2(consult 空验收)**:出方案时 consult 对纯报告型目标返回空 `worker_acceptance_criteria`,proposal 落店校验(`validate_role_acceptance_criteria`)当场拒收——方案没建成、零消耗、零派发。根因:consult 提示词对三类验收的示例全是写单口味,只读单被判「无 worker 事实」。修:consult 提示词补死规则「纯咨询/只读/盘点类目标三类验收同样不许为空,worker 验收=口供硬要求」。
11. **attempt-3(worker 真跑通·闭环卡回程带出)**:worker 在真项目派发→完成→只读侦察高质量交付(promise_verdicts 5 条带 README:line 原文 + top_5_issues 前 5 问题带 file:line;P0 delta-time 缺失等真 bug),**项目目录逐字节零写**(7 文件 SHA-256 全对基线)。但完整口供**没到主管手里**:worker 回程契约 `WorkerReport` 只认 `did/outputs/status/evidence`(为写单设计),worker 自造的 `promise_verdicts/top_5_issues` 顶层字段被 serde 静默丢弃 → 主管 inspect 只见摘要 → 判证据不足 → 试图 follow_up 撞授权闸 → 停 waiting_user 请用户决定。**这是拆包疏漏**:任务单要 worker 交结构化侦察报告,却没和回程契约对齐。worker 完整口供已存证 `evidence/raw/.../worker-full-testimony.json`(6.3KB)。用户拍板修法 ①。
12. **① 修复(回程加通用结论字段)**:`WorkerReport` 加 `#[serde(default)] findings: Vec<String>`(只读/分析/审查/盘点类单的结论正文,每条带 file:line+原文;写单留空;不进 `has_help_fields`、不触发 blocked)+ 契约文本 `WORKER_REPORT_CONTRACT_TEXT` 补 findings 说明与只读示例、明示「勿自造 promise_verdicts/top_5_issues」+ inspect 投影 `normalized_raw_worker_report` 带出 `findings`。冻结判决体/经典链 consume/落库均不动(findings 不影响 did/status/help 逻辑)。测试:worker_report 层 2 条(findings 解析、自造字段丢弃 findings 保留)+ 投影层 1 条 end-to-end(read_worker_report 带出 findings、仍 reported_completed、自造字段不现)。验证:cargo test --lib **874/0/43**;fmt 仅 3 历史漂移。**待真机重发验证主管终标闭环**。
13. **attempt-4 PASS（2026-07-13）**：UI 新建 proposal/authorization/work item/run/native thread 五件套；`allowed_tools=[read_file]`、`allowed_checks=[node --check game.js]`、`allowed_write=[]`。主管只派发一个 worker，worker 逐行读取四个文件并按契约把承诺判断、前 5 问题和总评放入 `findings`；控制核心成功 inspect，随后 advisory `finalize(pass)` 与 `report_user` 均落账，`follow_up_count=0`。前后 `git status`、7 文件清单、7 个 SHA-256、`node --check` 全一致。证据：`evidence/2026-07-13-orchestrator-station3b-mario-test-readonly-real-run-v1.md`。

## 01 一句话

在真实项目 mario test 上跑通主管编排**第一单真活**:主管派 1 个只读 worker 侦察项目 → 口供带证 → advisory 终标 → 报告用户——全程物理不可写,全新五件套身份,证据单独存。

## 02 只读单(worker 任务原文)

> 盘点 `/Users/yoyi/Documents/mario test` 现状:
> ① 对照 README.md 的承诺逐条判「已实现 / 未实现 / 部分实现」;
> ② 给出问题清单(bug / 手感 / 代码结构),**每条必须带 file:line + 原文引用**,按影响排序取前 5;
> ③ 50 字内总评。
> 产出 = 结构化口供;**不写任何文件**。

为什么算真活:每条结论可逐条 grep 核对;报告对用户后续迭代该游戏直接可用。

## 03 开闸设计(高危#3 · 最小面)

**死原则:**
- `workflow_engine_test_project_unsealed`(commands.rs:2105)与 `require_test_project_path_lock`(≈:2112)**函数体零字节不动**——全局 S1 闸与 legacy 封条不放宽;
- 新增**并列小闸**,建议名 `station3b_readonly_project_unsealed(project_root, allowed_write_roots)`:仅当 `project_root == "/Users/yoyi/Documents/mario test" && allowed_write_roots 为空` 才真;**只挂主管编排链路**的入口,逐处 OR,不挂任何 legacy / 写路径;
- 冻结核(`command_plan_for` / `run_phase_b` / 沙箱物化 / 人闸)**0-diff**;若 worker 发射撞冻结核,照站 2 launcher 先例**旁建只读变体**,不改冻结件。

**已勘座标(改动落点):**

a. 前端 `src/views/projects/ProjectJiaobanPanel.tsx:46-57`——`TEST_PROJECT_ROOT` 常量旁加 3B 常量;`supervisorPilotUnavailableReason` 加分支:3b 根 && 写根空 → 可用(返回 null);3b 根 && 写根非空 → 「站 3b 仅限只读(零写根)」。同名常量在 `WorkflowCommandConsoleView` 也有一份注释提及,勘一眼是否需同步。
b. `src-tauri/src/supervisor_session_launcher.rs:559` `ensure_supervisor_pilot_write_scope`——注意现状 `all()` 对空集为真(空写根已能过);要补的是**显式语义**:3b 项目必须空写根,非空即拒(错误话术照 canon 风格)。
c. 同文件 :1044-1048 / :1067 主管 argv `-C` / `current_dir` 硬编码测试根——**建议不动**:主管会话仍留固定测试根(它不需要项目文件,读 MCP 只读投影即可,面最小)。若实现中发现主管必须进 3b 目录才能工作,**先停下来报**,不擅自扩。
d. 同文件 :1119 `validate_supervisor_argv`——`workflow_engine_test_project_unsealed(project_root)` 处 OR 上 3b 小闸;`--sandbox read-only` 检查与 `--ignore-user-config` 禁令**死线不动**。
e. **worker 派发链路**(本包最重的勘察活):grep 全仓 `workflow_engine_test_project_unsealed|require_test_project_path_lock` 全部调用点,逐个分类「挂 3b 小闸(仅主管编排派发所经)/ 不动(其余全部)」,**分类清单进证据**;`supervisor_action_controller` 派发适配器到真 runner 的路径亲勘;worker argv 必须物化 `--sandbox read-only`(写根空 ⇒ 只读沙箱,不能只靠写根为空兜底)。
f. h5 fail-closed(空写根 = 零写授权)3a 已修,3b 直接受益,不重复改。

**案发测试(每层都要):**其它真实项目根 → 拒;3b 根 + 非空写根 → 拒;3b 根 + workspace-write argv → 拒;legacy 入口喂 3b 根 → 仍 blocked;既有测试(如 supervisor_session_launcher.rs:2274 write-scope 案发测)**一条不许放松**。

## 04 发射流程

0) 应用项目列表若无 mario test → UI 添加(轻档)。
1) **发射前基线**：保存 `git status --short`、全部 7 个内容文件清单与逐文件 SHA-256；该仓库无 commit，不能用 `rev-parse HEAD` 代替内容基线。
2) 正常交办流:出方案(§02 单文)→ 用户[允许并开始](主管编排模式)→ dispatch → inspect(追问在预算内)→ finalize(advisory)→ report_user。
3) **全新身份五件套**:authorization / work item / supervisor run / native worker thread / binding——严禁复用 v7 / 站 2 任何 id。
4) 发射后:重跑基线命令,逐字节比对。

## 05 验收(预写死)

1. **越权 0**:账本无任何写动作;主管 + worker argv 均含 `--sandbox read-only`;
2. **物理零写**：前后 `git status --short`、7 个内容文件清单和逐文件 SHA-256 全一致；`.workbench/` 三个历史探针纳入同一哈希基线，不当本轮新增证据；
3. **链路闭环**:dispatch → inspect → finalize → report 完整;advisory 不写链态(`workflow_chain_state_written=false` 维持);
4. **口供质量**:5 条问题全部带 file:line + 引文;
5. **身份全新**:五件套 id 与 v7 / 站 2 零交集(grep 证明);
6. **回归**:`cargo test --lib` 全绿(867 基线只增不减)+ 新增案发测试;`npm run typecheck` 过;
7. **S1 原闸 0-diff**:`git diff` 中两个原闸函数体零改动;
8. **证据**:`evidence/raw/2026-07-12-station3b-mario-test-readonly/`(基线 / argv / sidecar / 口供原文 / 前后 git status / SHA256SUMS)+ 总证据 md;
9. **交付真活**:口供报告本身对用户可用(「第一单真活交付+用户认」的「交付」半边;「认」半边在用户)。

## 06 变更辐射面

- 碰安全闸文件:commands.rs(**只加不改**)/ supervisor_session_launcher.rs / ProjectJiaobanPanel.tsx;
- **路径含空格**:全链禁 shell 字符串拼接路径,argv 单元素传递;grep 派发链上 `format!` 拼 command 的点,发现即报;
- 主管 cwd 不动(§03-c),辐射面不含主管 argv 构建主体;
- 不碰:经典管线、人闸、冻结核、legacy 封条、店锁、binding 迁移、`.workbench` 旧探针。

## 07 边界

- 不做:写单、其它项目、自动连环、把 advisory 升级为自动决定、产品化按项目授权 UI(Phase D 的活);
- 卡住 / 歧义 → 停下来报,不擅自扩权;
- 完成报告必须「怎么验的 + 证据」;没验 = 「已实现,未验证」,不许说做好了。
