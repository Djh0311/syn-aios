# 阶段13 DeepSeek Harness 方法吸收、Syn 原生核心与自升级计划校准

总计划：product-line 唯一基线与 Harness Lite 切换

目标：根据用户当前明确要求，在 5600X WSL 权威仓库中落实 DeepSeek Harness / Cordis 官方研究，确认 Syn 不以 DSH 为核心依赖、原生持有治理与默认执行核心，并把受治理自升级纳入正式长期路线。全面校准架构、master、M5–M10 和新增 M11；不重开 M1–M4，不实现产品代码。

当前用户边界：

- 项目已经迁移到 5600X WSL，文档必须落实到 `/home/synadmin/workspace/syn`；
- 不直接采用 DSH 作为核心，借鉴它的实现方法建设 Syn 自己的核心；
- 用户早于 DSH 已提出自升级平台，本轮要把该方向正式纳入计划；
- 全面梳理计划，但不授权 DSH 安装、真实模型 / Provider、产品代码、外部动作、Git 提交 / 推送或发布。

干完的标准：

- 官方一手资料研究与发布日旧报告均进入 `docs/research/`，旧报告有校准状态，研究索引可发现；
- 当前决定、产品正本、权威登记与架构正本一致确认“原生治理根 + 可替换 AgentRuntime + 受治理自升级”；
- master 与计划索引覆盖 M1–M11，M1–M4 历史完成范围不重写；
- M5–M10 各自吸收与其 owner 对应的 DSH 方法和限制；M11 有独立阶段合同；
- `docs/current-state.md` 只记录 WSL 权威、D0D01 证据上限和文档计划事实，不冒充产品实现；
- 所有相对 Markdown 链接、状态词、M1–M11 引用和 targeted diff 通过；既有 dirty WIP 零覆盖；
- D1A01 与 stage-13 归档后无 current leaf；stage-12、D0C04 / D0C05 仍保持原状态。

允许动：

- decisions/2026-08-16-syn-native-governance-core-and-governed-self-upgrade-direction-v1.md [新增]
- decisions/README.md
- docs/product/syn-product-canon-v1.md
- docs/product/authority-register-v1.md
- docs/research/README.md
- docs/research/2026-07-09-self-evolution-frontier-and-syn-design-v1.md
- docs/research/2026-08-14-deepseek-harness-reference-research-v1.md [新增]
- docs/research/2026-08-16-deepseek-harness-ai-opc-reference-research-v1.md [新增]
- docs/workbench-system-architecture-v1.md
- docs/plans/README.md
- docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md
- docs/plans/2026-08-01-syn-stage-5-project-supervisor-and-execution-loop-plan-v1.md
- docs/plans/2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md
- docs/plans/2026-08-01-syn-stage-7-memory-personal-model-and-skill-governance-plan-v1.md
- docs/plans/2026-08-01-syn-stage-8-connector-and-credential-reference-plan-v1.md
- docs/plans/2026-08-01-syn-stage-9-read-model-migration-and-legacy-retirement-plan-v1.md
- docs/plans/2026-08-01-syn-stage-10-full-day-pilot-and-release-hardening-plan-v1.md
- docs/plans/2026-08-16-syn-stage-11-governed-self-upgrade-platform-plan-v1.md [新增]
- docs/current-state.md
- docs/harness/plan.md
- docs/harness/stages/stage-13.md [新增]
- docs/harness/leaves/D1A01-syn-dsh-method-and-self-upgrade-plan-reconciliation.md [新增]
- docs/harness/done/2026-08/D1A01-syn-dsh-method-and-self-upgrade-plan-reconciliation.md [新增]
- docs/harness/done/2026-08/stage-13.md [新增]

不许动：

- 产品源码、测试、依赖、lockfile、运行数据、SQLite、配置、凭据和设备设置；
- `docs/product/candidate-register-v1.md` 与 `docs/product/syn-primary-edge-core-distributed-runtime-architecture-candidate-v2.md` 的既有 WIP；
- stage-12、D0C04、D0C05、既有 Harness migration / audit / report / usage 字节；
- M1–M4 冻结合同与具名完成结论；
- Git add / commit / push / merge / rebase / reset / clean / stash、部署、发布和删除。

停止与退场：

- 任一目标文件远端 preimage 与冻结基线不符、同名新文件已出现、diff 越过允许路径或相对链接失败时停止，不覆盖、不猜合并；
- 不把 DSH Developer Preview、插件 seam、Session Log、Sandbox 或 worker 自报扩大成 AI OPC 已成熟；
- 完成后只归档 D1A01 与 stage-13；stage-12 和两个 unfinished 叶保持不变。

## 叶子

- [x] D1A01 DSH 方法吸收、原生核心决定与 M1–M11 / 自升级计划校准

## 退场结论（2026-08-16）

- `STAGE13_DOC_PLAN_RECONCILIATION=PASS`；
- 本阶段只形成研究、决定、架构与计划字节，没有安装或集成 DSH，也没有实现产品代码；
- stage-12 继续保持打开，D0C04 / D0C05 继续保持 unfinished；本阶段未扩大其授权或改写其证据；
- 详细变更与验证回执见同目录的 D1A01 完成记录。
