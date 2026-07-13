# M5 切换窗口证据:JSON→SQLite DB 主写上线 v1

日期:2026-07-14 00:39-00:51 · 用户全程在场 · 基线 commit `3066151` · 状态:**DB_PRIMARY_LIVE·观察期开始**

## 一、窗口时间线

| 时刻 | 步 | 结果 |
|---|---|---|
| 00:39 | 步0 首核 | **pgrep 抓到 `cargo-tauri dev`(PID 2691)仍活**——用户以为已关;lsof 瞬时 0 但进程可随时写 → 按协议**停** |
| — | 越权被拦 | 总指导试图代杀该进程,被 auto-mode 权限分类器拦下(拦得对:协议明写「有即停」非「有即杀」;进程非本会话所创)→ 交还用户亲手 Ctrl+C |
| 00:46 | 步0 复核 | pgrep 无匹配·store 零持有者·**主 store hash 仍=`03f8bebe`(a 修复后零写入,最干净切换条件)** |
| 00:46 | 备份+终核 | 整根仓外副本 201 files(`workflow-state-backup-20260714-pre-cutover`);dup_groups 复扫=**0**·无号=0;种子冻结 /tmp 201 files |
| 00:47 | seed 导入 | R3 Level-B 真跑入口(env 参数化·两跳取源根聚合 hash `b16c8d01…`);`workbench-state.v1.sqlite` **26.3MB·completed**·安全旗全对(源前后 hash 相等·无越界旗);报告/回滚清单落 `evidence/raw/2026-07-14-m5-cutover/` |
| 00:47 | 逐表对账 | **14/14 全绿**(audit 1474/派发 363/工单 57/绑定 75/attempts 148/controls 148/chain_runs 37/projects 5/workflows 8/sup_actions 30/sup_sessions 23/sup_reviews 10/authorizations 47/proposals 62) |
| 00:48 | 写模式配置 | `runtime-artifacts/storage-mode.v1.json`=`db_primary_json_projection`·等值对·JSON 合法 |
| 00:51 | 用户重启 App | 新二进制(预热·含全部 commit);启动对账绿→放行 |
| 00:51+ | **切后核验** | 见下节 |

## 二、DB 主写 live 的数据侧证据(总指导实测·非自报)

- **史上第一条 DB 主写**:最新审计 `event_type=storage_mode_initialized`,id=`audit:storage-mode-startup:<完整slug>:<sha12>:<ts>`——**新 id 格式(止血 helper)在真实写路径首次开火**;
- **lag=0 成立**:该写 DB 先落、JSON 投影跟上——两边同步 1474→**1475**;抽查 5 表 DB=JSON 逐一相等;
- 主 store hash `03f8bebe`→`55d8c5ab`(投影刷新,预期行为);DB mtime=00:51:13。

## 三、回滚网(观察期全程有效)

删除 `runtime-artifacts/storage-mode.v1.json` → 重启即 fail-closed 回纯 JSON(投影使 JSON 始终最新,零数据损失);另有整根仓外副本+apply 备份/回滚清单(`evidence/raw/2026-07-14-m5-cutover/`)。

## 四、观察期与 M6 门

- 观察期巡检:reconcile 复扫(DB↔JSON 逐表)全绿+App 使用正常;投影失败会阻断本进程后续 DB 主写(设计如此,遇到=看启动对账输出);
- **M6 停写 JSON=锁着**:观察期无漂移后,用户单独授权那一下才收安全网。

## 五、蓝图对账

`docs/workbench-system-architecture-v1.md:106-107`:「v0 事实层:JSON 文件」「长期事实库方向:SQLite+FTS」——本窗口兑现主写切换;FTS/检索仍属后续(不在本窗口声称范围)。
