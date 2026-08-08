# M2a · T2 真隔离 App 崩溃恢复验收记录 · 2026-08-04

任务：`tasks/2026-08-04-syn-m2a-t2-isolated-crash-recovery-package-v1.md`（含用户批准的 R4 harness 扩展授权）
工作目录：`/Users/yoyi/workspace/product-line-syn-fnd-002`，分支 `syn-fnd-002-dev`
证据等级：**ISOLATED-RUNTIME**（真隔离 App + console invoke 打生产命令 + SIGKILL + 读库核对）

## 0. 隔离机制与现场

- **R4 profile root（canonical）**：`/private/var/folders/nj/y6s1fvl936xgfwg20w08sk6r0000gn/T/syn-r4-acceptance-m2a-t2`
  - run_id `syn-r4-a4b6c8d0e2f4a6b8`，manifest `expires_at_ms` = 启动时 +12h（记录在 profile.json）
  - 校验说明：R4 校验器要求 root 直接位于 canonical `std::env::temp_dir()`（本机为 `/private/var/folders/.../T`，非 `/private/tmp`），名称前缀 `syn-r4-acceptance-`，0700，非 symlink，同 uid
- **store 绝对路径**：
  - JSON：`<root>/workflow-state/workflow-state.v0.json`
  - SQLite：`<root>/runtime-artifacts/workbench.sqlite`（db_primary，operator 预置全量 schema + fixture 镜像行）
  - storage-mode 配置：`<root>/runtime-artifacts/storage-mode.v1.json`
  - App 日志：`<root>/logs/` + tauri dev 全程日志 `/tmp/m2a-t2-dev*.log`
- **启动命令**（每轮一致）：
  `HOME=/private/tmp/m2a-t2-home SYN_R4_ACCEPTANCE_PROFILE=<root>/profile.json RUSTUP_HOME=/Users/yoyi/.rustup CARGO_HOME=/Users/yoyi/.cargo tauri dev --config /private/tmp/m2a-t2-tauri-override.json`
  （override 仅 `{"app":{"withGlobalTauri":true}}`，repo `tauri.conf.json` 未动；HOME 仅供 toolchain/临时目录）
- **harness 扩展**（用户授权，详见 §6）：R4 容忍可选 `runtime-artifacts/`；`.r4-initialized`（run_id 匹配）崩溃重进；三个 debug-only 验收门。首次启动过 first-init 全量校验（无 marker），marker 在首启后落盘，之后全部重启走重进模式。
- **场景输入准备**：fixture 为 R4 合同裸 workflow（无节点，R4 校验要求 nodes/edges 为空）；操作者在停机后向 JSON 与 DB 双侧注入默认 7 节点/6 边结构 + 1 个 draft work item（hash 按 `validate_stored_record_hash` 的 compact-JSON sha256 形式，重启对账绿）。work_item_id 见各场景。

## 1. 场景矩阵与实物证据

### S1 冷启动 ✅

| 项 | 实物 |
|---|---|
| 启动 | first-init 校验通过，日志 `storage mode=db_primary_json_projection db_path_hash=ce1939fc…`（/tmp/m2a-t2-dev.log），无 blocked |
| 合法命令 | console `update_work_item_state` draft→ready_to_dispatch 返回"已推进工作项状态：草稿 -> 待派发" |
| DB 读数 | command_receipts=1（COMMITTED `e610ce5a-…`）、events=1、audit_records=1、current_snapshots=1、work_items.state=`ready_to_dispatch` |
| 指纹 | JSON sha256 `3f1f8a7f…`、DB sha256 `6d0127a8…`（推进后） |
| 重启 | SIGTERM 54538 → 重进模式启动绿（/tmp/m2a-t2-dev2.log），数据可读研（上表读数为重启后所读） |

### S2 commit 前强退 ✅

| 项 | 实物 |
|---|---|
| 确定性窗口 | 门文件 `<root>/runtime-artifacts/acceptance-gates/pre-commit.pause`（armed 后 console invoke 在 `immediate_transaction_attempt` 的 commit 前阻塞，App stderr `acceptance_gate_armed:pre-commit: waiting for operator release` @/tmp/m2a-t2-dev4.log） |
| 强退 | 哨兵监听 gate-armed 日志行后 `kill -9` App PID **86254** @14:03:32（/tmp/m2a-t2-s2-kill.log；进程数归 0） |
| 杀后读库 | command_receipts=1、events=1、audit_records=1、current_snapshots=1（与杀前全同）、work_items.state=`ready_to_dispatch`——**零半提交**：M2 四表与业务行均无该命令痕迹 |
| 重启 | 重进启动绿（/tmp/m2a-t2-dev5.log），对账无异常；未提交命令未变成已提交 |

### S3 commit 后强退（receipt 丢失窗口）✅

| 项 | 实物 |
|---|---|
| 确定性窗口 | 门文件 `post-commit.pause`；invoke ready_for_review 在**事务 commit 后、JSON 投影前**阻塞（`acceptance_gate_armed:post-commit` @/tmp/m2a-t2-dev7.log） |
| 强退 | 哨兵 `kill -9` App PID **57143** @15:19:33（/tmp/m2a-t2-s3-kill.log） |
| 杀后读库 | command_receipts=3、events=3、audit_records=3、snapshots=1、**DB** wi=`ready_for_review`（commit 已落盘）；**JSON** wi=`running`（投影未发生，陈旧） |
| 重启恢复 | 启动对账 **fail-closed blocked**（/tmp/m2a-t2-dev8.log）：`db_json_reconciliation_not_green: workflow_nodes:hash_mismatches=[review 节点]、work_items:db_leading=[wi]、workflow_audit_events:db_leading=[audit]`；降级 json_only、数据无损、需重 seed 恢复 DB 主写（产品当前语义原文）。**注**：两侧 updated_at 新旧决定 replay vs fail-closed（`compare_record_freshness`）；本轮 fail-closed 受操作者夹具时间戳（人为未来值）影响，与 S4 的 replay 分支互为对照，均如实记录 |
| 附带观测 | 同窗口一次 gate 超时（非 SIGKILL）变体（running 已提交未投影）重启表现同为 blocked——同状态殊途同归 |

### S4 投影失败 ✅

| 项 | 实物 |
|---|---|
| 注入点 | 门文件 `projection-fail.pause` → 投影闭包内 `acceptance_injected_failure`（debug-only，profile 限定） |
| 命令错误 | console 返回 **错误** `acceptance_injected_failure:projection-fail`（未伪装成成功） |
| 双侧指纹 | DB：receipts=4、events=4、audit_records=4、snapshots=1、wi=`accepted`（事务已提交）；JSON：wi=`ready_for_review`（投影失败未写） |
| 重启对账 | 启动日志无 blocked（/tmp/m2a-t2-dev10.log）：对账判 db_leading（DB updated_at 更新）→ **replay**：JSON wi 更新为 `accepted`（updated_at=1785828558312），对账转绿 |

### S5 重复 command ✅

| 项 | 实物 |
|---|---|
| 两次返回 | 第二次 console 返回"幂等重放：该状态推进命令已处理，返回既有 receipt，未新增任何变更"，`audit_event_id=idempotent-replay:c46bf6e9-…` |
| 行数差 | receipts=4、events=4、audit_records=4、snapshots=1 与重放前**全同**；返回的 receipt_id 与既有 COMMITTED receipt `c46bf6e9-…` 相同 |

### S6 JSON-leading ✅

| 项 | 实物 |
|---|---|
| 构造 | 停机手改 JSON：wi accepted→needs_changes、updated_at=1785829477803（新于 DB）、revision 10→11；构造前 DB sha256=`508eae7e…`（/tmp/t2-s6-db-hash-before.txt） |
| 启动日志 | `blocked; 已降级 json_only…reason=db_json_reconciliation_not_green:work_items:json_leading=[wi]`（/tmp/m2a-t2-dev11.log） |
| 两侧指纹 | 重启后 DB sha256 仍 `508eae7e…`（**未被证据输入覆盖**）；JSON 保持 needs_changes/revision=11（**未反向覆盖**） |
| 后续命令 | blocked 状态下 invoke needs_changes→ready_to_dispatch 返回成功（BlockedJsonOnly 降级）：JSON 推进为 ready_to_dispatch，**DB wi 仍 accepted、receipts/events 仍 4、DB sha256 仍 `508eae7e…`**——降级不伪装成 DB 写入 |

## 2. 真实 HOME 零接触核对

- 前置指纹（验收前）：`~/Library/Application Support/CodexGovernanceWorkbench/**` 905 文件逐文件 sha256（/tmp/t2-real-home-fingerprint-before.txt）；主 store mtime 2026-08-01 01:35:38。
- 后置指纹：**904/905 逐字节一致**。唯一差异 `workflow-state/exec-process-registry.v1.json`：今天 13:22:50 一次仅 revision 递增写入（entries 仍空、9 条旧事件无新增）。**用户当场确认 13:22 前后打开过本机 Codex 系 app**——其启动时 startup reap 的持久化，与本任务隔离运行无关；本任务全部隔离 App 均绑定隔离 HOME/R4 路径（代码路径已核：profile 有效时所有派生路径指向 profile root）。
- 主 store（workflow-state.v0.json）前后 hash 一致、mtime 不变。

## 3. 改动的 Rust（最小 debug-only、profile-gated）

| 文件 | 改动 |
|---|---|
| `acceptance_runtime_profile.rs` | ① root 布局容忍可选 `runtime-artifacts/` 目录；② `.r4-initialized` 崩溃重进模式（marker 由操作者落盘、内容须同 run_id，否则按 reused 拒绝；重进仍强校验 location/symlink/0700/owner/manifest 身份，跳过全新夹具内容检查）；③ debug-only 验收门 `acceptance_gate_armed` / `acceptance_wait_for_gate_release`（120s 兜底 fail-closed 报错、不提交）/ `acceptance_injected_failure` |
| `workbench_sqlite_repository.rs` | `immediate_transaction_attempt` 在 operation 完成、commit 之前插入 `#[cfg(debug_assertions)]` 的 pre-commit 门（release 编译消失） |
| `workflow_run_dispatch_entrypoints.rs` | `update_work_item_state_db_primary`：事务返回后、投影前插入 `#[cfg(debug_assertions)]` post-commit 门；投影闭包内插入 `#[cfg(debug_assertions)]` projection-fail 注入 |
| `acceptance_runtime_profile_tests.rs` | 5 个拒绝路径测试：首启容忍 runtime-artifacts、重进脏 root 接受、错 run_id marker 拒绝、重进未知条目拒绝、无 profile 时门全惰性 |

约束核对：普通 App（无 profile）门恒惰性（测试锁定）；门调用点全部 `#[cfg(debug_assertions)]`（release 不含）；生产超时/事务语义零改动（门为附加阻塞点，超时 fail-closed 不提交）；无通用环境变量开关（门文件必须落盘于已校验 profile root 内）。

## 4. 验证命令与末行数字

| 命令 | 退出码 | 末行 |
|---|---|---|
| `cargo test --lib acceptance_runtime_profile` | 0 | `test result: ok. 24 passed; 0 failed` |
| `cargo check --lib` | 0 | 694 warnings（与 T1-R2 基线一致，零新增） |
| `cargo test --lib`（全量） | 0 | `test result: ok. 1348 passed; 0 failed; 45 ignored`（日志 /tmp/t2-final-test.log） |
| `git diff --check` | 0 | 零输出 |
| `node scripts/harness-v2/project-context.js --target .` | 0 | Advisories: none |

## 5. 边界声明

- 六个场景全部完成，无 HOLD；"强退"均为对隔离 App 真实进程的 `kill -9`（PID 与时刻见各场景）。
- 崩溃窗口全部由门文件界定（armed 日志行 → 哨兵 SIGKILL），非时间竞速。
- 操作者夹具注入（节点结构/work item/修复性同步）与手工 JSON-leading 构造均为**场景输入准备**，已在 §0/§1 显式标注；崩溃与恢复的被测行为全部经生产 Tauri 命令与启动对账真实执行。
- `m2_isolated_app_acceptance.rs` 的进程内函数未用作本记录任何结论的依据。
- T2 完成不等于 M2 完成；T3/T4 与最终 M2a 收口仍属指导线动作。
