# SYN M2a · T2 派发包：真隔离 App 崩溃恢复验收

date: 2026-08-04
status: ACTIVE — T1-R2 与 T4-A 已由指导线验收后派发
dispatcher: 总指导线
executor: 新执行会话

---

## 0. 对齐块（先读，不猜）

- `authority_chain`: 当前用户 2026-08-04 同意 T4-A 更正、验收并派发 T2 → `decisions/2026-08-03-syn-m2-blanket-authorization-v1.md`（含真实 App 强退/崩溃注入）→ `tasks/2026-08-03-syn-m2a-kickoff-v1.md` §2 T2、§3、§4 → `docs/harness/CURRENT.md` → `AGENTS.md`。
- `plan_anchor`: M2a kickoff §2 的 T2；目标是推翻 DAT-008 的“进程内函数算隔离 App”假验收，不是再写一层单测。
- `existing_before_new`: T1-R2 已把生产 Tauri 命令 `update_work_item_state` 接入 SQLite M2 UoW；其隔离 App 记录在 `test-fixtures/m2a-acceptance/acceptance-record-2026-08-04-t1r2.md`。FND-006 已验证 `HOME=/private/tmp/...` + `RUSTUP_HOME`/`CARGO_HOME` + 临时 `withGlobalTauri` override 的隔离启动方式。`m2_isolated_app_acceptance.rs` 现有函数只是 UNIT，不构成本任务证据。
- `capabilities_touched`: 仅限隔离 App 启动、console invoke `update_work_item_state`、隔离 HOME 内 JSON/SQLite/日志的测量与受控故障验收；若确有必要，可增加仅在隔离 debug profile 可达的验收控制。
- `forbidden_alternatives`: 不触碰真实 HOME store、`.codex`、外部 provider、T3 manifest、T4 grant/code-map；不把库函数、直接 SQLite 改行、睡眠竞速、`--test-threads=1` 或“预期行为”称为真机崩溃恢复。

工作目录：`/Users/yoyi/workspace/product-line-syn-fnd-002`，分支 `syn-fnd-002-dev`。不切分支、不 merge、不 push、不 reset/clean/stash。

## 1. 目标与范围

在一个新的、canonical 的隔离 HOME（建议 `/private/tmp/m2a-t2-iso`）启动真实 Tauri App，走生产 `update_work_item_state` 命令，拿到以下场景的 **ISOLATED-RUNTIME** 证据：

1. 冷启动：全新隔离 store 启动、建项目/草稿、合法状态推进，重启后数据仍可读。
2. commit 前强退：在真实命令尚未提交的确定性边界杀掉 App；重启后不得出现半笔 receipt/event/audit/snapshot 或错误的 work item 状态。
3. commit 后 receipt 丢失：以代码中已有或本包新增的确定性、隔离 profile 限定注入点制造该窗口；重启后的恢复行为、receipt 与业务行数必须经读库说明。
4. 投影失败：在真实 App 命令路径制造 JSON 投影失败，核对 fail-closed/降级行为与重启后的 DB/JSON 对账结果。
5. 重复 command：同一命令重放返回同一 receipt，业务表行数不增加。
6. JSON-leading：构造隔离 JSON 比 DB 新的情况，重启后必须按当前产品 fail-closed 行为处理；记录启动日志、读写可用性和两侧文件指纹，不得反向覆盖证据。

“强退”必须是对隔离 App 真实进程的 `SIGKILL` 或等价进程终止；“窗口”必须由可观察、可重复的暂停点界定。若当前代码没有这种暂停点，只能增加最小的 debug-only 控制，且仅当 `SYN_R4_ACCEPTANCE_PROFILE` 有效、debug build、受控 `/private/tmp` profile 时可达；普通 App 路径不得得到新的故障开关。

## 2. 先做勘察，后做任何改动

1. 读 `commands.rs` 的 `update_work_item_state` Tauri 入口及 T1-R2 记录的 bootstrap/create-draft console setup；确认本次使用的干净项目根、workflow、work item 与数据库路径。
2. 读 `workbench_sqlite_storage_mode.rs` 的启动对账、DB-leading、JSON-leading 和投影失败逻辑；列出每个场景所需的真实可观察边界。现有仅在 `#[cfg(test)]` 使用的 injected failure 不能直接当作运行时证据。
3. 先记录真实 HOME 工作台 store 的绝对路径与 mtime/sha256 指纹；后续结束前再做一次相同指纹。真实 HOME 必须零接触。
4. 用新的 `/private/tmp/m2a-t2-*` 根准备隔离 HOME、临时 override（仅 `{"app":{"withGlobalTauri":true}}`）和隔离 profile；`RUSTUP_HOME=/Users/yoyi/.rustup`、`CARGO_HOME=/Users/yoyi/.cargo` 只用于 toolchain，绝不读取 `.codex`。

启动命令的基本形态为：

```text
HOME=/private/tmp/m2a-t2-iso RUSTUP_HOME=/Users/yoyi/.rustup CARGO_HOME=/Users/yoyi/.cargo tauri dev --config /private/tmp/m2a-t2-tauri-override.json
```

若需 `SYN_R4_ACCEPTANCE_PROFILE`，profile manifest 必须通过现有校验，根目录为新的 canonical `/private/tmp/m2a-t2-*`，并在证据中记录其路径和有效期。不得改仓库 `tauri.conf.json` 的 `withGlobalTauri`。

## 3. 验收矩阵

| 场景 | 必须给出的实物 | 通过判据 |
|---|---|---|
| 冷启动 | App 启动日志、console 返回、JSON/SQLite 绝对路径与启动后指纹 | App 确实使用隔离路径；合法命令在 DB 与 JSON 可见 |
| commit 前强退 | 暂停点来源、App PID、终止时刻、重启日志、四类业务表前后计数 | 不存在半提交；重启可解释且不把未提交命令变成已提交 |
| receipt 丢失 | 注入点/隔离文件操作的源码或命令、重启日志、receipt/event/audit/work_item 查询结果 | 恢复语义由实物坐实；不能只删一行后口头称恢复 |
| 投影失败 | 注入点、命令错误、DB/JSON 指纹和重启对账日志 | 失败不被伪装成成功；当前 fail-closed/降级语义可读库复核 |
| 重复 command | 两次 console 返回、同 receipt_id、所有相关表行数差 | 第二次不新增 domain/event/audit/receipt/snapshot 行 |
| JSON-leading | 构造前后两侧文件 hash、启动日志、后续命令结果 | 产品按现有 JSON-leading 防线处理，DB 不被证据输入覆盖 |

每场景在独立、可重启的隔离根中执行，或在记录中说明前置状态、清理方式及其 hash。不得以时间竞速判断“刚好在 commit 前”；没有确定性窗口即记为 HOLD，不得声称本场景通过。

## 4. 允许的最小实现与禁止范围

- 默认任务是验收，先不改 Rust。若没有可重复的运行时故障边界，可在 `acceptance_runtime_profile`/Tauri 命令附近新增最小 debug-only、profile-gated 控制及其拒绝路径测试；禁止通用环境变量开关、普通 profile 可调用入口、生产超时/事务语义改动。
- 任何源码改动必须同时证明：普通构建不含或拒绝该控制、隔离 profile 才可到达、每个窗口仍由真实 App 进程执行。
- 不改 `m2_isolated_app_acceptance.rs` 的进程内 API 来伪装完成；它可以保留为 UNIT 辅助，但不得进入 ISOLATED-RUNTIME 结论。
- 不处理 T3/T4、schema 大迁移、grant store、code-map 或外部接入。

## 5. 证据、验证与交付

创建 `test-fixtures/m2a-acceptance/t2-isolated-crash-recovery-record-2026-08-04.md`，逐场景列：命令、PID/重启事实、App 日志绝对路径、JSON/SQLite 文件绝对路径、读库查询与结果、前后 hash/mtime、证据等级。没有完成的场景明确标 `HOLD` 与缺失的可重复边界，不得降级改叫通过。

若改 Rust：运行相关 focused tests、`cargo check --lib`、`cargo test --lib`，记录实际末行数字；若只新增证据文件，仍运行 `git diff --check` 与 `node scripts/harness-v2/project-context.js --target .`。启动 App、终止进程和临时目录动作都写进 evidence；真实 HOME 指纹前后相同是退场条件。

提交前列出精确文件，不用 `git add -A`；commit message 含 `catch:`，回写 `docs/harness/CURRENT.md`，并在交付后单独核 `git status --short`、`git log -1`、`HEAD^{tree}` 与 `git write-tree`。交付报告只能陈述已实际完成的场景；T2 未完整时不得宣称 M2 完成。

## 6. 退件条件

- 没有真实 App PID/启动日志/隔离 store 路径，或只给 Rust 单测。
- 写入或读取真实 HOME store、`.codex`、外部 provider。
- 用非确定性 sleep/重试证明 commit 窗口，或跳过无法做的场景却写“全部通过”。
- 证据数字、路径、文件 hash 不能由命令输出或读库复核。
