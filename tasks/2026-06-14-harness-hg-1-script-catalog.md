# Harness HG-1：治理脚本索引（catalog）

## 任务名

给 `scripts/harness/` 的 79 个脚本建一份一行可读的索引，并在 AGENTS.md 加一行指针指过去。

## 所属开发线

开发治理 harness 改进线（Claude 开发线，worktree `harness-hg`）。

## 背景

只读审计 `docs/harness-script-audit-2026-06-14.md` 发现：79 个脚本里只有 2 个真在用，其余"装好从没接上电"，连维护者都不知道有什么——最近险些重造已存在的 verification 工具。根因之一是**没有任何 agent 会读到的脚本索引**：唯一发现入口是 `harness.config.json` 和 `harness.js --help`，而流程文档不指向脚本库。本包补这个缺口。数据源 = 上述审计报告附录命中表。

## 目标

1. 新增 `docs/harness-catalog.md`：79 行，每行 = 脚本名 / 干啥 / 状态(桶+接没接+该不该用) / 怎么调。
2. 在 `AGENTS.md` Product-Line Override 的"Project-specific rules"加**一行指针**指向索引。
3. 特别标注：agentmemory 9 件标"休眠·待定（待用户定是否永久退役，本包不决定）"；`capability-map` 标"退役候选（HG-2 ⑩ 落实）"。

## 允许读取

`docs/harness-script-audit-2026-06-14.md`、`scripts/harness/**`、`AGENTS.md`、`harness.config.json`、`scripts/harness/harness.js`。

## 允许写入

`docs/harness-catalog.md`（新增）、`AGENTS.md`（仅加一行指针）、本任务包、`tasks/2026-06-14-harness-hg-1-*`。附带把数据源 `docs/harness-script-audit-2026-06-14.md` 与姊妹篇 `docs/harness-source-package-audit-2026-06-14.md`（原在 main 工作区 untracked）带进本分支，使分支自洽、索引引用可验证。

## 禁止事项

改/删任何脚本；改产品代码（src-tauri/prototypes）；开 hooks/CI（`hooks.enabled`/`ci.required` 保持 false）；重写 AGENTS.md 既有规则（只许加一行指针）；顺手接线（接线是 HG-2）。

## 形状影响

- 任务类型：治理任务包（纯文档）。
- 新增代码落点：无代码。新增文档 `docs/harness-catalog.md`；`AGENTS.md` +1 行。
- 是否触碰棘轮文件：否。不碰 `workbench-shape-gate.js`、不碰产品 ratchet/waterline 文件。
- 预计行数变化：新增索引 ~110 行；AGENTS.md +1 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务基线 commit：70f5557。
- 本任务完成 commit：见执行结果。

## 验收标准

- 索引条目数 = `ls scripts/harness/*.js`(66) + `ls scripts/harness/lib/*.js`(13) = 79，一一对应。
- AGENTS.md 指针行确已加入且指向 `docs/harness-catalog.md`。
- agentmemory 9 件、capability-map 的特别标注到位。
- `git diff --check` 干净。
- 不触碰 `workbench-shape-gate.js`，shape gate 行为零变化（本包不跑 gate 改动验证，因未碰 gate/产品）。

## 执行与验证结果

做了什么：新增 `docs/harness-catalog.md`（79 行索引，状态=桶+接没接+该不该用）；`AGENTS.md` Project-specific rules 加一行指针（行 33）；把数据源审计与姊妹篇带进分支。

自验原始结论（在 worktree `harness-hg` 跑）：

- 条目数：`ls` top=66 lib=13 = 79；catalog top=66 lib=13 = 79，一一对应。
- 完整性：全部 79 个脚本都有行（0 missing）；无 stale 行（每行都对应真实脚本）。
- 特别标注：agentmemory 脚本行 = 9（精确匹配）；capability-map 行 = `退役候选（…HG-2 ⑩ 落实）`。
- 指针：`AGENTS.md:33` 指向 `docs/harness-catalog.md`。
- `git diff --check` 干净。

有依据的结论：索引覆盖全 79 脚本、与磁盘一一对应、指针已接入 agent 必读的 AGENTS.md。
仍不确定/边界：`怎么调`列的部分 flag 取自 harness.js 路由/config 推断，未逐脚本 `--help` 实跑核对（HG-2 会实跑其中 5 组，届时可回校）；agentmemory 是否永久退役未决（待用户）。
未动：任何脚本、产品代码、`workbench-shape-gate.js`、hooks/CI 开关。
完成 commit：见本包提交（git log harness-hg）。
