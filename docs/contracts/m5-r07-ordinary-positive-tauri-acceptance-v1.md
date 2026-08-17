# M5R07 U02 ordinary disposable positive Tauri acceptance v1

- 版本：v1（2026-08-18）
- 状态：**ADDITIVE / 非冻结正本 / 不构成 M5 或 stage-14 完成**
- 关系：补充 `m5-isolated-app-and-candidate-v1.md`、`m5-r07-ui-control-recovery-addendum-v1.md`、`m5-r07-terminal-retry-lineage-addendum-v1.md`。**不改 M1–M4 冻结合同正文与 hash，不改 shared-isolated authority/profile。**
- 范围：**只覆盖 U02 ordinary disposable fixture-only positive Tauri runner**。U01a / U01b / U01c 已 scoped PASS，本包不重做。本包不是 legacy ordinary GUI composition，不是 shared-isolated positive，不是 closeout。

## 1. 本包不重做的已落地项

- 默认入口保持 `jiaoban` 与正式 `ProjectSupervisorPanel`。
- `load_m5_execution_control` / `apply_m5_execution_control` 字段封闭；renderer 不传 operation / grant / dispatch / attempt / effect / fault。
- RETRY 只做新 lineage 准备，不执行 runtime；唯一生产 runtime 入口仍是 `run_m5_authorized_runtime_with_state`。
- shared-isolated profile 继续 M1/M3 uninstalled；`run-m5-isolated-app-acceptance.mjs` 继续 unavailable-only。

## 2. Composition

本包是 **ordinary disposable AppState + 服务器端 fixture**：

- 使用普通 `try_new_with_ordinary_product_ports` / OrdinaryInstalled authority。
- 独立 env / 不可猜 capability / profile 校验。无这些 env 时普通生产 byte-semantics 不变。
- 不得走 `try_new_with_isolated_product_profile`，不得把 shared-isolated 改成 positive。
- 明确标记 `ORDINARY_DISPOSABLE_FIXTURE_ONLY`、`NOT_LEGACY_COMPOSITION`、`NOT_STAGE_CLOSEOUT`。

## 3. Fixture 与 authority

- 临时 index 至少含一个安全 fixture project。其 locator 与 M1 exact alias **显式对应**。
- 只有 acceptance-only Rust 通过现有 M1 `register_exact_alias` 登记；再通过现有 M3 `provision` / `load`。
- Node / renderer 不得直接写 M1 registry、M3/M5 SQLite。
- 请求不得携带 actor / session / grant / dispatch / effect / fault 等 authority 字段。
- M5 不得自动登记或 path-hash fallback。

## 4. 纵向场景

1. 真实 Vite + Tauri 普通组合；默认 `jiaoban`；正式 `ProjectSupervisorPanel` 自动 open；binding / session 来自服务端。
2. 同一 fixture 项目 proposal reject，零 runtime effect。
3. 再 approve；acceptance-only Rust 把当前正式链落成 authoritative known-no-effect `FAILED` 或 `TIMED_OUT`；UI load control 显示 `can_retry`。
4. UI 点 RETRY，只创建新 Attempt / Grant / Dispatch / effect，不执行 runtime；再显式点 runtime 并成功；重复 runtime 不得产生第二 effect / receipt。
5. 同 app-data 第二进程 reopen，canonical project / binding / formal state 可再读。

known-no-effect terminal seed 只能由当前 stored chain 在 acceptance-only Rust 模块派生。

## 5. Receipt

- Rust 写 backend-derived receipt。
- renderer 写 DOM interaction receipt。
- Node launcher 只负责临时目录、启动 / 等待 / 进程管理与聚合。
- runner 以 exact window / process receipt 为准；不得使用 root / largest-window 截图 fallback。
- 未截图必须记 `NO_WINDOW_CAPTURE`，不得伪造。

## 6. 明确未关闭

- 真实 legacy `ProjectRecord` → M1 canonical / exact alias 的可信创建 / 迁移 / ordinary GUI composition
- shared-isolated 正向 scene / window / restart
- `RUNNING` / `LEASED` authoritative cancel
- `OUTCOME_UNKNOWN` 同 effect reconcile
- STOP / RESUME 真实 Tauri 覆盖（本包不阻塞）
- M5 / stage-14 closeout / M6 激活
