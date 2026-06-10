# Evidence：Codex Native App Conversation List Repair v1

状态：阶段 A/B/C 已完成；阶段 D 已执行四步修复。磁盘侧元数据验证通过；当前运行中的 Codex app 仍持有旧列表缓存，需完全退出并重开后验收。

## 用户确认

- 问题对象：Codex 原生 app 左侧 / 会话列表，不是工作台智能体页。
- 用户允许：只读读取 `/Users/yoyi/.codex` 元数据。
- 用户样本：截图显示 `金融`、`harness-engineering`、`gamework`、`game-harness`、`agents`、`videocut` 等项目下为“暂无对话”，`kt-erp` 只显示一条近期对话。
- 用户允许的第一阶段写入范围：`/Users/yoyi/.codex/.codex-global-state.json`；允许创建 `/Users/yoyi/.codex/backups_state/native-conversation-list-repair/` 备份；不允许写 sqlite、session index、rollout、auth/token/config。
- 用户反馈第一阶段运行后仍未成功；随后通过权限确认允许写 `/Users/yoyi/.codex/session_index.jsonl` 并创建备份。
- 用户反馈第二阶段运行后仍未成功；随后通过权限确认允许再次写 `/Users/yoyi/.codex/session_index.jsonl`，把保存项目的代表线程提升到 native app 最近窗口。
- 用户反馈第三阶段后仍未成功；随后通过权限确认允许写 `/Users/yoyi/.codex/state_5.sqlite`，把保存项目的代表线程提升到 sqlite `updated_at` 最近窗口。

## 读取范围

已读：

- `/Users/yoyi/.codex/state_5.sqlite`：表结构、thread 计数、`id/cwd/archived/rollout_path/updated_at_ms` 等元数据。
- `/Users/yoyi/.codex/session_index.jsonl`：行级 id / 名称 /更新时间元数据统计。
- `/Users/yoyi/.codex/sessions/**/rollout-*.jsonl`：只读文件名计数。
- `/Users/yoyi/.codex/archived_sessions/**/rollout-*.jsonl`：只读文件名计数。
- `/Users/yoyi/.codex/.codex-global-state.json`：项目列表、项目顺序、thread workspace hints、heartbeat permission 计数。

未按计划读取：

- auth/token/secret 文件正文。
- rollout 正文。
- sqlite transcript 正文。

偏差记录：

- 有一次字符串搜索范围过大，误扫到 `backups_state/provider-sync/**/session-meta-backup.json` 这类备份文件，工具输出中可能包含历史会话片段。后续诊断结论没有使用这些内容，本文不记录其中正文。

## 初始数据源诊断

- Codex CLI：`codex-cli 0.134.0`。
- `/Applications/Codex.app` 和 `/Applications/Codex++.app` 的标准 `CFBundleShortVersionString` / `CFBundleVersion` 字段不存在。
- `state_5.sqlite` 表：`threads`、`thread_dynamic_tools`、`thread_spawn_edges`、`agent_jobs`、`agent_job_items` 等。
- `threads` 总数：379。
- sqlite duplicate thread id：0。
- `sessions` rollout 文件：321。
- `archived_sessions` rollout 文件：58。
- rollout id 总数：379。
- sqlite thread id 与 rollout id 完全匹配：无缺失。
- sqlite `rollout_path` 不存在数量：0。
- `session_index.jsonl`：330 行，266 个唯一 id，45 个重复 id，64 行重复；相比 sqlite 缺 115 个 thread，其中 104 个是未归档可见 thread。

截图项目在 sqlite 中的可见 thread 数：

| 项目 | sqlite 可见 | `thread-workspace-root-hints` 匹配 | heartbeat permission 匹配 |
| --- | ---: | ---: | ---: |
| `/Users/yoyi/Documents/金融` | 5 | 0 | 2 |
| `/Users/yoyi/harness engineering/harness-engineering` | 11 | 0 | 2 |
| `/Users/yoyi/gamework` | 48 | 0 | 11 |
| `/Users/yoyi/Documents/Codex/game-harness` | 16 | 0 | 2 |
| `/Users/yoyi/Desktop/superpowers/agents` | 1 | 0 | 0 |
| `/Users/yoyi/Desktop/kt-erp` | 106 | 0 | 5 |
| `/Users/yoyi/videocut` | 1 | 0 | 0 |

保存的 13 个 workspace root 下，sqlite 未归档可见 thread 为 285 条；当前 `thread-workspace-root-hints` 对这些项目匹配为 0。

## 原因分类

主因：

- `app_cache_stale` / 原生 app 项目归属缓存缺失。sqlite 和 rollout 都完整，但 `.codex-global-state.json` 的 `thread-workspace-root-hints` 未把可见 thread 归回保存的项目 root。

次因：

- `session_index_missing`：`session_index.jsonl` 比 sqlite 少 115 个 thread，其中 104 个可见。
- `duplicate_or_conflict`：`session_index.jsonl` 有 45 个重复 id。

非主因：

- `state_sqlite_missing`：未成立。
- `sqlite_points_to_missing_rollout`：未成立。
- `parse_or_permission_failed`：未发现于本轮元数据层。

## 修复尝试

已按用户确认范围尝试写入：

- 备份：`/Users/yoyi/.codex/backups_state/native-conversation-list-repair/20260603144529/.codex-global-state.json.before`
- 写入目标：`/Users/yoyi/.codex/.codex-global-state.json`
- 写入内容：从 sqlite `threads.id/cwd/archived` 重建保存 workspace root 的未归档 thread hints。
- 写入结果：脚本报告新增 284 条、修正 1 条，total hints 变为 309。

第一次写入后的只读验证曾发现：

- 当前 `.codex-global-state.json` 又回到 total hints 25，且全部映射到 `/Users/yoyi/Documents/Codex`。
- 文件 mtime 在写入后又更新为 2026-06-03 22:50:01。
- 判断：运行中的 Codex 原生 app 把内存中的旧全局状态重新刷回磁盘，覆盖了外部写入。

## 已新增工具

- `tools/codex-native-conversation-diagnostic/repair-global-state-hints.mjs`
- `tools/codex-native-conversation-diagnostic/repair-session-index.mjs`
- `tools/codex-native-conversation-diagnostic/README.md`

dry-run 结果：

```json
{
  "savedWorkspaceRoots": 13,
  "sqliteThreads": 379,
  "added": 284,
  "changed": 1,
  "unchanged": 0,
  "skipped": 94,
  "totalHintsBefore": 25,
  "totalHintsAfter": 309
}
```

## 当前结论

用户反馈第一阶段运行后仍未成功。复查显示 `.codex-global-state.json` 的 `thread-workspace-root-hints` 已基本持久化，保存 workspace 的可见 thread 已归属到项目；但 `session_index.jsonl` 仍缺大量未归档可见 thread，且仍有重复 id。下一阶段最小修复对象应改为 `session_index.jsonl`。

2026-06-03 复查只读计数：

- `session_index.jsonl`：330 行，267 个唯一 id，64 条重复额外行，0 个解析错误。
- `state_5.sqlite`：380 个 thread；当前会话进行中，计数可能随本线程更新。
- sqlite 中未出现在 `session_index.jsonl` 的 thread：115。
- 未归档可见 thread 缺失：104。
- 保存 workspace 下未归档可见 thread 缺失：84。
- 新增工具：`tools/codex-native-conversation-diagnostic/repair-session-index.mjs`。
- dry-run 显示将删除 64 条重复额外行，补齐 104 条未归档可见 thread，生成后的索引为 371 条唯一记录。
- 新增条目的 `thread_name` 只使用 sqlite `title` 的前 36 个字符，避免长标题污染左侧列表；脚本输出不打印标题。

## 第二阶段写入结果

已按用户确认范围写入：

- 备份：`/Users/yoyi/.codex/backups_state/native-conversation-list-repair/20260603150605/session_index.jsonl.before`
- 写入目标：`/Users/yoyi/.codex/session_index.jsonl`
- 写入内容：去重保留每个 id 最新索引行，并从 sqlite 未归档 thread 补齐缺失索引。

写后验证：

- `session_index.jsonl`：371 行，371 个唯一 id。
- 解析错误：0。
- 重复额外行：0。
- sqlite 未归档可见 thread：322。
- 未进入 `session_index.jsonl` 的未归档可见 thread：0。
- 保存 workspace 下未进入 `session_index.jsonl` 的未归档可见 thread：0。
- `.codex-global-state.json` dry-run 仅剩当前新会话 1 条 hint 未写入；截图旧项目的可见 thread hints 已存在。

第二阶段修复前，截图相关项目在 `session_index.jsonl` 中缺失：

| 项目 | sqlite 可见 | `session_index` 已有 | 缺失 |
| --- | ---: | ---: | ---: |
| `/Users/yoyi/Documents/金融` | 5 | 1 | 4 |
| `/Users/yoyi/harness engineering/harness-engineering` | 11 | 9 | 2 |
| `/Users/yoyi/gamework` | 48 | 34 | 14 |
| `/Users/yoyi/Documents/Codex/game-harness` | 16 | 9 | 7 |
| `/Users/yoyi/Desktop/kt-erp` | 106 | 101 | 5 |

第二阶段写后验证中，保存 workspace 下未进入 `session_index.jsonl` 的未归档可见 thread 已归零。

## 第三阶段窗口修复

用户反馈第二阶段后仍未恢复。进一步源码与 app 接口诊断显示：

- Codex app 版本：`26.601.21317`，bundle id：`com.openai.codex`。
- app bundle 代码中存在 `.codex-global-state.json`、`thread-workspace-root-hints`、`electron-saved-workspace-roots` 等键。
- `codex_app.list_threads` 当前运行态只返回约 25 条最近 thread；查询 `金融`、`gamework`、`game-harness`、`harness-engineering`、`agents`、`videocut` 均为空。
- 磁盘 `session_index.jsonl` 中，截图项目代表 thread 原本排名较后：`金融` 123、`gamework` 129+、`game-harness` 183+、`agents` 363、`videocut` 366+。

已新增工具：

- `tools/codex-native-conversation-diagnostic/promote-saved-projects-in-session-index.mjs`

已按用户确认范围写入：

- 备份：`/Users/yoyi/.codex/backups_state/native-conversation-list-repair/20260604040616/session_index.jsonl.before-promote`
- 写入目标：`/Users/yoyi/.codex/session_index.jsonl`
- 写入内容：为 10 个保存 workspace 各选 1 个代表 thread，将该 thread 在 `session_index.jsonl` 的 `updated_at` 提升到当前时间附近，使其进入 native app 最近窗口。
- 未写：sqlite、rollout、auth/token/config。

写后磁盘验证：

- `session_index.jsonl`：371 行，371 个唯一 id，0 个解析错误，0 个重复。
- 第三阶段 dry-run：`promotedCount: 0`，说明代表 thread 已在窗口内。
- 目标项目代表排名：
  - `/Users/yoyi/Desktop/superpowers/agents`：1
  - `/Users/yoyi/Documents/Codex/game-harness`：2
  - `/Users/yoyi/Documents/金融`：5
  - `/Users/yoyi/gamework`：8
  - `/Users/yoyi/harness engineering/harness-engineering`：9
  - `/Users/yoyi/videocut`：10

运行中 app 接口验证：

- 写后立即调用 `codex_app.list_threads` 查询上述项目仍为空。
- 判断：当前运行中的 Codex app 已缓存线程窗口，不会立即重读磁盘 `session_index.jsonl`。下一步必须完全退出并重新打开 app 后验收。

## 第四阶段 sqlite 窗口修复

用户反馈第三阶段后仍未恢复。进一步对比显示：

- app bundle 中桌面端会通过内部 client 调用 `listThreads({ limit, sortKey: "updated_at", archived: false })`。
- `codex_app.list_threads` 返回的近期 25 条与 `state_5.sqlite.threads.updated_at_ms` 排序一致，而不是 `session_index.jsonl` 排序。
- 截图项目在 sqlite 中有未归档可见 thread，但最新 thread 排名很靠后：
  - `/Users/yoyi/gamework`：约第 107。
  - `/Users/yoyi/harness engineering/harness-engineering`：约第 124。
  - `/Users/yoyi/Documents/金融`：约第 127。
  - `/Users/yoyi/Documents/Codex/game-harness`：约第 182。
  - `/Users/yoyi/Desktop/superpowers/agents`：约第 316。
  - `/Users/yoyi/videocut`：约第 318。

已新增工具：

- `tools/codex-native-conversation-diagnostic/promote-saved-projects-in-state-sqlite.mjs`

已按用户确认范围写入两次：

- 备份 1：`/Users/yoyi/.codex/backups_state/native-conversation-list-repair/20260604042052/state_5.sqlite.before-promote`
- 写入 1：提升 9 个保存 workspace 代表 thread 的 `threads.updated_at` / `threads.updated_at_ms`。
- 备份 2：`/Users/yoyi/.codex/backups_state/native-conversation-list-repair/20260604042435/state_5.sqlite.before-promote`
- 写入 2：补提升 `/Users/yoyi/gameai/crazytown` 的 1 个代表 thread，避免第一轮提升后它被挤出前 25。
- 未读/未写：rollout 正文、auth/token/config。

写后磁盘验证：

- 第四阶段 dry-run：`promotedCount: 0`。
- `session_index.jsonl` dry-run：371 行，371 个唯一 id，0 个解析错误，0 个重复，保存 workspace 缺失为 `{}`。
- `.codex-global-state.json` dry-run：截图旧项目 hints 已存在；仅当前 `/Users/yoyi/workspace` 新线程有 1 条 hint 未写入。
- sqlite 前 25 已包含截图目标项目：
  - `/Users/yoyi/harness engineering/harness-engineering`
  - `/Users/yoyi/Documents/金融`
  - `/Users/yoyi/gamework`
  - `/Users/yoyi/Documents/Codex/game-harness`
  - `/Users/yoyi/Desktop/superpowers/agents`
  - `/Users/yoyi/videocut`
  - `/Users/yoyi/Desktop/kt-erp`

运行中 app 接口验证：

- 写后立即调用 `codex_app.list_threads`，仍返回旧窗口；查询 `金融`、`gamework`、`game-harness`、`harness-engineering`、`agents`、`videocut` 仍为空。
- 判断：当前运行中的 Codex app / app worker 已缓存线程窗口，不会立即重读 sqlite 和 `session_index.jsonl`。

仍禁止直接写：

- rollout JSONL
- auth/token/config
- Local Storage / Session Storage，除非用户单独确认

下一步需要用户完全退出并重新打开 Codex 原生 app，验收左侧项目列表是否恢复。若重启后仍失败，下一层应定位 Electron renderer/worker 的持久缓存或 Chromium Local Storage，而不是继续改 sqlite/rollout。
