# 咨询线审计：甲·中转 manual relay 实现（mock 包）v1

日期：2026-06-18
审计线：咨询线（Claude）
审计对象：Codex 交付的 manual relay 实现（提交前工作树）；任务包 `tasks/2026-06-17-codex-relay-manual-implementation-v1.md`、设计正本 `docs/plans/2026-06-17-codex-relay-stepping-stone-design-draft-v1.md`。
前序复核：独立复核线 Hegel（agent `019ed696-09cb-78d3-897a-1a60f259a2c5`），初审 `FINDINGS`（3 P1 已修）→ 复扫 `CLEAR_WITH_P2`。

## 结论

**STATUS: PASS**（接受为合格 mock 包，等同复核 `CLEAR_WITH_P2` 口径）。mock 包合格收口；**真实 relay 前有必修 3 条（见下），不阻断本 mock 包**。

## 实物核验（非橡皮图章，逐项对 git / 代码 / 测试实核）

- **旧闸未动**：`session_continuation_store.rs` / `k3_b1_recovery.rs` / `real_execution_command.rs` / `codex_local_runner.rs` / `h5_project_dispatch_bridge.rs` 五文件 `git diff` 空。relay 是**叠加**调用既有 `inspect_codex_local_execution_guard`，未改其逻辑。
- **无真执行 / 无外发 / 无 .codex 读写**：`manual_relay.rs` 全文 grep 无 `Command::new` / `process::Command` / `spawn` / `codex exec` / `reqwest` / `http` / `TcpStream`；`.codex` 仅 3 处命中 = deny-list（`denied_material_requested`，行 737-756）+ 测试恶意样本。回执 `real_codex_executed=false` / `syn_read_codex_home=false` / `syn_wrote_codex_home=false` / `prompt_sent=false`。
- **新增 Tauri 命令净增 = 4**：`git diff b56bad8 → 工作树` 的 `#[tauri::command]` 新增 4、删 0，全是 relay wrapper（preview / confirm / run / stop）。shape-gate 报 `104 vs baseline 97` 的 +7 = relay 4 + 会话引擎 3（已在 b56bad8、baseline 滞后未棘轮），**非私货**。
- **独立重跑测试**：`cargo test --lib manual_relay` = 5 passed 0 failed（咨询线亲跑，不取报告）。

## 红队 5 点

| 查 | 结论 |
|---|---|
| 真没跑 Codex / 没碰 .codex | ✅ |
| 旧闸真没放宽 | ✅（5 文件 diff 空） |
| 合设计·不伪装 H2 / K3-B1 | ✅（新 `ManualRelayEnvelope` contract；`payload_layers` 空 + `future_hooks` 预留） |
| 测试钉死安全门 | ✅（payload 逐字三重 / 三种 hash 不匹配 / duplicate / 一次性消费 / secret deny / command plan 无 shell / stop 定向 / dirty 不自动回滚） |
| shape-gate +7 | ✅（= relay 4 + baseline 滞后 3） |

## 真实 relay 前必修 3 条（对到设计已有 gate / 本分，非另起炉灶）

1. **路径精确**（必修 1）↔ 设计本分二「target 校验、不靠 fallback 推断」。现状 `normalize_path_text`（行 758）`canonicalize` 失败时走词法 `clean_path` 兜底——mock 路径不存在无害；**真跑前必须改为"canonicalize 成功才放行"**，否则 symlink / 别名 / 不存在路径可能让 target_hash 失真、发错地方。
2. **一次一发原子**（必修 2）↔ 设计本分三「一个 confirmation_id、terminal 后重新确认」。两层：①**前端发完清空 + 置灰**（用户 2026-06-18 提，第一道、本就该有，防手抖连点）②**后端 consume 改原子**（现 check 行 327-333 与 insert 行 400-403 分离 lock window，TOCTOU，mock 单线程无害，**真跑并发前必须 reserve/consume 原子**）。
3. **stop 真杀进程**（必修 3）↔ 设计「停」原文「无法提供可点击 stop、只靠 timeout，不应宣称能停」。现 stop（行 408）mock 下只移除内存 registry、标 `killed_by_user`（定向正确、测过不误伤），但"真 kill 真 child"未验（无真进程）；**真跑前必须接真 runner 实测"按停 → 进程真没了"**。

## 边界

本审计针对 mock 实现；未真跑 Codex、未解锁真实执行、未放宽任何旧闸。必修 3 条 + 用户在场授权语句 = **真 relay 前置任务包**（计划新增步骤）。不得据此声称：真实执行已解锁 / relay 能真跑 Codex / 旧闸已放宽 / 第一次真 relay 已做。
