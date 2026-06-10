# Stage K / K6 Real Tauri Dogfood And Stage Acceptance Freeze v1

日期：2026-06-10

状态：已完成，结论为 `accepted_with_deferred_items`。

初次执行结果记录见 `../evidence/2026-06-10-stage-k-k6-real-tauri-dogfood-and-stage-acceptance-freeze-v1.md` 与 `../handoffs/2026-06-10-stage-k-k6-real-tauri-dogfood-and-stage-acceptance-freeze-v1-result.md`，当时阻断于 `blocked_by_tauri_webview_blank_window`。后续 K6.1 / K6.2 已恢复真实 Tauri 可见截图链路；最终回到 K6 主任务并完成核心入口 window-only 截图验收，记录见 `../evidence/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1.md` 与 `../handoffs/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1-result.md`。本任务接受为 K6 真实 Tauri dogfood 核心入口验收和 Stage K acceptance freeze；不接受为 Stage K 严格无缺口完成、K3-B1 retry 成功、K3-B2 可开始、真实 retry / stop / restart / resume 已实现或 planned adapters 真实接入。

本任务包用于在 K5 非真实运行 / 待办 / 失败恢复和操作控制切片完成后，执行 Stage K 的真实桌面 dogfood 和阶段验收收口。K6 不是新的真实 Codex 执行授权任务；它默认只做真实 Tauri UI / 产品链路验收、截图证据、缺口矩阵和 Stage K 完成项 / deferred 项冻结。

本文不授权新的真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，不启动 K3-B1 retry，不启动 K3-B2，不实现真实 retry / stop / restart / resume。

如果 K6 执行中需要启动真实 Tauri、使用 macOS 截图、检查或清理端口，必须由主管线在执行前明确列出命令、路径、风险和降级方案；不能把普通浏览器 smoke 冒充真实 Tauri 验收。

## 1. 当前事实

- K4 已完成并收口为 `accepted_non_real_productization_slice`。
- K5 已完成并收口为 `accepted_non_real_productization_slice`。
- Stage K architecture calibration v2 and gate 已完成，gate strict 通过，0 error / 0 warning。
- K3-B1 已执行但失败分类；retry 申请再次被安全审查拒绝。
- K3-B2 依赖 K3-B1 成功和复核，当前不得启动。
- K1-K5 已形成日常可用工作台的主要普通 UI / read model / Product Command / workflow / memory / run queue 非真实产品化链路。
- K6 是 Stage K 的真实 UI dogfood 和验收冻结，不是 K3-Level-B 真实执行恢复。

## 2. 目标

K6 本轮交付：

1. 用真实 Tauri 桌面壳验证 Stage K 的核心用户路径。
2. 采集真实窗口截图或明确记录截图无法完成的具体原因。
3. 对照 Stage K 计划冻结完成项、deferred 项和不得声称完成项。
4. 确认普通 UI 继续以人类操作方式组织，不退回开发者控制中心。
5. 确认 K1-K5 的 UI / read model 能在真实壳中可理解展示。
6. 形成 Stage K 最终 evidence / handoff / 当前入口同步。

## 3. 非目标

- 不执行真实 Codex。
- 不发送 prompt。
- 不做 K3-B1 retry。
- 不启动 K3-B2。
- 不真实 retry / stop / restart / resume。
- 不新增真实操作 Tauri command。
- 不 kill Codex 进程。
- 不自动清理真实 Codex 本地状态。
- 不自动写 FormalMemory。
- 不新增 provider credential store 或 model verification。
- 不接 planned adapters 真实执行。
- 不把普通浏览器 smoke 当真实 Tauri 验收。

## 4. UI 显示边界确认

本任务会验收前端，原则上不新增产品功能：

- [ ] 不涉及 UI / read model / 文案。
- [x] 涉及真实 Tauri UI 验收。
- [x] 涉及截图 / 手动检查清单。
- [x] 涉及阶段完成项和 deferred 项冻结。
- [ ] 新增普通主导航入口。

普通 UI 应覆盖：

- 首页入口。
- 智能体对话页：项目、对话、消息、输入、发送前确认 / 预览状态。
- 运行中工作流页：运行队列、待确认、失败控制、操作控制 / 恢复建议、记忆待处理。
- 项目页：项目工作流、run units、节点详情和任务包 / 记忆包摘要。
- 记忆层：正式记忆、候选、捕获、待正式化、任务记忆包预览和补证。
- 知识库 / 想法箱 / Skill / Harness 的普通入口信息层级。
- 设置开发者区：开发者 / 内部边界信息应默认后撤。

普通 UI 不显示：

- raw JSON。
- sidecar 绝对路径。
- store revision。
- prompt body。
- full transcript。
- raw stdout / stderr。
- `/Users/yoyi/.codex` 内部路径内容。
- H/J/K/PCR 阶段术语作为用户操作文案。
- 真实 `codex exec` / `codex exec resume` 命令串。
- “自动重试中 / 已自动修复 / 已写正式记忆 / 结果数：0 / 已停止 / 已重启 / 已恢复 / 已 resume”等误导完成态。

## 5. 建议截图清单

截图路径建议：

`evidence/tauri-verification/2026-06-10-stage-k-k6/`

建议编号：

1. `01-home.png`：首页入口和主对象层级。
2. `02-agent-chat.png`：智能体对话页。
3. `03-agent-send-preview-or-boundary.png`：发送前确认 / 预览 / 边界状态。
4. `04-running-workflows.png`：运行中工作流总览。
5. `05-operation-control.png`：操作控制 / 恢复建议。
6. `06-project-workflow.png`：项目工作流和 run units。
7. `07-workflow-node-detail.png`：节点详情 / 任务包 / 记忆包摘要。
8. `08-memory-center.png`：记忆层普通入口。
9. `09-memory-candidate-task-packet.png`：候选 / 待正式化 / 任务记忆包预览。
10. `10-knowledge-base.png`：知识库普通入口。
11. `11-ideas-skills-harness.png`：想法箱 / Skill / Harness 普通入口。
12. `12-settings-developer.png`：设置开发者区。

如果某张截图无法稳定触达，应在 evidence 中记录为 `not_captured`，并写明原因；不得把缺失截图包装成通过。

## 6. 推荐验收步骤

1. 运行静态验证：
   - `npm run typecheck`
   - `npm run test:offline-interaction`
   - `npm run build`
   - `node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict`
2. 启动真实 Tauri 桌面壳。
3. 确认窗口标题、目标窗口区域和截图权限。
4. 按截图清单逐入口验收。
5. 截图后停止 Tauri，并记录是否释放端口 / 进程。
6. 扫描误导文案和旧口径。
7. 写 K6 evidence / handoff。
8. 同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`README.md`、`STAGE_PLAN.md` 和 Stage K 计划。

## 7. 验收标准

必须满足：

- K1-K5 主要普通 UI 在真实 Tauri 中可进入或明确记录未覆盖原因。
- 运行中工作流页能看到 K5 操作控制 / 恢复建议，并保持只读建议。
- `readback_unavailable` / `readback_failed` / `timed_out` / null result count 不显示为 0。
- retry / stop / restart / resume 不显示为已实现真实操作能力。
- 记忆候选 / observation / capture 不显示成 FormalMemory。
- 开发者字段默认后撤到设置或折叠详情，不铺普通首屏。
- Stage K 完成项 / deferred 项冻结清楚。

## 8. 降级规则

如果出现以下情况：

- Tauri 启动失败。
- 端口被占用且不能安全处理。
- 截图权限不足。
- 无法识别真实 Tauri 窗口。
- 只能打开普通浏览器 / Vite 页面。

则只能记录为：

`真实 Tauri / 截图验收未完成或部分完成`

允许保留普通浏览器 smoke、DOM 文案、命令输出或构建验证作为辅助证据，但不得把它们冒充真实 Tauri 完整验收。

## 9. 接受口径

可接受为：

- K6 真实 Tauri dogfood 和 Stage K 验收收口完成。
- Stage K 当前完成项 / deferred 项冻结完成。
- 普通 UI 信息层级和 K1-K5 关键路径完成真实桌面或缺口矩阵验收。

不接受为：

- K3-B1 retry 成功。
- K3-B2 可开始。
- 真实 retry / stop / restart / resume 已实现。
- 任意项目无限制自由控制台完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 自动写 FormalMemory 或自动技能化完成。
- G3-B 历史缺口全部补齐，除非 K6 逐项真实截图覆盖并记录。
