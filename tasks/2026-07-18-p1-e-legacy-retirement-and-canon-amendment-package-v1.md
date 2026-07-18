# 任务包:P1-E 旧路退役+修宪(诚实关门案)v1

日期:2026-07-18 · 档位:**轻档·前后端+文档**(高危 5 条零碰) · 执行者:执行线 · 上位:总执行计划 §一 P1-E(防跑偏总则先读) · 前置 P1-A..D 已收(`0969c95`) · 勘察笔记(先读透):`/private/tmp/claude-501/-Users-yoyi-workspace-product-line/cb32c0e7-68c5-4963-b125-c9fd5640ab66/scratchpad/p1e-dataflow-recon.md` · 用户拍板:**a=诚实关门**(07-18;唯一硬条件=**对话功能零影响**——固定测试项目全链照旧)。

## A·干什么

1. **A1 非测试项目诚实关门**:两个入口(交办中栏常驻框+工作流页「说目标」表单)对**非固定测试项目**=禁发+人话说明(文案基调:「这个项目还没接执行——当前版本先伺候固定测试项目,开放真实项目是后面阶段的事」;一句话,不加横幅不加新牌,复用常驻框 disabled 路由形态);**固定测试项目零变化**。判据用既有 path-lock 常量,禁新造判定。
2. **A2 塞纸条路退役**:①关门后三处路由(consultant_agent.rs:1036/director_agent.rs:469/488)的非测试 fallback 分支删除,固定测试项目全走常驻路;②**`readonly_codex_consult` 本体+共享器官(`parse_consultation_proposal`/`consultant_build_prompt`)保留零碰**(全局主管两钩/秘书/终标总结仍消费,勘察 §1 判定);③pilot 问人死胡同退役(`RequestUserDecision` 动作路径;**`waiting_user` 状态词承载四种活语义[格式错误保守停/worker blocked/崩溃恢复/report_invalid],只退问人动作本身,禁 grep 连坐**);④真死码清扫:`prepare_workflow_node_dispatch` 死接口对、`confirmProjectDirectorTaskSessionBindings` 前端 wrapper(tauri.ts:961)、`map_consultation_to_c1_input`、21 零调用 wrapper 清单中**属本包领地的**(勘察标了归属,他包领地零碰)。
3. **A3 修宪落 decisions**:按勘察 §2 清单出修订文件(新 decision:P1 对话化修宪 v1)——交办冻结令作废归档注记(实体在 craft-sweep 包,07-16 方向决策已作废)/交互宪法 §四.2 布局固化整段重写(依据=07-17/18 用户实物拍板原话)/§五 拆巨石方向作废注记/「变形不搬家」改写/「开发者详情废除」与 P1-C「工程详情」边界澄清;**DESIGN.md 同句拷贝同 commit 改**(防两皮,勘察点名)。
4. **A4 测试面翻案**:必翻=pilot 问人 4 处断言(含 controller:2928 整测删除);**禁翻**=pilot-switch 两段 `waiting_user` 夹具(格式错误保守停的脸)+conversation-center:486 P3-C 占位守卫;死码测试删除按**预登记制**:回传里先列「删测名单+每测原因+新基线数」,cargo 基线 988/0/47 允许按名单减,名单外零减(核复将按名单逐一对)。

## B·红线(违者停手报回)

1. **对话功能零影响**(用户唯一硬条件):固定测试项目的常驻会话/问答/常驻框三通道/方案·交货右区/P1-D 零停点直通——全链 diff 零碰;非测试项目仅入口禁发,不动对话组件本体。
2. `final_mark` 复核实证闸与待删 plan 方法同 impl 区**三行之遥**(勘察 §4)——diff 必须绕开,扫到即打回;S1/guard/path-lock/写域/高危 5 条零碰。
3. `readonly_codex_consult` 本体/共享器官/全局主管两钩/秘书路零碰;launcher 现 2999 行贴 3000 线,只许净删不许净增。
4. serde:勘察已证删枚举变体不炸旧数据(kind 落盘为 String),但任何持久化面改动仍双件齐+m5b/m5c 定向亲跑;闸对应最终 diff。
5. 关门文案=感受件:一句人话,真机走查用户过目,不满意随口改,不为它开设计回合。

## C·交付

1. 代码+翻案测试+死码删除;退役对照表(删了什么/保了什么/为什么);
2. 修宪 decision 文件+DESIGN 同步 diff;
3. 10 项回传模板:cargo 全量(按预登记新基线)+m5b/m5c 定向/typecheck 0/离线全绿(断言随形态)/gate 仓根 13/5/5 零净增/git status 前后;真机走查点清单(非测试项目关门脸+测试项目对话零变化,两点即可,不烧额度——关门脸不用模型)。
