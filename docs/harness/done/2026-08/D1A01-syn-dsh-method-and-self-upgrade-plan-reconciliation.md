# D1A01 DSH 方法吸收、原生核心决定与 M1–M11 / 自升级计划校准

阶段：stage-13 DeepSeek Harness 方法吸收、Syn 原生核心与自升级计划校准

目标：在 5600X WSL 权威仓库中建立官方 DSH 研究、原生核心决定、架构校准、M5–M10 方法吸收和 M11 自升级计划；不修改产品代码。

允许路径、禁止项、完成标准与退场规则精确继承 stage-13，不得扩大。

## 已完成步骤

1. 已冻结远端 `main@9103c3b26b060e854be119a8cedaa856a2a900ce`、dirty path 与所有目标 preimage；
2. 已在不读取仓库旧报告的前提下独立研究官方 DSH / Cordis 资料，随后读取并校准 2026-08-14 旧报告；
3. 已同步决定、产品正本、权威登记、架构、研究索引、master、M5–M10、M11 与 current state；
4. 已验证相对链接、Markdown 围栏、状态词、M1–M11 映射和 targeted diff；
5. 已归档本叶与 stage-13，stage-12 / D0C04 / D0C05 保持原状态。

## 完成回执（2026-08-16）

- 结果：`D1A01_DOC_PLAN_RECONCILIATION=PASS`；
- 新增 4 份正式内容：原生核心与自升级决定、DSH 历史报告归档、DSH / AI OPC 官方研究报告、M11 受治理自升级计划；
- 同步 15 份既有内容：产品正本、权威登记、系统架构、研究与计划索引、master、M5–M10、自演进旧研究和当前状态；
- 核心结论已统一为：Syn 原生持有不可被候选自动改写的治理根和默认执行核心；DSH 仅作为可选 AgentRuntime 适配器与方法来源；自升级必须走候选、隔离评测、独立验收、灰度、签名提升与回滚；
- 远端主补丁 `git apply --check`、targeted `git diff --check` 均通过；19 / 19 文件存在，相对链接与 Markdown 围栏通过，8 项关键边界断言通过；
- 应用主补丁后 status 从 79 条变为 98 条，新增的 19 条精确等于本叶 19 个正式文档目标；
- `authorization.json` 保持 `authorized:false`；stage-12、D0C04、D0C05、候选登记、Primary / Edge 候选稿和 WSL 迁移计划哈希保持不变；
- 未改产品源码、测试、依赖、运行数据、凭据或设备状态；未执行 Git add / commit / push / merge / rebase / reset / clean / stash，也未部署或发布。
