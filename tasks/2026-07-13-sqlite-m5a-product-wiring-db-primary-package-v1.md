# 任务包:M5-A 产品接线——DB 主写模式开关(默认关·休眠可翻)v1

日期:2026-07-13 · 基线 commit `b549b05` · 基线测试 909/45(+1 计时 flaky 家族)。
授权:用户「现在就做」=M5 明确授权;**本包合法触碰此前红线的活写文件**(它们即接线靶面),但**flag 默认关=产品行为零变化**;真实翻闸仍在包外窗口(用户在场)。
上承:M4 休眠 repository(六流行级事务·已测)、preflight v2(真实根 preflight_ready)、评审 §八 M5 原文「先 DB 主写并保留 lag=0 JSON 投影,再做有限 read-cut」。

## 一句话

给产品装上"存储模式开关":默认 `json_only`(字节级原行为);`db_primary_json_projection` 模式下六个代表写路径先走 M4 repository 行级事务落 DB,**同笔成功后**把 JSON 投影经既有 `write_validated`/sidecar 原子写路径刷新(lag=0)——回滚=关 flag,JSON 始终最新。

## 交付物

### W1 模式开关(fail-closed)
- 配置=显式文件(照 Level-B config 姿势:confirmed DB 路径逐字段等值+canonicalize),落 `runtime-artifacts/storage-mode.v1.json`;**文件缺失/解析失败/路径不符=一律 json_only**;模式读取一次成缓存,变更需重启(不做热切,少一类竞态)。
- 状态上报:模式+DB 路径 hash 进启动日志与一条 audit(经新 helper),不进 UI。

### W2 repository confirmed-path 打开
- 现 `open_rehearsal` 只认 temp;新增 `open_confirmed(config)`——路径闸照 production_apply Level-B(逐字段等值+canonicalize+禁词标记),WAL/busy/Immediate 全继承 M4。

### W3 六流接线(靶面=M4 勘察 §4 的六个 JSON 写入口)
| 流 | 接线点 | 
|---|---|
| proposal | project_consultation_proposal_store.rs `create_proposal` |
| authorization | plan_authorization_store.rs `create_authorization`(强制 CAS 语义对齐 M4) |
| dispatch 预留/记录 | workflow_execution_entrypoints.rs `write_prepared_dispatch`/`write_started_dispatch` |
| 主管动作 | supervisor_action_controller.rs `reserve_action`/`complete_action` |
| work_item 转移 | workflow_run_dispatch_entrypoints.rs `update_work_item_state_at` |
| audit 追加 | 各流内联 push 处经统一入口 |

接线形状(每处相同):模式关→原代码路径**一字不动**;模式开→①repository Immediate 事务落行(失败=整个操作失败,**不半写**)→②JSON 投影按**原路径原语义**刷新(锁/CAS/备份全走既有函数)→③两步间崩溃的恢复口径=启动时 DB↔JSON revision 对账,DB 领先则重放投影(记 audit),JSON 领先(不该发生)=停写报错 fail-closed。

### W4 对账器
- `reconcile_db_vs_json(config) -> report`:逐表计数+natural key+record hash vs JSON 数组;供窗口用+观动期巡检;只读。

### W5 案发测试(全 temp,零真实根)
- 模式关:六流行为与基线**字节级一致**(现有测试即证+新增 flag-off 显式断言);
- 模式开(temp DB+temp JSON 根):六流各落行+投影一致+对账器全绿;②流 CAS 冲突显式拒;①→②间注入崩溃→重启对账重放恢复;
- 配置文件缺失/坏/路径不符→json_only(fail-closed 三连测)。

## 红线

1. **flag 默认关,缺省行为零变化**——`cargo test --lib` 基线只增不减是硬验收;
2. 触碰活写文件仅限 W3 表列点位,改动形状=「模式分支包裹」,原 JSON 路径逻辑一字不动(diff 里原行为必须整段可辨认未改);
3. 不碰:安全闸/敏感谓词/preflight 判定/stable_id/棘轮(commands.rs 若必须挂模式读取,零逻辑只传参,回传列明);read_cut.rs 零加行;
4. 不新增 tauri command/sidecar 种类(storage-mode.v1.json 落 runtime-artifacts=运行时件非 sidecar,preflight v2 目录已豁免——回传里仍要点名说明);
5. 真实根/真实 DB 零写(全 temp);**不执行任何真实翻闸**;
6. 卡住/发现两步一致性有本包方案盖不住的窗口→停下报,不擅自扩。

## 验收(预写死)

模式关字节级等价+全量基线只增不减;模式开六流落行+投影 lag=0+对账全绿+崩溃恢复;fail-closed 三连;`git diff --check`/fmt 仅历史三漂移/shape gate 零净增;真实根 hash 前后一致。回传 10 项+每个接线点的 diff 形状说明。

## 总指导回收动作

核实物(六流 diff 逐点读"原路径未改"+全量亲跑+temp 开模式亲验+对账器亲跑)→ commit 问一次 → **切换窗口**(用户在场):关 App→终快照→`production_apply` 真库 seed 导入→逐表对账→写 storage-mode 配置→重启目检→观察期开始。
