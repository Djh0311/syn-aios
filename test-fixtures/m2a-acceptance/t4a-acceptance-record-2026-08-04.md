# M2a · T4-A 验收解阻证据记录 · 2026-08-04

任务：`tasks/2026-08-04-syn-m2a-t4a-acceptance-unblock-package-v1.md`（用户已批准；scope 经用户当场批准扩至同族 obsidian/manual_relay 夹具）
工作目录：`/Users/yoyi/workspace/product-line-syn-fnd-002`，分支 `syn-fnd-002-dev`，基线 HEAD `eb79443`
证据等级：**UNIT**（focused 测试与全量 `cargo test --lib` 均为进程内单测层级，未越级声称）

## 一、两条点名阻塞的诊断与修复

### A. `sqlite_production_preflight_blocked_creates_no_db_or_report`（preflight 应拒绝却放行）

**诊断**：缺陷在 fixture 输入，不在 preflight 分类或 apply 路由。
- `fixtures/r3-a9/production-preflight-blocked-denied-path/` 与 `production-valid-core-chain/` **逐字节相同**（`diff -rq` 零差异）；
- 回查创建提交 `52d6b4b`：该 fixture 自创建起即与合法 fixture 逐字节相同 → 该测试自 2026-06-11 引入起从未通过（kickoff 判定的"既有 bug"坐实）；
- 原始设计合同（`evidence/2026-06-11-...v1.md:139`）：该 fixture 应 "contains `.env` marker；验证 preflight blocked，不读取 body"。
- preflight 分类代码本身有拒绝分支单测覆盖且全绿（`workbench_sqlite_preflight.rs` denied name/path 系列），apply 路由在 DB/report 创建前调 `ensure_preflight_ready`（`workbench_sqlite_production_apply.rs:314`），两者无缺陷。

**修复**（最小正确修复，改 fixture 不改产品代码）：向 fixture 加入 `secret-token-fixture-marker.txt`（内容一行占位说明，preflight 按名拒绝、不读 body）。
- 不用文档原意的 `.env`：repo `.gitignore:7` 忽略 `.env`，且 `.githooks/pre-commit` 的 `git-gate.js --strict` 将 staged `.env` 判为 secret-bearing path 硬拒；占位内容也不在其 placeholder 白名单语义内。改用具名等价 denied marker（命中 `secret`/`token` 两个 marker 子串）。
- 机制核实：`denied_name_hit`（`workbench_sqlite_preflight.rs:560`）子串匹配 → `rejected` → `blocked_reasons=1` → `preflight_not_ready`，apply 在任何 DB/report 创建前返回。

**验证**（focused）：
```
cargo test --lib workbench_sqlite_production_apply::tests::sqlite_production_preflight_blocked_creates_no_db_or_report -- --exact --nocapture
→ test result: ok. 1 passed; 0 failed（0.04s）
```
断言语义原样保留：`err.contains("preflight_not_ready")` + `!db_path.exists()` + `!report_path.exists()`（未改测试、未删断言、未 ignore）。

### B. `codex_local_runner::tests::real_process_timeout_kills_and_reaps_mock_child`（PID 夹具全量并行不稳定）

**诊断**：竞态在子进程自报 pid 文件 vs 父进程 2s 超时杀之间——`/bin/sh` 新脚本首次 exec 延迟（实测见下）使 `echo $$ > mock-child.pid` 赶在 SIGKILL 前不成立（复现：focused 1/8 失败，`fs::read_to_string` NotFound @:2040）。

**修复**（确定性握手，不改产品超时语义）：
- `exec_process_registry.rs` 的 `#[cfg(test)]` 登记桩升级为测试专用 spawn 登记通道（`run_id → pid`，spawn 父进程同步点写入，**生产构建零代码变化**——`cargo check --lib` 不含该通道）；按 `codex_local_process_run_id(request)` 取回，与同进程并行测试隔离。
- 测试改用登记 pid 做回收核验（`kill -0` 探活）；子进程 pid 文件保留为交叉校验（落盘则必须与登记 pid 一致，实测 3/3 landed 且相等）；超时/回收/无 stale message 三类断言原样保留。
- mock 子进程改为 `/bin/sh -c <payload>`（见第三节根因），不再新建脚本文件。

**验证**（focused）：修复后 15/15 连续通过（修复前 1/8 失败）。

## 二、scope 扩展（用户当场批准）：同族进程夹具根因治理

全量复跑时同族其他成员翻转（catch-log 早已定性："该族三次全量各翻不同成员：codex_local_runner→obsidian→manual_relay"）。

**根因实测**（本沙箱，10 样本）：
- 新建脚本文件首次 exec：**155ms ~ 3198ms**（系统对新可执行文件的校验）；同文件再次 exec ~7ms；
- `/bin/sh -c <payload>`：**稳定 ~6-10ms**。
- 翻转成员的共同机制：夹具预算（1s deadline / 2s 超时窗 / 3s 就绪窗）被新脚本 exec 延迟撞穿——obsidian 两测失败时错误为 `obsidian_integration_timeout`（DIAG 实测坐实，非断言期望的领域错误）。

**修复**（同一确定性边界，全部只改夹具侧、产品零改动）：
- `obsidian_integration.rs`：`fake_executable`（新建脚本）→ `fake_sh_argv`（`/bin/sh -c` 载荷经 argv）；修复后 20/20 连续通过（修复前 ~70% 失败率），单次 0.05s（原 0.44~1.01s）。
- `manual_relay.rs`：新增 `mock_codex_process_sleep_sh:` 前缀（载荷入 argv，MockCodexSleepSh variant）；`manual_relay_app_shutdown_kills_active_process_group_children` 与 `supervisor_final_active_slot_collision_rekeys_pending_child_and_preserves_old_owner` 两个 3s 就绪窗测试改用之；模块 52 测全绿。
- 其余 sleep-mode 夹具用 5s 轮询预算且全量复跑从未翻转，未动。

**HEAD 对照**：obsidian 两测在 stash 本次改动后于 HEAD 复测 3/5 失败——预存环境族失败，非本次引入。

## 三、闸门命令与结果（普通构建档位，全量前台落盘）

| 命令 | 退出码 | 末行摘要 |
|---|---|---|
| `cargo test --lib workbench_sqlite_production_apply::tests::sqlite_production_preflight_blocked_creates_no_db_or_report -- --exact --nocapture` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1387 filtered out` |
| `cargo test --lib codex_local_runner::tests::real_process_timeout_kills_and_reaps_mock_child -- --exact --nocapture` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1387 filtered out` |
| `cargo check --lib` | 0 | **693 warnings**（与 T1-R2 指导线基线一致，零新增；唯一相关 `manual_relay.rs:4230 reserve_confirmation_once never used` 为预存） |
| `cargo test --lib`（全量 × 3 连续） | 0 | **`test result: ok. 1343 passed; 0 failed; 45 ignored; 0 measured`** ×3（修复过程中曾见 obsidian/manual_relay/quality_debt 翻转，逐个定性修复后稳定） |
| `git diff --check` | 0 | 零输出 |

全量日志：`/tmp/t4a-cargo-test-full.log`（首绿）、`/tmp/t4a-full-6.log`、`/tmp/t4a-full-7.log`、`/tmp/t4a-full-8.log`（连续三绿）。

## 四、边界声明

- 真实 HOME store 与 App 全程未触碰：无 App 启动、无 `~/Library/Application Support/**` 读写；全部改动为仓库内源码与 fixture。
- 无删断言、无 ignore、无 `--test-threads=1`、无重试包装、无串行化；产品超时/上限语义零改动（obsidian 的 1s/20ms 窗口、codex 的 2s 超时、manual_relay 的就绪预算全部原值保留）。
- `quality_debt_tests::timeout_triggers_one_auto_replan_with_facts_then_ran` 曾在第 5 次全量翻转一次（纯进程内 ScriptedRunner，非进程夹具；focused 6/6 通过、修复后连续 3 次全量通过）；未改它，记账观察。
- T1 验收与 T2 派发仍属指导线动作；本记录不构成 T1/M2 完成声明。

## 五、改动文件清单

1. `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a9/production-preflight-blocked-denied-path/secret-token-fixture-marker.txt`（新增）
2. `prototypes/productized-desktop-shell/src-tauri/src/exec_process_registry.rs`（cfg(test) spawn 登记通道）
3. `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`（测试用登记 pid + warm-sh mock）
4. `prototypes/productized-desktop-shell/src-tauri/src/obsidian_integration.rs`（fake_executable → fake_sh_argv）
5. `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`（MockCodexSleepSh variant + 两个就绪窗测试改造）
6. `test-fixtures/m2a-acceptance/t4a-acceptance-record-2026-08-04.md`（本文件）
7. `docs/harness/CURRENT.md`（状态回写：交付待指导验收）
