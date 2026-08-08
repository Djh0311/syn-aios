# 决策目录状态入口

> `decisions/` 保存整个项目演化过程中的拍板、薄修订、专项实现选择、阶段现场、验收和授权记录。**文件位于本目录，不等于它现在仍是产品权威或执行授权。**

## 唯一判定入口

决定是否仍有现行效力，只看 [`../docs/product/authority-register-v1.md`](../docs/product/authority-register-v1.md) 的精确登记。当前产品净结论看 [`../docs/product/syn-product-canon-v1.md`](../docs/product/syn-product-canon-v1.md)；知识与所有智能体的关系另看 [`../docs/product/knowledge-infrastructure-canon-v1.md`](../docs/product/knowledge-infrastructure-canon-v1.md)。

当前权威登记明确保留的决定是：

- [`2026-08-09-syn-product-canon-authority-and-knowledge-infrastructure-v1.md`](2026-08-09-syn-product-canon-authority-and-knowledge-infrastructure-v1.md)：产品正本、材料分层和知识基础设施确认；
- [`2026-08-01-whole-workbench-event-driven-operating-model-amendment-v1.md`](2026-08-01-whole-workbench-event-driven-operating-model-amendment-v1.md)：事件驱动、个人 / 项目范围和角色运行方式；
- [`2026-08-01-memory-self-capture-daily-consolidation-and-skill-governance-amendment-v1.md`](2026-08-01-memory-self-capture-daily-consolidation-and-skill-governance-amendment-v1.md)：当前记忆自捕获、每日整理和技能候选政策；
- [`2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md`](2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md)：Syn 原生知识工作区及其界面路线。

如果权威登记以后增删决定，以登记表为准；不要只改本 README。

## 其余材料怎么读

- **已吸收来源**：角色、秘书、会话优先、两轴治理、共享传输、简单主流程、技术栈、画布、项目内交互等旧决定的净原则已经由产品正本或专题正本收敛。原文件保留来路，不再单独与新正本竞争。
- **专项实现来源**：精确工具数、旧界面布局、视觉例外、测试项目、一次性会话和局部存储选择，只能解释对应版本或兼容面；没有权威登记时，不得扩成全产品规则。
- **被修订 / 取代 / 停止**：正文或状态头已写明后稿、停止原因或保留部分的，只能按注明范围使用。典型包括 Obsidian 中心化 v1 路线和已停止的独立交办旁路重建。
- **阶段现场**：旧阶段顺序、测试项目放行、双线并行和旧开发护栏决定只记录当时现场，不定义今天的下一步。
- **验收记录**：M1 关闭拍板只证明 M1 的具名范围，不是产品正本，也不激活后续阶段。
- **过期授权**：M2 整体预授权已随 M2 关闭而失效，不能用于 M3 或任何新任务。

## 几个容易误读的文件

- [`2026-07-14-skill-harness-vocabulary-draft-v1.md`](2026-07-14-skill-harness-vocabulary-draft-v1.md)：文件名中的 `draft` 是历史路径；其已确认术语已由新正本吸收，原文件不是活动候选。
- [`2026-07-14-interaction-model-canon-v1.md`](2026-07-14-interaction-model-canon-v1.md)及 07-18、07-19 薄修订：只保留项目内项目主管界面的来源价值，不是整个工作台总交互正本。
- [`2026-07-23-supervisor-read-only-exact-five-capability-surface-v1.md`](2026-07-23-supervisor-read-only-exact-five-capability-surface-v1.md)：是当时某个项目主管配置的兼容合同，不是所有智能体的知识能力上限。
- [`2026-07-23-development-harness-operating-model-v1.md`](2026-07-23-development-harness-operating-model-v1.md)：旧开发护栏已经退出，正文中的旧权威路由和待派阶段失效。
- [`2026-07-23-knowledge-and-conversation-parallel-workstreams-v1.md`](2026-07-23-knowledge-and-conversation-parallel-workstreams-v1.md)：双线活跃只属于当时现场；当前没有活动工程阶段。
- [`2026-08-03-syn-m1-closure-acceptance-v1.md`](2026-08-03-syn-m1-closure-acceptance-v1.md)：验收记录。
- [`2026-08-03-syn-m2-blanket-authorization-v1.md`](2026-08-03-syn-m2-blanket-authorization-v1.md)：已过期授权记录。

## 新决定如何进入

1. 用户确认新产品要求后，先更新产品正本或对应专题正本；
2. 需要保留拍板来路时，再新增一份边界明确的决定；
3. 同步更新权威登记；未进入权威登记前，不宣称它是当前决定；
4. 尚未拍板的内容只进入 [`../docs/product/candidate-register-v1.md`](../docs/product/candidate-register-v1.md)，不要再散落一份自称当前的建议稿；
5. 被取代的文件保留历史来源和替代指针，不静默改写成“从未发生”。

工程是否可以执行，仍由当前用户指令、`AGENTS.md`、当前轻量开发护栏链和授权文件共同决定；本目录不提供执行授权。
