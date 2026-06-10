# Workbench UI Information Architecture And Developer Settings Plan v1

日期：2026-06-08

状态：UI Shell 重构开发文档。本文不改变权威入口、不推进阶段结论、不声明中间版本完成；用于指导下一轮前端 UI 清理、信息层级重排和开发者内容归位。

## 1. 背景

当前工作台已经积累了项目、智能体、记忆、知识库、运行日志、诊断、审计、真实 `codex-local` 探针、任务包和权限边界等大量能力。

问题不是“内容不够”，而是显示层级混乱：

- 主界面把用户工作对象、运行状态、治理证据、内部边界、开发者调试信息混在一起。
- 许多页面像阶段验收面板或内部状态面板，不像可日常使用的桌面工作台。
- 为了守安全边界，UI 过度展示了 `sidecar`、`adapter`、`raw status`、`.codex`、`不写正式记忆`、`不执行 Codex` 等内部说明。
- 普通用户、全局主管、开发者三种视角没有分清。

下一轮 UI 改造的第一目标不是换风格，而是重建信息架构。

## 2. 固定原则

### 2.1 桌面边界

本 UI 只面向桌面端 Tauri 工作台：

- 不做手机端 UI。
- 不做移动端适配。
- 不做 mobile-first responsive 设计。
- 不以手机浏览器或移动视口作为验收目标。
- 允许保留极窄窗口的基础防溢出回退，但不能发展成移动端产品设计。

### 2.2 产品不是调试台

默认主界面只服务用户完成工作：

- 看项目。
- 看智能体。
- 看 Skill。
- 看 Harness。
- 看运行中工作流。
- 看待处理事项。
- 看必要的结果和风险。

默认主界面不展示内部实现细节：

- 不铺 `sidecar` 文件名。
- 不铺 raw enum / raw status。
- 不铺 raw event。
- 不铺 schema。
- 不铺 adapter / provider / credential 细节。
- 不铺 evidence / handoff / audit path。
- 不铺数据库、store integrity、runtime log 大表。

这些内容不是删除，而是归位到 `设置 > 开发者` 或 `管理` 的详情层。

### 2.3 三种用户视角

UI 必须区分三种视角：

- 普通使用者：关心项目、智能体、运行中、待办、结果、记忆是否可用。
- 全局主管：关心权限、风险、回收、审计摘要、阶段结论、哪些不能声称完成。
- 开发者：关心 adapter、provider、sidecar、raw status、runtime log、diagnostics、schema、测试证据和内部边界。

普通主界面以普通使用者视角为默认；主管信息进入待办、管理、详情；开发者信息进入设置里的开发者内容。

## 3. 顶层入口

### 3.1 普通一级入口

左侧主导航保留：

- `项目`
- `智能体`
- `Skill`
- `Harness`
- `运行中工作流`

说明：

- `Harness` 可以使用英文，不强制翻译为“运行器”。
- `Skill` 可以使用英文，不强制翻译为“技能”。
- 这五个入口是用户日常工作对象，不是内部治理入口。

### 3.2 辅助入口

以下内容不作为普通一级入口展示，放入对应位置：

- `记忆`：如果保留一级入口，需要明确它是用户可理解的“记住了什么 / 候选是什么 / 被任务如何使用”，不是 store viewer。若主导航空间不足，可并入项目和设置中的二级入口。
- `知识库`：可以作为资料空间入口，也可以并入项目文档/知识页。不得和正式记忆混同。
- `设置`：固定放在左侧底部。

### 3.3 开发者入口

所有开发 / 内部边界信息统一进入：

```text
设置 > 开发者
```

不得再作为普通主导航、普通首页模块或普通项目页大块展示。

`设置 > 开发者` 默认折叠或弱化，只有用户主动进入时才显示。

## 4. 右侧全局入口

右侧固定入口保留：

- `秘书`
- `通知`
- `待办`
- `运行中`
- `管理`

职责如下：

- `秘书`：解释、整理、提醒、帮助用户理解影响面；不直接派发、不批准、不裁判、不写正式记忆。
- `通知`：发生了什么，例如读取状态、错误、完成、候选写入、诊断提醒。
- `待办`：用户需要处理什么，例如权限确认、方案确认、记忆候选确认、结果确认。
- `运行中`：当前正在做什么、是否卡住、是否等待权限、是否读回失败。
- `管理`：健康状态、审计摘要、日志摘要、权限摘要、数据位置。

右侧入口不得混成一个动态列表。

## 5. UI 内容分类和显示位置

### 5.1 主工作对象

内容：

- 项目。
- 智能体会话。
- Skill。
- Harness。
- 运行中工作流。

显示位置：

- 首页核心卡片。
- 左侧一级入口。
- 对应主页面。

显示规则：

- 显示用户能理解的名称、状态、数量、最近结果和下一步。
- 不显示 raw id、raw path、raw enum，除非进入详情。

### 5.2 决策和待处理事项

内容：

- 权限确认。
- 任务派发确认。
- 项目方案确认。
- worker 汇报确认。
- 过程事实确认。
- 记忆候选确认。
- 正式记忆生命周期操作确认。
- 失败、超时、读回异常需要用户处理。

显示位置：

- `待办`。
- 权限弹层。
- 秘书辅助解释。
- 对应对象详情页。

显示规则：

- 文案必须说明：要做什么、为什么、影响哪里、风险是什么、允许后会发生什么。
- 不能用 `允许一次` 这种过宽文案承载不同风险级别的操作。
- 不能让秘书看起来像批准者或裁判。

### 5.3 运行状态

内容：

- running。
- waiting_for_permission。
- ready_to_dispatch。
- ready_for_review。
- retry_pending。
- blocked_by_guard。
- readback_unavailable。
- readback_failed。
- timed_out。

显示位置：

- `运行中工作流` 首页入口。
- 右侧 `运行中`。
- 项目工作流节点摘要。
- 智能体会话详情。

显示规则：

- `readback unavailable / failed / timed_out` 显示为“读回不可用 / 读回失败 / 读回超时”。
- `result_count = null` 显示为“未知 / 不可用”，不得显示为“空”或暗示 0 条。
- 运行状态只显示摘要，详细日志进入管理或开发者区。

### 5.4 通知

内容：

- 索引读取状态。
- 项目 warning。
- 诊断 note。
- 运行关注提醒。
- 权限等待提醒。
- 最近错误。
- 候选写入成功提示。
- 正式记忆写入成功提示。

显示位置：

- 右侧 `通知`。
- 首页只显示少量摘要。

显示规则：

- 通知只说明发生了什么。
- 如果需要用户处理，应转成待办。
- 如果是运行过程，应转到运行中。

### 5.5 记忆和知识

内容：

- 正式记忆。
- 记忆候选。
- observation。
- 任务记忆包。
- lint / 冲突 / 过期。
- 实体关系。
- 成熟模式。
- 知识库资料。
- 知识库引用。

显示位置：

- 记忆页。
- 知识库页。
- 项目详情中的项目记忆/文档 tab。
- 任务节点详情里的任务记忆包摘要。

显示规则：

- 正式记忆、候选、观察、知识命中必须视觉上区分。
- observation 不是正式记忆。
- candidate 不是正式记忆。
- knowledge hit 不是正式记忆。
- 任务记忆包只显示摘要和使用理由；不把内部筛选过程铺在主画布。

### 5.6 治理证据和管理信息

内容：

- audit event。
- runtime log。
- diagnostic summary。
- store integrity。
- evidence / handoff 引用。
- 数据位置。
- 状态文件读取结果。
- 权限摘要。

显示位置：

- 右侧 `管理`。
- 对象详情页的“证据 / 审计”折叠区。
- `设置 > 开发者` 的详细区。

显示规则：

- 管理默认只显示健康摘要、最近错误、关键阻断。
- raw log、internal id、evidence path、store detail 默认折叠。

### 5.7 开发 / 内部边界信息

内容：

- adapter descriptor。
- provider availability。
- model credential boundary。
- command preview。
- raw status enum。
- sidecar path。
- 缺字段列表。
- schema。
- fixture。
- test-only boundary。
- “不写正式记忆 / 不执行 Codex / 不读 .codex”等长边界说明。

显示位置：

```text
设置 > 开发者
```

显示规则：

- 普通页面只保留必要的一句话状态，例如“该能力需要配置后才能使用”。
- 详细边界、禁止项和内部字段进入开发者详情。
- 不得在首页、项目页、智能体页铺成大段说明。

## 6. 页面调整方向

### 6.1 首页

首页只回答五个问题：

- 我有哪些项目？
- 哪些智能体 / 会话需要关注？
- 哪些 Skill / Harness 可用或待配置？
- 哪些工作流正在运行？
- 我现在需要处理什么？

首页不显示：

- sidecar。
- raw path。
- evidence / handoff。
- 长阶段结论。
- 开发者诊断大表。

### 6.2 项目页

项目页以项目为核心，不以任务包为核心。

建议结构：

- 左侧：项目列表。
- 中间：项目工作区。
- 项目内 tab：工作流 / 智能体 / 文档 / 记忆 / 设置。
- 右侧：当前节点或当前项目详情。

工作流 tab：

- 主区域只显示画布。
- 节点只显示摘要。
- 任务包、权限、readback、memory packet、audit/evidence 放进节点详情。

### 6.3 智能体页

智能体页显示：

- `codex-local` 当前可用状态。
- 会话列表。
- 会话阅读区。
- send / resume 的权限预览和状态。
- readback 状态。
- planned adapters 的不可用状态。

不显示为主内容：

- provider credential 细节。
- adapter descriptor 大表。
- model verification 内部字段。
- raw transcript。

这些进入 `设置 > 开发者` 或对象详情。

### 6.4 Skill 页面

Skill 页面显示：

- Skill 列表。
- 来源：系统 / 本地 / 插件。
- 可用于哪些智能体或项目的摘要。
- 是否缺少加载状态。

不显示：

- 插件内部 manifest 大表。
- raw path 大量堆叠。
- 未实现的加载 / 删除 / 编辑按钮。

### 6.5 Harness 页面

Harness 使用英文命名。

Harness 页面显示：

- Harness 资源。
- Harness 候选。
- 所属项目。
- 是否可用、未验证、缺配置。
- 适配的智能体 / runner 类型摘要。

不显示：

- 清单路径、说明路径、entrypoint、权限级别等完整字段大表作为默认内容。

这些进入 Harness 详情或 `设置 > 开发者`。

### 6.6 运行中工作流

运行中工作流是独立工作对象入口，不只是右侧状态摘要。

显示：

- 当前运行中的项目工作流。
- 等待权限的工作流。
- 失败 / 阻断 / 超时工作流。
- 最近回读状态。
- 可查看证据和详情。

不显示：

- runtime log raw entries。
- audit event raw list。
- command argv。
- secret / transcript / rollout 内容。

### 6.7 设置

设置分为：

- 通用。
- 项目路径。
- 数据位置。
- 权限。
- 外观。
- 开发者。

`设置 > 开发者` 包含：

- adapter descriptor。
- provider availability。
- model credential boundary。
- sidecar / store 列表。
- runtime log 详情。
- diagnostics 详情。
- schema / raw status。
- evidence / handoff 路径。
- fixture / test boundary。
- planned adapter 内部说明。

## 7. 第一轮开发切片建议

### UI-1：导航和信息归位

目标：

- 左侧主入口调整为项目、智能体、Skill、Harness、运行中工作流。
- 设置固定在底部。
- 开发 / 内部入口移入 `设置 > 开发者`。

验收：

- 普通主导航无 `开发 / 内部` 分组。
- 普通主导航无模型/凭据/工具等内部入口。
- Harness 英文显示。
- 设置底部可进入开发者内容。

### UI-2：首页重排

目标：

- 首页只保留五个核心对象摘要。
- 去掉长边界说明和内部索引口径。

验收：

- 首页能一眼看到项目、智能体、Skill、Harness、运行中工作流。
- 不出现 sidecar / raw status / evidence path。

### UI-3：右侧入口分工

目标：

- 通知、待办、运行中、管理分离。
- 管理默认摘要，详情折叠。

验收：

- 待处理事项不混进通知。
- 运行状态不混进待办。
- 审计和日志不作为顶级图标散开。

### UI-4：项目页降噪

目标：

- 项目工作流画布为主。
- 任务包、记忆包、权限、readback、audit/evidence 进入节点详情。

验收：

- 工作流主区域不是任务包管理器。
- 普通用户能看懂节点当前状态和下一步。

### UI-5：智能体页降噪

目标：

- 会话和运行状态为主。
- adapter/provider/model 内部边界移入设置开发者。

验收：

- planned adapters 不显示执行按钮。
- readback unknown 不显示为空结果。
- 秘书不显示为派发者。

## 8. 文案规则

禁止默认主界面出现：

- `sidecar`
- `raw`
- `schema`
- `store revision`
- `full transcript`
- `.codex`
- `adapter descriptor`
- `provider availability`
- `model credential boundary`
- `evidence path`
- `handoff path`

允许在开发者内容出现。

普通用户文案应该改成：

- “需要配置后才能使用。”
- “需要你确认后才会执行。”
- “读回不可用，结果未知。”
- “这是候选，不是正式记忆。”
- “这是资料引用，不是正式记忆。”
- “已阻断，原因见详情。”

## 9. 验收要求

每个 UI 改造任务至少运行：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

涉及布局、导航、右侧入口、首页、项目页、智能体页、设置或开发者内容时，必须做真实桌面窗口或浏览器截图验收。

如果没有截图工具，不得声称 UI 验收完成，必须写明：

```text
真实窗口 / 截图验收未完成。
```

## 10. 不在本轮解决

以下内容不通过 UI 重构顺手完成：

- 不新增真实 Codex 执行能力。
- 不改 Rust runner。
- 不改 workflow state schema。
- 不改 sidecar schema。
- 不新增 planned adapter 真实接入。
- 不新增 provider credential 管理。
- 不把中间版本声明为最终完成。

UI 重构只做显示层和信息架构归位。
