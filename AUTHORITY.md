# Current Authoritative Documents（精简版 · 2026-07-26）

> 本文件是唯一人工权威索引，不复制 CURRENT 或计划正文。协作与安全规则以 AGENTS.md 为准；当前事实和唯一下一步以 CURRENT.md 为准。

## 一级入口

- AGENTS.md：协作规则与安全边界。
- CURRENT.md：当前短视图和唯一下一步。
- backlog.md：想法收纳，不是执行顺序。
- decisions/：已经拍板的边界。
- README.md：产品线定位。

## 当前业务路由（默认）

- decisions/2026-07-23-knowledge-and-conversation-parallel-workstreams-v1.md：当前执行拓扑决策；知识库与对话底座同时 active，但共享承重文件和同一真实 store 的运行验收不得并发。
- decisions/2026-07-23-supervisor-read-only-exact-five-capability-surface-v1.md：当前主管只读 MCP 工具面正本；精确五项为 `submit_proposal + knowledge_search/read/open/cite`，历史单工具验收谓词不再用于新运行。
- decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md：当前 L3 第二片路线正本；07-25 修订为 Syn 原生底座上的 Obsidian 核心桌面高保真界面，真嵌入、受限品牌资产、插件生态和私有 API 仍排除。
- docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md：当前 N0-N6 小阶段计划；R0、R1、R2、R3A、R3B 及经 R3C-R1 修正后的 R3C Canvas-first synthetic 范围均已获指导接受；当前只授权 R3D Graph 收敛，活动栏、右栏与真实 App 仍未授权。
- docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md：当前 N2R-R0 设计参考；已冻结 `1.12.7` Public、Default light 的核心结构/交互、Syn 品牌替换、双容器迁移与 `984 × 768` 真实参照。
- docs/design/2026-07-14-syn-redesign-design-brief-v1.md：Syn 全局设计语言正本；知识工作区的结构/比例按冻结 Obsidian 参考，品牌、颜色、字体与非知识页面继续按 Syn 设计语言。
- tasks/2026-07-25-l3-syn-n2r-r1-single-shell-convergence-package-v1.md 与 evidence/2026-07-25-l3-syn-n2r-r1-single-shell-convergence-offline-verification-v1.md：N2R-R1 React-only 单壳结构、响应式状态和折叠可访问性离线正本；指导结论为 `ACCEPTED_N2R_R1_OFFLINE / NOT_REAL_APP_ACCEPTED`。
- tasks/2026-07-26-l3-syn-n2r-r2-isolated-browser-visual-baseline-package-v1.md 与 evidence/2026-07-26-l3-syn-n2r-r2-isolated-browser-visual-baseline-v1.md：N2R-R2 synthetic-only 基线、隔离/量尺/焦点、Search 结果态和完整 P1/P2 差距矩阵正本；指导结论为 `ACCEPTED_N2R_R2_BASELINE / NEEDS_N2R_R3_VISUAL_CONVERGENCE / NOT_REAL_APP_ACCEPTED`。该接受不授权 R3 产品修复或真实 Syn/Tauri App。
- tasks/2026-07-26-l3-syn-n2r-r3a-search-and-overlay-convergence-package-v1.md 与 evidence/2026-07-26-l3-syn-n2r-r3a-search-and-overlay-convergence-verification-v1.md：R3A 施工与指导复核正本。Search/command/quick-open 行为、隔离和 `900×760` 量尺成立；Quick Open/Command 的不透明 backdrop 删除底层上下文，指导裁决为 `NEEDS_N2R_R3A_REWORK / NOT_ACCEPTED`。本条不授权返工、后续 R3 或真实 App。
- tasks/2026-07-26-l3-syn-n2r-r3a-r1-overlay-layering-rework-package-v1.md 与 evidence/2026-07-26-l3-syn-n2r-r3a-r1-overlay-layering-rework-verification-v1.md：R3A-R1 一行 CSS、red/green 浏览器合同、02/03 新图和指导复核正本；指导结论为 `ACCEPTED_N2R_R3A_R1_OVERLAY_LAYERING / ACCEPTED_N2R_R3A_SEARCH_OVERLAY / NOT_REAL_APP_ACCEPTED`。本条不授权 R3B、Rust、真实数据或真实 App。
- tasks/2026-07-26-l3-syn-n2r-r3b-central-tab-groups-and-split-package-v1.md 与 evidence/2026-07-26-l3-syn-n2r-r3b-central-tab-groups-and-split-verification-v1.md：R3B 施工、synthetic 浏览器取证与指导复核正本；指导结论为 `ACCEPTED_N2R_R3B_CENTRAL_TAB_GROUPS_AND_SPLIT / NOT_REAL_APP_ACCEPTED`。只接受单排真实中央标签组、最多两个左右组、单一草稿双投影、可调分隔与偏好迁移范围；Graph/Canvas 内容视觉、后续 R3、Rust、真实数据与真实 App 均未授权。
- tasks/2026-07-26-l3-syn-n2r-r3c-canvas-first-convergence-package-v1.md 与 evidence/2026-07-26-l3-syn-n2r-r3c-canvas-first-convergence-verification-v1.md：R3C 主施工与 synthetic 浏览器证据正本；其 Canvas-first 主结构、隔离、量尺和十张图成立。原先唯一回焦缺口由下一条 R3C-R1 关闭后，指导最终接受上游 R3C 的 synthetic 范围；不代表真实 App、完整 R0 或发布通过。
- tasks/2026-07-26-l3-syn-n2r-r3c-r1-file-panel-opener-focus-return-rework-package-v1.md 与 evidence/2026-07-26-l3-syn-n2r-r3c-r1-file-panel-opener-focus-return-verification-v1.md：R3C-R1 actual opener、取消/选择焦点策略、red/green 浏览器矩阵和指导复核正本；指导结论为 `ACCEPTED_N2R_R3C_R1_FOCUS_RETURN / ACCEPTED_N2R_R3C_CANVAS_FIRST / NOT_REAL_APP_ACCEPTED`。
- tasks/2026-07-26-l3-syn-n2r-r3d-graph-convergence-package-v1.md：当前唯一 active 的知识前端施工包；只授权 `KnowledgeGraphView.tsx`、Graph 对应 `styles.css` selector、`knowledge-graph.test.tsx` 与新 Graph synthetic evidence，把卡片阵列收为轻量节点/连线网络，并补节点键盘打开、焦点/ARIA 和窄宽量尺。Canvas、组壳、fixture、runner、活动栏、右栏、Rust、真实数据与真实 App 全部冻结。
- tasks/2026-07-23-l3-syn-native-knowledge-workspace-development-package-v2.md：原 N0-N6 开发合同和既有离线证据入口；其旧 UI 禁止项已被 07-25 路线修订取代，但本包不自动授权 N2R 实现。
- evidence/2026-07-23-l3-obsidian-embedding-feasibility-and-route-selection-v1.md：历史路线勘查与本机安装探针证据；解释为何停止真嵌入，不再决定当前实施。
- evidence/2026-07-23-l3-syn-native-knowledge-workspace-offline-verification-v2.md：N0-N5 离线实现、N6 fail-closed 只读 capability 与统一离线门证据；不代表真实 App 通过。
- evidence/2026-07-23-l3-syn-native-knowledge-workspace-real-app-acceptance-v2.md：N6 十二项真实 App 验收的安全停点记录；受信任 host dispatch 未获授权实现，十二项均未执行。
- evidence/2026-07-23-l3-syn-native-knowledge-workspace-guidance-review-v1.md：指导线独立复核裁决；N0-N5 离线通过，N6 安全停点成立但真实 App 未通过。
- evidence/2026-07-23-l3-knowledge-open-host-owned-relay-offline-verification-v1.md：执行线 R1 停点记录；relay WIP 局部测试通过，但 durable argv、raw stdout/stderr 与 pre-registration sink 未关闭。
- evidence/2026-07-23-l3-knowledge-open-relay-r1-blocker-guidance-review-v1.md：指导线独立确认上述 sink，并补充 outer safe-attempt 登记失败未回收 child/attempt 的 cleanup catch。
- tasks/2026-07-23-l3-knowledge-open-relay-secret-sink-closure-repair-package-v1.md 与 evidence/2026-07-23-l3-knowledge-open-relay-secret-sink-closure-offline-verification-v1.md：已获指导线独立验收的离线安全返工正本；只证明 secret sink、spawn 前 raw 闭锁、失败清理和 host-only recovery，不代表真实 App 通过。
- evidence/2026-07-23-l3-knowledge-open-host-owned-relay-offline-verification-v2.md：恢复后的执行线离线记录；§4.3 的 2 + 6 个格式 block 与 R3 离线门已通过，shape 历史债单列；其“待指导验收”停点已由下一条指导验收正本解除。
- evidence/2026-07-23-l3-knowledge-open-host-owned-relay-r3-guidance-acceptance-v1.md：R3 指导验收正本；独立复核 2 + 6 格式 diff、核心 Rust/前端/格式门并接受 R3，只解锁 fresh Gate 0，不代表真实 App 通过。
- evidence/2026-07-23-l3-syn-native-knowledge-workspace-real-app-acceptance-v3.md：当前 R4 pre-Gate-0 安全停点正本；Syn 首屏自动呈现既有非验收 store 面，故未发主管首句、未调用 MCP、未读 vault、未执行十二项，不代表任何真实验收通过。
- tasks/2026-07-23-l3-knowledge-open-host-owned-relay-and-real-app-acceptance-package-v1.md：当前知识线执行包；R3 已获指导验收，R4 已在 pre-Gate-0 触发既有 store 面安全停点，等待明确隔离启动边界后才可重派。
- evidence/2026-07-24-l3-r4-pre-gate0-existing-store-surface-guidance-review-v1.md：指导线对 pre-Gate-0 停点的独立裁决；确认停止正确，并证明只改 app-data 不足，隔离还须覆盖 repo index/tasks、真实 Codex DB、workflow/vault/recovery/canvas。
- tasks/2026-07-24-l3-syn-r4-isolated-runtime-profile-preflight-package-v1.md 与 evidence/2026-07-24-l3-syn-r4-isolated-runtime-profile-preflight-verification-v1.md：知识线 pre-Gate-0 隔离技术正本；runtime profile、bundle-integrity 红绿合同与离线门已完成。v4/v5 的旧 fresh bundle 在 UI 枚举前 `SIGKILL`；manual dev 对照排除了 isolated profile 是充分原因，最小修复随后要求 fixed-path codesign seal + deep/strict verify。v6/v7 两轮 sealed fresh bundle 分别存活至少 88/90 秒并由 SIGTERM/SIGINT 受控结束，receipt 均记录 launcher 未主动 kill、父子/process-group/session 关系且无本线残留；两轮均未调用 UI 或触碰真实 store。技术状态仍为 `PENDING_AUTHORIZED_I5_HOME_ONLY_UI_DISCOVERY`，但 07-25 路线修订已把实际执行延后到 N2R 离线收口之后；Computer Use、Home 读取/截图、主管首句、Codex CLI/MCP、工具和十二项仍未授权。
- tasks/2026-07-23-shared-conversation-transport-parallel-restart-audit-package-v1.md 与 evidence/2026-07-23-shared-conversation-transport-parallel-restart-audit-v1.md：已完成并经指导线验收的对话底座恢复审计；保留真实根因未知、共享资源冲突和三句合同输入，不代表真实 App 通过。
- tasks/2026-07-23-shared-conversation-transport-real-app-reacceptance-package-v2.md：已冻结但 HOLD 的下一次三句真实 App 重验合同；知识 relay 前置已满足，当前只等待用户新的真实运行授权与无并行 writer 窗口。
- decisions/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-v1.md：当前对话底座架构正本。
- docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md：唯一业务执行计划；L3 第二片当前打开，conversation 真实 App 替代性验收保留挂账。
- tasks/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-offline-implementation-package-v1.md：已收口离线实施合同，不授予真实 App、真实 store 或自动续跑。
- evidence/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-offline-verification-v1.md：离线收口证据，不代表真实 App 通过。
- tasks/2026-07-23-shared-conversation-transport-real-app-substitution-acceptance-package-v1.md 与 evidence/2026-07-23-shared-conversation-transport-real-app-substitution-acceptance-v1.md：当前真实 App 失败裁决；首句止于 binding 未持久化，后两句未执行。下一步以 CURRENT 的 binding 建立链离线修复为准，不自动重跑 App。
- tasks/2026-07-23-shared-supervisor-conversation-binding-phase-semantics-and-failure-closure-rework-package-v1.md 与 evidence/2026-07-23-shared-supervisor-conversation-binding-phase-semantics-and-failure-closure-rework-verification-v1.md：当前 binding 阶段语义、失败闭锁与临时 fixture 验证正本；真实 App 根因仍未知，也不授予重跑。
- tasks/2026-07-23-shared-supervisor-conversation-binding-establishment-repair-package-v1.md 与 evidence/2026-07-23-shared-supervisor-conversation-binding-establishment-offline-verification-v1.md：历史建立链/私有副本记录；其四阶段完整性与终结结论已被上述返工包纠正，不能单独作为当前裁决。
- handoffs/2026-07-22-jiaoban-conversation-module-reuse-and-syn-mcp-capability-guidance-v1.md：capability audit / 复用指导入口。
- handoffs/2026-07-22-knowledge-vault-production-write-path-status-sync-v1.md 与 decisions/2026-07-21-knowledge-vault-audit-production-write-path-v1.md：知识库当前交接与生产写路纠偏正本。

## 长期参考（非默认派发）

- /Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md 与同目录 local-ai-workbench-ui-blueprint-v1.md：产品蓝图外部源。
- principles.md：长期原则。
- docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md：产品现状说明书。
- docs/memory-layer-consolidated-canon-v1.md：记忆层正本索引。
- docs/workbench-frontend-display-boundary-v1.md：前端平台边界。
- docs/plans/2026-07-16-conversation-first-direction-and-execution-plan-v1.md：方向与原则正本，不是第二个业务执行计划。

## 并行开发治理（不参与业务排期）

- decisions/2026-07-23-development-harness-operating-model-v1.md：开发 Harness 长期运行模型。
- docs/plans/2026-07-23-development-harness-routing-code-map-and-authority-governance-remediation-plan-v1.md：并行开发治理计划；阶段状态仅以其原文为准，本索引不在本次对齐。
- docs/2026-07-23-development-harness-phase0-baseline-and-consumer-audit-v1.md：Phase 0 基线与 consumer 审计，不是运行时或业务验收 evidence。
- docs/project-context.json：新会话只读、fail-open 短路由事实；它只导航决策、任务包、计划、下一步和安全提醒，不替代任务包、CURRENT 或业务验收。

## 历史（只读，非默认路由）

- archive/2026-07-23-current-before-short-view-v1.md：CURRENT 短视图前的逐字节历史快照（SHA-256：8df14369d800aff3e42b08daf808cd9924a615c76f0db8877f4511e91cfa8b21）；仅用于历史核对或重建，不参与默认路由或派发。
- tasks/2026-07-19-s1b-h2-supervisor-syn-natural-information-flow-package-v1.md 及 07-22 R3B/R4E/R4F/R4F-R1 包：resident/private-home 主运输、诊断与 live 合同，保留但不重新派发。
- docs/plans/2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md、docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md、docs/plans/2026-07-11-orchestrator-fast-path-five-stations-plan-v1.md：历史或已收官计划，不是当前执行入口。
- evidence/2026-07-12-orchestrator-station3a-control-core-bridge-v1.md 与 evidence/raw/2026-07-12-station3a-binding-id-migration/：旧主管编排证据。

## Superseded / 停用

- decisions/2026-07-23-l3-obsidian-full-interface-in-syn-route-v1.md、docs/plans/2026-07-23-l3-obsidian-full-interface-and-maximal-integration-small-stage-plan-v1.md、tasks/2026-07-23-l3-obsidian-full-interface-and-maximal-integration-development-package-v1.md：已由 L3 原生知识工作区 v2 取代；只保留可复用 WIP 和历史裁决。
- codex-multi-agent-safe-collaboration.md：旧重型科学家代号复核线，已废弃。
- requirements-matrix、task-queue、open-questions、context-checkpoints、sprint-contract、current-state 与旧 docs/decisions.md：停用的全局流程文档，不构成当前强制入口；历史仍可在 Git 与 archive/ 查阅。
