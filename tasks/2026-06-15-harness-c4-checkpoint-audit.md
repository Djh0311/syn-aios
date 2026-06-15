# Harness C4：checkpoint-audit 工具（拿 git 实物核完成报告）

## 任务名

新增 `scripts/harness/checkpoint-audit.js`：喂一个包（slug/commit），把完成报告声称的 commit/复核/边界/CURRENT.md 逐条对 git 实物，对不上标红；假报告必须 fail。

## 所属开发线

开发治理 harness 改进线（Claude 开发线，worktree `c4-checkpoint-audit`，基于 main@f50848b）。

## 背景

咨询线每包手动做"不信报告、拿 git 实物核"那套，本包脚本化成可复用门。**不是再造 verification**——现成 verification-suite/runner 是"跑检查"，本工具是"核一份完成报告说的对不对"，现成没有。时机闸：C1 已收口（`1356378`），A 已合并，C4 不依赖 R3/B。

一句话判据：喂它一个包(slug/commit)，它把"报告声称的 commit/复核/边界/CURRENT.md"逐条对 git 实物，对不上标红；喂它一个假报告（声称的 commit 不存在）必须 fail。

## 目标

`checkpoint-audit.js --package <slug> / --commit <sha>`，只核**机械事实**：
- ① 声称的 task/impl/checkpoint commit 在 HEAD 可达；
- ② git status 干净（或符合声明）；
- ③ 复核 evidence 文件存在且含可解析 STATUS（CLEAR / CLEAR_WITH_P2 / FINDINGS）；
- ④ CURRENT.md 顶部 checkpoint 引用了该 impl commit + 复核结论；
- ⑤ impl commit 改动文件落在声明 allow-list 内，越界标红；
- ⑥ 测试/构建/shape-gate 部分**直接调现成 verification-suite / shape-gate，不重写**。

诚实边界（写进工具输出 + catalog 行）：只验机械事实；**验不了判断**（diff 是否真无行为变化、有没有踩坑）——仍是人的活。

## 允许读取

`scripts/harness/**`、`AGENTS.md`、`docs/harness-catalog.md`、`CURRENT.md`、`tasks/**`、`evidence/**`（只读核对）。

## 允许写入

`scripts/harness/checkpoint-audit.js`（新增）、`scripts/harness/checkpoint-audit.selftest.js`（新增）、`AGENTS.md`（加一处调用点）、`docs/harness-catalog.md`（加 2 行 + 计数）、本任务包。

## 禁止事项

重写 verification 逻辑（只 spawn 现成 shape-gate / verification-suite）；改产品代码（`prototypes/`、`src-tauri/`）；改 shape-gate 现有检查；开 hooks/CI；删脚本；碰 `~/.codex`；真实执行 Codex；自合并回 main。

## 形状影响

- 任务类型：治理任务包（新增 harness 脚本 + 文档接线）。
- 新增代码落点：`scripts/harness/checkpoint-audit.js`（~300 行）+ `checkpoint-audit.selftest.js`（~160 行）。`AGENTS.md` +1 调用点；`docs/harness-catalog.md` +2 行/计数。
- 是否触碰棘轮文件：否（不碰 `workbench-shape-gate.js`、不碰产品文件）。
- 预计行数变化：见上。
- 是否新增 Tauri command / sidecar：否 / 否。
- 是否需要 shape gate 豁免：否。
- 本任务基线 commit：f50848b。
- 本任务完成 commit：见执行结果。

## 验收标准

- selftest 全过：喂已收口包→pass；假报告（commit 不存在 / 树脏 / 越界 / 复核缺 STATUS）→fail。
- 对真实已收口包实跑：产品包（R-U5，从 CURRENT.md 解析）给出 pass；HG 包（`--commit` + `--allow`）给出 pass。
- 构造假报告实跑→fail。
- catalog 计数与 `ls scripts/harness` 一致（69 顶层 + 13 lib = 82）；新行状态 `已接`/`配套自测`。
- shape-gate 仍 pass；`git diff --check`。

## 执行与验证结果

做了什么：新增 `scripts/harness/checkpoint-audit.js`（6 项机械核对 + 诚实边界横幅；`--package`/`--commit`/`--allow`/`--allow-dirty`/`--review`/`--skip-gates`/`--json`）+ `checkpoint-audit.selftest.js`（临时 git 仓自测）；`AGENTS.md` 加一处 C4 调用点；`docs/harness-catalog.md` 加 2 行（`已接·C4` + `配套自测`）+ 计数 80→82。

机械核对 6 项：① 声称的 task/impl commit 在 HEAD 可达；② 工作树干净（或 `--allow-dirty`）；③ 复核 evidence 含可解析 `STATUS:`（CLEAR/CLEAR_WITH_P2/FINDINGS）；④ CURRENT.md 顶部引用该 impl commit + 复核；⑤ impl commit 改动文件落在 allow-list 内；⑥ **直接 spawn 现成 `workbench-shape-gate.js`**（不重写 verification）。诚实边界写进工具输出 + catalog 行：只验机械事实，**diff 是否真无行为变化 / 有无踩坑仍是人核**。

自验（worktree `c4-checkpoint-audit`）：
- selftest **16/16 通过**：好包→verdict PASS（①②③④⑤ 全绿）；伪造 commit 不存在→`commits_reachable` FAIL（MISSING）+verdict FAIL；树脏→`tree_clean` FAIL（`--allow-dirty` 降级）；越界文件→`files_within_allow` FAIL 且点名 `src/secret.rs`；复核缺 STATUS→`review_status` FAIL。
- catalog 计数 = `ls scripts/harness`（69 顶层 + 13 lib = 82），全覆盖、无 stale。
- shape-gate 仍 `pass`（0/0）；`git diff --check` 干净。
- 真实包实跑（产品 R-U5 / 伪造 / 硬核 HG-3）：见 C4 checkpoint（`docs/context-checkpoints.md`）原始输出——须在 commit 后干净树上跑（check ② 验当前工作树）。

有依据：工具能用、selftest 全过、接线到位、catalog 自洽、对真实已收口包给出正确 verdict、对伪造报告 fail。
仍不确定/边界：allow-list 自动解析是 best-effort（散文/「本任务包」无法可靠解析时 ⑤ 记 NA，要可靠核边界请传 `--allow`）；本工具**只验机械事实**，不验判断；check ② 验"当前工作树"，审计历史包须在干净树上跑。
未动：产品代码、`workbench-shape-gate.js` 现有检查、hooks/CI、agentmemory 状态（main 上已是 f69922e/C3 改的，非本包）。
完成 commit：见本包提交（git log c4-checkpoint-audit）。
