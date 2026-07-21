# 知识库第一片 验收证据 v1

日期：2026-07-20 · 轻档含 Rust · 执行线施工，总指导回收核实物
任务包：`tasks/2026-07-20-knowledge-vault-first-slice-package-v1.md`
基线 commit：`112c029`（开工核对 HEAD=`112c029ca2407c74c8a560110e449839ea915360`，工作树干净）
拍板留痕：`decisions/2026-07-20-knowledge-vault-first-slice-v1.md`

## 一、结论

独立 vault + 浏览/手编/[[双链]] + AI 写入闸全做：Rust `knowledge_vault.rs`（根锁+slug+路径锁三例拒绝+5 命令+3 审计事件+6 单元测试）、前端笔记区（列表/阅读/编辑/新建/双链两态）、PermissionDialog 变体、记忆候选「存成知识库笔记」第一触发、受限渲染器、离线新册——四闸全绿，零新 npm 依赖，agent 零直写通道。

## 二、四闸

| 闸 | 结果 |
|---|---|
| `cargo test --lib`（cwd=src-tauri） | **1030 passed / 0 failed / 44 ignored**（基线 1024+新增 6 对平 ✓） |
| `npx tsc --noEmit` | 0 错 |
| `node scripts/run-offline-interaction-test.mjs` | **26 组全绿**（25→26·exit 0） |
| shape gate baseline+check | **13/5/5 零净增**（Tauri commands 136→141=+5 对平；sidecar unknown 5 同基线·vault=md 非 JSON 不涉） |
| `git diff --check` | exit 0 |

## 三、路径锁（§十.2）

单元测试三例拒绝全绿：`../escape`（ParentDir 组件锁）、`/etc/passwd`（绝对路径 RootDir 锁）、symlink→vault 外文件（symlink_metadata 锁·读写双拒）；slug 三例（CJK 保留+空白→`-`、剔 `\:*?"<>|`、空/全空白→`untitled`）；重名 `-2`；create→list→read→edit 回环+outlinks 提取；ai_write 空 source_summary 硬拒且零落盘+审计 actor/event/reason 对平。真机外路径构造尝试：单测内 symlink 指向 vault 外临时文件被拒（stderr 文案「拒绝符号链接路径」），外层命令复跑见 cargo log。

## 四、AI 写入闸「不允许=零写入」双锁

1. **离线断言锁**：弹窗变体只在 `knowledge-vault-ai-write` kind 渲染；[允许写入][不要]；全文预览/来源/边界文案齐（离线 26 组内既有 PermissionDialog 断言面不回归；本册未加变体断言=变体走 §三.9 枚举外，渲染路径与既有 47 变体同构）。
2. **Rust 锁**：`ai_write_note_at` 空 source_summary=Err 且 `标题.md` 不存在（测试实证）；agent 面无调用点（grep `knowledge_vault_ai_write` 全仓仅模块本体+注册+TS invoke 三处）；无常驻授权（每次写入必经 PendingAction→App 分发支）。
3. 审计三事件 `knowledge_vault_note_created`/`_user_edited`/`_ai_written` 字段（event_id/event_type/target_ref/actor_ref/permission_level/reason）对平，actor 两值（user_manual_edit / ai_proposed_user_confirmed）。

## 五、渲染器（§二.5·离线新册 8 组断言）

ATX 1-4 级/段落/粗/斜/行内码/围栏代码块逐字/无序+有序列表一层/wikilink/外链——逐类断言；非支持语法（表格/图片/嵌套列表/HTML）逐字纯文本断言；XSS 样例（`<script>`/`<img onerror>`）转义纯文本断言（markup 无 `<script>`/`<img>` 标签）；`extractWikilinks` 去重；渲染层零 innerHTML/dangerouslySetInnerHTML（grep 实证·全文本节点）。

## 六、视图与闸面

- wikilink 命中（大小写不敏感）=打开；未命中=「新建《Gamma》」问询面，用户点才建（断言 5/6）。
- 编辑=textarea raw·保存走 onSaveEdit/取消弃改（断言 7）；空态=EmptyState 定式必答下一步（断言 8）；unavailable=明说桌面壳才可读写。
- 记忆中心两册零搭车：vision-restyle/daily-inbox 的 CandidateMemoryDetail 均不传 onRequestAction → 新按钮不出现（grep 实证）；生产 MemoryCenterView 透传后才出现。
- 现有四面板一字未动（git diff KnowledgeBaseView 仅 import+挂载行+新增组件）；`lib/knowledgeBase.ts` 零碰。

## 七、对账（§十.5）

新 Tauri command=**5**（list/read/create/write/ai_write·shape 计数 136→141）；新 npm 依赖=**0**（package.json diff 空）；离线组 25→**26**；cargo 1024→**1030**；shape 三数零净增；lib.rs 棘轮**零碰**（命令注册走 command_registry.rs·派生写入面披露）；PermissionDialog +1 变体；styles.css 只加新类（零 hex 裸值 grep 实证）。

## 八、枚举外事项与被闸拦过的事

1. **派生写入面 3 处**（包 §三未列·落地必需·照 G2 catch 先例披露）：command_registry.rs（mod+注册·lib.rs 零碰）、App.tsx（分发支·确认唯一通道）、MemoryCenterView.tsx（+1 行透传）。详见 decisions §派生写入面披露。
2. **manual_relay 既有 flake**：首轮全量 cargo 跑 `manual_relay_gui_direct_stop_kills_mock_process_group_children` 失败 1-2 次（测试内硬编码 1000ms 等 mock 子进程 ready·竞态与本包无关——knowledge_vault 独立模块零进程面；复跑 1030/0/44 全绿·基线同测亦有过失败史=既有时敏件）。
3. **DB-primary 观察期口径**：vault audit 走 workflow-state JSON 原子写（M5 双写桥私有不可达·M5 文件禁碰）——DB-primary 模式下 vault 事件不进 DB 投影（模式默认关·挂账 decisions）。
4. **SSR 守卫**：KnowledgeVaultNotesPanel 无 window 时不挂 hooks 渲染 loading 静态面（现有知识库测试走 findElement 裸调·守卫同 ProjectDetail 先例）；离线断言打零 hooks 的 KnowledgeVaultNotesView 本体。
5. 被闸拦过的事：**无**（四闸终过；cargo flake 为既有竞态非本包触发，明写不隐瞒）。
