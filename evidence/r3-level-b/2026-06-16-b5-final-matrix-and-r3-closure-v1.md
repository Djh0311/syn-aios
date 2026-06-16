# R3 Level B B5 Final Matrix And Closure

日期：2026-06-16

状态：accepted_with_deferred_product_cutover（独立复核 Aquinas `019ece6b-4b39-7830-9553-86b979ec322c` CLEAR；用户 2026-06-16 拍板收口「受控迁移验证阶段 B0–B4」，产品切换类 deferred 另窗另批）

## 拍板摘要

建议批准：宣布 R3 Level B 的受控迁移验证阶段（B0-B4）完成。这里的“完成”只表示 B0-B4 的 preflight、hash calibration、production apply、limited read-cut、observation、stop-write decision 都已经有提交记录、evidence 和复核/核验记录，且 B5 矩阵将交独立复核线逐行核忠实性。

代价：无运行时代价。B5 是纯结论登记，不改代码、不跑 runner、不触碰真实数据、不触碰 `/Users/yoyi/.codex`。

不批的后果：R3 仍挂在“待收口判定”状态，依赖 R3 验证阶段收口的后续计划不能往下走。

关键澄清：这不等于产品已切到 DB，不等于已停写 JSON / sidecar，不等于完整存储迁移完成；这些都是另外的高风险窗口，必须另开窗、另写任务包、另经用户批准。

一句话判据：想知道 R3 收没收口，看 B0-B4 是否都“已提交 + 证据可追 + 复核/核验无 P0/P1 overclaim”；是，则受控验证阶段收口，产品切换类一律另窗另批。

## 口径

- `commit` 列采用“该窗口结果被提交进 main 的 checkpoint commit”。部分 execution record 中的 `git_head_before/after` 是 runner 执行时的前置 HEAD，可能早于提交 commit；本矩阵不混用这两个口径。
- `classification` 为 B5 对该窗口的收口分类：`accepted` 表示窗口目标已接受；`not_executed` 表示该窗口刻意未执行产品动作或因安全门中止；`deferred` 表示明确留到另窗。
- B5 本身不新增产品能力，只登记 B0-B4 的证据矩阵和收口判定输入。

## Final Matrix

| 窗口 | phase | status | commit | 证据路径 | 核过的关键结论 | 边界 | classification |
|---|---|---|---|---|---|---|---|
| B0 preflight | `B0_preflight` | 已完成 | `688108f` | `evidence/r3-level-b/b0-preflight-20260614-172731/` | 只读冻结真实 `WORKBENCH_STATE_ROOT`；source root hash before/after 均为 `2fbdb7bfdc71b30d5b4d2bec2dfdde50de98ab24942c8ba550d29b6b539d3b53`；真实状态为 `workflow-state.v0.json` + `plan-authorizations.v1.json`，其余 sidecar 多数缺失；production DB missing_before。 | 未建 DB、未切 read path、未停写 JSON/sidecar、未执行真实 Codex、未触碰 `.codex`。 | accepted |
| B0 hash 校准 | `B0_hash_calibration_refreeze` | 已完成 | `91b2225` | `evidence/r3-level-b/b0-hash-calibration-20260615-152136/` | 统一 B0 与 Level-B preflight aggregate hash 口径；canonical source root hash 冻结为 `31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801`；Ampere 复核 `CLEAR_WITH_P2`，P2 已修/澄清。 | 未执行 B1 apply、未建 DB、未写 source root、未切读、未停写、未触碰 `.codex`。 | accepted |
| B1 首次 apply | `B1_production_apply` | `failed_classified`，口径中止，benign | `97ec465` | `evidence/r3-level-b/b1-production-apply-20260615-150005/` | 安全门在 DB/backup/report/rollback 创建前中止：expected old B0 hash `2fbdb7bfdc71b30d5b4d2bec2dfdde50de98ab24942c8ba550d29b6b539d3b53` 与 actual canonical hash `31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801` 不同；分类为 hash 口径分叉，不是内容漂移。 | production DB 未创建；backup / rollback manifest / production apply report 未创建；read-cut / stop-write / 完整迁移均未发生。 | not_executed |
| B1 enablement | Level-B production apply entry | 已完成 | `370acd3` | `tasks/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1.md`; `evidence/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1.md`; `evidence/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-review-arendt-v1.md` | 新增显式、窄口径 Level-B confirmed-path production apply 入口；Level-A temp/R3 旧门未放宽；Arendt 补复核 `STATUS: CLEAR`。 | fixture-only enablement；未读取真实 state root、未建真实 DB、未执行 B1 apply、未切读、未停写、未触碰 `.codex`。 | accepted |
| B1 retry | `B1_production_apply` | `completed`，真库 `12d65f21...` | `789949c` | `evidence/r3-level-b/b1-production-apply-20260615-160110/` | 用 calibrated expected hash `31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801` 完成真实 production apply；created DB hash `12d65f21ae383b72afd1b23347548974502ba60ca6a4143ca6b6fc94270f03ba`；export verified；Beauvoir 复核 `CLEAR_WITH_P2`，P2 为 do_not_claim 机械误报风险，非执行越界。 | JSON 仍是权威；未 read-cut、未 stop-write、未完整迁移、未解锁多 agent、未执行真实 Codex、未触碰 `.codex`。 | accepted |
| B2a enablement | Level-B limited read-cut entry | 已完成 | `48513d5` | `tasks/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-v1.md`; `evidence/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-v1.md`; `evidence/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-review-arendt-v1.md` | 新增 Level-B confirmed-path limited read-cut 入口；`read_cut_report_path` P1 已修；Arendt 复核 `STATUS: CLEAR`。 | 未跑真实 B2 runner、未切产品全局 read path、未停写、未新增 Tauri/UI/startup 接入、未触碰 `.codex`。 | accepted |
| B2b read-cut | `B2_limited_read_cut` | `completed`，DB读 == JSON读 `dc0524de...` | `9edc2a7` | `evidence/r3-level-b/b2-limited-read-cut-20260615-191500/` | 受控 limited read-cut 验证通过：flag off 走 `json_fallback`，flag on 走 `db_limited`，两条路 `workflow_state_summary` projection hash 均为 `dc0524de763f785891148c71cde9a97ef0c451395199a5adffaf1a50fa70e0d5`；DB hash 前后 `12d65f21...`；Cicero 复核 `STATUS: CLEAR`。 | 只验证 `workflow_state_summary`；未切产品全局 read path、未停写、未改 UI/Tauri/startup、未执行真实 Codex、未触碰 `.codex`。 | accepted |
| B3a enablement | Level-B observation entry | 已完成 | `1a2db17` | `tasks/2026-06-15-root-treatment-r3-b3a-level-b-observation-entry-v1.md`; `evidence/2026-06-15-root-treatment-r3-b3a-level-b-observation-entry-v1.md`; `evidence/2026-06-15-root-treatment-r3-b3a-level-b-observation-entry-review-poincare-v1.md` | 新增 Level-B confirmed-path observation 入口；双样本稳定断言在；测试模块拆分后父/子均低于 3000 行；Poincare 复核 `STATUS: CLEAR`。 | 未跑真实 B3 runner、未切产品全局 read/observation path、未停写、未改 UI/Tauri/startup、未触碰 `.codex`。 | accepted |
| B3b observation | `B3_observation` | `completed`，两样本稳定 `0a79ba13...`，账本订正 M-0003 | `11beb3b` | `evidence/r3-level-b/b3-observation-20260615-225700/` | 真实 B1 DB 连读两次稳定：两样本 projection hash 均为 `0a79ba13d818bda886eea4b2abb0faa7710cd0c51b7018c37bd49c87405cb590`，export hash 均为 `1aef44c8ae3a046497be70a878720ea45c26e390bbb1edda2b5649b18f908326`；账本已删除误带入的假 flag-off result 并登记 M-0003；Parfit 复核 `CLEAR_WITH_NOTE`。 | 未产品全局观察上线、未切读、未停写、未完整迁移、未执行真实 Codex、未触碰 `.codex`。 | accepted |
| B4a enablement | Level-B stop-write decision entry | 已完成 | `26744ad` | `tasks/2026-06-16-root-treatment-r3-b4a-level-b-stop-write-decision-entry-v1.md`; `evidence/2026-06-16-root-treatment-r3-b4a-level-b-stop-write-decision-entry-v1.md`; `evidence/2026-06-16-root-treatment-r3-b4a-level-b-stop-write-decision-entry-review-parfit-v1.md` | 新增 Level-B confirmed-path stop-write decision 入口；构造上 decision-only，`approve_stop_write` 最高 `ready_but_not_executed`；Level-A temp/R3-A12 旧门未放宽；Parfit 复核 `STATUS: CLEAR`。 | 未跑真实 B4 runner、未真实 stop-write、未停写 JSON/sidecar、未切产品全局读写路径、未触碰真实 DB/source、未触碰 `.codex`。 | accepted |
| B4b decision | `B4_stop_write_decision` | `completed`，`ready_but_not_executed` | `6824402` | `evidence/r3-level-b/b4-stop-write-decision-20260616-020629/` | 受控真决策通过但不执行：Pass A `prepare_only` 探测按预期 `command_exit_code=101`，Pass B `approve_stop_write` runner `1 passed`；final report `ready_but_not_executed`；DB hash `12d65f21ae383b72afd1b23347548974502ba60ca6a4143ca6b6fc94270f03ba`，B4 fallback hash `ae0797f8c5fc4c156cc0f5f15ed686af9f7871642e42afffb45530a621edd061`，B3b projection hash `87f62158ceef5dbe303d7c704dd47a2c3ae3775181e7ed1efbe59ff182e82175`；Maxwell 复核 `STATUS: CLEAR`。 | 未执行真实 stop-write、未停写 JSON/sidecar、未切产品全局读写路径、未改 UI/Tauri/startup、未建库、未迁移、未触碰 `.codex`。 | accepted |

## 收口判定

B5 建议判定：R3 Level B 的受控迁移验证阶段（B0-B4）可以收口为 `accepted_with_deferred_product_cutover`。

理由：
- B0-B4 的所有窗口均已有 main commit 与 evidence 路径。
- 唯一 failed 窗口（B1 首次 apply）是安全门在写入前按设计阻断，随后通过 B0 hash 校准与 B1 retry 解决，分类为 benign / not_executed。
- B1 retry、B2b、B3b、B4b 均保留真实 DB / source 前后不变证据。
- B4b 只达到 `ready_but_not_executed`，未把 decision 伪装成 stop-write execution。

这份判定只关闭“验证阶段”，不关闭产品切换阶段。

## Deferred 清单

以下事项明确未做，均需另开窗口、另写任务包、用户另行批准：

- 真实停写 JSON / sidecar。
- 把产品全局 read path 切到 DB。
- 完整存储迁移，包括全部 sidecar，而不是仅当前真实存在的两个文件。
- 解锁多 agent 并行真实执行。
- 真实 Codex 执行。

## 不可声称

- 不得声称产品全局读写路径已切。
- 不得声称 stop-write 已执行。
- 不得声称完整存储迁移完成。
- 不得声称 R3 的产品切换阶段完成。
- 不得声称多 agent 并行真实执行已解锁。
- 不得声称本包执行过真实 Codex。
- 不得声称本包触碰过 `/Users/yoyi/.codex`。

## B5 边界

B5 只写本文档、独立复核文件与 `CURRENT.md` 收口条目。不改代码、不运行 runner、不触碰真实数据、不触碰 `/Users/yoyi/.codex`。
