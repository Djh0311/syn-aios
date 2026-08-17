# SYN-M5R07 U02 ordinary disposable positive Tauri runner

日期: 2026-08-18
阶段: stage-14 / leaf M5R07
状态: implementation candidate / NOT_ACCEPTED / NOT_M5_COMPLETE / NOT_CLOSEOUT
合同: `docs/contracts/m5-r07-ordinary-positive-tauri-acceptance-v1.md`
父提交: `d6c5d6bd6cbe3e6021c81aca8a138481068ee4cf`

U01a / U01b / U01c 已 scoped PASS，本包不重做。本包只关 ordinary disposable fixture-only positive Tauri 纵向主链。

## 做完的标准

- 独立 env + 不可猜 capability / profile；普通生产无 env 时 byte-semantics 不变。
- 普通 AppState / 普通 authority；Rust fixture 用现有 M1 `register_exact_alias` 与现有 M3 provision / load。
- fixture locator 与 exact alias 显式对应；M5 不自动登记、不做 path-hash fallback。
- 默认 `jiaoban` + 正式 `ProjectSupervisorPanel` 自动 open。
- reject 零 runtime effect；approve 后 acceptance-only seed 把当前正式链落成 known-no-effect FAILED/TIMED_OUT；UI `can_retry`。
- RETRY 只建新 lineage；显式 runtime 成功；重复 runtime 不产生第二 effect/receipt。
- 同 app-data 第二进程 reopen 可读。
- JSON 标明 `ORDINARY_DISPOSABLE_FIXTURE_ONLY`、`NOT_LEGACY_COMPOSITION`、`NOT_STAGE_CLOSEOUT`；无截图则 `NO_WINDOW_CAPTURE`。

## 明确仍未完成

- 真实 legacy M1 composition
- shared-isolated 正向证据
- STOP/RESUME 真实窗口覆盖
- RUNNING/LEASED authoritative cancel
- OUTCOME_UNKNOWN reconcile
- M5 / stage-14 完成或 closeout
