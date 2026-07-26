# 决策：知识库第一片——独立 vault + 浏览手编 + [[双链]] + AI 写入闸 v1

日期：2026-07-20
状态：**主体继续有效；“DB-primary 观察期/bridge 私有不可达/test-only writer”挂账已由 `decisions/2026-07-21-knowledge-vault-audit-production-write-path-v1.md` 纠偏取代。**
任务包：`tasks/2026-07-20-knowledge-vault-first-slice-package-v1.md`（轻档含 Rust）
上位拍板：`handoffs/2026-07-18-stage3-open-memory-ui-and-knowledge-base-handoff-v1.md` §二.2（独立 vault·不碰用户现有 Obsidian 库+浏览/手编为主+AI 写入=用户允许那一下+md 渲染/编辑/[[双链]]）

## 拍板固化

1. **vault 边界**：根死锁 `~/Library/Application Support/CodexGovernanceWorkbench/knowledge-vault/`（lib.rs:1327 先例·App 自有数据目录同 workflow-state 级）；md 文件即真相（非 sidecar JSON·不入 M5 DB-primary 口径·unknown_sidecar_json 闸不涉）；文件名=标题 slug（CJK 保留·空白→`-`·剔 `/\:*?"<>|` 与控制字符·空则 `untitled`·重名追加 `-2`）；路径锁三例拒绝=`..`/绝对路径/符号链接（组件锁+symlink_metadata 锁·单元测试锁死）。**不读不写用户任何既有目录**。
2. **用户手编**：新建（标题→种子 `# 标题` 落 vault→进编辑）/编辑（textarea raw·保存整文回写·取消弃改）/浏览默认；手编直写无确认（本人数据本人改=低危）。
3. **[[双链]]**：渲染期解析，命中（标题精确·大小写不敏感）=点开；未命中=出「新建《标题》？」用户那一下才建。反链/图谱/嵌入不做。
4. **AI 写入闸（高危口径形态）**：`PendingAction.kind="knowledge-vault-ai-write"` + `knowledgeVaultWrite{note_title,body,source_summary}` → PermissionDialog 变体（标题=action.label「AI 想往知识库写一条笔记」·正文=标题+来源+全文预览·[允许写入][不要]）→ App 分发支调独立后端命令 `knowledge_vault_ai_write`；审计 `knowledge_vault_note_ai_written`、actor_ref=`ai_proposed_user_confirmed`、source_summary 必填（后端硬拒空）。**无常驻授权、无自动沉淀、agent 零直写通道**。第一条真实触发=记忆候选详情「存成知识库笔记」。
5. **受限 md 渲染器**（零新依赖·纯函数 parse→节点树·全文本节点）：ATX `#`~`####`、段落、`**粗**`/`*斜*`/`` `码` ``、``` 围栏、`-`/`1.` 列表（一层不嵌套）、`[[wikilink]]`、`https?://` 外链（target=_blank rel=noreferrer）；其余语法一律纯文本逐字。
6. **写操作审计**：workflow-state audit_events JSON 通道三事件（`knowledge_vault_note_created`/`knowledge_vault_note_user_edited`/`knowledge_vault_note_ai_written`·前后端词表同步）。

## 派生写入面披露（照 G2 catch 先例：落地正确+披露，总指导核验）

包 §三列 10 处，以下 3 处为落地必需的派生面，已最小化并全披露：
1. `src-tauri/src/command_registry.rs`：`mod knowledge_vault;` + 5 命令注册（命令本体在 knowledge_vault.rs 内·**lib.rs 棘轮零碰**——优于包文「lib.rs 薄壳 ≤20 行/壳」，照 audit_ledger_read_model 模块自带命令先例）。
2. `src/App.tsx`：PendingAction 分发支 1 条（确认弹窗唯一通道·无它则闸不达后端）。
3. `src/views/MemoryCenterView.tsx`：+1 行（`onRequestAction` 透传进 CandidateMemoryDetail·payload 组装在 MemoryDetailPanels 内照包文）。

## 挂账

- **DB-primary 观察期**：vault audit 事件写 workflow-state JSON（`workflow_state_store::atomic_write`）；M5 DB-primary 双写桥函数私有不可达且 M5 文件禁碰——DB-primary 模式下 vault 事件不进 DB 投影（模式默认关；若开观察期需总指导定桥接）。
- 真机过目挂自然使用（同 P3-A/记忆中心先例·不单独烧额度）。
- vault 目录=用户数据非仓代码，仓外备份口径不含 vault。
