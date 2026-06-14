# 复核结论：Harness HG-1 / HG-3 / HG-2 批次（branch `harness-hg`）

日期：2026-06-14

Reviewer：Claude（claude-opus-4-8，复核线，独立只读复核）

复核对象（分支 `harness-hg`，4 commits ahead of `main`@70f5557，未合并）：

- HG-1 任务包：`tasks/2026-06-14-harness-hg-1-script-catalog.md`（commit `dd0e372`）
- HG-3 任务包：`tasks/2026-06-14-harness-hg-3-ugate-dedup-shape-gate.md`（commit `b72c36f`）
- HG-2 任务包：`tasks/2026-06-14-harness-hg-2-wire-five-groups.md`（commit `8fe77ff`）
- 批次 checkpoint commit `608560b`

复核基线：worktree `/Users/yoyi/workspace/product-line-harness-hg`（HEAD `608560b`），对照 `main`@`70f5557`。所有门 / 脚本 / diff 均在 worktree 实物上独立重跑，非转述执行线报告。

性质：只读独立复核。本文件是复核线唯一产出；复核线不改代码 / 不改 catalog / 不 commit / 不合并；发现问题只列不修。

---

## STATUS: FINDINGS

- P0：无
- P1：无
- P2：1 项（HG-1 catalog 漏登 HG-3 新增的 selftest 脚本）

验收硬门"结论不得有 P0/P1"**满足**——三包功能性正确、边界守住、纯加性、未合并、未碰产品代码。唯一 P2 是跨包完整性缺口（文档级，非运行级），建议合并前修补，但不阻断合并。修与不修、何时合并均属用户 / 执行线决策，复核线不替决。

---

## 1. 独立验证方法

在 worktree 上实物重跑：shape-gate（`--mode check` / `--strict` / `--json`）、HG-3 self-test、HG-2 全部 14 个脚本；`ls` 与 catalog 交叉比对；`git diff main..harness-hg` 逐文件 numstat；main 基线 gate `--json` 取证对账。复核全程 worktree 保持 clean（我的活动零落盘）。

## 2. 跨包 / 边界（全部核实通过）

- **未合并**：`merge-base --is-ancestor harness-hg main` = NOT_MERGED。
- **未碰产品代码**：`diff --name-only main..harness-hg | rg 'prototypes/|src-tauri/'` 零命中；10 个改动文件无一在产品树。
- **纯加性 diff**：`shortstat` = 10 files, 897 insertions, **1 deletion**；唯一 `-1` 在 `docs/context-checkpoints.md`（占位行 "No checkpoints yet." 换成真 checkpoint，numstat 36/1）。`AGENTS.md`（11/0）与 `workbench-shape-gate.js`（72/0）**零删除**——现有规则与现有 gate check 一字未改（下见 §4 取证）。
- **冻结边界**：HG-2 的 14 个脚本实跑 git status 前后均 clean、无 `~/.codex`、无真实执行；hooks/CI 仍关（`已接` = AGENTS.md 手动调用点，非自动执行）；capability-map 文件保留（仅标退役）。

## 3. HG-1 脚本索引

- **完整性（除 1 项外通过）**：`ls`（顶层 + lib，去重）vs catalog `.js` token 交叉比对——
  - "stale" 疑点 `harness.config.js` / `package.js` 经核实为**正则假阳性**（`harness.config.json` / `package.json` 的子串命中 prose），**非 stale 行**；catalog 无指向不存在脚本的行。
  - `context-pack.js` 顶层与 `lib/context-pack.js` 同名，**两行均已登记**（非重复、非遗漏）。
  - 状态统计自洽：承重 2（stage-k-architecture-gate、workbench-shape-gate）/ 退役 1（capability-map）/ 休眠·待定 9（agentmemory 簇，与特别标注列举一致）/ 已接 14 / 其余 休眠 11 + 未接 42，合计 79，与 catalog 79 行一致。
- **指针**：`AGENTS.md` 新增一行 call-point 指针指向 `docs/harness-catalog.md`（加性，0 删除）。
- **见 §6 P2**：catalog 声明范围"66 顶层 + 13 lib = 79"，但分支现有 **67 顶层**（HG-3 加入 selftest 后）。

## 4. HG-3 U-Gate 去重 check（shape-gate）

- **纯加性**：`workbench-shape-gate.js` numstat 72/0（0 删除）。gate 485 行（< 500 上限）。
- **现有行为不变（独立取证）**：worktree gate `--json` vs main `--json` findings 对比——9 vs 9 条，severity 分布 0/0/9 一致，`boundary` / `baselines` 完全相同；逐条比对**唯一差异**是 `baseline_commits.current_head`（`70f5557` vs `608560b`），系不同 commit 的内禀哈希，**非行为变化**。新增 metric `helper_duplicates` 为加性。
  - 注：执行线报告称 findings "byte-identical: true"，严格说不准确（current_head 必然不同）；但实质 check 行为确未变，此差异无害。
- **当前代码 0 误报**：`--mode check` 与 `--strict` 均 `Status: pass`，"Converged-helper dups outside utils/: **0**（12 deferred-whitelisted）"。12 deferred 即 R-U 有意保留的 per-store `sidecar_path` 薄封装。
- **warning-only 不破门**：self-test **8/8 PASS**——utils 外重复 `fn`→恰好 1 warn 且 severity=warn（绝不 error）、默认 Status 仍 pass、白名单→0 且记 deferred、utils 内→豁免、干净树→0。self-test 用隔离临时夹具，运行后 worktree 仍 clean（不碰真实产品代码）。

## 5. HG-2 接线 5 组 + capability-map 退役

- **14 脚本独立重跑全过**：guard-state-files / status-snapshot / stale-control-check（①）、mistake-check / mistake-query / mistake-new（②）、evidence-check / -freshness / -query / -index / -new（③）、config-check / config-policy（④）、capability-scan（⑩）—— `--target .` 全部 **exit 0**，运行前后 `git status` 均 clean（**零落盘**）；写类脚本确认 dry-run（`mistake-new`：no files modified；`evidence-new`：dry-run；`evidence-index`："wrote": []）。
- **AGENTS.md 调用点**：新增 5 组手动调用点子节（加性），明确"hooks/CI 仍关闭、不自动执行""每组实跑通过（默认只读 / dry-run）"——表述诚实。
- **catalog 翻状态**：`已接·HG-2` 14 条、`未接·HG-2` 残留 **0**。
- **capability-map 退役**：catalog + AGENTS.md 标退役，文件 `scripts/harness/capability-map.js` **仍在**（未删）。

## 6. 唯一 P2：HG-1 catalog 漏登 HG-3 新增 selftest

- **什么**：`scripts/harness/workbench-shape-gate.dedup.selftest.js`（HG-3 于 `b72c36f` 新增的顶层脚本）**未出现在 catalog 任何行**（`comm -23` 与 `rg dedup.selftest` 双重确认 SELFTEST_NOT_IN_CATALOG）。
- **在哪**：`docs/harness-catalog.md`——范围行（line 6 "66 顶层…= 79 条"）、顶层表头（line 27 "顶层脚本（66）"）、统计行（line 23 合计 79）三处均按 79 行计，缺第 80 个脚本的行。
- **为什么是问题**：① 违反 catalog 自述不变量（line 3 "动用或改造任何 `scripts/harness/` 脚本前先查这里"）与 HG-1 验收口径"每个脚本都有一行、无遗漏"；分支实际为 67 顶层 + 13 lib = 80，catalog 仅 79 行。② 这是跨包完整性缺口：HG-3 加脚本、HG-2 又动过 catalog，两步均未补该行；逐包自验各自为真，批次终态才暴露（正是批次复核的价值）。③ 执行线报告"catalog 79 行 = 完整"在批次终态已不准确。
- 影响级别：仅文档索引，零运行 / 产品 / gate 影响；属 P2。是否补行、或显式把测试类脚本排除出索引并改计数，属实现线裁量，复核线只列不修。

## 7. 留给用户的判断项（非缺陷，仅供合并决策）

- **2 份审计 md 进分支**：`docs/harness-script-audit-2026-06-14.md` + `harness-source-package-audit-2026-06-14.md`（共 352 行，catalog 数据源）随分支带入；合并即进 product-line。执行线已自陈"若不该进可单独剔除"——属内容决策，用户定。
- **commit 授权**：分支 4 个 commit 由执行线称"用户 spec 明确授权 per-package commit"。该 spec 复核线无从独立查证；按 AGENTS.md commit 需用户放行，请确认与你本意一致。
- **`已接` 语义**：= AGENTS.md 手动调用点引用，hooks/CI 关闭故不自动强制执行（执行线已披露）；是否够用取决于人工遵循，属本轮"不开 hooks"边界内取舍。
- **catalog "怎么调" 列**：未接 / 休眠脚本的调用式部分取自路由 / config 推断、未逐脚本 `--help` 核（执行线已披露）；HG-2 实跑的 14 个已用真实调用核过。
- **agentmemory 9 件**仍 `休眠·待定`，永久退役与否未决（执行线明确不替用户决定）。

## 8. 复核边界声明

- 本文件为复核线唯一产出，写在 worktree（branch `harness-hg`）`evidence/` 下、**未提交**；按既定模式应由实现线 / 用户随合并前 commit 落到分支（复核线不 commit、不合并、不改 catalog / gate / 产品代码）。
- §2-§5 全部为复核线在 worktree 实物上独立重跑，非转述执行线报告。
- 合并 `harness-hg` → `product-line` 是用户决策，复核线不执行合并。
- 结论仅针对本批次三包的实物正确性与边界；不接受为 hooks/CI 已启用、脚本永久退役已决、审计 md 应否进分支已决、或后续 harness 治理方向已定。
