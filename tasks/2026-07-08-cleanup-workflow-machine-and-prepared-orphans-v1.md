# 实现任务包:清理小包·两笔旧账——run_workflow_machine 废实现删(保签名保闸) + prepared 孤儿卫生 · 主导线 → 执行线 v1

日期:2026-07-08　性质:**轻档**(纯清理·零新功能;两部分一包;文件边界 §2.4)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **判死码铁律(本仓真踩过·RunningWorkflowsView 前科)**:删任何东西前**先扫全引用**(生产/测试/夹具/boundary spec/前端),被引用的要么保留要么先迁依赖——"看着像死码"不是证据。
- **主导线已核的事实(直接用)**:
  - **部分 a**:`run_workflow_machine_at` 本体在 `workflow_execution_entrypoints.rs:1494`;引用面 = command_registry(1)/`commands.rs`(4·**死线文件**)/lib.rs(6·旧测试)/main.rs(2)/real_execution_command(1)——**调用面全活着,不能整删**;canon(CURRENT ④a):真实现**已被 H5 受控会话续取代并封**(boundary spec `deprecated:true`),「真实现可删、留 blocked stub」;S0 曾试删被弹回:**真实现里有 helper 被另一测试共用**(纠缠点,先摸清再动);
  - **部分 b**:线上 state 店(`workflow-state.v0.json`)为**平铺 schema**(顶层 `nodes/execution_attempts/artifacts/audit_events/...`,**没有** project_workflows 嵌套——别按前端 Snapshot 形状想);粗扫 `"state": "prepared"` **148 处** = 孤儿规模真实存在;具体容器(execution_attempts?artifacts 任务包件?)由你按真 schema 定位;
  - **卫生先例照抄** = `store_hygiene.rs`(canvas-run 清扫·f4e907c):dry-run 默认/逐条 control_core 合法迁移校验/写前备份/防并发二次确认/审计/dev Tools 按钮。

## 1. 拍板摘要

- **要做的事**:a) 删掉被取代的四角色机器**真实现分支**(签名/命令/闸行为原样=调用面 0-diff);b) prepared 超龄孤儿合法收尾(store_hygiene 第二把扫帚)。
- **为什么**:两笔在册旧账(S0 deferred + 方案a 时代 deferred);死代码是误读源,孤儿是店膨胀源。
- **代价**:一轮。删除为主 + 一个 sweep 函数。

## 一句话判据

**「是不是只:真实现分支及其独占 helper 删除(签名/blocked 行为/全部调用面 0-diff)+ 一把照先例的 prepared 扫帚(dry-run 默认·合法迁移·可逆)——而共用 helper 不动、闸不动、任何活数据不碰?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 部分 a·run_workflow_machine 废实现删(保签名保闸)

1. **先测绘**(动手前产出引用地图进回交):lib.rs 6 处旧测试各测什么(blocked-stub 测 vs 真实现测)/ S0 说的共用 helper 是哪个、另一个使用者是谁 / commands.rs 4 处与 main.rs 2 处怎么调;
2. **删法 = 保签名换体**:`run_workflow_machine_at` 函数保留、签名不变;**真实现分支整删**,函数体只剩现有的闸/blocked 路径(任何调用者拿到的 blocked 响应与今天逐字节一致);真实现**独占**的 helper 一并删;**与他人共用的 helper 一根不动**(纠缠点如实报回,宁留勿断);
3. lib.rs:只删**专测真实现分支**的旧测试;blocked-stub 测试必须原样全绿(它们就是"行为没变"的证明);**lib.rs 只许删不许加**;
4. boundary spec `deprecated:true` 原样;`commands.rs`/`main.rs`/`command_registry.rs` **0-diff**(保签名的意义就在这);
5. 回交带删除统计(删了几行/几个 helper/几个测试)。

### 2.2 部分 b·prepared 孤儿卫生(store_hygiene 第二把扫帚)

1. **先按真 schema 定位**:prepared 记录的实际容器/字段/时间戳;盘点线上规模(主导线粗扫 148 处 `"state": "prepared"`,你给精确数+分布);
2. **孤儿判据(保守)**:`state=="prepared"` 且 超龄 **>7 天**(照 canvas-run 先例)且 不被任何 running/近 24h 链引用(有关联字段就查,没有就只按超龄+state);
3. **收尾走合法路**:核 control_core 迁移语义——prepared → 哪个终态是**单步合法且可逆**的(照 canvas-run「非 accepted→paused」的核法);**没有合法单步路 → 停、报回**(别造直写);
4. 实现进 `store_hygiene.rs` 新函数(`sweep_prepared_orphan_dispatches` 类名):dry-run 默认/写前备份/写时复核/逐条审计/人话汇总;命令注册(registry +1)+ dev Tools 卡第二按钮(照第一颗·页面内两步确认·**别用 window.confirm**——Tauri 不弹的老坑);
5. 单测:判据入选/排除(新鲜 prepared 绝不碰)/dry-run 零写/合法迁移断言/损坏跳过。

### 2.3 明确不做(§7 同)

整删函数或命令(调用面活着)/ 动共用 helper / 碰 running·completed·新鲜 prepared / 改闸或状态机本体 / lib.rs 加东西。

### 2.4 文件边界(越界即停)

- 允许:`workflow_execution_entrypoints.rs`(仅 run_workflow_machine 区)/ `lib.rs`(**仅删除**)/ `store_hygiene.rs` / `command_registry.rs`(+1)/ `ActiveWorkbenchView.tsx`(仅 Tools 卡窄口)/ `lib/tauri.ts`·`lib/types/*`(加法封装)/ 相关 css 若需;
- **0-diff**:`commands.rs` / `main.rs` / c4_c6 / controller / runner / control_core 本体 / manual_relay / 各 agent 模块 / 两意见 store / run_history_read_model / 交办前端。

## 3. 安全死线

- 删除类改动的证明责任**加倍**:blocked-stub 旧测试原样全绿 + 全量计数按预期变化(只降不增·降幅=删掉的真实现测试数,逐个点名);
- 卫生类:dry-run 默认/备份/合法迁移/审计四件套一个不少;真机执行那一下(点按钮清线上 148 片)= 用户做;
- fmt skip_children;高危清单 0 接触。

## 4. 验收

- a:引用地图 + 删除统计 + blocked-stub 测试按名全绿 + 调用面 4 文件 0-diff 自证;
- b:线上盘点数(精确)+ 判据单测 + dry-run 真店跑一遍的人话报告(只读)+ 三闸绿;
- 全量 cargo 计数变化逐项解释(删了 X 测 → 应降 X);§2.4 0-diff 自证。

## 5. 回交

- §4 证据 + 纠缠点实答(共用 helper 是谁/怎么处置)+ prepared 真实容器与判据实答 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 为了删干净把共用 helper 迁走(那是另一包)/ blocked 行为有任何字节变化 / 新鲜 prepared 被扫 / 直写状态绕 control_core / lib.rs 净增 / window.confirm。
