# 实现任务包:刀B·事实确认 + 记忆召回(确认属实→沉淀候选;出方案带上项目记忆)· 主导线 → 执行线 v1

日期:2026-07-06　性质:**轻档**(单线双面·文件边界 §2.4;全走现成记忆机器,零新 store)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(单线双面,当前无并行线)。**子线不 commit。** 全程中文。
- **背景**:Phase A「闭环后半段」最后一块。canon 流:「worker 汇报 → **事实确认** → 沉淀记忆 →(下个 worker/咨询)召回」。现状:汇报✓(口供落库)、上脸✓(刀A 黄牌);**确认无入口、沉淀不发生、召回恒空**。
- **主导线已核的接缝(直接用)**:
  1. `ProjectContext.memory_summary` 存在且**已接进咨询 prompt**(consultant_agent.rs:277「--- 项目记忆 ---」块)——但 239 行**恒 None**。填它=召回生效,零 prompt 改动;
  2. 正式记忆 store:`formal_memory_store.rs`(formal-memories.v1.json·线上实存 3 条·MemoryRecord 带 scope/生命周期态)——loader 现成;
  3. 候选创建:`create_memory_candidate` 命令已注册(input:project_root/scope/memory_type/claim/body/source_refs/generated_by_role/risk…·types.rs:3888);App.tsx 已 import——**核查它有没有现成 PendingAction kind**(有则复用确认弹层,无则报回);
  4. **采集钩子的绕行是对的**:`record_worker_structured_report` 命令包装层挂了记忆采集(commands.rs:976-981),但链直调 `_at` 绕过——正合 canon「未经确认的汇报不自动沉淀」。**别把钩子接进链**(§7)。
- **一句话**:交货脸绿任务行加[属实→沉淀]（经现成 create_memory_candidate,claim=自述、body=产出+证据）;咨询前从正式记忆 store 取本项目活跃记忆填 memory_summary(顶格 5 条·人话行),说脸显示「出方案会带上 N 条项目记忆」。

## 1. 拍板摘要

- **要做的事**:确认有入口、沉淀真发生、召回真生效——记忆闭环接进交办循环,Phase A 补完。
- **为什么**:路线图 Phase A 交付项原文;也是"下个 worker 不再从零开始"的地基。
- **代价**:一轮。后端一处填值 + 前端两小块。

## 一句话判据

**「是不是只:确认按钮→现成候选命令(候选≠正式·治理转正不动)、咨询前填 memory_summary(只读正式 store·prompt 零改)、说脸一行召回计数——而记忆生命周期/治理/store 校验/死线全 0-diff?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 事实确认(前端为主)
- 交货脸任务行(刀A 的 `JiaobanStepReportList`):**绿✓且有自述**的任务,行尾加小按钮**[属实,沉淀]**;黄牌/失败/跳过**不给**(没干完的事实不沉淀);
- 点击 → 经确认流走 `create_memory_candidate`:claim=自述(report_summary 的 did 部分)、body=任务标题+产出+证据、scope=project、generated_by_role="user"、generated_from 标 worker_report 来源、source_refs 带 workflow/任务锚;memory_type/risk_level/sensitive_level **按 validate 的合法枚举取最保守档**(核查后填,不猜);
- **走现成 PendingAction kind**(§0.3 核查):有则 onRequestAction 复用确认弹层;没有 → 停、报回(别自造第二套写入确认);
- 沉淀过的行显示「已沉淀 ✓」(本地态即可,防重复点)。

### 2.2 记忆召回(后端一处 + 前端一行)
- `consultant_agent.rs::run_project_consultation_inner`:load_project_context 之后,从 formal memory store(workflow_state_path 侧车)取**本项目 scope + 活跃态**记忆,按更新时间取**顶格 5 条**,渲染成人话行(「[类型] claim——body 首行」)填 `ctx.memory_summary`;0 条 → None(现状);store 读失败 → None + 不挡咨询(召回是增益不是闸);
- 预拆(`plan_preview`)同一缝**同享**(主管拆任务也带记忆);
- 说脸(或授权卡顶部)一行小字:「出方案会带上 N 条项目记忆」(N=同口径统计;0 → 整行不显)。前端取数:直调现成 `loadFormalMemoryStore` 或走已有 props——**选侵入最小的**,别新穿五层。

### 2.3 测试位置
后端测试进 `consultant_agent.rs` 自己的 `#[cfg(test)] mod`(照 worker_report 先例,**不进 lib.rs**);前端进现有离线 harness(新文件挂 `run-offline-interaction-test.mjs`——**允许改跑器加 1 行**,刀A 教训)。

### 2.4 文件边界(越界即停)
- 允许:`consultant_agent.rs`(填值+自测)/ `ProjectJiaobanPanel.tsx` / `projectWorkflowSidePanel.css` / `lib/tauri.ts`(若需补 loadFormalMemoryStore 封装)/ `tests/` 新文件 + 跑器 1 行;
- **0-diff**:formal_memory_store / formal_memory_lifecycle / memory_capture_bus / memory_daily_loop / create_memory_candidate 命令与校验本体 / commands / director_agent / worker_report / lib.rs / 全部死线。

## 3. 安全死线

- **候选≠正式**:确认只产候选,转正仍走既有治理生命周期(一字不动);敏感内容不进 claim/body(自述本就来自结构化字段);
- 召回**只读**正式 store、失败静默降级不挡咨询;记忆内容进 prompt = 已有设计(memory_summary 槽位本就为此而生),不新开信道;
- 渲染类真机过;fmt 老规矩。

## 4. 验收

- **单测·召回**(consultant mod):有活跃项目记忆 → summary Some 且 ≤5 条格式对;0 条 → None;store 坏 → None 不 Err;
- **离线 DOM**:绿任务行有[属实]、黄牌行没有;点后「已沉淀 ✓」;说脸 N>0 显示/N=0 隐藏;
- **真机**:跑一单 → 绿任务点[属实] → 记忆中心候选页里能看到这条(claim=自述);给测试项目转正/建一条正式记忆后再出方案 → 说脸「带上 1 条」+ 咨询产出可见引用(有就截,没有不硬造——tier-1 如实报);
- 三闸 + 0-diff 自证 + 计数不降。

## 5. 回交

- §4 证据 + PendingAction kind 核查实答 + memory_type/risk 枚举取值依据 + 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 不接受为:把采集钩子接进链(绕确认自动沉淀)/ 动了治理生命周期或校验 / 召回失败挡咨询 / 自造写入确认层 / 黄牌任务也能沉淀。
