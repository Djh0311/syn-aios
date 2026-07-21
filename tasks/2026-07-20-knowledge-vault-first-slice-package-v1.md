# 任务包：知识库第一片——独立 vault + 浏览手编 + [[双链]] + AI 写入闸 v1

日期：2026-07-20
状态：**已出包，待总指导派工**
档位：**轻档**（工作台自管数据目录读写+前端呈现；不碰高危清单 5 条——vault=App 自有数据目录同 workflow-state 级，非用户真实项目、非 `.codex`；**AI 写入面按高危口径设计闸**见 §二.4）
执行者：执行线；总指导回收核实物
所属开发线：蓝图能力层 L3 知识库（主线=记忆→skill/harness→知识库→agent）
上位拍板：2026-07-18 用户拍（`handoffs/2026-07-18-stage3-open-memory-ui-and-knowledge-base-handoff-v1.md` §二.2）：**独立 vault（工作台自管新目录·不碰用户现有 Obsidian 库）+浏览/用户手编为主+AI 写入=用户允许那一下即可写（授权机制随片带·不做自动沉淀）+md 渲染/编辑/[[双链]]跳转（反链/图谱后置）**
正本纪律：`docs/plans/2026-07-14-post-m5-stage-plan-v2.md` L3（设计谈话先行·已谈形态已拍）；`docs/memory-layer-consolidated-canon-v1.md` M8（Obsidian 兼容接口 placeholder+边界·原生同步 deferred）+§18.1（记忆 Markdown 展示页≠知识库笔记，别混）
勘察依据：总指导 2026-07-20 写前勘察（数字自带分列，禁用「等」字）
本任务基线 commit：`112c029`

## 一句话目标

知识库从 read-model 占位长出第一片真肉：工作台自管 vault 里用户能建/看/改/跳 markdown 笔记，AI 产物落 vault 必过「用户允许那一下」确认闸——**不做自动沉淀、不做反链图谱、不做原生 Obsidian 同步、零新 npm 依赖**。

## 一、勘察实录（全部实测）

1. **现状=占位**：`KnowledgeBaseView.tsx`（255 行）纯前端读模型（`lib/knowledgeBase.ts` 319 行·documents=既有权威文件/记忆派生·`source_kind:"frontend_read_model"`）；边界面板自述「Obsidian-compatible 占位·未自动扫描 vault」。后端 `available_knowledge_refs` 全空（types.rs:2485/4725 字段预留·lib.rs:9707/:14315·director_agent.rs:284/:5378 全 `vec![]`），**无任何 vault/笔记存储**。
2. **存储落点**：App 数据根=`~/Library/Application Support/CodexGovernanceWorkbench/`（lib.rs:1327 先例），vault=`…/knowledge-vault/`（新建·首次写入时创建）；笔记=`.md` 文件，**文件即真相**（非 sidecar JSON·不入 M5 DB-primary 口径·`unknown_sidecar_json_kind` 闸不涉）。
3. **闸机制成**：`PendingAction`（`lib/types/workflow.ts:1723`）联合型+PermissionDialog 变体族=既有「用户允许那一下」通道，本片加一变体（god-switch 47 变体架构另议·本片随既有形加一）。
4. **AI 内容源（第一条真实触发）**：记忆中心候选详情（`MemoryDetailPanels.tsx`）已有候选两步道（确认≠转正）；候选=AI 产物，「存成知识库笔记」=用户动作触发+确认闸=拍板原话的最小真实落地。
5. **渲染**：全仓零 md 依赖（dependencies 仅 react/react-dom/@tauri-apps/api/@xyflow/react）→自写受限渲染器（§二.5 口径）。
6. **审计**：写操作走 workflow-state audit 既有通道（事件族 `knowledge_vault_*`·前后端词表同步）。

## 二、核心拍板口径（不许自由发挥）

1. **vault 边界**：根死锁 §一.2 路径；文件名=标题 slug（CJK 保留·空白→`-`·剔 `/\:*?"<>|` 与控制字符·空则 `untitled`·重名追加 `-2`）；读写拒 `..`/绝对路径/符号链接逃逸（测试锁）。**不读不写用户任何既有目录**。
2. **用户手编**：新建（输入标题→空笔记落 vault→进编辑）/编辑（textarea raw md·保存整文回写·取消弃改）/浏览默认。手编直写无确认（本人数据本人改=低危）。
3. **[[双链]]**：渲染期解析 `[[标题]]`→链接；命中（按标题精确匹配，大小写不敏感）=点开该笔记；未命中=问「新建《标题》？」（用户那一下才建）。**反链/图谱/嵌入 `![[]]` 不做**。
4. **AI 写入闸（随片带·高危口径形态）**：新 `PendingAction.kind="knowledge-vault-ai-write"`+payload（note_title/body/source_summary）→PermissionDialog 变体（标题「AI 想往知识库写一条笔记」·正文=标题+来源+全文预览·[允许写入][不要]）→允许才调独立后端命令 `knowledge_vault_ai_write`（审计 actor=`ai_proposed_user_confirmed`·source_summary 必填）；**无常驻授权、无自动沉淀、agent（MCP/worker/resident）零直写通道**（本片不挂任何 agent 写入面）。第一条真实触发=记忆候选详情「存成知识库笔记」按钮（候选全文+来源说明进 payload）。
5. **受限 md 渲染器**（零新依赖·零 innerHTML·全文本节点）：ATX 标题 `#`~`####`、段落、`**粗**`/`*斜*`/`` `行内码` ``、``` 围栏代码块、`-`/`1.` 列表（一层不嵌套）、`[[wikilink]]`、`https?://` 外链（纯文本+a[target=_blank rel=noreferrer]）；**其余语法一律按纯文本逐字显示**（表格/图片/嵌套/HTML 不渲染）。
6. **M8/§18.1 边界**：现有四面板（资料列表/边界/详情/捕获）一字不动；笔记区=新区块，文案不与「正式记忆/资料」混（知识库=材料和笔记空间）；「Obsidian 原生同步 deferred」边界文案保留。
7. 人话纪律：全部 UI 文案人话（`machine_face_on_ui` 闸口径）；空态=EmptyState 定式（必答下一步）。

## 三、施工清单（逐文件枚举）

1. **Rust 新建** `src-tauri/src/knowledge_vault.rs`：vault 根解析（照 lib.rs:1327 先例）+slug+路径锁+`list_notes`（title/slug/mtime_ms/出链标题清单）+`read_note`+`write_note`（用户手编）+`create_note`+`ai_write_note`（§二.4 actor/source 审计）；写操作落 workflow-state audit `knowledge_vault_note_created/user_edited/ai_written` 三事件（词表前后端同步）；单元测试：slug 三例/路径锁拒绝三例（`..`/绝对/符号链接）/create→list→read→edit 回环/ai_write 审计字段。
2. `src-tauri/src/lib.rs`：5 个 `#[tauri::command]` 薄壳+注册（照既有命令形）。
3. `src/lib/tauri.ts`：5 个 invoke 封装。
4. `src/lib/types/workflow.ts`：PendingAction 加 `knowledgeVaultWrite?: {…}` payload；`PathActionKind` 加 `"knowledge-vault-ai-write"`。
5. `src/components/PermissionDialog.tsx`：§二.4 变体（随既有 47 变体形加一·架构另议不混本片）。
6. `src/lib/knowledgeVault.ts`（新建）：受限渲染器（§二.5·纯函数 parse→节点树）+wikilink 提取。
7. `src/views/KnowledgeBaseView.tsx`：笔记区块（列表/阅读/编辑/新建/[[ ]]跳转载荷）+「存成知识库笔记」按钮不出现于此（在记忆中心）。
8. `src/views/memory/MemoryDetailPanels.tsx`：候选详情加「存成知识库笔记」按钮→组 payload→`onRequestAction`（复用既有候选数据·零新读模型）。
9. **测试**：`tests/knowledge-vault-notes.test.tsx`（新建·渲染器各语法+wikilink 命中/未命中+编辑保存+空态）+run-offline 注册一行（**25→26 组**）；rust 测试随 `knowledge_vault.rs`（`cargo test --lib` 1024→1024+新增）。
10. **落档**：`evidence/2026-07-20-knowledge-vault-first-slice-verification-v1.md`、`decisions/2026-07-20-knowledge-vault-first-slice-v1.md`（vault 路径/闸形态/渲染器口径固化）、`CURRENT.md`（收口后总指导笔）。

## 四、允许读取

本包、上位拍板交接 §二.2、两正本（post-m5 plan L3/canon M8+§18.1）、`AGENTS.md`、`prototypes/productized-desktop-shell/`（src/**、src-tauri/src/**、tests/**、scripts/**）、`docs/harness-catalog.md`。

## 五、允许写入

§三列出的 10 处。**不许碰**：现有 `lib/knowledgeBase.ts` 与 KnowledgeBaseView 四面板代码、记忆候选两步道逻辑、M5 存储线任何文件、gate 四规则与 selftest、`run-offline-interaction-test.mjs` 既有 25 组注册行（只准追加 1 行）、workflow-state 既有写点。

## 六、禁止事项

1. 零自动沉淀（任何东西不许自己写进 vault）；零 agent 直写通道；零常驻授权（每次 AI 写入都要用户那一下）。
2. 不碰用户既有 Obsidian 库/任何 vault 根外路径；不扫 vault 外文件。
3. 零新 npm 依赖；渲染器禁 innerHTML/dangerouslySetInnerHTML；反链/图谱/原生同步不做。
4. 零文案机器话；不改既有 25 组测试一字；shape 13/5/5 零净增；`lib.rs` 棘轮只加薄壳（超 20 行/壳=打回）。
5. 不 stage、不 commit。
6. 范围超 §三先停回总指导。

## 七、变更辐射面

- 新 Tauri command×5+新 PendingAction 变体 → PermissionDialog 变体表（既有形加一）；离线 25→26 组。
- 记忆中心候选详情 +1 按钮 → memory-center 相关测试面（vision-restyle/daily-inbox 两册）不许破。
- audit 词表+3 事件 → 账本面可读（AuditLedgerView 既有人话映射缺口按「逐字事件名」保底呈现，不伪造翻译）。
- vault=新数据目录 → 仓外备份口径不含 vault（用户数据非仓代码·明写进 evidence）。

## 八、五态旅程走查

- 说/批/干/交货/卡住：zero-touch（知识库是独立面）；交货后「主管总结→记忆候选→存成知识库笔记」=AI 写入闸唯一真实动线（§二.4）。

## 九、形状影响

- 任务类型：**能力层新片**（蓝图 L3 第一片·绿地已拍形态）。
- 新增代码落点：`knowledge_vault.rs`（新模块）+`knowledgeVault.ts`（渲染器）+1 测试新册；命令薄壳×5。
- 棘轮文件：`lib.rs` 只加薄壳（每壳 ≤20 行）；`PermissionDialog.tsx` 只加 1 变体；styles.css 如需笔记区样式只加新类不碰既有（闸口径 13/5/5 零净增·零 hex 裸值）。
- 新增 Tauri command：**5**（§三.2·本包唯一新增面）。新增 sidecar JSON：**无**（vault=md 文件即真相）。
- 退役/删测：无。
- 本任务基线 commit：`112c029`。完成 commit：总指导核收后另定（执行线不 commit）。

## 十、验收标准

1. 四闸：`cargo test --lib`（cwd=`src-tauri`）**1024+新增/0/44 全绿**；`npx tsc --noEmit`=0；`node scripts/run-offline-interaction-test.mjs` **26 组全绿**；shape gate baseline+check **13/5/5 零净增**；`git diff --check` 过。
2. 路径锁：三例拒绝测试绿（`..`/绝对路径/符号链接）+evidence 给真机外路径构造尝试被拒输出。
3. 闸：AI 写入不经 PermissionDialog 允许=vault 零文件变化（离线断言+rust 测试双锁）；审计三事件字段对平。
4. 渲染器：§二.5 语法逐类断言+非支持语法逐字原样输出断言+XSS 样例（`<script>`/`<img onerror>`）按纯文本输出断言。
5. 对账：命令数=5、新依赖=0、离线组 25→26、cargo 数对平、shape 三数零净增。
6. **真机过目（用户自然使用时）**：本片离线可全验，真机走查挂自然过目（同 P3-A/记忆中心先例），不单独烧额度。

## 十一、必须回传（按 TASK_TEMPLATE 10 项）

做了什么 / 改了哪些文件 / 新增哪些测试或证据 / 哪些结论有依据 / 哪些仍不确定 / 风险和下一步建议 / shape gate baseline+check 摘要（含 cargo 新数）/ start-end commit / 是否新增 command·sidecar·触碰棘轮文件 / **被闸拦过的事**（无也必须写「无」）。

## 十二、总指导回收动作

- 亲跑四闸不信回传；路径锁三例亲手构造复跑；渲染器断言逐类核；AI 闸「不允许=零写入」亲验；PendingAction 变体 diff 核（不碰既有变体）；记忆中心两册测试面 grep 核零搭车。
- 判断 接受 / 需要修改 / 暂停 / 废弃，记 `docs/harness-catch-log.md`；收口 commit 同笔回写 CURRENT（L3 第一片落地+下一片=P2-B 或底2）。
