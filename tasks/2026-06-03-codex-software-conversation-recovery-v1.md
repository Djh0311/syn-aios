# Task Package：Codex Software Conversation Recovery v1

状态：已撤回实现，任务目标错误，已被 `2026-06-03-codex-native-app-conversation-list-repair-v1.md` superseded。  
用途：原本误写为修复“Codex 软件对话列表里有一些旧对话消失、不能被工作台识别”的问题；该描述混入了工作台语义，不再作为可派发任务。  
执行方式：先只读诊断，再按诊断结果做最小修复；不能一上来重建、迁移或写 Codex 数据。

## 0. 撤回和纠偏记录

本任务包对应的实现曾被错误执行为“工作台智能体页旧 Codex 会话恢复”。该实现已被撤回：产品代码中的 recovery Rust 模块、Tauri command、前端类型、面板、传参和测试残留已清理。

用户原始问题指向 Codex 原生软件自己的对话列表：旧对话在 Codex app 里消失、不能被 Codex 识别。工作台能不能显示旧会话，不是这个问题的验收标准。

因此本任务包不再派发。真正可派发任务是：

- `tasks/2026-06-03-codex-native-app-conversation-list-repair-v1.md`

该新任务包要求以 Codex 原生 app 会话列表作为验收对象，并且写 `.codex` / sqlite / session index 前必须另行取得文件级用户确认、备份和回滚方案。

## 1. 先说薄弱点

- 会话中心底座硬化后，sqlite 已成为会话目录主权威，`index.json` 只做缓存 / 兼容 / 辅助。
- 用户现在反馈“旧对话消失”，说明当前目录发现逻辑可能仍漏掉某些旧来源，例如旧 sqlite、归档目录、旧 sessions 目录、冻结 index 或历史 JSONL。
- 这个问题很容易误碰 `/Users/yoyi/.codex`、真实完整 transcript、Codex 内部状态库和敏感数据，所以必须分成诊断和修复两段。
- 本任务不是实现发消息、resume、停止、删除、导出或多 agent 接入。

## 2. 任务目标

让工作台能重新识别可合法展示的旧 Codex 对话：

```text
只读扫描合法会话来源
-> 对比 sqlite / index / sessions / archived_sessions / 旧 state DB
-> 生成缺失会话诊断报告
-> 明确每类缺失的修复策略
-> 用户确认后执行最小修复
-> 工作台智能体页可重新看到旧对话
```

接受为完成：

- 能解释旧对话为什么消失。
- 能把“可安全恢复”的旧对话重新纳入工作台会话目录。
- 不破坏当前 sqlite 主权威。
- 不把 `index.json` 重新变成 transcript 准入名单。

## 3. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/workbench-system-architecture-v1.md`

会话中心前置：

- `tasks/2026-06-03-session-center-foundation-hardening-v1.md`
- `evidence/2026-06-03-session-center-foundation-hardening-v1.md`
- `handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`
- `tasks/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`
- `evidence/2026-06-03-workflow-dispatch-readback-native-parser-v1.md`
- `handoffs/2026-06-03-workflow-dispatch-readback-native-parser-v1-result.md`

主要代码：

- `prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_transcript.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 4. 全局禁止

- 不执行 `codex exec`。
- 不执行 `codex exec resume`。
- 不停止、重启、重开、删除、移动或归档真实 Codex 会话。
- 不写 `/Users/yoyi/.codex`。
- 不修改 Codex 自己的 sqlite、session index、rollout JSONL 或内部状态库。
- 不读取真实完整 transcript 正文作为诊断证据。
- 不读取 auth、token、`.env`、密钥或授权文件。
- 不把 rollout 正文复制进 evidence / handoff。
- 不迁移数据库。
- 不改 workflow state JSON。
- 不写正式事实。
- 不写正式记忆。
- 不把 `index.json` 重新设计为 transcript 准入名单。

如果确实需要读取 `/Users/yoyi/.codex` 下的元数据或目录列表，必须在执行线程明确说明范围并取得用户确认；默认只允许读取文件名、路径、mtime、大小、thread id 等元数据，不读取完整正文。

## 5. 阶段 A：只读诊断

目标：先知道“哪些旧对话消失了、消失在哪一层”。

建议新增只读诊断命令或本地 helper，例如：

- `diagnose_codex_conversation_catalog`
- `scan_codex_session_sources_readonly`
- `compare_codex_catalog_sources`

必须统计这些来源：

1. 当前 Codex sqlite：
   - `codex_db::default_state_db_path()`
   - 所有能安全发现的 `state_*.sqlite`

2. 当前兼容 index：
   - 工作台现有 `codex-index.json` 或当前 index 读模型。

3. session JSONL 文件元数据：
   - `sessions/**/rollout-*.jsonl`
   - `archived_sessions/**/rollout-*.jsonl`
   - 只读文件名、路径、mtime、大小；不读取完整正文。

4. session index JSONL 元数据：
   - `session_index.jsonl`
   - 只读必要字段；不读取消息正文。

5. 工作台已缓存或绑定过的 thread：
   - workflow node bindings。
   - dispatch readback references。
   - 只读引用，不修改。

诊断输出必须至少包含：

```ts
type CodexConversationRecoveryDiagnostic = {
  scanned_at: string;
  sqlite_db_paths: string[];
  current_sqlite_thread_count: number;
  index_thread_count: number;
  rollout_file_count: number;
  archived_rollout_file_count: number;
  missing_from_sqlite_but_rollout_exists: RecoveredConversationCandidate[];
  missing_from_ui_but_in_sqlite: RecoveredConversationCandidate[];
  missing_rollout_for_sqlite_thread: RecoveredConversationCandidate[];
  duplicate_thread_ids: RecoveredConversationCandidate[];
  unsafe_or_unreadable_candidates: RecoveredConversationCandidate[];
  warnings: string[];
};
```

```ts
type RecoveredConversationCandidate = {
  thread_id?: string;
  source_kind: "sqlite" | "index" | "rollout_file" | "archived_rollout_file" | "session_index" | "workflow_binding";
  project_root?: string;
  title?: string;
  rollout_path?: string;
  rollout_exists: boolean;
  archived: boolean;
  updated_at_ms?: number;
  file_mtime_ms?: number;
  reason: string;
  safe_to_recover: boolean;
  recovery_strategy: "none" | "ui_include_from_rollout_metadata" | "compat_catalog_entry" | "sqlite_catalog_refresh" | "manual_review";
};
```

诊断 UI：

- 可以先放在 `智能体` 页的开发者模式 / 管理详情。
- 普通 UI 不默认显示路径大表。
- 必须用人话说明：
  - 发现多少旧对话候选。
  - 为什么当前没有显示。
  - 哪些可以恢复。
  - 哪些需要手动确认。

## 6. 阶段 B：确定缺失原因

至少区分这些原因：

- `sqlite_current_db_missing_old_thread`：当前 sqlite 里没有旧 thread。
- `sqlite_path_changed`：Codex 当前使用了另一个 `state_*.sqlite`。
- `rollout_exists_but_not_cataloged`：rollout 文件存在，但目录读模型没有纳入。
- `archived_rollout_hidden`：旧对话在 archived_sessions。
- `session_index_missing_or_stale`：session_index 或兼容 index 过旧。
- `project_filter_hidden`：UI 过滤 / 项目分组 / 状态过滤把旧对话藏起来。
- `rollout_missing_on_disk`：sqlite 有记录，但 rollout 文件不存在。
- `rollout_outside_allowed_dirs`：路径不在允许目录。
- `parse_failed`：rollout 元数据可见但 parser 无法安全解析。
- `duplicate_or_conflicting_thread`：同 thread id 多来源冲突。

验收：

- 不能只输出“找不到”。
- 必须给每类缺失一个明确原因和下一步策略。

## 7. 阶段 C：最小修复策略

优先级从保守到激进：

1. **UI include from read-only catalog**
   - 如果 rollout 文件合法存在，但不在当前 sqlite，可以把它作为只读旧会话候选显示。
   - 不写 Codex sqlite。
   - 不写 Codex session index。
   - 不读取完整正文，只有用户点开时才走现有安全 parser。

2. **Compat catalog entry**
   - 在工作台自己的兼容 catalog / cache 中记录旧会话元数据。
   - 这个 catalog 属于工作台，不属于 Codex 内部状态。
   - 必须有版本、来源、mtime、rollout path、恢复原因。

3. **SQLite catalog refresh**
   - 只允许刷新工作台自己的读模型或缓存。
   - 不写 Codex 原生 sqlite。
   - 如果执行者认为必须写 Codex sqlite，必须停止并另开高风险任务，由用户明确批准；本任务禁止。

4. **Manual review**
   - 对路径不安全、重复 thread、解析失败、缺 rollout 的候选，只能进入手动检查，不自动恢复。

推荐第一版实现：

- 新增工作台自己的 `codex-conversation-recovery.v1.json` sidecar，放在工作台应用数据目录或 workflow state 同目录。
- 只记录元数据和恢复策略，不记录 transcript 正文。
- 智能体页合并显示：
  - sqlite sessions。
  - 兼容 index fallback。
  - recovery sidecar 中 safe 的旧会话候选。

## 8. 必须保证的安全边界

- 恢复旧会话只是“让工作台识别和展示旧会话元数据 / 可安全读取的 transcript”，不是修改 Codex 软件本身。
- 不改变 Codex 原生 app 的会话列表。
- 不写 `/Users/yoyi/.codex`。
- 不重建 Codex sqlite。
- 不用工作台 sidecar 冒充 Codex 官方 sqlite。
- 不把解析失败的 rollout 当成可读会话。
- 不把路径越界的 rollout 纳入允许读取集合。

## 9. 测试要求

Rust 测试必须使用临时目录和 fixture，不读真实 `/Users/yoyi/.codex`。

至少覆盖：

1. `conversation_recovery_detects_rollout_missing_from_sqlite`
   - rollout 文件存在，sqlite 没有 thread，诊断为可恢复候选。

2. `conversation_recovery_detects_archived_rollout`
   - archived_sessions 下 rollout 存在，诊断为 archived 候选。

3. `conversation_recovery_rejects_rollout_outside_allowed_dirs`
   - 越界路径拒绝恢复。

4. `conversation_recovery_does_not_read_full_transcript`
   - 诊断阶段只读元数据，不调用 transcript parser 全量读取。

5. `conversation_recovery_merges_sqlite_and_recovery_catalog`
   - UI 会话读模型合并 sqlite session 和 recovery sidecar 候选，不重复。

6. `conversation_recovery_marks_missing_rollout_for_manual_review`
   - sqlite 有 thread 但 rollout 不存在，不当成恢复成功。

前端离线测试：

- 智能体页能显示“旧对话恢复候选”或“已恢复旧对话”摘要。
- 搜索和过滤能覆盖恢复候选。
- 恢复候选不能显示为正在运行。
- 错误提示不能说“正式恢复成功”，除非 recovery sidecar 已写入并通过验收。

## 10. 验证命令

至少运行：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib conversation_recovery
cargo test --lib codex_db
cargo test --lib codex_transcript
rustfmt --check src/codex_db.rs src/codex_transcript.rs
```

如新增模块，例如 `codex_conversation_recovery.rs`：

```text
rustfmt --check src/codex_conversation_recovery.rs
```

禁止用真实 `.codex` 全量扫描作为唯一验收。真实机诊断只能作为额外手动验收，并且必须记录是否获得用户批准。

## 11. 手动验收建议

如果用户允许真实只读诊断：

1. 打开工作台智能体页。
2. 记录当前可见 Codex 会话数量。
3. 运行只读诊断。
4. 查看诊断摘要：
   - sqlite thread 数量。
   - rollout 文件数量。
   - archived rollout 数量。
   - 可恢复旧对话候选数量。
   - 需要手动检查数量。
5. 用户确认恢复策略后执行恢复。
6. 回到智能体页，确认旧对话重新出现。
7. 随机打开 1 到 3 个恢复会话，只读取必要 transcript；不把完整正文写入 evidence。

## 12. evidence / handoff 要求

完成后新增：

- `evidence/2026-06-03-codex-software-conversation-recovery-v1.md`
- `handoffs/2026-06-03-codex-software-conversation-recovery-v1-result.md`

必须记录：

- 是否读取 `/Users/yoyi/.codex`；如果读取，读取了哪些元数据，是否读取正文。
- 是否写 `/Users/yoyi/.codex`；本任务预期必须是否。
- 诊断发现的缺失类别和数量。
- 恢复策略。
- 是否新增工作台 sidecar。
- 是否改 Codex 原生 sqlite；本任务预期必须是否。
- 哪些旧对话恢复到 UI。
- 哪些仍需手动处理。

## 13. 完成定义

接受为完成：

- 有只读诊断能力。
- 有明确缺失原因分类。
- 可安全恢复的旧对话能进入工作台智能体页。
- 不破坏 sqlite 主权威和 Rust parser 主路径。
- 不写 Codex 原生状态。

不接受为完成：

- 发消息 / resume / stop / restart 完成。
- Codex 原生 app 会话列表被修复。
- 删除 / 导出 / 收藏 / 分享完成。
- 多 agent 会话底座完成。
- 真实完整 transcript 已被读取或复制进 evidence。
