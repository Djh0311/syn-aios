# 决策：shape gate 新增 machine_face_on_ui 机械规则 + 既有违规 defer 白名单 v1

日期：2026-07-20
任务包：`tasks/2026-07-20-human-language-engineering-modules-and-machine-face-gate-rule-package-v1.md`（人话工程②）
法源：交互宪法 §四.3「禁机器内部术语上脸」（`decisions/2026-07-14-interaction-model-canon-v1.md`）

## 拍板

1. shape gate（`scripts/harness/workbench-shape-gate.js`）新增机械规则 `machine_face_on_ui`：
   **UI 组件禁直渲机器格式错误串，新增零容忍（error 级）；既有违规登记豁免、不沉默。**
2. 规则本体拆 `scripts/harness/lib/machine-face-rule.js`（gate 脚本 489+3=492 行，不破 500 软限；
   §2.3 解法 a，首选）。gate 本体只加 require/挂载/打印各一行。
3. warn-only 档 `machine_face_state_hint`：原始 `error.message` 进 state 形先观察不拦
   （照 `converged_helper_redefined` 先例），误报面大。

## 规则语义

| pattern | 级别 | 拦什么 |
|---|---|---|
| `jsx_error_message` | error | JSX 直渲 `{error.message}` / `{this.state.error.message}` 形 |
| `jsx_event_stderr_pre` | error | `<pre>stderr: {…stderr}</pre>` 形 |
| `state_error_message` | warn | `error instanceof Error ? error.message : String(error)` 进 state 形 |

扫描面：`prototypes/productized-desktop-shell/src/**`（.tsx 查 JSX 两形，.ts/.tsx 查 state 形）。
`<details>` 下钻 `raw_snippet` 合规格板（`JiaobanHistory.tsx:274`，离线断言锁
`tests/jiaoban-history-and-secretary-board.test.tsx:123-145`）不匹配以上三形，天然豁免；
selftest case 4 锁死不误伤。

## MACHINE_FACE_DEFER_WHITELIST 条目（与规则文件一一对应）

| 条目（pattern\|path） | 豁免理由 |
|---|---|
| `jsx_error_message\|…/src/main.tsx` | 启动失败屏 `<code>{this.state.error.message}</code>`（:46）；首屏诊断面，随③清单另包治平 |
| `jsx_event_stderr_pre\|…/src/views/agent/TranscriptViews.tsx` | 转录详情 `<pre>stderr: {event.stderr}</pre>`（:428/:603）；转录诊断面，另包治理（§十三挂账） |
| `state_error_message\|…/src/views/AuditLedgerView.tsx` | :87 `setLedgerError(error instanceof Error ? …)`；warn 档观察件 |
| `state_error_message\|…/src/components/SecretaryBrief.tsx` | :116 `reason: error instanceof Error ? …`；warn 档观察件 |
| `state_error_message\|…/src/views/projects/ProjectJiaobanPanel.tsx` | :613/:1139 `set*Error(error instanceof Error ? …)`；warn 档观察件 |
| `state_error_message\|…/src/views/WorkflowCommandConsoleView.tsx` | :19 本地 `messageOf` 同形；**§2.4「等」覆盖的既有观察件，勘察漏列，HEAD 既有（git show 实证），执行线补登并主动披露** |

App notice 三处（`App.tsx:175/206/281`，及同族第四调用点 :630）本包已治平
（`messageOf` 接 `src/lib/humanize.ts` 薄委托，命中族出人话、未命中原文逐字回退），
**不进白名单**。

## 纪律

- 新增直渲 = error，check 模式直接 fail；不许为过关把新违规塞进白名单（任务包 §六.5）。
- 白名单粒度照 `DEDUP_DEFER_WHITELIST` 先例 = `pattern|path` 文件级；同文件新增同形命中同样被 defer，
  属已知粒度代价，随③清单治理时一并清点。
- selftest：`scripts/harness/workbench-shape-gate.machine-face.selftest.js`（18 断言，夹具树模式照 dedup 先例）。
