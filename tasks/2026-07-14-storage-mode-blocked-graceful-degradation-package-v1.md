# 任务包:storage-mode Blocked 态优雅降级(小包·拆雷)v1

日期:2026-07-14 · 档位:**轻档**(单文件语义修+测试)· 基线 commit `9cb5d13`(或当前 HEAD)。
背景:M5-A 现状=Blocked 态(启动对账 JSON 领先/分歧,或运行中投影失败)下 `primary_repository_for_write` 返回 **Err → 六流产品操作直接报错**(storage_mode.rs:119-122)。而接线只覆盖六流,六流外任何写(派发完成/回读、链、绑定…)只进 JSON → **正常跑完一单任务+重启=必踩**:核心功能突然全坏。数据零损失但体验=砖。

## 目标(语义拍死)

**Blocked ⇒ 写路径行为=json_only 降级,不再报错**:
1. `primary_repository_for_write` 遇 Blocked → **`Ok(None)`**(六流走原 JSON 分支)+ 首次降级时 `eprintln` 一次+经 JSON 路径落一条降级审计(event_type=`storage_mode_degraded_json_only`,带 Blocked 原因;每进程只落一次,防刷屏);
2. health 仍保持 Blocked(**不自动痊愈**)——DB 从此冻结,恢复=将来重新 seed 的窗口(与"删配置回滚"同级语义,但不用删配置);
3. 启动日志已有的 Blocked eprintln 保留,措辞补"已降级 json_only,数据无损,需重 seed 恢复 DB 主写"。

## 允许写入

`workbench_sqlite_storage_mode.rs`(语义+其 m5a 测试更新:`m5a_projection_failure_blocks_further_writes_until_restart`、`m5a_db_ahead_replays_on_restart_and_json_ahead_blocks_writes` 等按新语义改写——**"阻断 DB 主写"断言保留,"产品操作报错"断言改为"降级走 JSON 且成功"**)+ 新增降级案发测试(Blocked 后六流照常成功+降级审计恰一条)。

## 红线

六流文件零碰(降级点在 storage_mode 一处,天然不用动它们);迁移面/安全闸/preflight 零碰;live 根零碰;不 commit;回传 10 项第 7 项 shape gate 必报(M-2026-07-13 在案)。

## 验收

Blocked 模拟下:六流全部成功走 JSON+降级审计恰 1 条+DB 零新写;非 Blocked 行为零变化;全量基线只增不减;fmt 仅历史三;真实根 hash 前后一致。
