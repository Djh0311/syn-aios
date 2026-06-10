# Handoff：Codex Native App Conversation List Repair v1

状态：未最终完成。第一阶段 `global-state` 修复后用户反馈仍未恢复；第二阶段补齐 `session_index.jsonl` 后仍未恢复；第三阶段提升 `session_index` 后仍未恢复；第四阶段已把保存项目代表线程提升进 sqlite 最近窗口。磁盘侧验证通过，真实原生 app 左侧需完全退出重开后验收。

## 做了什么

- 确认用户问题是 Codex 原生 app 左侧列表消失，不是工作台智能体页。
- 只读诊断 `/Users/yoyi/.codex` 元数据。
- 判断对话本体未丢失：sqlite 与 rollout 文件完全匹配。
- 第一阶段判断主因是 `.codex-global-state.json` 的项目归属 hints 缺失 / 被运行中的 app 覆盖。
- 新增 helper：`tools/codex-native-conversation-diagnostic/repair-global-state-hints.mjs`。
- 已尝试一次写入 `.codex-global-state.json`，并备份到：
  `/Users/yoyi/.codex/backups_state/native-conversation-list-repair/20260603144529/.codex-global-state.json.before`
- 后续 dry-run 显示 hints 已基本持久化；用户仍反馈 app 没恢复。
- 复查发现 `session_index.jsonl` 仍缺 104 条未归档可见 thread，且有 64 条重复额外行。
- 新增 helper：`tools/codex-native-conversation-diagnostic/repair-session-index.mjs`。
- 已写入 `/Users/yoyi/.codex/session_index.jsonl`，备份：
  `/Users/yoyi/.codex/backups_state/native-conversation-list-repair/20260603150605/session_index.jsonl.before`
- 写后验证：371 行，371 个唯一 id，0 个解析错误，0 个重复；322 条 sqlite 未归档可见 thread 全部进入索引。
- 用户反馈第二阶段仍不恢复。源码和 app 接口显示当前 app/list API 只暴露约 25 条最近 thread，旧项目代表 thread 在 `session_index` 中排名 100+ 到 300+。
- 新增 helper：`tools/codex-native-conversation-diagnostic/promote-saved-projects-in-session-index.mjs`。
- 已再次写入 `/Users/yoyi/.codex/session_index.jsonl`，备份：
  `/Users/yoyi/.codex/backups_state/native-conversation-list-repair/20260604040616/session_index.jsonl.before-promote`
- 写后验证：目标项目代表 thread 已进入 `session_index` 前 10；第三阶段 dry-run 为 `promotedCount: 0`。
- 写后当前运行中的 `codex_app.list_threads` 仍查不到这些项目，判断为运行中 app 内存缓存未刷新。
- 继续诊断发现原生 app/list API 实际按 `state_5.sqlite.threads.updated_at_ms` 窗口返回近期线程，`session_index` 提升不会改变当前左侧窗口。
- 新增 helper：`tools/codex-native-conversation-diagnostic/promote-saved-projects-in-state-sqlite.mjs`。
- 已写入 `/Users/yoyi/.codex/state_5.sqlite` 两次，均使用 sqlite `.backup` 先备份：
  - `/Users/yoyi/.codex/backups_state/native-conversation-list-repair/20260604042052/state_5.sqlite.before-promote`
  - `/Users/yoyi/.codex/backups_state/native-conversation-list-repair/20260604042435/state_5.sqlite.before-promote`
- 写入内容：只更新保存项目代表 thread 的 `threads.updated_at` / `threads.updated_at_ms`，不读写 rollout 正文，不碰 auth/token/config。
- 写后验证：`promote-saved-projects-in-state-sqlite.mjs --dry-run` 为 `promotedCount: 0`；sqlite 前 25 已包含截图目标项目。
- 写后当前运行中的 `codex_app.list_threads` 仍是旧窗口，说明 native app / worker 仍持有运行态缓存。

## 当前最小修复步骤

建议用户现在完全退出 Codex 原生 app，再重新打开验收。不要从当前会话里 kill/restart Codex app。

如需复查元数据，可运行：

```bash
cd /Users/yoyi/workspace/product-line
node tools/codex-native-conversation-diagnostic/repair-session-index.mjs --dry-run
node tools/codex-native-conversation-diagnostic/repair-global-state-hints.mjs --dry-run
node tools/codex-native-conversation-diagnostic/promote-saved-projects-in-session-index.mjs --dry-run
node tools/codex-native-conversation-diagnostic/promote-saved-projects-in-state-sqlite.mjs --dry-run
```

重新打开后验收：

- `金融` 应至少恢复 5 条可见对话。
- `harness-engineering` 应至少恢复 11 条。
- `gamework` 应至少恢复 48 条。
- `game-harness` 应至少恢复 16 条。
- `agents` 应至少恢复 1 条。
- `kt-erp` 应至少恢复 106 条。
- `videocut` 应至少恢复 1 条。

## 不要做

- 不要写 `session_index.jsonl`，除非用户另行文件级确认。
- 不要修改 rollout JSONL。
- 不要读取或写入 auth/token/config。
- 不要读取或写入 Chromium Local Storage / Session Storage，除非用户单独确认。

## 风险

- 如果 app 仍在运行，左侧可能继续显示旧内存缓存，必须重启后再判断。
- 新增 `session_index` 条目的 `thread_name` 来自 sqlite `title` 的前 36 个字符；这避免长标题污染左侧列表，但可能不如 app 官方生成标题自然。
- 第三阶段会让每个保存项目的一个代表 thread 在 UI 中看起来像最近更新；这是显示修复，不改 sqlite 的真实更新时间。
- 第四阶段会让每个保存项目的一个代表 thread 在 UI 中看起来像最近更新；这是显示修复，改的是 sqlite thread 元数据，不改 rollout 本体。
- 如果完全重启后仍不生效，需要定位 Electron renderer/worker 持久缓存或 Local Storage。继续诊断时仍要避开 rollout 正文、auth/token/config。
