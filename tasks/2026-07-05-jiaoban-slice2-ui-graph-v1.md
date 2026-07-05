# 实现任务包:交办·刀2-UI(批前看图 + 画布依赖连线 + 所批即所跑接线)· UI 专线 v1

日期:2026-07-05　性质:**轻档·前端**(`src-tauri/**` 0-diff;后端件全在刀2-backend `39315a1`,只接不改)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是 **UI 专线**。只改前端。**子线不 commit。** 决策正本 `decisions/2026-07-02-project-jiaoban-tab-final-design-v1.md` §条件式工序图。
- **现状(用户 07-05 真机确认)**:Phase A 主干真机验收**已过**;任务节点**已经**出现在「工作流」画布上(刀2-backend 的 prepare 落任务级节点+链完成刷状态,画布 1:1 透传)。**缺的**:① 节点之间没有依赖连线(只读画布不翻 depends_on)② 批前看不到图 ③ 批的图没有被原样带进执行(所批即所跑的 UI 半边没接)④ 任务节点没挡运行入口。
- **后端已备好、只等你调**(全部已注册·核过实物):
  - `preview_pending_proposal_director_plan(project_root, proposal_id)` → `{planned_tasks, warnings}`:对 pending 方案只读预拆图(零写盘·1-7 分钟·可能 flaky);
  - `confirm_and_start_authorized_run` 的 **`approved_planned_tasks` 可选参**:传了就照这份图跑、不重拆;
  - `ProjectConsultationProposal.suggest_workflow: bool`(咨询判"值得按工作流")。
- **关键实现坑(主导线核过·照抄别踩)**:只读画布 `projectCanvas.ts` 的 `buildEdges`(~1068-1085)只合成固定角色边——把 `derivedWorkflow.nodes[].depends_on` 翻成 `ProjectCanvasEdge` 时,**canvas id 必须复用 `buildWorkflowNodeCanvas` 的 slug 规则**(~961:`split(":node:").pop()`),否则端点过滤会**静默丢边**。read model 已把 edges 反推成节点的 `depends_on` 字段(现成数据·不用新后端)。
- **先读**:`ProjectJiaobanPanel.tsx`(授权卡/干态/交货态)、`projectCanvas.ts:941-1085`(节点/边合成)、`lib/tauri.ts`(补 preview 封装)。

## 1. 拍板摘要

- **要做的事**:复杂活批前看图(文字先出、图后浮现)、批的图原样执行(UI 把预拆结果回传合流)、画布上依赖连线 + 跑到哪亮到哪、任务节点只看不点。
- **为什么**:决策已拍;这是"看到再批、所批即所跑、真图落画布"三件套的 UI 半边,后端全就绪。
- **代价**:一轮·前端(卡上图区 + buildEdges 翻译 + 合流传参 + 小清理)。

## 一句话判据

**「是不是只在前端接刀2-backend 现成命令/数据(预拆/approved 传参/depends_on 翻线),简单活零新等待,src-tauri 0-diff?」** 是 → 做;否 → 停。

## 2. 建什么

### 2.1 批前看图(条件式·授权卡)
- 触发:`proposal.suggest_workflow === true` **或**用户在卡上点「按工作流来」开关。**简单活不触发、不加一秒等待**(好用判据)。
- 触发后:卡照常先显文字方案;图区显「正在画工序图…(1-7 分钟)」,异步调 `preview_pending_proposal_director_plan`;回来后画**迷你图**(任务名 + 谁等谁;残图/警告按 warnings 显虚线或小字;不用重型图库,列表+缩进箭头即可)+「去工作流 tab 看大图」链接(切 tab)。
- 预拆失败(flaky)→ 图区一行人话「工序图没画出来(可重试);不影响批」+ [重试];**卡照常可批**(优雅降级)。
- **持住预拆结果**(state+模块缓存):批时要用(2.2)。

### 2.2 所批即所跑(核心接线)
- 用户看过图后点[允许并开始] → `confirmAndStartAuthorizedRun` **带 `approved_planned_tasks: 预拆返回的那份`** → 后端跳重拆、照图跑。
- 没触发图(简单活)→ 不传(现状:批后 LM 拆)。预拆过但用户改了要求重出方案 → 旧图作废(清缓存)。

### 2.3 画布依赖连线 + 任务节点只看不点
- `projectCanvas.ts buildEdges`:把 `depends_on` 翻成边(**照 §0 的 slug 规则**;`edge_type` 视觉上与角色边区分,如虚线/箭头)。
- 任务级节点(node_id 含 `:node:task:`):只读画布**不给运行入口/不进可运行判定**(它们是窗口不是操作台——决策 §条件式工序图);编辑画布里也别让它冒充可跑节点(能显示即可)。
- 顺手:交货脸(fix6 记档的 cosmetic)改喝 `thisRoundChainStatus`。

### 2.4 词表照旧
- 主路径禁黑话;图上显任务标题,不显 node_id/planned_task_id。

## 3. 安全死线

- `src-tauri/**` 0-diff;[允许并开始]仍显式点击;`approved_planned_tasks` 只传预拆**原样返回**的数据(前端不改其内容——改了 guard 也会拦,但别造脏数据);渲染类**必须真机过**。

## 4. 验收(UI 线真机·测试项目)

- **简单活回归**:不开工作流开关 → 卡秒出、无图区、两下跑通(零回退)。
- **复杂活**:开关/建议触发 → 文字先出 → 图浮现(任务+依赖对得上)→ 批 → **后端不重拆**(结果任务与图一致)→ 画布上节点+**连线**在、跑到哪亮到哪。
- **降级**:模拟预拆失败 → 人话+重试+仍可批。
- **任务节点**:画布上无运行入口。
- 三闸绿;`git diff` 仅前端。

## 5. 不做

- 画布编辑工序图再跑(推后·决策圈外);方案 a;工序图跨 app 重启恢复;后端任何改动。
- 「每单一条独立工作流」:决策默认如此但刀2-backend 现状是**挂选中工作流**,用户真机见过未异议——**本包维持现状**,要切另议。

## 6. 回交

- §4 证据(截图:卡上图/画布连线/亮进度/简单活无图)+ 改动清单 → 主导线核。**子线不 commit。**
