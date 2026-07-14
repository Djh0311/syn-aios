# 重 seed 恢复窗口证据:降级 json_only → DB 主写恢复 v1

日期:2026-07-14 16:15-16:28 · 用户在场 · 基线 commit `ad79381`(M5-B 补遗已收口) · 状态:**DB_PRIMARY 恢复·观察期重开**

## 一、窗口时间线

| 时刻 | 步 | 结果 |
|---|---|---|
| ~16:10 | 步0 | 用户亲手关 App+dev 进程;总指导核:pgrep 两查=1(零活进程)·lsof=1(零 DB 持有者)——退出码经上次「管道尾 head」教训后按真实码核 |
| 16:15 | 备份 | 整根仓外副本 **215 文件** `~/workbench-backups/workflow-state-backup-20260714-pre-reseed/`(workflow-state 全根+runtime-artifacts+stale 旧 DB 三件挪入 `stale-db/`);主 store hash 快照 `399cc4f7…` |
| 16:2x | 两跳 seed | R3 env 授权入口(`R3_B1_APPLY_CONFIRM`+期望源 hash);第一跳占位 hash 被闸拦并吐真 hash `595255ef…`(hash 闸正面开火);第二跳 **completed**·export hash `55abbc1b…` |
| 16:2x | 逐表对账 | 主 store 七面 1476/363/57/75/148/148/37 全相等;授权 47+审计 481(**含当日两条肇事审计,本次入 DB**);方案 64+112;编排会话 23+189(补遗新接两表首次入账);安全旗:源零写/零 read_cut/零停写/`.codex` 零碰 |
| 16:27 | 用户开 App | dev 重编译后启动;**两条新 `storage_mode_initialized`**(16:27:54/16:28:17·dev 启动序跑两遍·均对账绿·无害);DB=JSON=**1478** 同步·lag=0;降级审计仍仅 13:55 历史一条 |

## 二、报告与回滚

- apply 报告/备份 manifest/回滚清单:`evidence/raw/2026-07-14-reseed/`;
- 回滚网:删 `runtime-artifacts/storage-mode.v1.json` 即回纯 JSON;整根副本+stale 旧 DB 均在备份区。

## 三、与降级事件的闭环

13:55 降级(`storage_mode_degraded_json_only`)根因=M5-B 前的未接线写(`auto_dispatch_scope_checked`×2)造成 JSON 领先——肇事写点已在补遗包接桥(`ad79381`),两条肇事审计本次随全量 seed 入 DB,起点一致。**同型事故不再可能由该写点触发**;剩余 4 个 DIRECT+DB 族不在对账聚合源(亲核分表制),不构成降级面,M5-C 扩包挂账(M6 硬前置)。
