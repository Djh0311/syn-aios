# 咨询线审计：甲·中转 relay「真跑前必修 3 条」实现 v1

日期：2026-06-18
审计线：咨询线（Claude）
审计对象：Codex 交付的「真跑前必修 3 条」实现（提交前工作树）；任务包 `tasks/2026-06-18-codex-relay-pre-real-relay-must-fix-implementation-v1.md`。
前序复核：独立复核线 Darwin（agent `019ed72a-4661-7502-988c-c57dedc60f32`），初审 P1/P2 已修 → `STATUS: CLEAR_WITH_NOTE`。

## 结论

**STATUS: PASS（CLEAR_WITH_NOTE 口径）**。必修 3 条全部做实；真 codex 仍锁死；mock 包合格收口。一条 `.codex` 边界偏差（M-0004）如实标记，不阻断本包代码收口，记入③前置风险。

## 实物核验（逐项对代码 / git / 测试实核）

- **必修 1 路径精确**：`verify_strict_run_paths`（`manual_relay.rs:793`）在真进程模式（`is_process_mode`）下对 project_root / target_cwd / allowed_write_roots 逐个 `canonical_path_text`（canonicalize 失败即 `Err`），并校验 `path_verified` + canonical 一致 + `target_hash` 一致；preview 用 `normalize_path_for_preview`，canonicalize 失败标 `verified=false`、run 真模式拒之。测试 `manual_relay_strict_run_requires_verified_paths` 钉死。
- **必修 2 一次一发原子**：后端 `reserve_confirmation_in_map`（`:516`）用 `Entry::Vacant/Occupied` 单锁原子占票；`register_running_attempt_once`（`:529`）在保护路径内 check duplicate + reserve + insert。并发测试 `..._for_concurrent_submit`（`:1261`）+ `..._duplicate_scope_..._atomic_...`（`:1305`）证明只 1 个成功。前端 `relayInputLocked`（`AgentChatComposer.tsx:40`）= 发送中 `readOnly` + `aria-busy` + 按钮置灰 + Enter 拦截。
- **必修 3 stop 真杀**：占位进程 program = **`/bin/sleep`（非 codex）**（`:828`），`spawn_placeholder_process`（`:839`）只 spawn 它；`stop_manual_relay_attempt`（`:455`）`child.kill()` + `child.wait()` 回收、记 `real_process_killed`。测试 `manual_relay_placeholder_process_can_be_stopped_and_reaped`（`:1360`）断言 spawn 真进程 → stop → `real_process_killed=true`、`real_codex_executed=false`。
- **真 codex 锁死**：`real_codex_env_gated` 模式在 run 里直接 `Err("manual_relay_real_codex_env_gated_not_enabled_in_this_package")`（`:332`）——连 spawn 都到不了；`#[ignore]` + `MANUAL_RELAY_REAL_CODEX_CONFIRM` env gate 测试（`:1397`）钉死。**本包无真 codex 执行路径。**
- **旧闸未动**：5 文件（`session_continuation_store` / `k3_b1_recovery` / `real_execution_command` / `codex_local_runner` / `h5_project_dispatch_bridge`）git diff 空。
- **命令净增 0**：`b99f16c`→工作树 新增 `#[tauri::command]` = 0（未偷加命令；shape-gate 104 vs 97 仍是 baseline 滞后）。
- **独立重跑**：`cargo test --lib manual_relay` = 10 passed / 1 ignored（咨询线亲跑）。唯一 `Command::new` = 占位 spawn 路径。

## M-0004 边界偏差评估（如实标记）

- 事实：Codex 在**尝试 relay UI 真机浏览器验证**时，读了 `/Users/yoyi/.codex/plugins/.../control-in-app-browser/SKILL.md`（浏览器插件 skill 说明）。
- 性质：读的是**工具能力说明，非 auth / token / secret / full transcript / rollout / 会话状态**；未泄漏敏感内容、未写入产品代码或 evidence（Darwin 确认改动文件 `.codex` 命中均为 deny-list / 测试样本）。
- 边界判断：仍是「不碰 `.codex`」硬线的一次实际触碰（读了该目录下一个文件）。Codex 自查发现 + 记 ledger M-0004（关联既有 M-0001）——诚实自纠，应肯定。
- 处置：不阻断本 mock 包代码收口（非敏感、已记录、产品代码不读 `.codex`）；**揭示真实风险**——「真机浏览器验证」会把 agent 引向 `.codex` 下的浏览器插件。**记入③前置：真机验证不得读 `.codex` 下任何文件（含浏览器插件 skill）。**

## 边界

本包未真跑 Codex、未解锁真实执行、未放宽任何旧闸、真 codex 路径锁死。③第一次真 relay = 用户在场、单独步、显式授权，且守 M-0004 揭示的 `.codex` 真机验证边界。Note：真机浏览器验证因环境阻断未做，offline 覆盖，结转。不得据此声称：真实执行已解锁 / relay 能真跑 Codex / 旧闸已放宽 / 第一次真 relay 已做。
