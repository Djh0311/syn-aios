# Task Package Template

## 任务名

一句话说明任务。

## 所属开发线

例如：Codex 数据盘点线 / 索引内核线 / 桌面应用线 / 验证线。

## 背景

说明为什么要做。必须引用阶段计划或上游证据。

## 目标

列出本任务要交付的具体结果。

## 允许读取

列绝对路径或项目内路径。

## 允许写入

列绝对路径或项目内路径。

## 禁止事项

必须写清楚不能改哪些文件、不能做哪些动作。

## 形状影响

每个任务包必须填写。不能写“无影响”后跳过细项。

- 任务类型：功能任务包 / 治理任务包 / 紧急缺陷 / spike。
- 新增代码落点：列文件或目录；没有新增代码也要写明。
- 是否触碰棘轮文件：例如 `src-tauri/src/lib.rs`、`real_execution_command.rs`、`ProjectsView.tsx`、`AgentView.tsx`、`types.ts`、`styles.css`、离线测试主文件，或 `workbench-shape-gate.js` 输出的 ratchet list。
- 预计行数变化：列出主要文件的预计增减；治理任务包必须记录前后指标。
- 是否新增 Tauri command：如新增，必须说明落点；新增 `#[tauri::command]` 不得写进 `lib.rs`。
- 是否新增 sidecar JSON 种类：默认不得新增；确需新增时必须先取得用户确认并写入 `decisions/**`。
- 是否需要 shape gate 豁免：默认不需要；任何豁免都必须有 `decisions/**` 记录，不允许沉默豁免。
- 本任务基线 commit：
- 本任务完成 commit：

## 治理任务包规则

治理任务包的验收口径是：行为不变 + 形状指标改善 + evidence 记录前后指标。治理任务包也走既有任务包、evidence、handoff 和回收流程，不另起制度。

解冻恢复功能开发后，配额为每 3 个功能任务包至少配 1 个治理任务包，跑一个 Stage 后可复盘调整。配额例外必须写入 `decisions/**`，不能沉默跳过。

## 验收标准

写可检查的标准，不写感觉判断。

必须按改动范围选择验证，并默认包含：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

## 必须回传

开发线回传必须包含：

1. 做了什么
2. 改了哪些文件
3. 新增了哪些测试或证据
4. 哪些结论有依据
5. 哪些仍不确定
6. 风险和下一步建议
7. shape gate baseline / check 摘要
8. start commit / end commit；如无 git，必须标记 `no_git_blocked_for_r2_r3`
9. 是否新增 command、sidecar 或触碰棘轮文件

## 总指导回收动作

总指导回收时必须判断：

- 接受
- 需要修改
- 暂停
- 废弃

并说明依据。
