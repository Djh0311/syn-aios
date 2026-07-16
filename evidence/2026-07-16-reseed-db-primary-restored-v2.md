# 重 seed 恢复窗口证据:降级 json_only → DB 主写恢复 v2

日期:2026-07-16 22:30-22:46 · 用户在场 · 基线 commit `d5d08ad`(M5-B/C 全桥在) · 状态:**DB_PRIMARY 恢复·观察期重开重计(22:45 起)**

## 一、窗口时间线

| 时刻 | 步 | 结果 |
|---|---|---|
| 22:3x | 步0 | 用户 archive 真单+亲手关 App;总指导核:pgrep(App/dev/codex 三查)+lsof(DB+主 store 两查)=退出码全 1,零活进程零持有者 |
| 22:34 | 备份 | 整根仓外副本 **304M** `~/workbench-backups/workflow-state-backup-20260716-pre-reseed/`(workflow-state 全根+runtime-artifacts;DB 三件挪入 `stale-db/`);主 store hash 快照 `abdef3b8…` |
| 22:36 | 第一跳 | R3 env 授权入口占位 hash 被闸拦并吐真源根 hash `b47603a7…`(hash 闸正面开火,exit=101 留证 scratchpad) |
| 22:4x | 第二跳 | **Claude Code auto-mode 分类器拦「写生产 DB」那一下→用户亲手终端执行**(重档「授权那一下」实体化);`test result: ok. 1 passed`·report `completed`·export hash `c7ba116f…` |
| 22:44 | 对账 | 安全旗全绿(源零写/零 read_cut/零停写/`.codex` 零碰/before==after 源 hash);`source_records` 3815 vs 源计数和 3821=**差 6 与 07-14 v1 完全同型(3591/3597)=导入层口径常态**;业务四面亲数 DB=JSON 全等:audit 1590/work_items 58/workflows 8/nodes 66;sidecar 两日增量合理(+12 复核/+26 授权/+148 主 store) |
| 22:45:40 | 用户开 App | `storage_mode_initialized` 对账绿;**零新降级**(degraded 总数仍 4=全历史);启动事件同笔 DB=JSON=**1591**·lag=0 |

## 二、报告与回滚

- apply 报告/apply-backup/回滚清单:`evidence/raw/2026-07-16-reseed/`;第一跳/第二跳原始输出:scratchpad `reseed-jump1.log`(第二跳在用户终端,报告文件为权威)。
- 回滚网:删 `runtime-artifacts/storage-mode.v1.json` 即回纯 JSON;整根副本+stale 旧 DB 在备份区(07-14/07-16 两代并存)。

## 三、与降级事件的闭环

07-14 21:55 降级根因=M5 双桥合入**前**已写下的 JSON-only 记录(boundary review 等)无法回填(B 裁定无修,`193c373`);其后 fail-safe「每启再降」属安全语义。本次全量 seed 把含降级留痕在内的全部 JSON 记录种入 DB,起点一致→启动对账绿→循环断。观察期天数自 2026-07-16 22:45 重计,M6(停写 JSON)前置照旧=观察期攒够+用户另授权。
