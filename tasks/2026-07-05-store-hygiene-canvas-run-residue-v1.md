# 实现任务包:卫生包(canvas-run 历史残料合法归档:ready_for_review → paused)· 主导线 → 执行线 v1

日期:2026-07-05　性质:**轻档**(只读盘点 + 合法状态迁移;默认 dry-run;不删任何数据)。**可与方案a并行**(文件面不撞·见 §2.4)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(后端)。**子线不 commit。**
- **背景**:线上 store 里有 **~20 条 canvas-run 形状的工作项**(id 含 `:canvas-run:`)停在 `ready_for_review`——是 6 月画布线真跑攒下的历史残料(没人会去审),**不挡交办**(id 空间不同),但一直挂着:视图里当"待审"陈列、store 膨胀。主导线已核:交办残料(fix4/fix5)与此无关,本包只管 canvas-run 这批。
- **合法归宿已查定(迁移表 control_core:17-48)**:`after=="paused"` 从任何非 accepted 态**一步合法**,且 `paused → ready_to_dispatch` 可逆——**paused = 合法"搁置"态,不删、可逆、留审计**。这就是归档方式,不新增迁移、不直接改字段。
- **一句话**:一个维护命令,盘点(dry-run 默认)→ 执行时把「canvas-run 形状 + ready_for_review + 超龄」的工作项**逐个经 `update_work_item_state_at` 迁到 paused** + 审计;附一颗 dev-only 触发按钮。

## 1. 拍板摘要

- **要做的事**:历史残料合法归档,视图清爽、账不丢、可逆。
- **为什么**:用户拍板卫生包;残料放着是慢性噪音。
- **代价**:一轮·后端为主(新文件)+ dev 视图一颗按钮。

## 一句话判据

**「是不是只对『canvas-run 形状 + ready_for_review + 超龄』的工作项经合法迁移迁到 paused(默认 dry-run·带审计·不删·别的记录一概不碰),迁移表/闸/死线 0-diff?」** 是 → 做;否 → 停。

## 2. 建什么

### 2.1 维护命令(新文件,如 `store_hygiene.rs`)
- `sweep_canvas_run_residue(project_root?, dry_run: bool=true, now_ms)`:
  - 扫 work_items:id 含 `:canvas-run:` **且** state=="ready_for_review" **且** 超龄(如 > 7 天·常量写死带注释);
  - **dry_run(默认)**:只返回盘点清单(条数 + 每条 id/岁数),**零写**;
  - **execute**:逐条经 `update_work_item_state_at` 迁 `paused`(合法一步·自带审计),另 append 一条汇总审计(`canvas_run_residue_swept`,带条数);返回结果清单。
- 只碰命中三条件的;交办形状(`planned-task`)/别的项目/别的状态一概不扫。注册进 command_registry。

### 2.2 dev 触发口(唯一前端触点·允许本包顺手做)
- dev-only Tools 视图(`devNavItems` 的 `tools`)加一颗「清理画布历史残料」按钮:先 dry-run 显示盘点(「找到 N 条,预览如下」),用户点「执行归档」才真迁。**只碰 Tools 视图一个文件**,不碰交办面板/画布/外壳。

### 2.3 测试放新文件自己的 mod(**别进 lib.rs**——给方案a让路,防并行撞车)
- dry-run 零写(字节比对);execute 迁 paused + 审计在 + 可逆(paused→ready_to_dispatch 单测);三条件筛选逐类断言(交办形状/非 ready_for_review/未超龄 全不动);幂等(重跑无新变化)。

### 2.4 并行纪律
- 本包文件面:新文件 + command_registry(1 行)+ Tools 视图。**不碰** director_agent.rs / lib.rs / ProjectJiaobanPanel.tsx(方案a的地盘)。

## 3. 安全死线

- 迁移表/`update_work_item_state_at`/控制核心/全部死线 0-diff;**不删任何记录、不直接写 state 字段**(全经合法迁移);dry-run 默认;prepared 孤儿派发/链历史记录/审计——**全不碰**(有引用关系,另议)。

## 4. 验收

- §2.3 全绿;`cargo test --lib` 计数不降;0-diff 自证;fmt;dev 按钮真机截图(dry-run 清单 → 执行 → 复扫为 0)。
- **对线上真库执行 = 用户在 dev Tools 里点那一下**(先看 dry-run 清单再点执行·可逆兜底)。

## 5. 不做

- prepared 孤儿派发清理、链历史压缩、审计归档(引用关系另议);自动定时清理(维护动作必须人点);交办残料(fix4/5 已管)。

## 6. 回交

- §4 证据 + 文件面清单(证没碰方案a地盘)→ 主导线核实物。**子线不 commit。**
