# Syn 产品正本入口

状态：**当前产品文档入口。**

这里回答“Syn 最终是什么、为什么做、哪些原则已经确定”。这里不记录当前开发进度，也不自动授权修改代码、运行真实智能体、访问真实账号、提交、合并或发布。

## 阅读顺序

1. [`syn-product-canon-v1.md`](syn-product-canon-v1.md)：产品正本，回答 Syn 为谁服务、核心能力、角色关系、工作闭环和反目标。
2. [`knowledge-infrastructure-canon-v1.md`](knowledge-infrastructure-canon-v1.md)：知识基础设施专题正本，回答所有智能体怎样获得所需资料和技能。
3. [`authority-register-v1.md`](authority-register-v1.md)：权威登记表，回答哪些材料现行、哪些只是计划、候选、历史或证据。
4. [`candidate-register-v1.md`](candidate-register-v1.md)：统一候选登记表，只保存仍待用户拍板的产品和架构问题。
5. [`../../decisions/2026-08-09-syn-product-canon-authority-and-knowledge-infrastructure-v1.md`](../../decisions/2026-08-09-syn-product-canon-authority-and-knowledge-infrastructure-v1.md)：本轮整理决定。

软件结构另看 [`../workbench-system-architecture-v1.md`](../workbench-system-architecture-v1.md)，界面显示边界另看 [`../workbench-frontend-display-boundary-v1.md`](../workbench-frontend-display-boundary-v1.md)，当前实现事实另看 [`../current-state.md`](../current-state.md)。

## 使用规则

- 当前用户的新指令永远优先；新决定与旧正文冲突时，按权威登记表和明确取代关系处理。
- 产品正本定义长期目标，不冻结具体表名、字段名、阶段拆法和界面像素。
- 候选登记不等于采纳，计划不等于授权，验收报告不等于产品定义。
- 历史材料可以解释来路，但不能因为仍在仓库里就恢复成现行要求。
- 开发护栏（Harness）、任务包、报告和交接只管理施工与证据，不重写产品方向。
