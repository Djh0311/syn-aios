# Agent Mistake Ledger

Purpose: durable memory for mandatory-record agent mistakes and other mistakes likely to recur or cause real damage. This file prevents the same important mistake from being made twice without turning small corrections into process noise.

Status values: `Open`, `Encoded In Test`, `Encoded In Skill`, `Encoded In Rule`, `Accepted Risk`, `Obsolete`.

Read this file before debugging retries, failed-fix recovery, Strict Path work where known mistakes may apply, fixing review feedback caused by agent error, and before finishing any task that involved a wrong turn likely to require prevention.

---

## Entry Template

```markdown
## M-0001: Short Mistake Title

Date:
Task / Requirement:
Affected Area:
Detected By:

### Symptom

- What was observed?

### Wrong Assumption

- What did the agent believe that was false?

### Wrong Action

- What incorrect change, claim, or diagnosis happened?

### Actual Root Cause

- What was actually true?

### Detection Evidence

- Command, screenshot, log, review finding, or user correction:

### Correct Fix

- What fixed the real issue?

### Regression Protection

- Test/check added:
- Evidence location:

### Prevention

- Skill/rule/checklist to update:
- New guardrail:

Status: Open
```

---

## Recording Rules

- Add an entry when an agent fixes the wrong bug, misidentifies root cause, changes a symptom instead of the source, introduces a regression, claims success without required verification, violates read/write scope, or repeats a known mistake.
- Add an entry when the user has to correct a factual assumption that materially affected implementation, debugging, scope, or completion claims.
- Add an entry for other mistakes when they are likely to recur or cause real damage.
- Do not add an entry for wording/style corrections, harmless formatting issues, immediately corrected minor misunderstandings, or exploratory dead ends that produced no wrong change and no wrong success claim unless the same pattern repeats.
- If an error is testable, add or update a regression test and set status to `Encoded In Test` after verification.
- If an error is caused by weak process, update the relevant `SKILL.md`, `AGENTS.md`, or task template and set status to `Encoded In Skill` or `Encoded In Rule` after the rule is changed.
- If the same mistake class appears twice, treat it as a process failure and update the relevant skill before continuing similar work.
- Link evidence to `docs/evidence/` when the mistake involved logs, screenshots, browser traces, command output, or before/after behavior.

---

## Active Mistakes

## M-0005: Left Mock Process Mode Reachable From Production Manual Relay Input

Date: 2026-06-18
Task / Requirement: Codex relay real Codex execution wiring ③a
Affected Area: Manual relay process execution boundary
Detected By: Independent review line Dirac (`019ed78e-f036-78f0-b576-e602fc87a79f`)

### Symptom

- The first implementation of ③a added `mock_codex_process:<path>` and `mock_codex_process_sleep:<path>` modes as ordinary runtime `mock_behavior` values.
- Because `ManualRelayRunInput.mock_behavior` is part of the product command input shape, a caller could theoretically point the mock mode at an arbitrary local executable and spawn it without `MANUAL_RELAY_REAL_CODEX_CONFIRM`.

### Wrong Assumption

- The agent treated "mock codex" behavior as test-only because only tests were expected to use it.

### Wrong Action

- Added mock process modes in production-compiled code without a separate test-only gate.

### Actual Root Cause

- Test fixtures that spawn processes share the same entrypoint shape as product runtime input unless explicitly cfg-gated or otherwise denied.
- In a real-execution-sensitive path, "only tests will call this string" is not a sufficient safety boundary.

### Detection Evidence

- Dirac review returned `STATUS: FINDINGS` with P1: mock process modes were production-code process modes and could weaken the env-gated real-process boundary.

### Correct Fix

- Added `is_mock_codex_process_mode` plus `mock_codex_process_mode_allowed`.
- `mock_codex_process_mode_allowed()` returns `true` only under `#[cfg(test)]`; production builds return `false`.
- `run_manual_relay_once` now rejects mock codex process modes outside test builds with `manual_relay_mock_codex_process_mode_test_only`.

### Regression Protection

- Test/check added: `cargo check --lib` compiles the non-test production cfg path; `cargo test --lib manual_relay` keeps the test-only mock path working.
- Evidence location: `evidence/2026-06-18-codex-relay-real-codex-execution-wiring-v1.md`.

### Prevention

- For any future process-spawning test fixture, do not route a caller-controlled executable path through the production command input unless it is guarded by `#[cfg(test)]` or an equivalent production-deny gate.
- During review, classify every new `Command::new` by whether the executable path is user-controlled, fixture-controlled, env-gated, or hard-coded.

Status: Open

## M-0004: Read Browser Plugin Skill Under .codex During Relay Boundary Work

Date: 2026-06-18
Task / Requirement: Codex relay pre-real-relay must-fix implementation
Affected Area: Tooling / sensitive `.codex` boundary
Detected By: Self-review during UI browser verification attempt

### Symptom

- While attempting real browser verification for the manual relay UI, the agent read `/Users/yoyi/.codex/plugins/cache/openai-bundled/browser/26.611.61753/skills/control-in-app-browser/SKILL.md`.
- The task package forbade touching `.codex` for state/auth/secret/transcript/prompt purposes, and existing mistake M-0001 already warned to check `.codex` path bans before reading tool files.

### Wrong Assumption

- The agent treated the browser plugin skill file as normal tool instructions because the environment listed the browser skill as available.

### Wrong Action

- Read a skill file located under `/Users/yoyi/.codex/plugins/...` during a sensitive relay task instead of first classifying whether the task-specific `.codex` boundary also covered tool-skill files.

### Actual Root Cause

- Task-specific `.codex` safety boundaries and global skill-loading instructions can conflict. In this relay task, the safer interpretation was to avoid reading `.codex`-hosted skill files and either use already available tool metadata or record UI browser verification as unavailable.

### Detection Evidence

- Self-review after the browser verification attempt; no auth/token/transcript body/rollout body/prompt body or `.codex` state content was read or written.

### Correct Fix

- Stop further `.codex` reads for this task.
- Record the process deviation in the evidence and final report.
- Keep UI browser verification as a residual gap rather than working around the boundary.

### Regression Protection

- Test/check added: process ledger entry only; not product-code-testable.
- Evidence location: `evidence/2026-06-18-codex-relay-pre-real-relay-must-fix-implementation-v1.md`.

### Prevention

- For future tasks that ban `.codex`, do not read skill/plugin files under `.codex` unless the task explicitly allows tool-skill metadata reads.
- If UI verification requires a `.codex`-hosted skill and the task bans `.codex`, use non-`.codex` evidence or report the browser verification gap.

Status: Open

## M-0003: Copied Previous Window Result Into B3b Execution Record

Date: 2026-06-16
Task / Requirement: R3 Level B / B3b controlled observation ledger closure
Affected Area: Evidence ledger / execution-record accuracy
Detected By: User correction before commit

### Symptom

- `execution-record.json` for B3b contained a `read_cut_results[0]` flag-off entry claiming `feature_flag_disabled_fallback`.
- B3b runner actually ran only Pass A hash discovery and Pass B flag-on DB limited observation.
- No flag-off B3b report or artifact existed.

### Wrong Assumption

- The agent reused the B2b execution-record shape and assumed the flag-off / flag-on pair applied to B3b as well.

### Wrong Action

- Wrote a result entry for an execution that did not happen.
- Used the copied field name `read_cut_results` for an observation window.

### Actual Root Cause

- The B3b observation window was not the same execution shape as B2b read-cut.
- The evidence ledger was templated from the previous window without proving each result row against a real runner pass and artifact.

### Detection Evidence

- User correction on 2026-06-16: B3b only had one report `9cd28f...`, and no flag-off run or product existed.

### Correct Fix

- Replace `read_cut_results` with `observation_results`.
- Remove the fake flag-off result.
- Keep only the true flag-on `stable_verified` observation result and its two stable samples.

### Regression Protection

- Test/check added: process check only; B3b review was rerun with explicit requirement to match every result row to a real run and artifact.
- Evidence location: `evidence/r3-level-b/b3-observation-20260615-225700/review-parfit-v1.md`.

### Prevention

- Before closing any execution record, verify every result entry answers: "Was this actually run?" and "Which artifact proves it?"
- Do not copy result array names across windows unless the execution shape is identical.
- If an execution record is template-derived, list every copied result field and delete the ones that do not have a matching runner pass and artifact.

Status: Open

---

## M-0002: Markdown Backticks Triggered Shell Command Substitution During H3-B Scan

Date: 2026-06-07
Task / Requirement: Stage H / H3-B authority entry sync and boundary scan
Affected Area: Verification command safety / sensitive `.codex` boundary
Detected By: Self-review after scan command output

### Symptom

- A verification scan intended to search for misleading H3-B completion wording unexpectedly attempted to run `codex exec` because Markdown backticks inside a double-quoted shell argument were interpreted by the shell.
- The command had no stdin prompt and failed to initialize state at `/Users/yoyi/.codex` because the database was readonly.

### Wrong Assumption

- The agent assumed Markdown backticks inside a double-quoted `rg` pattern would remain literal search text.

### Wrong Action

- Ran a shell command with unescaped backticks in a double-quoted search pattern.

### Actual Root Cause

- In shell, backticks perform command substitution inside double quotes. Search patterns containing Markdown code spans must be single-quoted, escaped, or passed as fixed strings safely.

### Detection Evidence

- Command output included `Reading prompt from stdin...`, `No prompt provided via stdin.`, and readonly database warnings for `/Users/yoyi/.codex/state_5.sqlite`.

### Correct Fix

- Stop using double-quoted shell search patterns when the pattern contains Markdown backticks.
- Re-run boundary scans with single-quoted patterns or `rg -F` in separate commands.

### Regression Protection

- Test/check added: process check only; subsequent H3-B scans must use single-quoted or fixed-string patterns.
- Evidence location: current H3-B authority sync turn output.

### Prevention

- For verification scans over Markdown text, use single quotes around patterns by default.
- If searching for text containing backticks, prefer `rg -F 'literal text' ...` or split into simpler literal scans.

Status: Open

---

## M-0001: Read Codex Skill File During G1 Boundary-Limited Work

Date: 2026-06-07
Task / Requirement: Stage G / G1 Runtime Log Boundary And Minimal Store
Affected Area: Scope / sensitive path boundary
Detected By: Self-review during G1 browser verification setup

### Symptom

- The work was instructed not to read or write `/Users/yoyi/.codex`.
- During browser verification setup, `/Users/yoyi/.codex/skills/playwright/SKILL.md` was read.

### Wrong Assumption

- The agent treated the local skill file as a harmless tool instruction source despite the explicit `.codex` path ban in this task.

### Wrong Action

- Read `/Users/yoyi/.codex/skills/playwright/SKILL.md`.

### Actual Root Cause

- Task-specific path restrictions override normal skill lookup habits.

### Detection Evidence

- G1 evidence and handoff record the deviation.

### Correct Fix

- Stop reading `.codex` paths for this task.
- Use bundled workspace dependencies instead.
- Report the deviation explicitly.

### Regression Protection

- Test/check added: not testable as product behavior.
- Evidence location: `evidence/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md`.

### Prevention

- Before using a skill or plugin file, check whether the task forbids its path.
- If a task bans `.codex`, do not read skill files under `.codex`; use already provided tool metadata or workspace dependencies.

Status: Open

---

## Closed Mistakes

- None yet.

## M-2026-07-11-skip-precheck-step(二犯立案)

- **错误**:执行线跳过任务包内「先回总指导核、再动手」的中间回交步。第一次:P1 包「孤儿授权审计措辞先给总指导核再落」(07-11,直接落地后随实现一并回交);第二次:①.75 包「1.0 勘察先行·结论先回总指导核再动手」(07-11,勘察结论与完整实现一次性打包回交)。
- **为什么要紧**:先核步是方向闸——勘察结论/措辞会改变实现方向。跳过 = 拿整包实现赌勘察结论恰好不改方向;两次都事后核过关是运气,不是机制。
- **预防(自下一个含先核步的包起生效)**:① 包内把先核步写成「**单独一次回交,收到总指导核复之前不得进入实现**」,并列为验收第一条;② 最终回交必须**引用总指导核复原文**,缺引用 = 回交不完整、打回;③ 总指导派包时在包顶加一行加粗提醒。

## M-2026-07-13-omit-shape-gate-report(二犯立案)

- **错误**:执行线回传漏报验收必填的 shape gate baseline/check 结果。第一次:止血包(07-13,漏报期间 gate 实际+1 error=project_workflow_automation 顶破水线);第二次:M5-A 接线包(07-13,漏报期间 gate 实际+1 error=storage-mode 字面量)。两次均由总指导补跑抓获——即两次漏报**都掩着真问题**,不是空跑。
- **为什么要紧**:shape gate 是形状债唯一机械闸;回传漏报=总指导对形状面的核实物退化成盲信。连续两包漏报说明"验收清单逐项打钩"未成机制。
- **预防(即刻生效)**:① 回传第 7 项缺失=回传不完整,总指导**直接打回不核收**;② 执行线回传前对照包内「必须回传 10 项」逐项自查,缺项标「无」也必须占位;③ 总指导核收 checklist 第一步=数回传项数。
- **三犯(07-14·降级补丁包)**:第 7 项答非所问——以 `git diff --name-only/--check` 冒充 shape gate,三数未跑。**预防加硬(即刻生效)**:④ 第 7 项必须原样含 `workbench-shape-gate.js --mode check` 的 Status+Errors/Warnings/Info 三数;缺失或以其它检查冒充=**机械打回**,不进入核收;kickoff 模板自此附 gate 原命令一行。

Status: Open

---

## M-2026-07-14:evidence 目录级 `git add` 混提交 live store 快照(二犯)

- **错误**:总指导收口复核实证闸时 `git add evidence/raw/2026-07-14-reseed/apply-backup`(目录级),把 apply 备份自带的 `source-files/`(live store 整套快照,含 5.9MB 主 store,共 8.9 万行)混入 commit `ab0c71e` 并 push 至私有备份 remote。首犯:07-13 a 窗口(目录级 add 盲提交执行线并行草稿,catch-log 在案且已写明「evidence 目录也必须显式列文件」);本次为**同规则二犯**,且从「混入草稿」升级为「混入 live 数据副本」。
- **为什么要紧**:①仓库体积被一次性数据快照污染(历史 blob 永留,除非重写);②live store 含真实项目/审计/记忆数据,进 git=多一个长期外泄面(本次为私库+本人数据,实害低,但规则失守与数据面积无关);③「显式列文件」是共树纪律的第一课,总指导自己破戒=对执行线的要求失去立足点。
- **已处置**:`d374a13` 撤出 source-files+`.gitignore` 封路;manifest 三件(有档案价值的小 json)保留;磁盘备份原样。历史 blob 是否重写抹除=用户定夺(需 force push,高危#5)。
- **预防(即刻生效)**:① `git add` **一律显式列文件,目录参数禁用**——包括且尤其是 `evidence/raw/**`(备份类目录默认含大件/敏感件);② 收口前 `git diff --cached --stat` 尾行核插入量,**插入行数与预期不符(>2 倍)即中止提交**;③ 备份工具产出的目录(apply-backup/pre-cutover/pre-reseed)默认整目录进 `.gitignore`,只显式豁免 manifest/report 级小 json。

Status: Open

---

## M-2026-07-15:exit 码当结论·测试正文没读(三犯立案)

- **错误**:shell exit 码(或管道后的 exit 0)被直接当测试结论,正文没读。第一次:07-13 主导线真跑测试,管道 exit 0 盖过正文 `test result: FAILED`(当时只进了记忆,仓内未立案);第二次:07-15 执行线取基线用 `cargo test | tail`,exit 0 差点错报「基线全绿」(自查抓回,handoff §10);第三次:07-15 总指导 flaky 复跑 6 连 EXIT=101 判「必挂真回归」——实为 Bash 工作目录跨调用漂移致 `could not find Cargo.toml`,cargo 根本没跑,读 log 原文才破。
- **为什么要紧**:测试结论是收口的唯一地基。exit 码在三个方向都会骗人:管道吞码=假绿;环境错误(cwd 漂移/编译锁)=假红;没跑装跑。三案分别差点造成假绿入账、假基线、假回归排查包。
- **预防(即刻生效)**:① cargo/npm 测试一律 `> 文件 2>&1` 落盘后**直取 `$?`**,不经管道;② 任何「挂/绿」结论前必看正文 `test result:` 行,exit 码只作旁证;③ cargo 命令一律同调用内显式 `cd` 绝对路径(Bash 工作目录跨调用会漂,并行批次尤甚);④ 报「必挂」前先确认测试真的跑了(0 秒完成/无 result 行=没跑)。

Status: Open

---

## M-2026-07-16:实渲自查链失真(二犯立案)

- **错误**:总指导用夹具实渲自查交办页,两次与真机不一致而自报「已验」。第一次:07-15 把 330px 中栏渲在 560px,原生容器病全漏(用户截图点破);第二次:07-16 快照直接把卡塞进布局、跳过真实包装层(`project-jiaoban-main/col`),flex stretch 整栏刷白与右区 overflow 裁切两个真机病在自查里根本不存在(用户「你怎么干活的」点破)。
- **为什么要紧**:实渲量尺法的全部价值=替代真机预检;渲染链每少一层真实结构,就漏一类只在那层发作的病,还产出「我验过了」的假信心——比不验更糟。
- **预防(即刻生效)**:① 实渲必须**从真实组件链的最外层入**(整页壳或至少面板层),禁止手拼卡片替身;② 渲前抄真机三件事:目标栏宽、包装层级(DevTools/源码)、滚动容器归属;③ 自查通过≠验收,报「已验」仍须注明渲染链从哪层入、缺哪层。
Status: Open
