# Current Authoritative Documents（精简版 · 2026-07-28）

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
- docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md：当前 N0-N6 小阶段计划；N2R R0→R4 已完成 synthetic 收口并在 `a13599e` 入库，中央 chrome 为用户拍板的有意分歧；D1-R1 已获离线接受，D3-R1-R1 已获 synthetic 接受并关闭 D3 范围。I5-R2 已由用户真实确认 synthetic Home、知识工作台与非空 Graph；I5-R3 已获指导离线接受，真实组件已证明 Markdown basename 直接可见且点击保持 exact relative path。用户明确延期关系图视觉问题，并单独授权 I5-R4 一发用户肉眼 isolated real App，只复看 `00-index.md` 直接可见和可打开。UI 先行门、Gate 0 与 N6 仍未通过。
- docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md：当前 N2R-R0 设计参考；已冻结 `1.12.7` Public、Default light 的核心结构/交互、Syn 品牌替换、双容器迁移与 `984 × 768` 真实参照。
- docs/design/2026-07-14-syn-redesign-design-brief-v1.md：Syn 全局设计语言正本；知识工作区的结构/比例按冻结 Obsidian 参考，品牌、颜色、字体与非知识页面继续按 Syn 设计语言。
- tasks/2026-07-25-l3-syn-n2r-r1-single-shell-convergence-package-v1.md 与 evidence/2026-07-25-l3-syn-n2r-r1-single-shell-convergence-offline-verification-v1.md：N2R-R1 React-only 单壳结构、响应式状态和折叠可访问性离线正本；指导结论为 `ACCEPTED_N2R_R1_OFFLINE / NOT_REAL_APP_ACCEPTED`。
- tasks/2026-07-26-l3-syn-n2r-r2-isolated-browser-visual-baseline-package-v1.md 与 evidence/2026-07-26-l3-syn-n2r-r2-isolated-browser-visual-baseline-v1.md：N2R-R2 synthetic-only 基线、隔离/量尺/焦点、Search 结果态和完整 P1/P2 差距矩阵正本；指导结论为 `ACCEPTED_N2R_R2_BASELINE / NEEDS_N2R_R3_VISUAL_CONVERGENCE / NOT_REAL_APP_ACCEPTED`。该接受不授权 R3 产品修复或真实 Syn/Tauri App。
- tasks/2026-07-26-l3-syn-n2r-r3a-search-and-overlay-convergence-package-v1.md 与 evidence/2026-07-26-l3-syn-n2r-r3a-search-and-overlay-convergence-verification-v1.md：R3A 施工与指导复核正本。Search/command/quick-open 行为、隔离和 `900×760` 量尺成立；Quick Open/Command 的不透明 backdrop 删除底层上下文，指导裁决为 `NEEDS_N2R_R3A_REWORK / NOT_ACCEPTED`。本条不授权返工、后续 R3 或真实 App。
- tasks/2026-07-26-l3-syn-n2r-r3a-r1-overlay-layering-rework-package-v1.md 与 evidence/2026-07-26-l3-syn-n2r-r3a-r1-overlay-layering-rework-verification-v1.md：R3A-R1 一行 CSS、red/green 浏览器合同、02/03 新图和指导复核正本；指导结论为 `ACCEPTED_N2R_R3A_R1_OVERLAY_LAYERING / ACCEPTED_N2R_R3A_SEARCH_OVERLAY / NOT_REAL_APP_ACCEPTED`。本条不授权 R3B、Rust、真实数据或真实 App。
- tasks/2026-07-26-l3-syn-n2r-r3b-central-tab-groups-and-split-package-v1.md 与 evidence/2026-07-26-l3-syn-n2r-r3b-central-tab-groups-and-split-verification-v1.md：R3B 施工、synthetic 浏览器取证与指导复核正本；指导结论为 `ACCEPTED_N2R_R3B_CENTRAL_TAB_GROUPS_AND_SPLIT / NOT_REAL_APP_ACCEPTED`。只接受单排真实中央标签组、最多两个左右组、单一草稿双投影、可调分隔与偏好迁移范围；Graph/Canvas 内容视觉、后续 R3、Rust、真实数据与真实 App 均未授权。
- tasks/2026-07-26-l3-syn-n2r-r3c-canvas-first-convergence-package-v1.md 与 evidence/2026-07-26-l3-syn-n2r-r3c-canvas-first-convergence-verification-v1.md：R3C 主施工与 synthetic 浏览器证据正本；其 Canvas-first 主结构、隔离、量尺和十张图成立。原先唯一回焦缺口由下一条 R3C-R1 关闭后，指导最终接受上游 R3C 的 synthetic 范围；不代表真实 App、完整 R0 或发布通过。
- tasks/2026-07-26-l3-syn-n2r-r3c-r1-file-panel-opener-focus-return-rework-package-v1.md 与 evidence/2026-07-26-l3-syn-n2r-r3c-r1-file-panel-opener-focus-return-verification-v1.md：R3C-R1 actual opener、取消/选择焦点策略、red/green 浏览器矩阵和指导复核正本；指导结论为 `ACCEPTED_N2R_R3C_R1_FOCUS_RETURN / ACCEPTED_N2R_R3C_CANVAS_FIRST / NOT_REAL_APP_ACCEPTED`。
- tasks/2026-07-27-l3-syn-real-app-first-look-followup-package-v1.md 与 evidence/2026-07-27-l3-syn-real-app-first-look-followup-verification-v1.md：07-27 D1 种数据、D2 知识页通栏和 D3 对比度的施工记录；指导仅接受 D2 离线范围和 D3 token 分层（保留 `.feed-item-time` 正文残留）。该包的 D1 pre-profile 写入与 exit 78 是历史 blocker，当前已由下一条 D1-R1 的离线指导接受取代；这不等于真实 App 已复验。
- tasks/2026-07-27-l3-syn-d1-r1-post-profile-seeding-rework-package-v1.md 与 evidence/2026-07-27-l3-syn-d1-r1-post-profile-seeding-rework-verification-v1.md：四文件 post-profile seeding 返工、9 项合同、warning 身份与指导 A/B 归因正本。该包写授权已消费；当时裁决为 `ACCEPTED_D1_R1_POST_PROFILE_SEEDING_OFFLINE / FULL_LIB_GATE_BASELINE_ENVIRONMENT_FAILURE_REMAINS / REAL_APP_RETRY_NOT_AUTHORIZED`，意思是该离线包自身不自动重跑。随后 I5-R1 已在独立新授权中真实证明 profile 与 7 篇 seed materialization；全库 Rust 门仍红。
- tasks/2026-07-27-l3-syn-d3-r1-feed-time-contrast-and-r3e-successor-lock-package-v1.md 与 evidence/2026-07-27-l3-syn-d3-r1-feed-time-contrast-and-r3e-successor-lock-verification-v1.md：D3-R1 已消费并受控停止。static red 为 `6/1`，fixture 当时 HTTP 200；Chrome PID `66179` 在首个 context 前、macOS application registration 路径 `SIGABRT`，产品 CSS 未改，predecessor `126/3` 与 successor 均未形成。指导接受停止但不接受产品完成：`ACCEPTED_D3_R1_CONTROLLED_STOP / BLOCKED_AT_CHROME_APPLICATION_REGISTRATION_BEFORE_FIRST_CONTEXT / PRODUCT_UNCHANGED / NEEDS_NEW_AUTHORIZATION / NOT_REAL_APP_ACCEPTED`。
- tasks/2026-07-27-l3-syn-d3-r1-c0-chrome-capability-preflight-package-v1.md 与 evidence/2026-07-27-l3-syn-d3-r1-c0-chrome-capability-preflight-verification-v1.md：C0 已消费并经指导接受为受控部分结果。Chrome PID `62804` launch/clean exit 成立，但 runner 在首个 context 前调用 Playwright `Browser` 不存在的 `browser.process()`；完整 C0 不接受、产品未改、D3 重试未授权。
- tasks/2026-07-27-l3-syn-d3-r1-c0-r1-runner-api-correction-and-capability-preflight-package-v1.md 与 evidence/2026-07-27-l3-syn-d3-r1-c0-r1-runner-api-correction-and-capability-preflight-verification-v1.md：C0-R1 已消费并获指导接受。冻结 lane 的唯一 PID `79990` 完成 context/about:blank/DOM/close 与正常退出，产品、staged、端口和 DiagnosticReports 未变；裁决为 `ACCEPTED_D3_R1_C0_R1_CHROME_CAPABILITY / FROZEN_LANE_MINIMUM_BROWSER_CAPABILITY_PROVEN / PRODUCT_UNCHANGED / D3_RETRY_ELIGIBLE_FOR_SEPARATE_AUTHORIZATION / NOT_REAL_APP_ACCEPTED`。
- tasks/2026-07-27-l3-syn-d3-r1-r1-feed-time-contrast-and-r3e-successor-retry-package-v1.md 与 evidence/2026-07-27-l3-syn-d3-r1-r1-feed-time-contrast-and-r3e-successor-retry-verification-v1.md：D3-R1-R1 已消费并获指导接受。只改 `styles.css` 两个窄 hunk，static `6/1 → 6/0`、R3E `126/3 → 126/0`；裁决为 `ACCEPTED_D3_R1_R1_FEED_TIME_CONTRAST_AND_R3E_SUCCESSOR_SYNTHETIC / D3_SCOPE_CLOSED / NOT_REAL_APP_ACCEPTED`。browser process 输出仅留摘要的非阻断 catch 已入账，不授权重跑。
- tasks/2026-07-27-l3-syn-i5-r1-isolated-home-only-post-profile-seeding-preflight-package-v1.md 与 evidence/2026-07-27-l3-syn-i5-r1-isolated-home-only-post-profile-seeding-preflight-verification-v1.md：I5-R1 已消费并获指导受控停止接受。PID/executable、build、receipt、隔离与 7 篇 seed 实物成立，证明真实 profile + post-profile seed materialization；Computer Use 未发现窗口，Home/AppState/window/知识 UI 保持未知。该包不得重试。
- tasks/2026-07-27-l3-syn-i5-r2-human-visible-home-and-knowledge-first-look-package-v1.md 与 evidence/2026-07-27-l3-syn-i5-r2-human-visible-home-and-knowledge-first-look-verification-v1.md：I5-R2 已消费并获指导窄范围接受。用户真实确认 synthetic Home、知识工作台和非空 Graph；物理 7 篇 seed 完整，但 `00-index.md` 在 UI 未找到、精确 7 篇 UI 计数 unresolved。指导裁决为 `ACCEPTED_I5_R2_PARTIAL_HUMAN_VISIBLE_HOME_AND_KNOWLEDGE_FIRST_LOOK / UI_00_INDEX_DISCOVERABILITY_REWORK_REQUIRED / GRAPH_VISUAL_REWORK_DEFERRED_BY_USER / NOT_UI_GATE_ACCEPTED / NOT_GATE0_AUTHORIZED`；旧包不得重试。
- tasks/2026-07-28-l3-syn-i5-r3-index-filename-ui-discoverability-rework-package-v1.md 与 evidence/2026-07-28-l3-syn-i5-r3-index-filename-ui-discoverability-rework-verification-v1.md：I5-R3 已消费并获指导离线接受。真实组件 red/green 证明 root/nested Markdown basename 直接可见、点击仍回传完整 relative path；typecheck、固定 37-entry runner、raw 34/34、冻结 SHA、dirty WIP、diff/staged 均成立。裁决为 `ACCEPTED_I5_R3_INDEX_FILENAME_UI_DISCOVERABILITY_OFFLINE / INDEX_FILENAME_COMPONENT_CONTRACT_PROVEN / GRAPH_VISUAL_REWORK_DEFERRED / REAL_APP_RECHECK_REQUIRES_SEPARATE_AUTHORIZATION`；旧包不得启动 App 或续写。
- tasks/2026-07-28-l3-syn-i5-r4-human-visible-index-filename-real-app-recheck-package-v1.md：**当前唯一 active**。用户已单独授权一次既有 isolated real App 启动，先由用户确认 synthetic Home、精确 identity 与零真实内容，再只确认文件树中字面 `00-index.md` 无需 hover 即直接可见并能只读打开。零产品写入；不数 7 篇，不进入 Graph/Canvas/Search/设置，不使用 Computer Use/Sky/浏览器/Vite/截图，也不进入 UI 先行门、Gate 0、N6 或十二项。
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
- tasks/2026-07-24-l3-syn-r4-isolated-runtime-profile-preflight-package-v1.md 与 evidence/2026-07-24-l3-syn-r4-isolated-runtime-profile-preflight-verification-v1.md：知识线 pre-Gate-0 隔离技术历史正本；runtime profile、bundle-integrity 红绿合同与离线门已完成，历史 v6/v7 sealed fresh bundle 分别存活至少 88/90 秒。07-27 第二次 preflight 因旧 D1 在 profile 初始化前写入必须为空的 `app-data` 而 exit 78；D1-R1 已修复且 I5-R1/I5-R2 已真实证明 post-profile seed。该历史包不是重跑入口；当前唯一新运行授权来自 I5-R4，只允许一发用户肉眼文件名复看，主管首句、Codex CLI/MCP、工具、Graph 与十二项仍未授权。
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
- handoffs/2026-07-27-syn-knowledge-and-conversation-guidance-handoff-v2.md：当前双线指导交接；记录 D1-R1 离线接受、D3-R1-R1 synthetic 最终接受、I5-R1 真实 profile/seed、I5-R2 用户真实 Home/知识第一眼的窄接受、I5-R3 文件名可发现性离线接受、关系图视觉延期、当前 I5-R4 单发用户肉眼真实 App 复看、全库 baseline 环境红、对话三句 HOLD、工作树与接手顺序。

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
