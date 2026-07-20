# 人话工程①②：错误翻译族前后端收编 + shape gate machine_face_on_ui 规则 验收证据 v1

日期：2026-07-20 · 轻档 · 执行线施工，总指导回收核实物
任务包：`tasks/2026-07-20-human-language-engineering-modules-and-machine-face-gate-rule-package-v1.md`
基线 commit：`24bde2e`（开工核对：HEAD=`24bde2eaf99102623879644ad7211744355c73a7`，工作树干净）
规则留痕：`decisions/2026-07-20-machine-face-gate-rule-and-defer-whitelist-v1.md`

## 一、结论

三件事全做、四闸全绿、零净增：

1. **前端收编**：新建 `src/lib/humanize.ts`（272 行），§2.2 表列函数**逐字原样迁入**（人话输出串/判据/分支/兜底零变化），6 个迁出文件 import-back / re-export 保导入面；`App.tsx` `messageOf` 接 `humanizeNoticeMessage` 薄委托（命中族出人话、未命中原文逐字回退）。
2. **后端收尾**：`secretary_agent.rs`、`global_supervisor_agent.rs` 两个 `humanize_consult_error` 薄委托壳删除，3 个调用点直调 `run_error_translation::humanize_error_for_display`；真源本体与 `codex_local_runner.rs:402` 承重壳零碰。
3. **shape gate 新规则** `machine_face_on_ui`：error 级拦 `{error.message}` 直渲与 `<pre>stderr:` 形（新增零容忍），`state_error_message` 形 warn-only；规则本体拆 `scripts/harness/lib/machine-face-rule.js`（gate 脚本 489→492 行，未破 500 软限）；既有违规 6 条进 `MACHINE_FACE_DEFER_WHITELIST`；selftest 18 断言全绿；catalog + decisions 已登记。

**口径说明**：包文「10 函数」与 §2.2 表实际 11 行不符；按表全迁 11 个函数（含判据件 `isAlreadyConfirmedRejection`）。

## 二、迁移保真证明（grep 实物）

- 11 函数全部在 `src/lib/humanize.ts`（`grep '^export function'`：humanizePreviewError / isAlreadyConfirmedRejection / humanizeProviderUnavailable / humanizeAuthorizeError / normalizeTranscriptError / friendlyLiveTitle / friendlyLiveDetail / historyErrorFamilyLabel / humanizeVerdict / humanizeWriteRoots / humanizeChainProgress + 新增 humanizeNoticeMessage + 随迁私有依赖 commandFromArguments / firstLine）。
- 随迁依赖披露：`friendlyLiveDetail` 依赖 `commandFromArguments`/`firstLine`（TranscriptViews 私有件，逐字随迁并经 humanize.ts 导出供原文件 import-back）；`humanizeChainProgress` 依赖 `countDoneNodes`（随迁为 humanize.ts 私有件）。
- `TranscriptErrorCategory` / `TranscriptErrorInfo` 类型随 `normalizeTranscriptError` 迁出并导出。
- 迁出文件零函数体残留：`humanize_consult_error` grep 清零（仅 `run_error_translation.rs` 注释历史提及·本体零改）；后端 3 调用点全部直调。
- 离线测试**零改动**（断言文本逐字未动），全部经 import-back / re-export 链路透传。

## 三、变更辐射面（git diff --stat）

11 个跟踪文件，+57/−252：

| 文件 | 变化 |
|---|---|
| ProjectJiaobanPanel.tsx | −59（棘轮水位只降不升·顺向） |
| AgentConversationShell.tsx | −77（含类型与映射表） |
| TranscriptViews.tsx | −49 |
| JiaobanHistory.tsx | −44 |
| JiaobanRunningStates.tsx | −24 |
| JiaobanAuthorizeStates.tsx | −12 |
| App.tsx | +7（import + messageOf 薄委托 + 注释） |
| secretary_agent.rs / global_supervisor_agent.rs | −10 / −12（删壳改直调） |
| workbench-shape-gate.js | +3（require / 挂载 / 打印各一行） |
| docs/harness-catalog.md | 登记（顶层 70、lib 14、合计 84） |

新增未跟踪：`src/lib/humanize.ts`（272 行 < 2000 新文件限）、`scripts/harness/lib/machine-face-rule.js`、`scripts/harness/workbench-shape-gate.machine-face.selftest.js`、decisions、本 evidence。

## 四、App notice 前后对照样例（真实代码经 esbuild 转译后 node 直跑 `humanizeNoticeMessage`）

| 输入（原始错误） | 修前显示 | 修后显示 | 变？ |
|---|---|---|---|
| `codex_provider_unavailable:codex 额度用完了，明天再试` | `读取失败：codex_provider_unavailable:codex 额度用完了，明天再试` | `读取失败：codex 额度用完了，明天再试` | 命中族·剥前缀取内嵌人话 |
| `Reconnecting... 5/5 ERROR: unexpected status 403 Forbidden: {"code":"SUBSCRIPTION_NOT_FOUND"}` | 原文整串 | `读取失败：codex 服务不可用（常见：额度用完 / 订阅过期 / 登录失效）——处理后点重试；若是网络抽风，重试一次通常就过。` | 命中族·供给人话 |
| `some totally novel error blorp` | `读取失败：some totally novel error blorp` | `读取失败：some totally novel error blorp` | **未命中·逐字不变** |

**主动披露**：包列 messageOf 三调用点（:175/:206/:281）；`messageOf` 另有第四调用点 `App.tsx:630`（`动作失败：…` 同族 notice），随本体接委托一并治平——未命中逐字回退保证该点未命中时显示串零变化。四处均不进白名单。

## 五、四闸（全亲跑）

| 闸 | 结果 |
|---|---|
| `cargo test --lib`（src-tauri） | **1024 passed / 0 failed / 44 ignored**，exit 0（包写时口径 1009；基线 24bde2e 实况 1024，只增不减 ✓） |
| `npx tsc --noEmit` | exit 0 |
| `node scripts/run-offline-interaction-test.mjs` | exit 0，全部套件输出「全过/passed」（offline interaction 15 + 各注册组 + r4×4） |
| shape gate `--mode baseline` | **13 errors / 5 warnings / 5 infos，exit 0，零净增**；machine-face 0 error-form / 0 warn（9 笔 deferred）；gate 脚本 492 行 ≤ 500 软限（自身零新增 warning） |
| shape gate `--mode check` | 13/5/5 同数，exit 1 = 历史债非零退出（既有口径，如实报告） |
| `git diff --check` | exit 0 |
| 加跑：machine-face selftest | **18/18 PASS**（直渲→error、白名单→deferred、`<details>` 样板→不误伤、state 形→warn-only、干净树→0） |
| 加跑：dedup selftest | 8/8 PASS（不回归） |

## 六、被闸拦过的事

1. **shape gate 首跑 Warnings 5→6**：新规则抓到 `WorkflowCommandConsoleView.tsx:19` 既有 state 形（`error instanceof Error ? error.message : String(error)` 本地 messageOf）——§2.4「等」覆盖的既有 warn 档观察件、勘察漏列。`git show HEAD` 实证非本轮新引入，按 §2.4 warn 档补登白名单并在 decisions/回传主动披露。**未**为过关塞任何新违规（error 级两形零登记新增）。
2. 包文「10 函数」与 §2.2 表 11 行不符：按表全迁 11 个并在此留痕。
3. `App.tsx:630` 第四调用点勘察未列：随 messageOf 本体治平并披露（见 §四）。

catch 建议：勘察漏列 1 件既有 warn 档观察件（WorkflowCommandConsoleView:19）——总指导定夺是否记 `docs/harness-catch-log.md`。

## 七、红线自查

- 函数体逐字迁移 ✓（人话输出串/判据/兜底零变化；仅加 export 与模块头注释）；离线断言零改动 ✓。
- 零碰 `run_error_translation.rs` 本体、两承重前缀、`classify_codex_resume_failure`、director retry、`lib.rs:9939`；未碰 `lib.rs` ✓。
- 零新增 Tauri command / sidecar JSON ✓；未收 `*Label` 枚举族、未动 §十三重复簇 ✓。
- 白名单仅 §2.4 既有违规（含「等」补登 1 件，实证留痕）✓；未 stage、未 commit ✓。

## 八、start-end commit 与挂账

- start = end = `24bde2eaf99102623879644ad7211744355c73a7`（执行线不 commit，总指导核收后另定）。
- 挂账照旧（§十三）：`*Label` 枚举族约 167 个另包；重复簇治理另包；`TranscriptViews.tsx:428/603` 与 `main.tsx:46` 白名单登记随③清单治平；③清单随走查攒。
