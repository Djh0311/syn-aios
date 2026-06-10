# Evidence：Memory Layer M8 Knowledge Base And Obsidian-compatible Interface Placeholder v1

日期：2026-06-05

## 结论

M8 已完成。接受范围仅限知识库 / Obsidian-compatible 接口占位和边界：

- `知识库` 一级入口已从 placeholder 替换为最小知识库资料 UI。
- 新增前端 `KnowledgeDocumentReadModel` 派生逻辑，从 `WorkbenchSnapshot.projects[].authority_files`、formal memory `source_refs`、memory candidates 和 task package `available_knowledge_refs` 派生摘要。
- 知识库详情可显示项目归属、来源锚点、关联正式记忆、关联候选和任务包知识引用数量。
- 可从明确知识库资料提出 `MemoryCandidate`，只写 `memory-candidates.v1.json`，不写正式记忆。
- `knowledge_doc` 来源在记忆中心显示为“来自知识库资料”。
- Obsidian-compatible 仅为占位和边界说明；未执行 Obsidian 原生同步，未自动扫描 vault。

不接受为：

- Obsidian 原生能力接入。
- vault 自动扫描。
- 知识库文档直接写正式记忆。
- 正式记忆生命周期操作。
- 关系治理、维护任务、成熟模式、跨项目记忆或中间版本记忆系统完成。
- 真实 worker / Codex 执行。

## 主要改动

- `prototypes/productized-desktop-shell/src/lib/knowledgeBase.ts`
  - 新增知识库前端读模型和候选草案派生。
- `prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx`
  - 新增知识库最小入口 UI。
  - 展示资料列表、边界面板、反向引用摘要和“提出记忆候选”动作。
- `prototypes/productized-desktop-shell/src/App.tsx`
  - `knowledge` view 接入 `KnowledgeBaseView`。
  - 新增 `create-memory-candidate` 确认分支，复用既有 `createMemoryCandidate` wrapper。
- `prototypes/productized-desktop-shell/src/lib/types.ts`
  - 新增 `create-memory-candidate` action kind。
  - `PendingAction` 新增 `memoryCandidateCreation`。
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
  - 新增知识库候选确认弹层详情。
  - 明示只写 `memory-candidates.v1.json`、只生成候选、不写正式记忆、未执行 Obsidian 原生同步。
- `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`
  - `knowledge_doc` 来源显示为“来自知识库资料”。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 新增知识库入口布局样式。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 新增知识库边界离线测试，覆盖读模型、UI 文案、禁用文案、候选 action 和确认弹层。

## 验证

通过：

```text
npm run test:offline-interaction
offline interaction tests passed: 9
```

```text
npm run typecheck
tsc --noEmit
```

```text
npm run build
tsc --noEmit && vite build
215 modules transformed
built in 605ms
```

禁用文案扫描：

```text
rg -n "已接入 Obsidian 原生同步|vault 已自动扫描|知识库已自动记住|文档已成为正式记忆|知识命中已成为正式记忆|知识命中已注入任务包|中间版本记忆层已完成" product-line/prototypes/productized-desktop-shell/src
```

结果：`src` 无命中。测试文件保留这些文案作为“不得出现”的负向断言。

真实浏览器 smoke：

- 本地 Vite 预览启动：`npm run dev -- --host 127.0.0.1`。
- 沙箱内监听 `127.0.0.1:5173` 被 `EPERM` 阻止，已按权限规则用批准的外部运行启动。
- Browser 打开 `http://127.0.0.1:5173/`，点击 `知识库`。
- DOM 可见：
  - `知识库资料`
  - `Obsidian-compatible 占位`
  - `未执行 Obsidian 原生同步`
  - `未自动扫描 vault`
  - `知识库是材料和笔记空间；正式记忆是经过确认、来源、版本、审计和权限治理的行为上下文。`
- 因 Vite 预览不是 Tauri 窗口，页面显示预期降级提示：`读取失败：当前页面不在 Tauri 窗口中运行`，真实 sidecar 数据未加载。
- Browser 截图已在会话中捕获和展示；尝试保存到 `evidence/2026-06-05-memory-layer-m8-knowledge-base-ui-smoke.png` 时 Browser runtime 报 `EPERM`，因此没有落盘截图文件。

未新增 Rust 测试说明：

- 本轮没有新增 Rust 命令、后端 store 或正式记忆写入路径。
- 知识库候选生成复用既有 `create_memory_candidate`，数据写入仍受现有 `MemoryCandidateStore` 测试覆盖。
- M8 新增风险集中在前端读模型、UI 边界和确认弹层，所以新增离线 UI / read model 测试覆盖。

## 边界与偏差

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未写 `formal-memories.v1.json`、`observations.v1.json`、`memory-lint.v1.json`、`workflow-state.v0.json`。
- 未执行 Obsidian CLI。
- 未扫描或改写 Obsidian vault。
- 为完成浏览器 smoke，按当前 Browser 插件技能说明读取了浏览器插件说明文件；该文件位于 `.codex/plugins/...`。未读取用户 Codex 会话、索引、rollout 或其他 `.codex` 数据，也未写 `.codex`。

## 后续

- M9：正式记忆生命周期操作任务包尚未拆出。
- M8 之后仍不能宣称中间版本完整记忆系统完成。
