# 决策兼容入口

> 本文件保留给仍查找 `docs/decisions.md` 的旧工具和历史链接。它不是第二份决策清单，也不把 `decisions/**` 整个目录升格为当前权威。

当前入口：

- [权威登记](product/authority-register-v1.md)：精确判断哪些决定仍有现行效力，以及它们能决定什么；
- [决策目录状态说明](../decisions/README.md)：解释目录内历史、来源、验收和过期授权材料怎么读；
- [Syn 产品正本](product/syn-product-canon-v1.md)：收敛已经确认的完整产品要求；
- [候选登记](product/candidate-register-v1.md)：唯一开放候选入口，候选不是权威；
- [当前事实](current-state.md)：说明当前主线和实现证据上限。

只有权威登记精确列为当前有效的决定才继续参与对应专题判断。其余决定按目录说明中的历史、来源、验收、被取代或过期授权状态使用，不能因为文件仍位于 `decisions/` 就恢复效力。

工程执行另按当前用户指令、工作区与项目 `AGENTS.md`、`docs/harness/plan.md`、活动阶段、唯一活动叶和 `docs/harness/authorization.json` 判断。产品正本、决定、候选、计划和历史授权都不能自行激活工程。
