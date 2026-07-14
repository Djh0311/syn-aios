# 任务包:M5-C——剩余 4 族 sidecar 接桥+对账面扩全(M6 硬前置)v1

日期:2026-07-14 · 档位:**轻档**(总指导已对 real_execution_command 做安全审查,见下;判决体零碰) · 上游:M5-B 补遗 C 清单核定(`ad79381` 收口·CURRENT §一) · 基线:HEAD 含复核实证闸(全量 959/45)。

## 目标

补遗核定的 4 个 DIRECT+DB 写族全部接 DB-primary 桥;reconcile/投影/回放扩到含其表;M6「停写 JSON」的最后一块硬前置就此闭合。

## 范围(16 写点·4 族)

**A·`global_supervisor_review_store.rs`**(:284/:345,2 点):照 M5-A/B 现成范式(mode-off 原样;mode-on DB delta+审计同笔 Immediate→JSON 投影)。
**B·`session_continuation_store.rs`**(:169/:347/:549/:868/:1187/:1503/:1645,7 点):同范式;若有集中原子写函数可单点收口(照 update_store 先例),没有则逐点。
**C·`runtime_log_store.rs`**(:146,1 点):同范式。
**D·`real_execution_command.rs`**(:993/:1222/:1481/:1580/:1948/:2285,6 点):**总指导安全审查结论**——6 点全为事后记账写(prepare/decision/phase A/phase B×2 的 sidecar 落盘),判决体 `decide_real_execution_command`(:185)/boundary_spec/blocked_message/k2 authorization 构造/denied_paths/baseline_hash **全部零碰**;仅在 `write_real_execution_product_command_store_atomic` 调用点包桥。测试必须走 `_with_runner` 注入形态+mock runner+temp,零真跑。
**E·对账面扩全**:reconcile/`DbProjectionData`/`load_db_projection_data`/`replay_workflow_state_projection`/`compare_record_freshness` 扩至 4 族全部表(supervisor_reviews 系/session_continuation 3 张/runtime_log 2 张/product_command 4 张;表全已存在零 DDL);**m5a/m5b 全族 fixture 同步扩**(不同步=假红,前科三次);repository 缺的 mutation API 照 Immediate+审计同笔补。

## 红线

判决体/安全闸/沙箱/解封面(station3b/4/S1/path-lock)零碰;复核实证闸(`supervisor_review_evidence`)零碰;迁移面 importer/apply/schema/exporter/preflight 零碰;L1 记忆区零碰(memory 系 9 写点=报回清单,**本包不接**);read_cut 零加行;live 根/真实 DB 零写(测试全 temp+mock);不 commit;automation 5059 不破;新增文件 <3000(超限拆模块,shape gate 上包已拦过一次);fmt 仅历史三。

## 验收(预写死)

- 16 写点逐点核销清单(file:line)入回传;反向 grep:4 族原子写函数生产直调点清零(残点仅 mode-off 分支/投影闭包/replay/测试区);
- 案发测试:每族至少一条 mode-on(DB 先写+审计同笔+投影)+mode-off(原样)双分支;D 族含 mock runner 的 phase 记账桥测试;
- reconcile 扩表后 m5a/m5b 全族绿;temp 端到端:db_primary 下「prepare→decision→phase 记账→重启对账」绿;
- 全量基线只增不减(959/45 起);gate 14/5/5 零净增(仓根);
- 回传 10 项模板,第 7 项 gate 三数必含(缺=机械打回)。

## 收口后(包外)

观察期继续巡检;4 族接完+巡检攒天数全绿 → 总指导提请 M6(用户授权窗口:停写 JSON+SQLite 备份纪律+正文外置判废+production-db 异地备份)。
