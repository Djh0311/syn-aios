# 咨询线审计：甲·中转 relay ③b-1「真发入口」v1

日期：2026-06-18
审计线：咨询线（Claude）
审计对象：Codex ③b-1 实现（提交前工作树）；任务包 `tasks/2026-06-18-codex-relay-real-relay-runner-entry-b1-v1.md`。
前序复核：Einstein（Newton 执笔），`STATUS: CLEAR_WITH_NOTE`，无 P0/P1/P2/P3。

## 结论

**STATUS: PASS（CLEAR_WITH_NOTE）**。真发入口写对且锁死、本包未真跑 codex；placeholder 模式也补了 cfg(test) gate（采纳③a 审计 note①）。一个证据缺口（TDD 红测原始输出丢失，Codex 诚实交代、未伪造）可接受。**真发入口就绪——③b-1 真发 = 用户在场、设 env、单独步**。

## 实物核验（逐项对代码 / git / 测试实核）

- **真发入口**：ignored test `manual_relay_b1_real_codex_runner_entry_requires_user_present_env`（`#[ignore]` + 明示用户在场）→ `std::env::var("MANUAL_RELAY_REAL_CODEX_CONFIRM")` 校验 `CONFIRMED_USER_PRESENT_REAL_RELAY` → `run_input.mock_behavior="real_codex_env_gated"` → **真 `run_manual_relay_once` → 真 spawn codex** → `wait_*` 等完成 → **断言 `hello.txt` 真被创建 + 内容**。
- **第一次真发安全默认（照任务包）**：target = **temp 项目**（`std::env::temp_dir()` + `git init`，**非 mariotest**）；prompt = 最小无害（"创建 hello.txt 写一行 hi，然后回 MANUAL_RELAY_B1_REAL_CODEX_OK"）；workspace-write 限 temp；可观察（文件 + 回执）、可回滚（temp + git）。
- **Codex 未真跑**：`manual_relay.rs` 无 `env::set_var`（只 ignored test 里 `env::var` + 负向 test `env::remove_var`）；ignored 真发 test 未跑（本包 15 passed / **2 ignored**）。
- **placeholder 也 gate**（采纳③a note①）：`placeholder_process_mode_allowed()` 仅 `#[cfg(test)]` 为 true、`#[cfg(not(test))]` 为 false；与 mock_codex 模式同。**"所有非真-codex 进程模式生产不可达"统一达成**。
- **旧闸 5 文件 diff 空**；`5ea7c48`→工作树 Tauri 命令净增 0；未接前端；readback 只 last-message。
- `Command::new` 三处：777/1183 统一 spawn（前审过）、2144 = test 里 temp fixture `git init`（非 codex 执行）。`.codex` 仅 1491 denied prompt 测试样本。
- 独立重跑 `cargo test --lib manual_relay` = 15 passed / 2 ignored（咨询线亲跑）。

## TDD 红测缺口评估（如实标）

evidence「TDD Note」：B1 mock runner-entry test 据续传摘要曾先跑红，但当前 terminal 无可追回的 raw red 输出，故 evidence 未把它当 fresh proof、只证 green。Codex **主动交代、未伪造**。我的评估：**可接受**——不影响"现在 green + 锁死"的实物（我重跑 green），非安全关键路径，缺口已记录在案。

## 边界

本包未真跑 codex、未解锁、未放宽旧闸、未接前端。**③b-1 真发 = 用户在场、设 `MANUAL_RELAY_REAL_CODEX_CONFIRM=CONFIRMED_USER_PRESENT_REAL_RELAY`、跑该 ignored test、看 codex 真在 temp 创建 hello.txt** —— Syn 第一次真启动 Codex 的验证，单独步、不可逆性质。不得据此声称：已真跑 codex / 已解锁 / 第一次真 relay 已做。
