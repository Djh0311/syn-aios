# 任务包:M5-B 全面接线——主 store 54 写点+sidecar 9 后续态全入 DB 主写(两批)v1

日期:2026-07-14 · 档位:**轻档**(接线沿用 M5-A 已证范式·flag 语义不变·不翻新闸)· **硬前置:降级补丁包先收口**(Blocked=优雅降级,批间部分接线才无砖化风险)。勘察正本:2026-07-14 M5-B 写路径勘察(62 生产写点=六流 8+闸外 54;形状五类;file:line 全录,执行线照勘察坐标干,别重数)。

## 目标

db_primary 模式下**所有**主 store 与六流 sidecar 写路径=DB 先写行级事务→JSON 投影 lag=0;M6 的硬前置就此闭合。

## 范围(三块,缺一不可)

**A·六流 sidecar 的 9 个后续态写**(最近爆炸半径·直接命中已对账表):proposal `record_decision`(:291)、plan_authorization `record_user_confirmation`(:275)/`record_global_boundary_review`(:355/:444)/`revoke_authorization`(:547)、supervisor_action `record_rejected_action`/`record_guard_rejection`/`prior_or_recover_result`/`record_supervisor_transport_failure`/`record_supervisor_system_result`(:502-:1276 各区)。

**B·主 store 闸外 54 写点,两批**(全按勘察 §1 清单逐点接,批次判据=真实使用热度,勘察 §6):
- **批 1(热路)**:completed/failed/readback dispatch(wee:829/:988/:1041)、commands:2776+submit_draft:3329、record_worker_structured_report(c4_c6:471)、bind/unbind/migrate(wrde:857/:1155/:1071)、director 链全家(24 写点·含 run_director_task_chain_inner 13 写)、chain controller 5 写+stop、reviews 家族(wee:1141+c4_c6:622/:739/:831)、task draft/fields/package 三件(lifecycle:442/:658/:832)、c4_c6:333(**其内 ~:240 直推 prepared dispatch 的绕闸点必须改走 repository 同一 insert,消"第二入口"**)、automation 6 写。
- **批 2(低频)**:initialize/bootstrap(lifecycle:76/:170)、offline 三件(wee:1383/:1504/:1603)、store_hygiene:277、operation_control:374、supervisor pilot 回填(launcher:793)、abandon binding(mcp:691)、record_permission_decision(wee:1228)、session birth/role loop audit(director:3486/:3763)。

**C·对账面同步扩**(随批 2 收口一起落,防半扩假红):reconcile 7→全写面表(+bindings/chain_runs/workflows/edges/artifacts/reviews/execution_attempts);`DbProjectionData`/`load_db_projection_data`/`replay_workflow_state_projection`/`compare_record_freshness` 四件套同扩;**m5a 七连测的 `bootstrap_json_state`/`seed_db_from_json` fixture 必须同步扩**(勘察 §4:不扩=全族假红);freshness 缺序字段的记录(edges/部分 artifacts)给确定性排序键或明确落 hash_mismatch 的处置策略。

## 实现要求(沿用+新增)

1. 范式照 M5-A:原函数开头闸+`*_db_primary` 兄弟函数;**director/commands 已超 3000 存量线,其兄弟函数落新 include 文件**(照 lib.rs include! 惯例,如 `workflow_db_primary_wiring.rs`,<3000),其余文件就地;
2. repository 新增 mutation API(缺口=勘察 §3:dispatch **update**、execution_attempts insert、work_items create/批量翻转、bindings insert/lifecycle/迁移、chain_runs upsert+节点态+finalize+stop——chain helpers 恰好集中在 controller:121/:186/:223/:240 四个函数,包一层即可、workflows/nodes/edges 定义级 upsert、artifacts insert+单字段、reviews insert)——全部 Immediate 事务+审计同笔,表全存在零 DDL;
3. **3 个无 audit 裸字段写**(director:3444/:3624、launcher:793)接线时补配套 audit 行(对齐"每写必审计");
4. 吞错写点(director:2056 `.is_ok()`、:3281 闭包)保留吞错语义但内部保序 DB→投影,回传注明;
5. 循环多写(链 5 写/13 写)每写各自成笔;条件写/幂等早退语义原样;
6. 启动序:migrate(:586)在对账前——batch2 接它时确认"真迁移发生→对账前 DB 也已同笔"或明确让降级兜(回传写清选了哪个);
7. worker_report 链两笔(completed+structured_report)同批接,防半炸。

## 红线

flag 默认关分支=原路径一字不动(逐点 diff 可辨);迁移面 importer/apply/schema/exporter/preflight 零碰;安全闸/谓词/stable_id 零碰;read_cut 零加行;live 根/真实 DB 零写(测试全 temp);不 commit;回传 10 项第 7 项 shape gate 必报。

## 验收(预写死)

- 62+9 写点逐点清单核销(file:line 对勘察表,一点不许漏);grep 证明生产 `write_validated` 无未闸裸点(除 replay/startup 两个反向点);
- temp 根端到端演练:建项目→方案→决定→授权→确认→派发→完成→回读→structured report→重启对账**绿**;二轮含链路径同绿;
- m5a 全族+新增案发测试绿;全量基线只增不减(当前 965 总·920/45 口径);fmt 仅历史三;gate 14 零净增(新 include 文件 <3000);
- 分批交付:**批 1+A 先回传一次**(总指导核后再进批 2+C)——M-2026-07-11 先核步规矩,包内即验收第一条。

## 回传

10 项模板×2 次(批 1+A / 批 2+C);每次附写点核销清单。
