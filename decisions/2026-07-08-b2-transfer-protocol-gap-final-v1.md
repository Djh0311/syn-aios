# 决策:B2 中转协议差量定稿(C0 回交裁定·七拍)v1

日期:2026-07-08 · 拍板:主导线(依 C0 调研 `docs/research/2026-07-08-agent-collab-transfer-reference-for-b2-v1.md`·已核实物:恒空四字段/启发式/未接线声明三处亲验)· 上承:`decisions/2026-07-08-phase-b2-execution-loop-final-v1.md`。

## 七拍

1. **契约严格度 = 分级(§1.3 待决策点)**:完成类汇报**保持软着陆**(黄牌哲学:呈现不阻断,链不因措辞卡死);**求助类信号 = 强信号,不可被软着陆**——落地规则:① 求助是与完成汇报**并列的一类回程报文**(status=blocked + 结构化 help 块),不是完成汇报里的可选字段;② 解析到求助 → 该任务进等待、链停该任务不崩、主管必见;③ **疑似求助但解析失败**(报文含 blocked/求助特征而 json 坏)→ 保守升级「疑似求助·主管必看」+ 任务停,**不许**降级成普通 warning;④ 完成汇报解析失败照旧软着陆。
2. **求助信号三条并行 → 收敛为一条(§5.3)**:**实现A(worker_report 契约链)为唯一真源**——C3 升级契约让四个恒空字段有真源;实现B(`derive_subagent_reports`)的 `contains("direction")` 启发式**退役**,读模型改为只投影真值(过渡期必须标注「派生·非自述」);第三机制 `unresolved_direction_risk` bool 在 C3 核写入侧后**接真源或删**。原则:**单一真源,读模型只投影不发明**。
3. **Codex `multi_agent`/`memories` = 双双入「不吸收」清单**(§5.10 事实增补·I0 之后新特性):不让 Codex 自治 spawn 替代工作台派发链;其原生 memories 与工作台记忆层**互不相认**。**并给 C1 加一条运行时防护实测**:exec/resume 路径下这两特性可否 per-exec 关闭(config flag),worker 会话会不会自发子 agent——能关则关,关不了则实证 exec 面不触发并记录风险与缓解;做不到两者之一 → C1 停手报回。
4. **前序口供摘要 = 独立新通道**(§5.8 M4 不能接·采纳):prepare 物化直拼(质量债重拆先例的链内正向版),**不**走「口供先转正式记忆再召回」(每任务过确认门 = 治理成本荒谬);`available_memory_refs` 照旧走 Builder 正式记忆通道——两通道并行、语义不混。
5. **C2 首务 = 三层命名统一**(§5.1):统一到正本 §3.4 命名(`task_goal`/`allowed_read_scope`/`report_format`),物化层随之改名或建显式映射表;新字段(`timeout_policy`/`failure_policy`/`available_skills`/`available_knowledge_refs`)只许落在统一后的命名体系里;`forbidden_actions`/`model_id` 从硬编码改按任务可配置。
6. **C4 缺口确认**(§5.6/§5.7):补「主管七查+终标」独立治理动作(现状=链解析成功即自动 completed);补 failed 后**四选一**(重试/退回/换会话/结束——现状只有"结束");`workflow_transition_allowed` 等未接线声明作设计起点;**知悉记录**:C4 主管终标是完成判定、**不构成对抗式复核**(单一判定者),审查智能体可选位照旧后置。
7. **C5 词表对齐**(§5.2):链 `workflow_chain_node_*` 命名空间向正本 13 词表建映射分支;`entry_type` 裸 String 升运行时校验。

## 附带裁定

- dispatch 可逆终态(cancelled)随 **C3** 落(waiting_decision 机器同批·前拍「并入 C1/C3」收窄到 C3);
- C0 报告转正为 B2 设计参考正本;报告中全部「未核·怎么核」项归各切片包内消化,不悬置。

## 生效

即日;C1 包同日出(含第 3 拍的运行时防护实测项)。
