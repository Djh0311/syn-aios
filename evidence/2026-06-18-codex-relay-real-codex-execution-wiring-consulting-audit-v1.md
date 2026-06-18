# 咨询线审计：甲·中转 relay③a「接通真 codex 执行路径」v1

日期：2026-06-18
审计线：咨询线（Claude）
审计对象：Codex ③a 实现（提交前工作树）；任务包 `tasks/2026-06-18-codex-relay-real-codex-execution-wiring-v1.md`。
前序复核：Dirac（`019ed78e-f036-78f0-b576-e602fc87a79f`），初审 `FINDINGS`（P1：mock 模式生产可达）→ 修复 → `CLEAR_WITH_P2`（文档）→ 终 `CLEAR`。

## 结论

**STATUS: PASS（CLEAR）**。真 codex 执行路径已接通但**双锁锁死**、本包未真跑 codex；mock 测试桩已修为生产不可达（M-0005）。一个非阻断 note（placeholder 模式建议一并 gate）。③b 第一次真发 = 用户在场单独步。

## 实物核验（逐项对代码 / git / 测试实核）

- **真 codex 双锁**：① mock 进程模式（`mock_codex_process:<path>`，可 spawn 任意 path）**只在 `#[cfg(test)]` allowed、生产构建直接 `Err(manual_relay_mock_codex_process_mode_test_only)`**（`manual_relay.rs` 行 7-8 / 113-119）——P1 修复扎实；② 真 codex 模式 `real_codex_env_gated` 要 `ensure_real_codex_env_authorized()`（env=`CONFIRMED_USER_PRESENT_REAL_RELAY`），没设即 `Err`（行 169-172）。
- **本包未真跑 codex**：默认真 codex 测试 `#[ignore]`（1 ignored）；测试 `manual_relay_real_codex_env_gated_without_env_does_not_spawn` 钉死"没 env 不 spawn"；本包未设 env、未跑 ignored。
- **readback 只读** `command_plan.last_message_path`（不读 full transcript / rollout）。
- **stop** 只 kill 指定 `relay_attempt_id`；mock 进程 receipt `real_codex_executed=false`。
- **旧闸 5 文件 diff 空**；`a65e6d7`→工作树 Tauri 命令净增 0；未改前端 / command registry；重跑 `cargo test --lib manual_relay` = 13 passed / 1 ignored（咨询线亲跑）。
- 唯一 `Command::new`+`spawn`（行 309/327）是**统一 spawn**：mock 路径 = cfg(test) only、真 codex 路径 = env only——**生产无授权时到不了真 codex**。

## M-0005 评估

Dirac 抓的 P1：mock 进程模式作为普通 runtime `mock_behavior` 值，调用方理论上能指向任意本地可执行文件 spawn（无需 env）。已修为 `#[cfg(test)]` gate（生产不可达）。这是个真隐患（任意程序 spawn）、Dirac 抓得准、修得对、记 ledger——**肯定**。

## 两个 note（非阻断，记入③b 前置）

1. **placeholder 模式未 gate**：②遗留的 `placeholder_process_sleep`（写死 `/bin/sleep`）模式未 `#[cfg(test)]` gate——生产理论可达，但只能 spawn 写死的 `/bin/sleep`（无害）。建议③b 真 codex 接入前，把 placeholder 也 cfg(test) gate，统一"**所有非真-codex 进程模式生产不可达**"。
2. **前端未接"真发"按钮**：③a 只接后端真 codex 路径，前端 relay 模式未接"真发"。故③b 现实路径：**先用 env-gated test runner 真发一次验通后端真 codex 路径（用户在场），再决定是否接前端 GUI 真发**。

## 边界

本包未真跑 codex、未解锁真实执行、未放宽旧闸、真 codex 双锁。③b 第一次真 relay = 用户在场、单独步、显式授权。不得据此声称：真实执行已解锁 / relay 已真跑 codex / 第一次真 relay 已做 / 前端可真发。
