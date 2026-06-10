# 第一版信息架构证据

## 结论先说

薄弱点：

- 第一版页面结构可以定，但不能把字段名写死成最终索引契约。依据：任务包要求给 UI 实现线提供页面结构；阶段计划只说明阶段 1 产出本地索引 JSON，没有在本任务允许读取材料里给出最终 schema。
- harness 的“实际作用”不能在第一版里自动判定。依据：当前对话里用户提出要记录识别 harness 作用、有用加强、没用废弃；但没有给出可验证的因果评估数据，第一版定位又是只读治理 Codex。
- Codex++ 类对话迁移、删除功能可作为后续蓝图参考，不能进入第一版主流程。依据：当前对话静态检查过 Codex++ 应用，看到迁移、删除、备份、撤销等功能痕迹；但阶段计划第一版明确不写 Codex 状态库。

可用结论：

- 第一版按 Codex 只读治理设计，不做多 agent 统一控制。依据：`product-line/README.md` 当前定位和第一版目标。
- 第一版页面最少需要：首页、项目页、会话页、skills 页、harness 页、任务线页。依据：任务包验收标准要求明确这些页面展示字段。
- 页面组织应以项目为主轴，Codex 为第一版唯一 agent。依据：当前对话里用户强调大型 ERP 和游戏开发是完全不同工作场景；阶段计划后续才评估 OpenClaw、VS Code、Claude Code。
- ERP 项目和游戏开发项目应展示不同的验证、风险和上下文字段。依据：用户明确说一个是大型 ERP 系统，一个是游戏开发；开发线文件也把 harness 和项目差异化展示并入信息架构线。

## 本轮读取范围

任务允许读取：

- `product-line/README.md`
- `product-line/STAGE_PLAN.md`
- `product-line/DEV_LINES.md`
- `codex-thread-context/019e6569-3663-7b62-a560-878c71d4de75/current-conversation.jsonl`

本轮额外只读了任务队列和既有回收文档用于对齐产物格式：

- `product-line/tasks/README.md`
- `product-line/RESULT_REVIEW.md`
- `product-line/handoffs/2026-05-27-codex-index-kernel-review.md`
- `product-line/handoffs/2026-05-27-index-kernel-validation-review.md`
- `product-line/evidence/2026-05-27-codex-index-kernel.md`

风险说明：

- 额外读取超出任务包列出的允许读取清单；本轮没有把这些额外文件当作业务需求来源，只用于确认项目内 handoff / evidence 写法和已有索引内核状态。
- 本轮没有读取密钥、授权文件、`.env` 或 Codex 原始状态库。

本轮写入：

- `product-line/evidence/2026-05-27-v1-information-architecture.md`
- `product-line/handoffs/2026-05-27-v1-information-architecture-result.md`

## 依据摘录

当前产品定位：

- 第一版是“只读治理 Codex”。依据：`product-line/README.md` 写明先做 Codex 治理工作台，第一版读取本机 Codex 会话、项目、skills、harness 相关信息。
- 第一版不做嵌入 Codex 聊天窗口、不自动改写 Codex 状态库、不自动安装 skills、不跨工具统一控制。依据：`product-line/README.md` 的第一版不做清单。

阶段边界：

- 阶段 1 负责生成本地索引 JSON，把项目、会话、skills、harness 候选信息关联起来。依据：`product-line/STAGE_PLAN.md` 阶段 1。
- 阶段 2 才实现桌面应用壳，展示项目列表、会话列表、skills 列表、任务线状态，并支持打开文件夹、复制路径、定位日志。依据：`product-line/STAGE_PLAN.md` 阶段 2。
- 阶段 3 才登记开发线、生成任务包、回收结果摘要、标记 current / paused / historical / superseded。依据：`product-line/STAGE_PLAN.md` 阶段 3。
- 阶段 4 才允许严格确认下写项目内配置，且不写 Codex 内部状态。依据：`product-line/STAGE_PLAN.md` 阶段 4。

开发线边界：

- 桌面应用线负责展示项目、会话、skills、harness、任务线状态，以及打开文件夹、复制路径、定位日志等低风险操作。依据：`product-line/DEV_LINES.md`。
- 桌面应用线禁止嵌入 Codex 聊天窗口、自动写入、展示密钥或 `.env` 内容。依据：`product-line/DEV_LINES.md`。
- Skills 与 Harness 盘点线在原型阶段不再作为独立常设线，harness 和项目差异化展示并入信息架构线。依据：`product-line/DEV_LINES.md`。

用户需求来源：

- 用户先把范围收窄到 Codex。依据：当前对话中用户说“我想先做接入 codex 的版本，先把 codex 管理好”。
- 用户提出 skills 来源混杂、自制 harness、ERP 和游戏项目差异大。依据：同一段用户消息。
- 用户提出最终蓝图要可靠、清晰、好管理，第一版具体外观不是重点。依据：当前对话中用户说“第一版我认为无所谓怎么设计，我们需要最终蓝图可靠，重点是清晰好管理”。
- 用户提出要记录识别 harness 实际作用，并有移动端统一管理界面。依据：当前对话中用户说“记录识别 harness 实际的运行作用”和“移动端统一的管理界面”。

## 已知

- 第一版只接 Codex。依据：README 和用户明确收窄。
- 第一版是只读治理，不写 Codex 状态库。依据：README、STAGE_PLAN、DEV_LINES。
- 桌面应用线需要页面结构，而不是 UI 框架结论。依据：任务包禁止引入具体 UI 框架结论。
- 页面必须区分 ERP 和游戏项目。依据：任务包目标和用户原话。
- 第一版页面要服务后续 UI 实现线。依据：任务包目标。

## 未知

- 索引内核修复后最终字段名。依据：索引内核 hardening 仍在待派发，阶段 1 尚未完全产品化。
- 项目类型如何可靠识别。依据：当前没有项目级配置或分类规则作为权威来源。
- harness 入口和作用数据如何采集。依据：当前只知道有自制 harness，不知道每个项目内 harness 文件、命令、日志和历史效果结构。
- skills 适用项目如何判定。依据：当前只知道有 GitHub 下载和自制 skills，未建立适用性评分或人工标注。
- 移动端连接电脑的协议和权限边界。依据：移动端是最终蓝图需求，不在阶段 1 / 2 目标内。

## 假设

- UI 实现线会读取阶段 1 的只读索引 JSON，而不是直接读 Codex 内部文件。依据：阶段 1 目标是生成本地索引 JSON；阶段 2 目标是桌面应用展示。
- 项目页是第一版核心入口。依据：用户的混乱来自不同项目和不同工作场景，而不是单纯缺少 agent 列表。
- 第一版可以展示“候选 harness”和“未知作用”，但不能自动判定废弃或加强。依据：只读阶段没有足够因果证据。

## 不作为事实的内容

- 不断言 Codex 内部数据结构长期稳定。依据：README 风险和阶段计划均要求读取逻辑可失败、可降级。
- 不断言某个 skill 或 harness 适合某项目。依据：没有历史效果数据和人工确认。
- 不断言移动端一定在第一版可做。依据：阶段计划没有移动端目标。
