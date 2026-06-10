# Stage K Architecture Calibration Plan v3

日期：2026-06-10

状态：Stage K 收口前架构校准和继续推进计划。本文不改变 Stage K 原目标，仍以“自由操控 Codex + 自动化工作流 + 记忆层记录”为交付方向。v3 的作用是：在 K4 / K5 已完成、K6.2 已恢复真实 Tauri 窗口截图能力之后，先把后端架构、执行主路径、记忆一致性、UI 信息层级和验收证据链再校准一次，再继续 K6 主任务补齐真实桌面 dogfood。

本文不是新的真实执行授权，不授权 `codex exec` / `codex exec resume`，不授权发送 prompt，不授权读写 `/Users/yoyi/.codex`，不授权 K3-B1 retry 或 K3-B2。本文也不是 Stage K 完成声明。

## 1. 当前事实

- Stage K 原目标没有变化：自由操控 Codex、自动化工作流、记忆层记录 / 分析 / 候选化。
- K0、K1、K2、K2.5、K3-Level-A、K3-Level-B 字段冻结、K3-B0、K3-B1.0、K3-B1.1、architecture gate、K4 和 K5 已完成。
- K3-B1 retry 被安全审查再次拒绝；K3-B1 仍未完成，K3-B2 仍不得启动。
- K6 真实 Tauri dogfood 已执行但未通过；K6.1 分类为旧截图链路捕获白屏；K6.2 已用 ScreenCaptureKit window-only harness 捕获一张真实 Tauri 首页截图。
- 当前应继续 K6 主任务，补齐核心路径截图和缺口矩阵；但在 K6 收口前，需要做一次架构校准复核，避免把 UI 截图通过误当成底层架构完全稳定。

## 2. 校准判断

不建议暂停 Stage K 原计划做大重写。

原因：

- 当前问题不是“底座不存在”，而是“多轮 checkpoint 后需要确认主路径没有被 probe、legacy、fixture、UI 补丁和旧执行入口污染”。
- 打补丁不是天然错误；错误的是补丁没有被主路径吸收、没有边界、没有测试、没有后续删除或封存策略。
- 现在应该做的是架构校准：证明当前代码仍符合蓝图分层，找出必须立刻修的 P0/P1，剩余 P2 进入后续重构清单。

暂停范围：

- K3-B1 retry。
- K3-B2 workspace-write / new-session 真实执行。
- 任何新的真实 Codex 执行点。
- planned adapters 真实接入。
- provider credential / model verification。

继续范围：

- K6 真实 Tauri UI dogfood。
- Stage K architecture gate。
- 只读代码架构复核。
- 不触发真实 Codex 的 UI 信息层级和读模型修补。
- K6 evidence / handoff / deferred freeze。

## 3. 蓝图对齐标准

本轮校准按三份权威设计对齐：

- `docs/workbench-system-architecture-v1.md`：本地模块化单体、项目单元隔离、控制核心、适配器、事件账本 + 当前快照 + 读模型。
- `docs/workbench-frontend-display-boundary-v1.md`：桌面 Tauri、普通 UI / 详情 / 设置开发者分层、智能体像对话而不是控制中心。
- `docs/memory-layer-design-v1.md`：Observation、MemoryCandidate、FormalMemory、Knowledge hit 和索引必须分层，不能互相冒充。

硬性架构约束：

- 前端不能直接写事实层。
- 前端不能直接拼真实 Codex CLI。
- 真实执行只能经 Product Command / permission envelope / runner / runtime log / audit / readback 主路径。
- 旧 dispatch、workflow machine、canvas experiment、fixture probe 不能成为普通产品执行入口。
- 适配器只能声明能力、接收受控命令、返回结果或错误；不能直接推进 workflow state 或写正式记忆。
- 聊天上下文、检索命中、LLM 摘要、工具输出和知识库命中都不是正式事实。
- 记忆正式化必须有来源、版本、权限、审计和确认链路。

## 4. 校准对象

### 4.1 后端执行主路径

检查目标：

- `codex-local` 真实调用只存在于批准 runner。
- Product Command 是唯一普通真实执行归口。
- Phase A / Phase B、permission、continuation、runtime log、audit、readback 的链路可追踪。
- K2 / J2 / H5 / PCR9 / K3-B fixture 常量不泄漏成普通产品逻辑。
- legacy Tauri command 和 CLI helper 要么 blocked，要么 developer / test only，要么明确迁移计划。

P0：

- 普通 UI 或 Tauri wrapper 可以绕过 Product Command 直接真实调用 Codex。
- 新增路径会发送 prompt 或读写 `.codex`，但没有任务级授权。

P1：

- 执行结果无法追溯到 runtime log / audit / readback。
- readback failed / unavailable / timed_out 被显示为真实 0 条结果。

P2：

- 历史命名、fixture 常量、兼容 helper 仍存在，但不在普通主路径。

### 4.2 工作流编排架构

检查目标：

- 用户目标到 run units 的链路来自项目工作流服务，不来自 UI 拼接。
- run unit 派发必须能追到 Product Command attempt。
- worker report、process fact、final review、memory capture 的引用链完整。
- K3-Level-A 非真实链路和 K3-Level-B 真实执行点在 evidence 和 UI 上清楚区分。

P0：

- 工作流 UI 直接触发旧 workflow machine 真实 runner。
- 工作流状态由前端直接写入。

P1：

- run unit 完成态没有 readback / worker report / audit 证据。
- K3-B1 未完成却允许 K3-B2 开始。

### 4.3 记忆层一致性

检查目标：

- ProductCommand、Continuation、RuntimeLog、MemoryCaptureEvent、Observation、MemoryCandidate、FormalMemory 之间的 ref 语义一致。
- Observation 不等于 FormalMemory。
- Candidate 不等于 FormalMemory。
- Knowledge hit 不等于 FormalMemory。
- 缺链路时 UI 显示“需要补证 / 待确认 / 不可用”，不显示为完成。

P0：

- 自动把聊天、工具输出、knowledge hit、observation 或 candidate 写成正式记忆。
- 正式记忆写入缺少确认、来源、版本或审计。

P1：

- 记忆候选来源无法追到 capture event / observation / workflow evidence。
- 任务记忆包把候选或知识命中当作正式 included memory。

### 4.4 UI 信息层级

检查目标：

- 普通左侧入口符合 Stage K 产品形态：项目、智能体、想法箱、知识库、记忆层、Skill、Harness、运行中工作流，设置在底部。
- 智能体页是项目 / 对话 / 消息 / 输入的对话工作区，不是控制中心。
- 运行中工作流显示正在做什么、哪里卡住、需要用户做什么。
- 开发者 / 内部边界进入设置或折叠详情。
- 不做手机端 UI，不把浏览器移动视口当验收。

P0：

- 普通 UI 提供未授权真实执行按钮。
- 普通 UI 展示 prompt body、secret、token、`.env`、keychain、OAuth、provider credential 或 full transcript。

P1：

- 普通 UI 大面积展示 raw sidecar、store revision、audit ref、runtime log raw path、阶段术语。
- retry / stop / restart / resume 显示成已经真实实现。

P2：

- Skill / Harness / 设置仍有偏开发者的文字，但默认不影响主路径操作。

### 4.5 Tauri dogfood 证据链

检查目标：

- 截图必须来自真实 Tauri window id 的 window-only capture。
- 截图路径、window id、PID、title、bounds、命令、sha256 都要记录。
- 普通浏览器 smoke 只能辅助，不能替代真实 Tauri 验收。
- 若导航无法完成，记录 `not_captured` 和原因，不包装成通过。

P0：

- 把普通浏览器截图冒充真实 Tauri。
- 用全屏截图或误截其他窗口后声明通过。

P1：

- 截图没有可复核的 window id / PID / sha256 / bounds。
- 核心路径缺图却声明 K6 全量通过。

## 5. 分线职责

全局主管线：

- 维护 K3-B1 / K3-B2 冻结事实。
- 决定 P0/P1 是否阻断 K6 收口。
- 只在 checkpoint 完成、阻断或阶段边界变化时同步权威入口。
- 不催促开发线用未完成结果冒领。

架构复核线：

- 只读复核后端执行主路径、工作流状态、记忆一致性和 UI 信息层级。
- 输出 P0/P1/P2，必须带文件行号或命令证据。
- 不改代码，不执行真实 Codex，不读写 `.codex`。

修补开发线：

- 只修 P0/P1 或主管明确纳入的 P2。
- 不做大重写，不顺手改无关文件。
- 不新增真实执行入口。

UI 线：

- 只处理信息层级和桌面 Tauri 可用性。
- 不改变真实执行语义。
- 参考 Xuanji 只学习层级，不复制风格或源码。

验证线：

- 运行 architecture gate、npm / cargo 必要验证、Tauri screenshot harness。
- 记录失败也算证据；不能把失败说成通过。

## 6. 执行顺序

1. 写入本 v3 校准计划。
2. 运行 Stage K architecture gate strict，确认当前无 P0/P1 架构 gate 命中。
3. 做一次只读代码架构抽样复核，范围覆盖执行主路径、工作流、记忆和前端入口。
4. 若发现 P0/P1，先修补并重新验证；若只有 P2，登记到 deferred。
5. 继续 K6：使用 ScreenCaptureKit window-only harness 对当前真实 Tauri dev 窗口补齐核心截图。
6. 每张截图记录 window id、PID、title、bounds、路径、sha256 和视觉结论。
7. 写 K6 evidence / handoff，明确 completed / deferred / blocked。
8. 只有 K6 状态变化后，才同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`README.md`、`STAGE_PLAN.md` 和 Stage K plan。

## 7. 重写与补丁判定标准

继续补丁式修补适用：

- 问题在 UI 文案、信息层级、读模型展示或 guard 分类。
- 主路径清楚，局部代码命名或历史兼容需要收口。
- 测试可以覆盖，变更可回滚。

需要小重构适用：

- 同一概念在多个文件重复派生，导致 UI 与读模型口径不一致。
- helper 已经承载多个职责，继续补丁会扩大条件分支。
- 普通路径和 developer / legacy 路径混在同一组件里。

需要暂停大重写适用：

- 存在可绕过 Product Command 的真实执行路径。
- workflow state 或 sidecar schema 已经无法表达当前事实。
- 记忆正式化链路出现不可追溯写入。
- UI 层直接决定权限、事实或执行状态。

当前默认判断：

```text
不暂停 Stage K 做大重写；先做 v3 校准 + P0/P1 修补 + K6 dogfood。
```

## 8. 验收口径

v3 可接受为：

- Stage K 收口前架构校准计划完成。
- 明确原目标不变，不暂停 Stage K 主线。
- 明确哪些问题必须阻断 K6 / Stage K 收口。
- 明确补丁、局部重构和大重写的判定标准。
- 明确 K6 下一步仍是 ScreenCaptureKit window-only 真实 Tauri 核心截图。

v3 不接受为：

- Stage K 完成。
- K6 完成。
- K3-B1 retry 成功。
- K3-B2 可开始。
- 任意项目无限制自由控制台完成。
- 自动 retry / stop / restart / resume 已真实实现。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 自动写 FormalMemory。

## 9. 当前下一步

当前不进入 K3-B1 retry，也不进入 K3-B2。执行顺序为：

```text
v3 plan freeze
-> architecture gate strict
-> 只读架构抽样复核
-> 必要 P0/P1 修补
-> K6 fresh Tauri window 核心截图
-> K6 evidence / handoff / checkpoint sync
```
