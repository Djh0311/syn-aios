# M5-F1-R1 真实存储恢复 + S1B-H2 两句 App 验收 现场证据 v1

日期：2026-07-19 23:55 – 2026-07-20 01:11（+0800） · 用户在场
状态：**恢复闸全绿（reseed 完成、DB-primary 恢复、lag=0、零新增降级）；H2 落卡闸未达成——首句已入 canonical，主管回合因 Codex 额度上限未运行，按止损挂起，待额度恢复后复验**
任务包：`tasks/2026-07-19-m5-f1-r1-live-reseed-and-s1b-h2-real-app-verification-package-v1.md`
H2 实现包：`tasks/2026-07-19-s1b-h2-supervisor-syn-natural-information-flow-package-v1.md`
raw 证据：`evidence/raw/2026-07-19-m5-live-reseed-h2/`
start commit：`97fca19bc8d3effd4959dec8cc4827e27cac31e6` · end commit：同（未 commit、未 stage）

## 一、结论

1. 恢复目标全部达成：新仓外备份逐文件校验绿；用户亲手 R3-B1 production apply 一次完成；静态 17 业务面 DB=JSON；启动对账 `storage_mode_initialized` 同笔、lag=0、`storage_mode_degraded_json_only` 零新增（11）。真实存储从 07-17 旧 DB 现场恢复到 `revision 274 → 276` 的 DB-primary。
2. H2 验收未完整执行：第一句「我想给这个游戏里的标题改成小马里奥」已入 canonical（`supervisor_resident_user_message_recorded` 7→8），但主管 one-shot 因 **Codex 额度上限**未能运行（无 rollout/stderr/last-message 新产物，registry 全程空）。第二句未发送，未落卡。
3. 现场健康：DB↔JSON 在消息写入后仍 lag=0；无新增降级；chain 40、proposals 74、测试项目、generation/thread 全部不变。
4. 非产品失败：额度属外部条件。H2 复验入口 = 同一真实 App 直接续聊（canonical 已有首句），或按总指导另定。

## 二、Gate 0 基线（23:55–00:20，全部现场重取，未沿用包快照）

- 进程/端口/`lsof`/registry 全空：无 Workbench App、probe、Vite、cargo-tauri；DB 三件与关键 JSON 无持有者；registry entries=[]（rev 1119）。
- 既有无关进程未动：PID 14847（6/30 `__probe_run.mjs` 孤儿）、PID 98835（7/15 起的 node api，:3000）。
- HEAD `97fca19…`；staged 唯一项 = 开工前既有 mockup rename。
- 五关键源码 SHA-256 与包 §二表逐一相等（无 BLOCKED_DIRTY_OVERLAP）。
- 现场与包快照的漂移已如实记录：SHM mtime 07-19 21:37（出包后有人开过 DB 连接，用户在场确认继续）；registry rev 1074→1119。
- 冻结值全录：`raw/…/gate0-baseline.md`、`gate0-mario-test-hashes.txt`。

## 三、Gate 1 构建（00:24）

- `prototypes/productized-desktop-shell` 下 `../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug`，exit 0（`raw/…/gate1-build.log`）。
- 新产物：`src-tauri/target/debug/codex-governance-workbench`，SHA-256 `f9d028f3d9ebb942877506bc12b586d1d1cb60fac536a61fa4ba97b4b1db01f3`，66399400B，mtime 07-20 00:24:05（晚于 R1 源 18:35:51）。
- 含 R1/H2 论证：cargo 增量重链（旧 66379576B→新 66399400B）+ 五源 hash 构建前后不变 → 产物出自冻结源。
- 旧 `.app`（`bundle/macos/CodexGovernanceWorkbench.app`，内件 14:01）为项目 `bundle.active=false` 配置下未更新的残物，本轮禁用；live 启动 = 终端直接执行新裸二进制。
- 构建后进程/`lsof` 复查全空。

## 四、Gate 2 仓外备份（00:30–00:33）

- 唯一新目录：`/Users/yoyi/workbench-backups/workflow-state-backup-20260719-pre-reseed-003058/`（07-14/07-16 旧备份未触碰）。
- `snapshot/` = workflow-state 全根 + 外层 runtime-artifacts + production-db 全目录（含 storage-mode 配置）。
- manifest：`manifest.json`（`227330d9…`）/ `manifest.txt`（`b042c41f…`），**467 文件 / 405,017,547 字节，逐文件 SHA-256 对源全 PASS**。
- `ROLLBACK-NOTE.md` 写明回滚源与「尚未执行 apply」。
- `stale-db/` 归档：三件逐个解析绝对路径、目标预确认不存在后移入；移后 hash 与 Gate 0 冻结相等；`production-db/` 清空。
- 全录：`raw/…/gate2-backup-record.md`。

## 五、Gate 3 apply 命令（00:39，先回传后执行）

- 源 hash 现场重算 = 07-16 第一跳先例：占位探针触发既有 hash 闸吐出真值 `c3038dc407fa9decf1323fed21909b6a72beb50f13aaf3dd30524c31326540f2`（exit=101 预期闸火，零写入；`raw/…/gate3-hash-probe.log`）。
- 回传总指导：完整命令、cwd、五个 confirmed path、源 hash、备份 manifest hash、生产 DB 不存在证明（`raw/…/gate3-apply-command-record.md`）。

## 六、Gate 4 用户亲手 apply（00:50，一次完成）

- 前两次尝试在 `~` 未带 `cd`，cargo 未找到 manifest 即退出，**不构成 apply**（每次均当场复核零写入）；第三次单行命令成功：`1 passed; 0 failed`，15.76s。
- report（`raw/…/production-apply-report.json`，`a3949363…`）：`status=completed`、level B、`failure_point=null`。
- 安全旗九项全绿：`production_apply_performed=true`、`production_db_created=true`、`source_json_written=false`、`production_root_written=false`、`codex_home_touched=false`、`read_cut_enabled=false`、`stop_write_json=false`、`production_restore_performed=false`、`product_read_path_changed=false`。
- `before_source_hashes == after_source_hashes`（11 源文件）；source_root_hash 与 Gate 3 一致。
- `export_verification.status=verified`；rollback boundary dry-run only。
- source_records 4142 vs 源计数和 4148（差 6，与 07-16 同型 = 导入层口径常态）。
- 新 DB：`e302a0fb…`（32,157,696B，00:49:58，apply 时刻快照）。
- 产物与执行事实全录：`raw/…/gate4-apply-execution-record.md`。

## 七、Gate 5 对账（静态 00:50 / 启动 01:02–01:06）

### 静态（App 关闭，sqlite3 -readonly）

17 业务面 DB=JSON 全等：workflows 8、nodes 66、edges 50、work_items 58、audit 1771、chain 40、attempts 164、dispatches 404、artifacts 27、projects 5、reviews 11、permission_requests 1、execution_controls 164、session_bindings 76、proposals 74（Pending 17/confirmed 56/rejected 1）、plan_authorizations 56、orchestrator sessions 25。

### 启动（用户开 Gate 1 新二进制）

- 新增 `storage_mode_initialized`（1784480531919）在 DB/JSON **同笔同 event_id**；initialized 25→26。
- audit：JSON 1772 = DB 1772，**lag=0**。
- `storage_mode_degraded_json_only` 仍 11，**零新增**。
- 源 JSON 经 projection 正常演进（revision 274→275），非 apply 写入。

## 八、Gate 6 H2：首句入 canonical，主管因额度未回（止损点）

- 01:06:51 用户发送第一句「我想给这个游戏里的标题改成小马里奥」：
  - canonical 已记录：`supervisor_resident_user_message_recorded` 7→8，event `supervisor-resident-message:user:1784480811921…`，client_request_id `4c711575-617b-4225-aef0-16994543dd81`。
  - 写入后 DB/JSON 仍 1773=1773，lag=0。
- 主管回合未运行：Codex 额度上限（用户现场确认）。runtime-artifacts 无新 rollout/stderr/last-message；resident home 停留 07-19 11:09；registry 全程 entries=[]。
- 主管答复未落（recorded 仍 3）；第二句未发；proposal/chain/项目不变；无点卡、无 chain、无 worker。
- H2 四项首句证明中「canonical 记录」成立；「主管答复/回合完成/同 thread 续接」未发生，属外部额度阻断而非产品失败。

## 九、Gate 7 收尾（App 关闭后补全最终冻结）

- 待用户关闭 App 后：registry/`ps` 无本轮孤儿复核、最终 DB/JSON/proposal/chain/generation/项目冻结值，见 `raw/…/gate7-final-freeze.md`。
- 保留恢复材料：仓外备份 `workflow-state-backup-20260719-pre-reseed-003058/`（snapshot+stale-db+manifest+ROLLBACK-NOTE）、apply 三件套（report/rollback-manifest/apply-backup）。
- 零新增 catch（本轮无 harness 拦截事项；catch-log 不追加）。

## 十、变更面声明

- 零源码改动、零新增 command/sidecar、未 stage、未 commit；start=end=`97fca19bc8d3effd4959dec8cc4827e27cac31e6`。
- 改动文件仅限本包允许面：仓外备份目录、`evidence/raw/2026-07-19-m5-live-reseed-h2/**`、本 evidence、`CURRENT.md` 最小回写；构建产物 `dist/`、`target/` 单独列出。
- 真实写面：新生产 DB 一件、经既有产品路径的 storage_mode_initialized 一笔与首句 canonical 一笔。
