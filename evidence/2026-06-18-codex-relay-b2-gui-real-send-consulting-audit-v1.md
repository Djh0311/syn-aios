# 咨询线审计：甲·中转 relay ③b-2「GUI 直接发」v1

日期：2026-06-18
审计线：咨询线（Claude）
审计对象：Codex ③b-2 实现（提交前工作树）；任务包 `tasks/2026-06-18-codex-relay-b2-gui-real-send-v1.md`。
前序复核：Erdos（`019ed9ba-3f4c-72a1-b595-6487fceb7b6b`），初始 `CLEAR_WITH_P2`（两 P2 已修）→ `STATUS: CLEAR`。

## 结论

**STATUS: PASS（CLEAR）**。GUI 直接发链路做对、**命根子（codex 审批/sandbox 不放松）守住**、Codex 未真发；真机界面截图为证。一个可接受证据缺口（TDD 红测原始输出，同 ③b-1）。**首次 GUI 真发 = 用户在场、先 temp。提交注意：`tmp/b2-gui-real-send-browser/` 临时目录不进 git。**

## 实物核验（逐项对代码 / git / 测试 / 截图实核）

- **命根子：codex 真发 argv 审批 / sandbox 不放松**（重点亲核）：真发 argv 带 `--sandbox workspace-write` + `--add-dir <project_root_canonical>`（写入限项目内）；`codex_approval_bypass_arg`（`manual_relay.rs:1306-1310`）拒 `--full-auto` / `dangerously-bypass` / `--approval*` / `full-auto`，校验点 `:1293-1295` 命中即 `Err(manual_relay_gui_direct_approval_bypass_arg_forbidden)`。测试 `manual_relay_gui_direct_send_uses_bound_target_without_approval_bypass`（`:1950`）断言 argv 含 `--sandbox`、**不含** `--full-auto`/`dangerously-bypass`。→ **防误改靠 codex 自身 sandbox 限项目 + 不绕审批 = 等于直接用 codex**。
- **直接发链路**：前端 `data-send-mode="manual_relay_direct"`、Enter / Send **仅在绑定 project + Codex session 时**调 GUI 直发 handler；**target strip 常驻可见**（项目 path / 会话标题 / exact `thread_id` / `manual_once` / `auto_chain=false` / `sandbox=workspace-write`）；非 Codex 会话显 `仅 Codex 会话可用`、Enter 不触发。三本分守：原话逐字、target 精确（绑定 + 常驻可见 + canonical/hash 校验）、一次一发。
- **真机截图实物**：Codex 启动真 Tauri app（非仅 Vite 页）截图 `evidence/2026-06-18-codex-relay-b2-gui-real-send-artifacts/tauri-app-real-launch-post-p2.png`（SHA-256 `357a997d36c031176c0df5c89ed5d9bf35fb139b3b64dbf4bbece3a6e6ffa456`）：显示 `GUI direct relay 已绑定` + target 常驻条 + `Enter 将直接发送给：全局主管新` + Stop 区。
- **Codex 未真发**：未设 `MANUAL_RELAY_REAL_CODEX_CONFIRM`（仅 test 里 `env::remove_var`）、未点发送、未跑 ignored、未真跑 codex（Erdos 同核）。
- **旧闸 5 文件 diff 空**；命令净增 +1（`run_manual_codex_relay_gui_direct`，GUI 直发入口，预期）；shape-gate 105 vs 97（=会话引擎 3 + relay 4 + ③b-2 1）；重跑 `cargo test --lib manual_relay` 16 passed / 2 ignored（咨询线亲跑）；旧门 focused 测试全绿。

## 证据缺口（如实标，同 ③b-1）

TDD 红测原始输出未持久化，Codex 诚实交代、用 regression test + fresh green 证明。可接受（不影响 green + 锁死事实，非安全关键）。

## 边界 / 下一步

本包未真跑 codex、未默认发真项目、未放宽旧闸、未放松 codex 审批。**首次 GUI 真发 = 用户在场、先连 temp / 受控项目、看 codex 真跑 + 审批层兜底**，验通再放开 mariotest / 真项目。**`tmp/b2-gui-real-send-browser/` 不进 git。** 不得据此声称：GUI 已日常真用 / mariotest 已真发 / 通用真实执行已解锁 / 乙·自动连环已开。
