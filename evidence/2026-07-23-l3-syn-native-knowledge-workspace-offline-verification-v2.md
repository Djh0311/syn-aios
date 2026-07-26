# L3 Syn 原生知识工作区离线验证 v2

- 日期：2026-07-23
- 当前阶段：N0-N5 已完成离线实现与根级验证；N6 已完成只读 capability 的 fail-closed 离线收口，但因受信任 host dispatch 缺口，十二项真实 App 验收均未执行。
- 权威路线：`decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md`
- 开发合同：`tasks/2026-07-23-l3-syn-native-knowledge-workspace-development-package-v2.md`

## N0 转向冻结

### 工作树与真实环境边界

- 起始 HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；本线开始和 N0 收口核对时暂存区均为空。
- 保留既有大量未暂存改动，承重脏文件仅 merge-only；未执行 reset、clean、stash、stage、commit 或 push。
- 未安装或启动 Obsidian、未注册 CLI、未请求辅助功能权限、未访问任何 vault 或真实项目；N0 的所有证据均为离线代码与测试证据。

### v1 WIP 的保留与停止

| 类别 | N0 处理 | 结果 |
| --- | --- | --- |
| 固定 Syn vault、Markdown、冲突拒绝、Batch 2 audit | 保留 | `knowledge_vault.rs` 仍是唯一知识文件根；冲突提示改为中立的“外部来源或另一窗口”。 |
| typed Obsidian bridge | 保留并降级 | 仅保留固定 vault 的 `status/open/read/search` 兼容层；未安装不会阻塞 Syn 原生工作区。 |
| Obsidian 主状态区 | 降级 | 知识页后的收起式“可选兼容与外部打开”入口；未展开不请求状态。 |
| companion/Accessibility/受管窗口 | 停止并移除本线 WIP | 删除 companion payload、`set_companion_mode`、AX probe、外部控制命令和注册；没有真嵌入声明。 |
| `knowledge_search/read/open/cite` | 保留 | registry、可信 binding 与 fail-closed 回归保持；未增加知识写能力。 |

### 验证实物

| 检查 | 结果 |
| --- | --- |
| `cargo test obsidian_integration --lib` | 8/8 通过：固定 argv、注入拒绝、超时/输出上限、非阻塞兼容状态和已停止外部控制面。 |
| `cargo test knowledge_vault --lib` | 23/23 通过：固定根、路径/symlink 拒绝、Markdown/Canvas/附件/恢复、审计与 CAS 冲突零覆盖。 |
| `cargo test capability_registry --lib` | 4/4 通过。 |
| `cargo test supervisor_conversation_binding --lib` | 8/8 通过。 |
| `npm run typecheck` | 通过。 |
| `node scripts/run-offline-interaction-test.mjs` | 通过，15 个离线入口，含 N0 原生工作区兼容入口断言。 |
| N0 范围 `git diff --check` | 通过。 |

### 已知、刻意保留的红测

`knowledge_open` 已不再调用旧 external bridge；它只返回 `target=syn_native_view`、`dispatch_status=trusted_host_dispatch_required` 和 `opened=false` 的安全意图。`cargo test mcp::supervisor_orchestrator::knowledge_capabilities::tests::trusted_host_dispatch_must_settle_before_native_view_can_be_claimed_open --lib -- --ignored --exact` 被刻意强制执行时预期失败（0/1）：没有受信任 host dispatcher 时绝不声称已打开。

精确缺口留给另行授权的 host-owned 短期 relay：它必须把已验证的 `relative_path` 从 MCP stdio 子进程交给 Syn 主进程原生视图，而不把路径塞入 binding、静态全局或第二真相源。此前不宣布 `knowledge_open` 的原生打开完成。

### 历史 warning

目标 Rust 测试有 18 条既有 warning；N0 已清除 bridge 新增的未使用字段 warning。没有把历史 warning 计为 N0 成功或失败。

## N1 已实现的 Vault 与索引合同

- 新工作区只接受内部 `ValidatedVaultRelativePath`：拒绝绝对/父级/空段/隐藏段/控制字符/反斜杠/选项形态和桥接层已拒绝的通配、引号、`=`；每次解析现有路径时逐段精确大小写比对，并拒绝根、祖先或叶子 symlink。
- 旧五个 `knowledge_vault_*` 命令和单层 slug 不迁移、不放宽；新工作区命令使用带扩展名的 `relative_path`，例如 `research/plan.md`。
- 文件字节是唯一真相；索引仅为内存重建投影，不写 `.index`、SQLite 或旁路 JSON。Markdown 上限为 64 KiB，Canvas 分类上限为 256 KiB，受限附件上限为 10 MiB。
- N1 仅解释 Markdown；`.canvas` 和 `attachments/` 先作为受限分类条目，具体 Canvas/附件解析留 N4/N5。Frontmatter 只读取 `title`、`tags`、`aliases` 与一层标量 `properties`，其余 YAML 原文保留但不执行。
- 新文件或移动/重命名/删除在操作前检查 mtime/hash，Markdown 写入采用同目录临时文件加 rename；这防止通常 stale 覆盖，但不宣称覆盖恶意并发 TOCTOU 替换。

### 实现与定向验证

- `knowledge_index.rs` 是仅内存、可从固定 vault 重建的 Markdown/目录/Canvas/附件分类投影；不会生成 `.index`、SQLite 或旁路 JSON 真相源。
- 受限工作区命令为 `snapshot/search/read_markdown/create_directory/create_markdown/write_markdown/move_entry/rename_entry/delete_entry`；路径、根、filesystem 与 shell 都不由前端传入。
- 新增的 `write_markdown` 只更新固定 vault 内已存在的嵌套 `.md`，携带 mtime/hash CAS，成功才产生 `knowledge_workspace_markdown_updated` 审计；目录、Canvas、附件、缺失条目、逃逸路径和 stale 都拒绝，失败零正文写入、零审计。
- 红测：`cargo test workspace_write_markdown_updates_only_an_existing_nested_markdown_with_cas --lib -- --exact` 在实现前因缺少 `workspace_write_markdown_at` 报 3 处 E0425；绿测使用完整名 `cargo test knowledge_vault::tests::workspace_write_markdown_updates_only_an_existing_nested_markdown_with_cas --lib -- --exact`，1/1 通过。
- 根级复核：`cargo test knowledge_index --lib` 为 8/8；`cargo test knowledge_vault --lib` 为 12/12；`cargo check --lib` 通过。

## N2 原生 Markdown 编辑工作区

### 已完成范围

- `NativeKnowledgeWorkspace` 是知识页的主层：左侧已验证文件/目录树，中间 Markdown 源码、受限渲染预览和分栏阅读，右侧安全 frontmatter 属性、标签、正向/反向链接。
- 浏览器内只调用 `knowledgeWorkspace` 的九个固定 typed host command；SSR/离线渲染使用无 hook、无 Tauri invoke 的静态壳。没有 raw command、vault root、任意 filesystem 或 shell 接口。
- 支持标签页、快速打开/全文搜索、Syn 内联“新建目录/Markdown”命令面板、唯一匹配的 wikilink 内部跳转，以及未命中时仅由用户点击准备新建。
- 保存使用当前 Markdown 的 mtime/hash；冲突保留草稿并只提供显式“重新读取”，不提供无声覆盖。工作区布局尚未持久化，按计划留给 N5。
- 旧单层笔记区仍保留在后方；官方 Obsidian 仍是收起式、非阻塞的可选外部打开。JSON Canvas 明确留给 N4，不在 N2 声称已完成。

### 红绿与离线证据

- UI 红测：在组件不存在时，`node scripts/run-offline-interaction-test.mjs` 如预期报 `Could not resolve ../src/views/knowledge/NativeKnowledgeWorkspace`；此前已有 15 项离线入口仍继续通过。
- N2 typed-client 红测：固定命令表缺少写命令时，runner 失败，typecheck 明确缺少 `markdown_updated`、`writeMarkdown` 与 command union；实现后固定 ninth command 的 exact lower-camel CAS payload 通过合同断言。
- 根级绿测：`npm run typecheck` 通过；`node scripts/run-offline-interaction-test.mjs` 通过（含 `knowledge-vault-notes` 和 `native knowledge workspace N0 optional compatibility and N1/N2 fixed client contract`）。
- N2 UI 场景是离线/SSR 与 typed-client 合同证据，不冒充真实桌面 App 操作；未启动 Syn App、Obsidian、CLI 或真实 vault。

### 文件归属

- 后端：`src-tauri/src/knowledge_index.rs`（新增）、`src-tauri/src/knowledge_vault.rs`（merge-only）、`src-tauri/src/command_registry.rs`（merge-only）。
- 前端合同：`src/lib/tauri.ts`（merge-only）、`tests/native-knowledge-workspace.test.tsx`（新增）。
- 工作区 UI：`src/views/knowledge/NativeKnowledgeWorkspace.tsx`（新增）、`src/views/KnowledgeBaseView.tsx`、`src/styles.css`、`tests/knowledge-vault-notes.test.tsx`（均 merge-only）。

## N3 知识关系图

### 已完成范围

- `knowledge_workspace_graph` 只从 N1 已验证 Markdown、wikilink 与 backlink 的可重建投影产生数据；不写布局、索引、Markdown 或旁路状态。
- 范围只允许 `global` 和 `local`：全局图保留孤立 Markdown；局部图只保留焦点及直接邻居。query 是有界普通文本，tag 是精确受限 scalar；未知范围、焦点缺失、大小写漂移或被筛选掉的焦点均 fail closed。
- 图节点的 `id` 与 `relative_path` 相同，边只连接已输出节点；确定性上限为 512 节点、1024 边，超限返回 `truncated` 与有界诊断。
- 前端只复用现有 `@xyflow/react` 原语。SSR 为无 hook、无 Tauri 调用的静态关系账页；浏览器只调用固定 `knowledgeWorkspace.graph`，不暴露 raw invoke、vault root、外部 URI 或工作流语义。
- 图节点只把后端返回的 `relative_path` 交给知识页的 typed callback，再复用 `NativeKnowledgeWorkspace` 的既有 `readMarkdown` 固定客户端读取。重复选择同一节点带递增 UI-only sequence，避免用户先切换笔记后再次点击同一节点而漏读；它不成为路径、路由或持久化状态接口。

### 红绿与离线证据

- 后端红测：缺少图谱 API 时，`cargo test n3_graph_projection_starts_red_with_validated_links_and_isolated_markdown --lib` 如预期报 `KnowledgeWorkspaceGraphRequest`、`KnowledgeWorkspaceGraphScope` 与 `workspace_graph_at` 缺失的 E0422/E0433/E0425。
- UI 红测：新增并注册图谱测试后，`node scripts/run-offline-interaction-test.mjs` 如预期仅新增 `Could not resolve ../src/views/knowledge/KnowledgeGraphView`；此前离线项继续通过。
- 根级复核：`cargo test knowledge_index --lib` 为 11/11；`npm run typecheck` 通过；`node scripts/run-offline-interaction-test.mjs` 通过，包含 `native knowledge graph static shell and typed handoff tests passed`。定向 Rust 测试仍只显示 18 条既有 warning。
- 所有 N3 UI、关系图和连接契约仍是离线/SSR/typed-client 证据；未启动 Syn App、未访问 vault/Obsidian。实际图谱交互留给 N6 的真实 App 十二项验收。

### N3 文件归属

- 后端：`src-tauri/src/knowledge_index.rs`、`src-tauri/src/command_registry.rs`、`src/lib/tauri.ts`（均 merge-only，前者为 N1 新模块）。
- UI：`src/views/knowledge/KnowledgeGraphView.tsx`、`tests/knowledge-graph.test.tsx`（新增），以及 `src/views/knowledge/NativeKnowledgeWorkspace.tsx`、`src/views/KnowledgeBaseView.tsx`、`src/styles.css`、`scripts/run-offline-interaction-test.mjs`（merge-only）。

## N4 JSON Canvas

### 已完成范围

- 新增 `knowledge_canvas.rs`，按 JSON Canvas 1.0 读取、校验并 roundtrip `text`、`file`、`link`、`group` 节点和边；顶层 `nodes`/`edges` 可省略并按空数组处理，非数组结构、重复 ID、悬空边、非法坐标和超限 Canvas 都 fail closed。
- 所有未识别的根、节点和边字段以原始 JSON 局部 patch 保留；节点内容和 URL 只作数据，不执行、不打开外部程序。
- 文件节点只接受已验证的固定 vault 内 `.md`、`.canvas` 或 `attachments/` 中受限的 `.png/.jpg/.jpeg/.gif/.webp/.pdf/.txt/.csv`；分组背景只接受 `attachments/` 下受限栅格图；引用逐段验证、拒绝 symlink/目录/路径逃逸，并执行 10 MiB 附件上限。
- `create_canvas`、`write_canvas` 复用固定 vault、同目录原子替换、mtime/hash CAS 与 `knowledge_vault_audit`。读取时缺失旧引用只给出有界诊断，写入时仍拒绝，防止把已失效引用静默固化。
- `KnowledgeCanvasView` 只复用现有 `@xyflow/react`：SSR 壳不调用 Tauri；浏览器仅调用 `snapshot/readCanvas/createCanvas/writeCanvas` 四个 fixed typed client 方法。画布可新建、选择、拖动、局部编辑四类节点、创建/删除边和节点、显式保存/重读；冲突保留本地草稿，不自动覆盖。

### 红绿与离线证据

- 后端红测：标准 fixture 尚无解析入口时，`n4_standard_json_canvas_fixture_starts_red` 如预期不能编译；实现后 `cargo test knowledge_canvas --lib` 为 8/8，包括结构、引用白名单、超限、symlink、CAS/原子写/审计和未知字段保留。
- 前端红测：注册 Canvas 场景而组件未落盘时，离线 runner 如预期只报告 `Could not resolve ../src/views/knowledge/KnowledgeCanvasView`；实现后 `npm run typecheck` 通过，完整离线 runner 通过并明确输出 `native knowledge canvas static shell, typed calls, and local JSON Canvas patch tests passed`。
- 根级复核：`cargo test knowledge_index --lib`（11/11）、`cargo test knowledge_vault --lib`、`cargo check --lib` 均通过。定向测试仍只有 18 条既有 warning；`cargo check --lib` 输出 598 条项目既有 warning，不归因给 N4。
- `rustfmt --edition 2021 --check src/knowledge_canvas.rs src/knowledge_index.rs`、`rustfmt --edition 2021 --check --config skip_children=true src/command_registry.rs` 和 N0-N4 写面 `git diff --check` 均通过；暂存区为空。未启动 Syn App、未访问真实 vault 或 Obsidian，故这些不是 N6 真实 App 证据。

### N4 文件归属

- 后端：`src-tauri/src/knowledge_canvas.rs`（新增），`src-tauri/src/knowledge_vault.rs`、`src-tauri/src/knowledge_index.rs`、`src-tauri/src/command_registry.rs`（merge-only）。
- 前端：`src/views/knowledge/KnowledgeCanvasView.tsx`、`tests/knowledge-canvas.test.tsx`（新增），`src/lib/tauri.ts`、`src/views/KnowledgeBaseView.tsx`、`src/styles.css`、`scripts/run-offline-interaction-test.mjs`（merge-only）。

### 格式、差异与历史债

- 受限 N0-N2 已跟踪文件 `git diff --check` 通过；三个新增 N1/N2 文件的 `git diff --no-index --check /dev/null <file>` 均无 whitespace 输出（退出码 1 仅表示新增文件与 `/dev/null` 不同）。
- `rustfmt --edition 2021 --check src/knowledge_index.rs` 与 `rustfmt --edition 2021 --check --config skip_children=true src/command_registry.rs` 通过。普通递归 rustfmt 仍会命中 `knowledge_vault.rs` 约 1295-1418 行的五处旧测试格式差异；该段不属于 N1/N2 写面，未为格式化改写脏树。
- `cargo check --lib` 通过但输出 598 条项目既有 warning；定向 `cargo test` 输出 18 条既有 warning。两者均不作为本阶段成功的替代证据。
- 全树 `node scripts/harness/workbench-shape-gate.js --mode check --target .` 为 fail（16 error / 5 warning）：历史 ratchet、既有超限文件和未知 sidecar 共同造成，不能从这份大量脏树中归因给 N2。该次扫描同时显示 UI hardcoded-hex 与 machine-face error 为 0；全树 shape gate 仍非绿色，留作 N6 前的历史债单列，不宣称已解决。
- 本次根级核对时 staged 为空；未执行 reset、clean、stash、stage、commit 或 push。

## N5 附件、刷新与恢复（离线实现）

- 附件导入的唯一入口是受限 `bytes + displayName + MIME`；宿主只接受允许的扩展/MIME 配对、最大 10 MiB，固定写入 `attachments/`，拒绝源路径、URL、覆盖和非附件读取。Markdown 与 Canvas 只保存固定 vault 内相对引用；缺失引用只展示可恢复提示，不启动外部程序。
- vault manifest 是从固定 vault 即时重建的投影，不写 `.index`、旁路 JSON 或 SQLite 真相。恢复备份只保存在受控 app-data 下，以不透明 ID 指向单条条目；创建、恢复都做 mtime/hash CAS，不能整体回滚 vault，也不能先创建恢复目录再接受 stale 请求。
- 工作区布局只保留可丢弃的本地偏好。窗口聚焦、手动刷新、旧笔记新建/保存以及确认式 AI 写成功后都会发出同一受限刷新事件；Markdown、Canvas 和旧笔记面板均遵循 clean 才可回填、dirty 保留、dirty 且磁盘变化即 conflict 的规则。
- 读取、刷新、保存回读还同时比对本地草稿 revision、编辑器 request generation 和当前相对路径；迟到的回包只保留当前草稿并给出提示，不能覆盖用户期间编辑或后选条目。Markdown/旧笔记在同目标保存期间检测到本地后续编辑时进入显式重读态，Canvas 进入 conflict 态。
- 离线复核：`cargo test knowledge_ --lib` 为 42 通过、1 个已知 N6 host-dispatch 红契约 ignored；其中包含附件、恢复、Canvas、索引、路径、MCP 只读和 binding 相关场景。`npm run typecheck` 与 `node scripts/run-offline-interaction-test.mjs` 均通过，后者含附件/恢复边界、统一刷新事件和异步草稿竞态断言。
- 未启动 Syn App、未访问真实 vault，也没有把离线结果冒充为附件导入、外部修改、重启恢复的真实 App 证据；这些场景与其余 N6 十二项一起记录为未执行。

## N6 AI/MCP（离线安全收口，真实 App 未执行）

- `knowledge_search/read/cite` 只读取 N1 已验证的嵌套 `relative_path` 与可重建索引投影；无 raw vault 扫描、无绝对路径、无 shell、无写入。`knowledge_open` 先执行同样的路径验证，再返回未派发的 Syn 原生视图意图，不默认调用 Obsidian。
- capability registry 保留既有 `submit_proposal`，并仅增加 `knowledge_search/read/open/cite` 四个只读能力；`knowledge_write`、`canvas_write`、`attachment_write` 被显式拒绝。Active binding 热扩 `knowledge_write` 的回归证明拒绝后 lifecycle 仍为 Active，未改变 binding lifecycle。
- 定向复核：`cargo test knowledge_capabilities --lib` 为 4 通过、1 ignored；`cargo test capability_registry --lib` 为 4/4；`cargo test supervisor_conversation_binding --lib` 为 8/8；`cargo test supervisor_orchestrator --lib` 为 58 通过、2 个既有/受控 ignored。强制执行 host-dispatch 红契约如预期失败，错误只说明 `opened=false`，不产生真实打开主张。
- `cargo check --lib` 通过；目标 Rust `rustfmt --check` 和 `command_registry.rs` 的 scoped fmt 通过；`git diff --check` 通过。`cargo check --lib` 仍输出 598 条项目既有 warning，定向测试显示 18 条既有 warning，均未归因给本线。

## 最终离线边界与未完成项

- N6 的真实 App 十二项、截图和日志没有生成：MCP stdio 子进程尚无受信任、短期、host-owned 的原生视图 dispatch；现有 binding 只存生命周期/结果，不能安全携带路径。这个 relay 涉及不在本包 N6 写面的主进程/transport/UI 文件，按任务包停点没有擅自扩写。
- 全树 shape gate 仍为 `16 error / 5 warning`，与 N0-N4 记录的历史聚合相同，不是绿色门；本线未扩展 gate 的错误类别。`offline-permission-dialog.test.tsx` 的占位断言已等行替换，避免对 HEAD 再增行数。全树既有 ratchet、超限文件和未知 sidecar 仍需独立治理。
- 根级 staged 为空；未 reset、clean、stash、stage、commit 或 push；未启动 Syn App、Obsidian、CLI，未访问任何真实 vault、其他 vault 或真实项目。真实停点详见 `evidence/2026-07-23-l3-syn-native-knowledge-workspace-real-app-acceptance-v2.md`。
