# CURRENT（短视图 · 2026-07-26）

> 只保留当前操作面。完整旧正文已逐字节冻结于 archive/2026-07-23-current-before-short-view-v1.md（SHA-256：8df14369d800aff3e42b08daf808cd9924a615c76f0db8877f4511e91cfa8b21）；规则见 AGENTS.md，唯一人工索引见 AUTHORITY.md。

## 一、现在真能用什么

- 共享 Conversation Transport + Syn capability plane 的离线实现与离线验证已收口：profile-driven transport、主管 read-only + 空写根、宿主可信 turn binding、服务端 registry/role allowlist 与交办页共享接线都在既定白名单内。
- 07-23 真实 App 替代性验收已执行但未通过：首句只新增 canonical recorded，主管 conversation turn binding 在 JSON/SQLite 均未持久化，零 injected/reply/卡/chain/worker；第二、第三句按停止合同未发。
- binding 建立链的阶段语义与失败收口已在临时夹具上复核：construct、store prepare、DB-primary persist、JSON projection、activate、transport start、terminate-unconfirmed 各自固定；transport/activate 失败仅在 JSON/SQLite 都确认 `Failed` 后才这样标记，终结失败只报告未确认并闭锁工具。前序私有副本 `25/0 → 26/1` 仅证明 Starting 写入；两类离线结果均不解释、更不等于修复真实 App 首句失败。
- L3 知识库第一片仍可用：Syn 自管 Markdown vault、浏览/手编、`[[双链]]`、确认式 AI 写入与 Batch 2 audit 已存在；第二片的 N0-N5 原生能力已离线收口。N2R-R1 已把旧纵向双容器收为唯一 React 单壳，并由指导线接受离线结构范围；视觉、高保真和真实 App 仍未通过。
- node scripts/harness/project-context.js --target . 的 READY 只是短导航可用，不是实施授权、业务完成或产品验收。

## 二、在做什么

- 用户 07-25 修订 L3 第二片 v2：Syn 原生底座、开放文件格式和安全边界不变；知识库主界面改为 Obsidian 核心桌面工作区的高保真复刻。R0 已在官方 `1.12.7`、Default、light、16 px、0 缩放和无隐私演示 vault 中完成 `984 × 768` 真实参照冻结，覆盖核心结构/交互、Syn 品牌替换和现有双容器迁移表；不嵌入 Obsidian，不复制商标/受限资产，也不复刻插件生态、私有 API、Sync、Publish 或移动端。
- 07-25 的 N2R-R1 React-only 单壳，以及 07-26 的 R2 synthetic-only 基线、R3A Search/overlay、R3B 中央标签组/左右分栏、经 R3C-R1 修正的 R3C Canvas-first 均已获指导接受。R3D Graph 已于 07-26 收口：执行线 `8 contexts / 75 assertions / 0 failed`（red 先行 `35 / 17`），指导线拷 runner 独立重跑得到 JSON 深度 diff 零差异、四图 SHA-256 逐张相同，并自算冻结 hash 与自跑全部门禁，结论 `ACCEPTED_N2R_R3D_GRAPH_CONVERGENCE / NOT_REAL_APP_ACCEPTED`。R3D 同时记账两条 catch：环形布局半轴恒为 `110/160`、与节点数无关，6 节点刚好排满、12 节点起互压（后端上限 512），规模外零覆盖；脏共享文件窄写只冻 hash、无基线副本，hunk 边界不可核。活动栏 icon ribbon 与右栏层级仍是剩余 R2 差距，完整 R0 与真实 App 均未通过。
- 用户已拍板知识库线与对话底座线并行：两线独立任务包、独立 evidence、独立验收；共享承重文件不得同时写，同一真实 store 的 App/MCP 验收不得并发。
- conversation-first 仍是总业务方向；07-22 对话底座和其真实 App 替代性验收保留挂账，但本轮不自动重跑，也不回到旧 resident/private-home 主路线。
- 已收口离线包与其验证证据只供复核；resident/private-home 主运输、S1B-H2/R3B/R4E/R4F/R4F-R1 均为历史参照，禁止重新派发或自动续跑。
- 其余长期事实、已完成线和历史逐包原文：先由 AUTHORITY.md 定位，完整原文见上述 archive。

## 三、并行下一步

- 知识库线：**R4 已收口获接受、N2R 视觉线收官（07-26），当前无 active 包、零新增写授权**。下一步不再是视觉包：先做**代码入库**（知识前端 + 未跟踪的后端 `knowledge_index.rs` 自 07-20 起全部未提交，需先复核后端与其他线 diff），再谈 isolated UI discovery → UI 先行门 → Gate 0。**指导线 07-26 实核补充**：`cargo check --lib` 通过（0 error / 598 warning，1m55s）；前端调用的 19 个 `knowledge_workspace_*` 命令在后端均有对应 `fn` 并经 `command_registry.rs` 登记进 invoke handler——接线在源码层通，但**从未真机验证**：N2R R1→R4 全部证据都出自浏览器夹具 + mock IPC。R3E 收口记录（保留供追溯）：结论 `ACCEPTED_R3E_D1_ACTIVITY_RIBBON / ACCEPTED_R3E_D2_RIGHT_CONTEXT / ACCEPTED_R3E_D3_GRAPH_SCALE`（synthetic）/ `NOT_REAL_APP_ACCEPTED`：活动栏收为 42px 图标条（三档零断行、八个可访问名称逐字未改）、右栏收为「属性 / 反向引用 / 来源上下文」三区可折叠（假的"大纲"已删字、注入方未改、折叠真退出 Tab）、Graph 环形布局随节点数分层（`1…512` 零重叠、512 为 13 环、大图按拍板只作俯瞰）。指导线亲跑：green `126/0` 且与执行线 JSON 深度 diff 零差异；自写探针 bundle 产品导出函数全量重算 `1…512`（按矩形重叠判据零对，最紧 `n=478 → 161.01`）；`shasum -c` 校验基线副本 7/7 后自行 diff 每个窄写文件、CSS 范围全在白名单；全部门禁自跑同结果；R3B/R3C/R3C-R1 目录与 R3D 四图/JSON 均未被覆盖。**下一段 = R4 最终视觉对照**（未起草、未授权）。
- **R4 已收口获接受，N2R 视觉线收官**（`ACCEPTED_R4_D2_BODY_TYPOGRAPHY` / `ACCEPTED_R4_D3_ICON_TOOLTIP` / `ACCEPTED_R4_D1_SKELETON_WITH_RATIFIED_DIVERGENCE`，synthetic / `NOT_REAL_APP_ACCEPTED`）：正文提到 16px（新增 `--text-body`，既有七档字号未动、chrome 字号逐项未变）、活动栏加与 `aria-label` 同值的 `title`（指导线自写 CDP 探针证实名称来源仍是 `aria-label`、`title` 被 superseded）、骨架七项在 R0 参照带内。指导线亲跑 green `73/0` 且与执行线 JSON 深度 diff 零差异，基线副本 3/3 校验后自行 diff，17 代码冻结件 + 6 冻结 runner 全 MATCH，门禁自跑同结果。
- **唯一超标项已由用户拍板接受为有意分歧**：中央 chrome `132.19` vs R0 `74`（文档头常驻"路径+标题+投影标签"两行块）。落 `decisions/2026-07-26-central-document-head-band-divergence-v1.md`。**收官口径固定为"骨架七项在带内 + 中央 chrome 为已记档分歧"，不得表述为完整 R0 通过 / 像素级 / 1:1。** 同轮记两条 catch：绿测集合不含已知超标项（新规矩：超标项必须以显式断言进集合）、两张必交截图字节相同（新规矩：核图要核图间差异）。
- 对话底座线：只读恢复审计已完成并经指导线验收；真实缺口仍是首句没有 durable binding。`tasks/2026-07-23-shared-conversation-transport-real-app-reacceptance-package-v2.md` 继续 HOLD；R3D 产品写入、Vite/浏览器取证期间只可静态只读准备，不得构建或启动三句重验。

## 四、锁着的 / 没接

- R3B、R3C、R3C-R1、R3D 与 R3E 的产品写授权均已消费并收口。**当前零在途写授权**；Graph、活动栏、右栏两个新组件、`NativeKnowledgeWorkspace.tsx`、`styles.css`、三个合同测试、fixture、注入方 `KnowledgeBaseView.tsx`、Shell、Canvas、maintenance、typed client、runner、依赖、Rust 一律回到冻结只读；真实 Syn/Tauri App、Codex CLI/MCP server、真实 store/消息/卡/chain/worker、非测试真实项目均未授权；主管仍只读、空写根，sandbox/服务端 allowlist 不得放宽。
- Obsidian 真嵌入、伴随窗口、Electron 迁移、品牌资产和插件生态复刻仍停止；禁止解包/改签/逆向/绕过 Gatekeeper。其他 vault、任意 filesystem/shell、登录/付费、stage/commit/push 仍未授权。
- M6 停写 JSON 与 audit_ledger 读源切换、记忆切 DB + 多 agent 专属门仍锁着；真实运行、高危边界和任何翻闸都不能由离线绿或短路由 READY 推导。
- 底1/底2只在替代性验收通过后按总计划重排；本次不进入 Harness Phase 3 或 Code Map。
