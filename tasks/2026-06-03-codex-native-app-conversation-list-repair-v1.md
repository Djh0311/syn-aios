# Task Package：Codex Native App Conversation List Repair v1

状态：待执行。  
用途：修复“Codex 原生软件自己的对话列表里，有一些旧对话消失 / 不被 Codex 识别”的问题。  
执行方式：先只读诊断，输出明确修复方案；用户确认方案后才能写 Codex 原生数据。  

## 0. 纠偏说明

本任务包不是 `2026-06-03-codex-software-conversation-recovery-v1.md` 的继续执行。

上一份任务包把用户的问题错误解释成“工作台智能体页识别旧 Codex 会话”，实际做的是工作台侧只读恢复诊断、工作台 sidecar 和工作台 UI 合并展示。那不等于修复 Codex 原生 app 的会话列表。

本任务包只针对 Codex 原生软件：

```text
Codex 原生 app 左侧 / 会话列表
-> 为什么某些旧对话不显示
-> Codex 自己的数据源里缺了什么 / 被隐藏了什么 / 索引坏了什么
-> 备份后最小修复
-> 重启或刷新 Codex 原生 app 后旧对话重新可见
```

工作台里能看到旧会话，不算本任务完成。

## 1. 先说薄弱点

- 这会触碰 Codex 原生软件的数据边界，比工作台读模型修复风险高。
- 当前不能假设 Codex 原生列表到底以哪个数据源为准：可能是 sqlite、session index、rollout 元数据、缓存或它们的组合。
- 旧对话消失可能是 UI 过滤、归档、索引陈旧、数据库切换、schema 变更、权限问题或数据损坏，不能先入为主。
- 不拿到至少一个“消失旧对话”的线索时，只能做全量差异诊断，不能证明某个具体对话已修复。
- 本任务不是工作台开发任务；不要把工作台 `codex-conversation-recovery.v1.json` sidecar 当成 Codex 官方状态。

## 2. 任务目标

修复 Codex 原生 app 对旧对话的识别：

```text
确认症状和样本
-> 只读识别 Codex 原生 app 当前使用的数据源
-> 对比旧会话文件 / sqlite / session index / archived 状态 / app 缓存
-> 输出每类缺失原因和精确修复方案
-> 用户确认后备份
-> 最小写入或刷新 Codex 原生数据
-> 重启 / 刷新 Codex 原生 app
-> 确认旧对话回到 Codex 原生列表
```

接受为完成：

- 能说明 Codex 原生 app 为什么不识别旧对话。
- 至少一种可安全修复的缺失类型被修复。
- 用户指定或抽样的旧对话能在 Codex 原生 app 会话列表重新出现。
- 当前新对话没有丢失。
- 所有写入都有备份和回滚方案。

不接受为完成：

- 只让工作台智能体页显示旧会话。
- 只写工作台 sidecar。
- 只证明 sqlite / rollout 文件存在。
- 只输出“可能是 index 问题”但没有修复或明确不可修复原因。

## 3. 必须先确认

执行前必须向用户确认或记录：

- 用户说的是 Codex 原生软件自己的会话列表，不是工作台智能体页。
- 是否允许只读访问 Codex 原生数据目录，例如 `/Users/yoyi/.codex`。
- 是否有 1 到 3 个消失旧对话的线索：
  - 大概标题。
  - 大概日期。
  - 所属项目路径。
  - 可能的 thread id。
  - 截图或用户描述。
- 是否允许在诊断后进入写入修复。只读诊断通过不等于允许写。

如果用户没有具体旧对话线索，也可以执行全量诊断，但 evidence 必须写明“没有样本，只能按数据源差异判断”。

## 4. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `docs/workbench-system-architecture-v1.md`

需要作为反例读取：

- `tasks/2026-06-03-codex-software-conversation-recovery-v1.md`
- `evidence/2026-06-03-codex-software-conversation-recovery-v1.md`
- `handoffs/2026-06-03-codex-software-conversation-recovery-v1-result.md`

读取目的：

- 明确上一轮只是工作台侧恢复，不是 Codex 原生 app 修复。
- 复用其中的只读诊断数据只能作为参考，不能作为完成结论。

## 5. 全局禁止

默认禁止：

- 不执行 `codex exec`。
- 不执行 `codex exec resume`。
- 不读取真实完整 rollout / transcript 正文作为诊断证据。
- 不复制真实 transcript 正文进 evidence / handoff。
- 不读取 auth、token、`.env`、密钥或授权文件。
- 不删除、移动或改写 rollout JSONL。
- 不重建整个 Codex 数据目录。
- 不在没有备份的情况下写 sqlite、session index、缓存或任何 Codex 原生状态。
- 不用工作台 sidecar 冒充 Codex 原生状态。
- 不把工作台 UI 验收当成 Codex 原生 app 验收。

需要用户明确批准后才允许：

- 读取 `/Users/yoyi/.codex` 下的元数据。
- 读取 Codex 原生 sqlite 的表结构和行级元数据。
- 读取 `session_index.jsonl` 的必要元数据。
- 写 Codex 原生 sqlite、session index 或缓存。
- 重启 Codex 原生 app 或要求用户重启。

即使用户批准写入，也仍禁止读取或写入 auth/token/secret。

## 6. 阶段 A：只读诊断

目标：确认 Codex 原生 app 当前到底依赖什么数据源，以及旧对话在哪一层丢失。

只读诊断必须输出：

```ts
type CodexNativeConversationListDiagnostic = {
  scanned_at: string;
  codex_app_version?: string;
  codex_cli_version?: string;
  codex_home?: string;
  native_data_sources: CodexNativeDataSource[];
  suspected_primary_source?: "state_sqlite" | "session_index" | "rollout_files" | "app_cache" | "unknown";
  visible_list_evidence: CodexNativeVisibleListEvidence;
  source_counts: CodexNativeSourceCounts;
  missing_candidates: CodexNativeMissingConversationCandidate[];
  hidden_or_archived_candidates: CodexNativeMissingConversationCandidate[];
  corrupt_or_unreadable_candidates: CodexNativeMissingConversationCandidate[];
  duplicate_or_conflicting_candidates: CodexNativeMissingConversationCandidate[];
  warnings: string[];
};
```

```ts
type CodexNativeDataSource = {
  source_kind: "state_sqlite" | "session_index" | "rollout_dir" | "archived_rollout_dir" | "app_cache" | "unknown";
  path: string;
  exists: boolean;
  readable: boolean;
  writable: boolean;
  schema_or_format?: string;
  record_count?: number;
  last_modified_ms?: number;
  warnings: string[];
};
```

```ts
type CodexNativeMissingConversationCandidate = {
  candidate_id: string;
  thread_id?: string;
  title?: string;
  project_root?: string;
  rollout_path?: string;
  updated_at_ms?: number;
  evidence_sources: string[];
  missing_from_sources: string[];
  suspected_reason:
    | "session_index_missing"
    | "state_sqlite_missing"
    | "archived_hidden"
    | "hidden_flag_or_filter"
    | "app_cache_stale"
    | "rollout_exists_but_metadata_missing"
    | "sqlite_points_to_missing_rollout"
    | "schema_changed"
    | "parse_or_permission_failed"
    | "duplicate_or_conflict"
    | "unknown";
  safe_repair_strategy:
    | "none"
    | "refresh_app_cache"
    | "unhide_or_unarchive_metadata"
    | "rebuild_session_index_entry"
    | "insert_state_sqlite_metadata"
    | "repair_sqlite_rollout_pointer"
    | "manual_review";
  requires_write: boolean;
  requires_user_confirmation: boolean;
  risk_level: "low" | "medium" | "high";
};
```

诊断必须检查：

- Codex 原生 app / CLI 版本。
- Codex home 实际路径。
- 当前 `state_*.sqlite` 是否有多个，哪个最近被使用。
- sqlite 表结构、关键表、thread 数、archived/hidden 字段、rollout path 字段。
- `session_index.jsonl` 是否存在、行数、thread id / title / cwd / updated 元数据。
- `sessions/**/rollout-*.jsonl` 文件元数据。
- `archived_sessions/**/rollout-*.jsonl` 文件元数据。
- app 缓存或索引文件是否存在，是否比 sqlite / session index 陈旧。
- 如果能安全判断，记录 Codex 原生 app 当前可见列表数量；如果不能自动读取，要求用户提供截图或手动数量。

诊断阶段不允许修复。

## 7. 阶段 B：原因分类

必须把缺失原因分清楚，不能只说“旧对话不见了”。

至少区分：

- `session_index_missing`：rollout / sqlite 有记录，但 session index 缺条目。
- `state_sqlite_missing`：rollout / session index 有记录，但当前 state sqlite 缺 thread。
- `archived_hidden`：旧对话被归档或位于 archived 目录，原生列表默认隐藏。
- `hidden_flag_or_filter`：sqlite 或 index 中有隐藏 / archived / deleted / project filter 标记。
- `app_cache_stale`：数据源已有记录，但 app 缓存或前端状态没有刷新。
- `rollout_exists_but_metadata_missing`：rollout 文件存在，但缺少 app 列表需要的元数据。
- `sqlite_points_to_missing_rollout`：sqlite 有记录，但 rollout path 不存在。
- `schema_changed`：Codex 版本升级后旧字段不再被原生列表读取。
- `parse_or_permission_failed`：文件权限或格式导致原生 app 无法读取。
- `duplicate_or_conflict`：同一 thread 在多个来源冲突。

每类原因都必须给出：

- 证据。
- 影响的候选数量。
- 是否可自动修复。
- 修复风险。
- 回滚方式。

## 8. 阶段 C：修复方案确认

只读诊断后，执行者必须先输出修复方案，等用户确认后才能写。

方案必须包含：

- 要写哪些文件。
- 每个文件写入前的备份路径。
- 要新增 / 修改 / 删除哪些记录。
- 为什么这是最小修复。
- 如果失败如何回滚。
- 修复后如何验证 Codex 原生 app 列表。

用户确认文案必须具体到文件级，例如：

```text
允许写入：
- /Users/yoyi/.codex/state_5.sqlite
- /Users/yoyi/.codex/session_index.jsonl

不允许写入：
- rollout JSONL 正文
- auth/token/secret
```

不能用“可以”自动解释为允许所有写入；必须让执行者在 evidence 里记录确认范围。

## 9. 阶段 D：允许的最小修复类型

按风险从低到高：

1. **刷新 app 缓存 / 重启验证**
   - 只在确认数据源已有记录、问题是缓存陈旧时使用。
   - 可要求用户重启 Codex app。
   - 不写数据。

2. **取消隐藏 / 取消归档元数据**
   - 只改明确的 hidden / archived 字段。
   - 必须备份。
   - 不移动 rollout 文件。

3. **重建 session index 条目**
   - 只从已有 sqlite / rollout 元数据重建列表所需字段。
   - 不写 transcript 正文。
   - 保留旧 index 备份。

4. **插入 state sqlite 元数据**
   - 仅当已证明 Codex 原生 app 以 state sqlite 为列表权威。
   - 只插入列表识别所需元数据。
   - 必须使用事务。
   - 必须先在临时 sqlite fixture 复现并测试。

5. **修复 sqlite rollout pointer**
   - 仅当 sqlite thread 存在但 rollout path 明确错误，且正确文件在合法目录内。
   - 不改 rollout 内容。
   - 必须记录 before / after。

不允许：

- 批量迁移全部历史会话。
- 删除旧记录。
- 改写 rollout JSONL 正文。
- 人工伪造 transcript。
- 直接覆盖整个 sqlite 或 session index。
- 为了让列表显示而伪造时间、标题或项目路径。

## 10. 实现建议

建议新增独立诊断 / 修复脚本或后端 helper，不要混进工作台会话中心读模型：

- `tools/codex-native-conversation-diagnostic/README.md`
- `tools/codex-native-conversation-diagnostic/dry-run.*`
- `tools/codex-native-conversation-diagnostic/apply-repair.*`

如果在 Tauri 后端实现，也必须与工作台 `AgentView` 读模型隔离：

- 诊断命令名必须包含 `codex_native_app`。
- 不复用 `codex-conversation-recovery.v1.json` 作为修复目标。
- 不把工作台 safe candidate 合并展示当成修复结果。

推荐命令：

- `diagnose_codex_native_app_conversation_list`
- `plan_codex_native_app_conversation_repair`
- `apply_codex_native_app_conversation_repair`

`apply` 命令必须要求：

- `dry_run_id`
- `approved_write_paths`
- `expected_backup_paths`
- `expected_candidate_ids`
- `user_confirmation_text`

## 11. 测试要求

测试必须用临时 fixture，不读真实 `/Users/yoyi/.codex`。

至少覆盖：

1. `native_conversation_diagnostic_detects_session_index_missing`
   - rollout / sqlite 存在，session index 缺失，诊断为 `session_index_missing`。

2. `native_conversation_diagnostic_detects_state_sqlite_missing`
   - rollout / session index 存在，sqlite 缺 thread，诊断为 `state_sqlite_missing`。

3. `native_conversation_diagnostic_detects_archived_hidden`
   - archived 候选不自动当成丢失。

4. `native_conversation_repair_requires_explicit_write_approval`
   - 没有文件级批准时，写入必须拒绝。

5. `native_conversation_repair_writes_backup_before_mutation`
   - 任意写入前必须生成备份。

6. `native_conversation_repair_rejects_rollout_body_mutation`
   - 修复不能改 rollout 正文。

7. `native_conversation_repair_sqlite_transaction_rolls_back_on_failure`
   - sqlite 修复失败必须回滚。

8. `native_conversation_repair_does_not_use_workbench_sidecar_as_success`
   - 工作台 sidecar 存在不等于 Codex 原生列表修复成功。

## 12. 验证命令

按实现位置选择。

如果新增 Rust 后端：

```text
cargo test --lib codex_native_conversation
cargo test --lib
rustfmt --check src/codex_native_conversation_repair.rs
```

如果新增脚本：

```text
<script-test-command>
<script-dry-run-command> --fixture
```

如需真实机验证，必须在用户批准后执行，并记录：

- 读取了哪些 `.codex` 元数据。
- 写入了哪些 `.codex` 文件。
- 备份路径。
- 修复前 Codex 原生可见数量。
- 修复后 Codex 原生可见数量。
- 用户抽样确认哪些旧对话重新出现。

## 13. 真实验收标准

必须验收 Codex 原生 app，不是工作台。

手动验收：

1. 用户打开 Codex 原生 app。
2. 记录修复前会话列表数量或截图。
3. 执行只读诊断。
4. 用户确认修复方案。
5. 执行修复。
6. 重启或刷新 Codex 原生 app。
7. 用户确认旧对话重新出现在 Codex 原生列表。
8. 抽样打开 1 到 3 条恢复会话，确认可读。
9. 确认新近会话仍在，没有被覆盖或隐藏。

如果无法让旧对话重新出现在 Codex 原生 app，必须输出不可修复原因，不能把工作台可见当作成功。

## 14. evidence / handoff 要求

完成后新增：

- `evidence/2026-06-03-codex-native-app-conversation-list-repair-v1.md`
- `handoffs/2026-06-03-codex-native-app-conversation-list-repair-v1-result.md`

必须记录：

- 用户原始问题是 Codex 原生 app 列表，不是工作台。
- 上一份工作台恢复任务已被标为 superseded / wrong target。
- 诊断读取范围。
- 是否读取 `/Users/yoyi/.codex` 正文；预期必须是否。
- 是否写 `/Users/yoyi/.codex`。
- 写入前备份路径。
- 缺失原因分类和数量。
- 修复策略和具体写入。
- Codex 原生 app 修复前后验收结果。
- 哪些候选没有修复及原因。

## 15. 完成定义

接受为完成：

- Codex 原生 app 会话列表问题被诊断清楚。
- 用户确认后的最小修复已执行。
- 至少一类可安全修复的旧对话重新出现在 Codex 原生 app。
- 写入前有备份，失败有回滚方案。
- 没有读取或泄露真实 transcript 正文。

不接受为完成：

- 只修工作台智能体页。
- 只写工作台 sidecar。
- 只做只读诊断但没有修复或不可修复结论。
- 未经用户确认写 `.codex`。
- 修改 rollout 正文。
- 破坏当前新对话。
