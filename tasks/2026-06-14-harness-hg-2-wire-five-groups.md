# Harness HG-2：接线 5 组工具 + capability-map 退役

## 任务名

把 5 组高价值工具接进流程（每组：实跑通过 + AGENTS.md 加一处调用点），并把 capability-map 标退役。

## 所属开发线

开发治理 harness 改进线（Claude 开发线，worktree `harness-hg`）。基于 HG-3 commit b72c36f。

## 背景

审计发现这些脚本一个都没跑过、可能装好从没跑通。本包前提：**每组先 node 实跑一遍**，跑通才接；跑不通记 deferred(broken)，不准假装接上。一句话判据：每组"实跑通过 + AGENTS.md 有一处调用点"两者都满足才算接上，缺一即 deferred。

## 目标

接 5 组（每组实跑取原始输出 → AGENTS.md 加一处调用点）：① 受保护路径守卫 guard-state-files/status-snapshot/stale-control-check；② 错误账本 mistake-check/-new/-query；③ 证据新鲜度 evidence-check/-freshness/-new/-index/-query；④ 配置校验 config-check/config-policy；⑩ 能力普查 capability-scan（并把 capability-map 在 catalog 标退役·被取代）。

## 允许读取

`scripts/harness/**`、`AGENTS.md`、`docs/harness-catalog.md`、`harness.config.json`、项目状态文件（脚本运行时只读扫描）。

## 允许写入

`AGENTS.md`（加 HG-2 调用点子节）、`docs/harness-catalog.md`（14 脚本翻已接、capability-map 翻退役、统计同步）、本任务包；窄修（≤几行，如脚本跑不通）。

## 禁止事项

开 hooks（`hooks.enabled` 保持 false）；开 CI；删任何脚本（capability-map 只标退役、文件留着）；改产品代码；碰 `~/.codex`；真实执行/解冻 backlog；假装接上跑不通的脚本。

## 形状影响

- 任务类型：治理任务包（接线 + 文档）。
- 新增代码落点：无新代码（未窄修任何脚本——14 个全部一次跑通）。改 `AGENTS.md`（+1 子节 5 条调用点）、`docs/harness-catalog.md`（状态列）。
- 是否触碰棘轮文件：否（不碰 `workbench-shape-gate.js`、不碰产品文件）。
- 预计行数变化：AGENTS.md +~12 行；catalog 状态列若干处。
- 是否新增 Tauri command / sidecar：否 / 否。
- 是否需要 shape gate 豁免：否。
- 本任务基线 commit：b72c36f。
- 本任务完成 commit：见执行结果。

## 验收标准

- 5 组各贴实际跑通的原始输出；跑不通的明确列 deferred。
- AGENTS.md diff 每组都有一处调用点（5 处）。
- catalog：已接 14、退役 1、未接 42，统计与行状态一致；`未接·HG-2` 余 0。
- shape-gate 仍 pass（本包不碰 gate/产品）。
- `git diff --check` 干净。

## 执行与验证结果

**实跑结果：14/14 全部跑通（exit 0，默认只读 / dry-run，git status 全程干净，无误写），0 deferred、0 broken、未做任何窄修。**

第一步实跑原始结论（`node scripts/harness/<x> --target .`，worktree `harness-hg`）：

| 组 | 脚本 | exit | 观察 |
| --- | --- | --- | --- |
| ① | guard-state-files.js | 0 | 输出受保护文件/envFiles JSON |
| ① | status-snapshot.js | 0 | 输出状态快照 + 推荐命令可用性 |
| ① | stale-control-check.js | 0 | 控制文件 tbd/warnings/failures（0） |
| ② | mistake-check.js | 0 | 输出 ledger 相关结构 |
| ② | mistake-query.js | 0 | FAIL/RELATED 查询（0 命中） |
| ② | mistake-new.js | 0 | `Mode: dry-run`，`Next ID: M-003`，未写文件 |
| ③ | evidence-check.js | 0 | 输出 hardEvidenceGates |
| ③ | evidence-freshness.js | 0 | 输出每项 timestamps/ageHours |
| ③ | evidence-query.js | 0 | matches=[] |
| ③ | evidence-index.js | 0 | `"wrote": []`（默认 dry，未写索引） |
| ③ | evidence-new.js | 0 | `Mode: dry-run`，content preview，未写 |
| ④ | config-check.js | 0 | 输出 config 形状（memory enabled=false 等） |
| ④ | config-policy.js | 0 | 输出 policy（hooks/prePush 等） |
| ⑩ | capability-scan.js | 0 | 输出工具能力（make=true、docker=false 等） |

接线（每组 1 处调用点，AGENTS.md「Harness 脚本调用点（HG-2 接线）」子节，行 52 起）：① 完成前/碰受保护文件前；② learning-from-mistakes 流程；③ 收口前查证据过期；④ 改 harness.config.json 后；⑩ pre-work。均为**手动**调用点（hooks/CI 仍关）。

catalog 同步：14 个目标脚本 `未接·HG-2X`→`已接·HG-2X`；capability-map `退役候选`→`退役·被 capability-scan 取代`（文件保留）；统计行 已接 14 / 未接 42 / 退役 1（合计 79）。

自验：catalog 79 行不变、状态计数自洽、`未接·HG-2` 余 0；AGENTS.md 5 组调用点各 1 处；shape-gate 仍 `pass`（0/0，去重 0/12 deferred）；`git diff --check` 干净。

有依据的结论：5 组对应脚本全部实跑可用、各有一处 AGENTS.md 调用点 → 5 组均"接上"（无 deferred）；capability-map 已退役（文件留着）。
仍不确定/边界：脚本只跑了**默认只读/dry-run 路径**；`--write` 落库路径（mistake-new/evidence-new 真写、evidence-index 真建索引）未实测（避免产生条目/污染）。调用点为手动引用，未开 hooks，故"是否真被每次执行"取决于人工遵循——这是本轮边界（不开 hooks/CI）内的有意取舍。
未动：任何脚本源码（0 窄修）、产品代码、`workbench-shape-gate.js`、hooks/CI 开关。
完成 commit：见本包提交（git log harness-hg）。
