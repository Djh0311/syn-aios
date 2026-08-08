# SYN-M2A-REMAINING-ONE-SHOT｜剩余 M2 一次性收敛任务包

你是既有 Syn M2 工作线主管。模型保持当前 gpt-5.6-terra / ultra。用户刚刚明确要求：**不保留内部阶段顺序与逐段验收；剩余 M2 一次做完，最后只回交一次总验收。**

## 0. 执行方式

- 这是一个 umbrella execution package，不再按“T2 验收后停一下 → T3 → T4 → 收口”逐段回交。
- T2 独立复验、T3、剩余 T4、M2 全量退出条件可按依赖关系并行/交错推进；中间不向主指导线请求 acceptance。
- 你自行按难度做多 agent 路由：证据审查、真实 HOME 只读核验、代码/测试可由独立 agent 并行；所有写入由单一主管串行协调，避免互相覆盖。
- 只有最终一次回交，给出一个总 verdict：PASS / PARTIAL(HOLD) / FAIL。
- 任一强制退出条件缺失，都不得把 M2 标成 COMPLETE；但某一项 HOLD 不妨碍把其他互不依赖的剩余项做完并如实汇总。

## 1. 固定起点与事实边界

执行前重新只读核验，不可只信本提示：

- 工作目录：/Users/yoyi/workspace/product-line-syn-fnd-002
- 分支：syn-fnd-002-dev
- 预期 HEAD：2a7229bde7f0b5bb6701f4a7aa21944973f1881f
- T2 已交付但尚未由主指导线独立验收。
- T4-A 已验收，只覆盖：
  1. preflight fixture bug；
  2. process fixture family。
- 剩余 T4 至少包括：
  1. real grant store + mint/load/verify；
  2. code-map advisory 清零；
  并必须按 M2 计划复核 forged report/grant 拒绝、Station 3b 拒绝语义；若机制属于后续阶段且当前不存在，只能给精确 HOLD/迁移边界，不能伪造完成。
- main 已在单机例外下快进到 e5269557b65998de56d09d83fa901c4bd92145bd；这只代表 FND-001 Git 集成，不代表 Harness closeout，也不代表 M2 完成。
- live Harness 仍为 generation 30、无锁，R1/I5 均 PARKED；本包不修改 live graph，不以其 PARKED 阻塞产品 M2 实施，也不宣称 Harness closeout。
- 当前 worktree 有 13 项既有战略愿景融合 WIP。执行前记录完整 status、staged 状态、路径和内容指纹；全部原样保留，不 reset/clean/stash/覆盖/批量 stage。它们不属于本包交付。

## 2. 必须一次性完成的工作面

### A. T2 独立总复验

对 commit 2a7229b 的六场景做独立事实核验，不照抄执行者报告：

1. 冷启动合法命令入 DB + JSON，SIGTERM 重启可读；
2. commit 前强退，四表/状态零半提交；
3. commit 后强退，DB committed + JSON stale，重启 fail-closed，数据不丢；
4. 投影失败返回真实错误，重启 replay 后转绿；
5. 重复 command 返回同 receipt_id，四表零新增；
6. JSON-leading 启动 fail-closed，DB hash 不变，JSON 不反向覆盖，降级写仅落 JSON。

检查绝对路径、PID/信号、日志、SQLite/JSON 前后读回、hash、故障门只在 debug/验收路径生效、普通路径惰性。现有证据不足时，按既有 M2 blanket authorization 在隔离 R4 root 重跑所需场景。不得用单元测试冒充真实隔离 App 证据。

### B. T3：DAT-001B 真实 live manifest

按 M2 权威计划完成真实 HOME 层核验：

- 仅限 CodexGovernanceWorkbench 管理的数据面，不得读取或写入 /Users/yoyi/.codex。
- 记录真实绝对路径、stat/hash、数据源类别、读取方式、结果与证据等级。
- 如果需要修改/删除真实 Workbench 数据，先在系统临时目录做完整外部副本，再执行；不得触及非 Workbench 数据。
- 必须得到真实 live manifest PASS，或形成精确且可复现的 HOLD。没有证据不得补写“通过”。

### C. 剩余 T4

落实并验证：

- real grant store：mint → persist → load → verify 的真实路径；
- 删除/封死 grant_id=dispatch_id、自签 wildcard 或等价伪授权路径；
- forged grant、forged execution report、过期/错 scope/错 subject 等拒绝路径；
- Station 3b 写入拒绝：若当前 session/attempt 机制已存在，给运行证据；若权威设计明确归 M3，写出精确接口边界与 HOLD，不得虚构；
- 新增/既有 M2 模块纳入 code map；相关 advisory 必须归零，包含 invalid domain path 的负向验证；
- 不接真实外部 provider，不扩大到 M3。

### D. M2 总体退出条件

对同一个 reference slice 做端到端闭环复核，不允许拼接不同样本：

- identity / actor / grant / prepared attempt；
- Unit of Work 与状态变更；
- denial audit；
- 当前快照；
- outbox；
- projector；
- shadow/parity/recovery；
- migration state 精确；
- rollback/export；
- isolated App crash/restart；
- common ports/schema/receipt/forbidden fields 已冻结且实现一致；
- 旧写路径关闭或有明确、受权威接受的迁移状态。

跑与变更直接对应的 focused tests、完整 Rust library check/test，以及权威计划要求的 Node/fixture/code-map/audit 验证。必须报告精确命令、exit、通过/失败/ignored/warning 数字，不能用局部测试冒充全量。

### E. 文档与证据

先把本消息原样落为：
/Users/yoyi/workspace/product-line-syn-fnd-002/tasks/2026-08-04-syn-m2a-remaining-one-shot-package-v1.md

随后允许在本包范围内新增/更新任务证据、验收记录和 /docs/harness/CURRENT.md。避免触碰 13 项既有 WIP 文件；若某个强制收口声明只能落在已被用户修改的 WIP 文件中，停止修改该文件，改在总回交中列为 DOC-OVERLAP，不覆盖用户内容。

只有全部强制退出条件都 PASS，才可把 M2 标为 CLOSED/COMPLETE；否则 CURRENT 必须保持 IN_PROGRESS，并准确列出 HOLD。无论结果如何，M3 都不得激活。

## 3. 授权与禁止

本包沿用并受限于：
/Users/yoyi/workspace/product-line-syn-fnd-002/decisions/2026-08-03-syn-m2-blanket-authorization-v1.md

允许：M2 范围产品代码、测试、fixtures、schema/migration、隔离 App 故障注入与强退、Workbench-owned data 的受控核验，以及本包/证据/CURRENT 写入。

禁止：

- 修改 /Users/yoyi/.codex、凭据或外部项目；
- 真实外部 provider；
- 修改 live Harness graph、manifest 或 Harness closeout 状态；
- merge、push、main/ref 更新、rebase、cherry-pick、reset、stash、clean；
- 覆盖或夹带 13 项既有 WIP；
- 宣称 OFF_MACHINE PASS（用户只有一台机器）；
- 把 FND Git 集成、同机 checkout、测试或 fixture 冒充 M2 产品完成。

本轮**不授权新 Git commit 或 staging**。完成全部实现与验证后保留清晰 diff，统一回交主指导线一次验收；commit 是否执行在验收后另行决定，不作为中间阶段门。

## 4. 最终一次回交格式

只在全部可做工作完成或遇到真正无法绕过的硬停止后回交一次，内容必须包含：

1. 总 verdict：PASS / PARTIAL(HOLD) / FAIL；
2. M2 是否 COMPLETE，逐条对应退出条件；
3. T2 六场景独立结论；
4. T3 live manifest 结论与证据等级；
5. T4 每个剩余项结论；
6. 实际改动文件与未改动边界；
7. 全部验证命令、exit 与精确数字；
8. 真实 App/PID/路径/hash/DB/JSON 证据索引；
9. before/after HEAD、index、status、13 项 WIP 指纹；
10. 未 commit、未 merge、未 push、未改 live Harness 的确认；
11. 若 HOLD：最小阻塞事实、已完成的其他项、解除 HOLD 所需的最小授权/外部条件。

达到上述最终回交点前，不需要主指导线逐段验收，也不要派发 M3。
