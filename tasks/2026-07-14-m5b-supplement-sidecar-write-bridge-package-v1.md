# 任务包:M5-B 补遗——sidecar 写函数漏点接桥+全仓再扫 v1

日期:2026-07-14 · 档位:**轻档**(沿用 M5-B 已证范式) · 背景:真机首巡降级 json_only(启动对账抓 JSON 领先 2 条,降级网首次真实开火·行为正确)。追根=M5-B 勘察反向 grep 只盯 `write_validated_workflow_state`,漏掉 sidecar 自有写函数两族。**本包收口是「重 seed 恢复 db_primary」的硬前置——顺序反了,批准流/真派发一开火就再降级。**

## 已核实的漏点(总指导亲查,坐标为当前 HEAD `6ea534a`)

**A·`plan_authorization_store.rs::inspect_auto_dispatch_authorization`(:641,写点 :677 `write_store_atomic`)**——正是本次降级肇事写点(`auto_dispatch_scope_checked` 审计落 sidecar)。同文件 create(:162)/update(:1075) 家族已 BRIDGED,接法照抄现成范式(mode-off 原样;mode-on DB delta+审计同笔 Immediate→JSON 投影)。

**B·`mcp/supervisor_orchestrator.rs::update_store`(:1332,写点 :1352)**——13+ 生产调用点全汇此函数(persist-worker-report/final-mark/pilot 四件/reserve-complete-fail-dispatch/follow-up 三件/update-worker-follow-up-result 等)。**单点接线一刀收**(在 update_store 本体加闸,勿逐调用点改)。repository 缺 orchestrator store 的 mutation API 则新增(表已存在=seed 14 表含 sup_sessions,零 DDL;Immediate 事务+审计同笔)。

**C·全仓 sidecar 写函数再扫(勘察方法修正)**:grep 所有 sidecar 自有原子写(`write_store_atomic` 各文件变体/`update_store` 变体/`workflow_state_store::atomic_write` 直调/其它 `fs::write` 落 sidecar JSON 的点),逐点定 BRIDGED/DIRECT。产出**全量清单**(file:line+分类)入回传。DIRECT 且数据在 DB 有表的→接桥;DB 无表的(如 exec-process-registry 运行时件)→列豁免名单+理由,**不擅自加表**;memory 系列(formal-memories/candidates/observations/capture-events/lint)若 DIRECT→**列缺口报回,总指导定夺,本包不接**(L1 通血刚动过该区,防交叉)。

**D·reconcile 面**:supervisor_orchestrator 数据 seed 时已入 DB(sup_sessions 表)但不在 20 表对账面→接线后**必须**把对应表纳入 reconcile/投影/回放(否则 DB 侧静默 stale 到 M6 才暴雷);`DbProjectionData`/`load_db_projection_data`/`replay_workflow_state_projection`/`compare_record_freshness` 四件套同扩;**m5a/m5b fixture 同步扩**(不同步=全族假红,前科两次)。

## 红线

flag 默认关分支原路径一字不动;迁移面 importer/apply/schema/exporter/preflight 零碰(reconcile 扩表在 storage_mode/repository 侧做,不动 schema=表已存在);安全闸/谓词/stable_id 零碰;read_cut 零加行;live 根/真实 DB 零写(测试全 temp);**不动 L1 记忆区代码**;不 commit;automation 水线 5059 不破;新增行若逼近既有文件 3000 限,兄弟函数落既有 include 惯例。

## 验收(预写死)

- A/B 两族接桥+案发测试(mode-on 走桥写 DB+审计、mode-off 原样;update_store 至少覆盖 persist-worker-report/reserve-dispatch 两个代表调用点);
- C 清单全量入回传(BRIDGED/DIRECT/豁免/报回四类,file:line 齐);
- D:reconcile 扩表后 `m5b_`/`m5a_` 全族绿;temp 根端到端:db_primary 下走一次「批准→inspect_auto_dispatch→重启对账」**绿**(案发:今天的降级路径不再触发);
- 全量基线只增不减(当前 932/45);fmt 仅历史三;shape gate 14/5/5 零净增(仓根跑);
- 回传 10 项模板,第 7 项 gate 三数必含(缺=机械打回)。

## 收口后(包外·总指导提请)

重 seed 恢复 db_primary=用户在场窗口(动真库,同 M5 切换惯例):删旧 DB→重 seed→storage-mode 配置回 db_primary→重启对账绿。
