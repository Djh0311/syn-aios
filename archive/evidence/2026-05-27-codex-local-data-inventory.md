# Codex 本地数据盘点证据

## 结论先说

薄弱点：

- `session_index.jsonl` 和 `state_5.sqlite` 的线程数量不一致，不能只靠 `session_index.jsonl` 做第一版权威索引。依据：`session_index.jsonl` 去重后 232 个线程 ID，`state_5.sqlite.threads` 有 289 个线程 ID；比对结果是索引中有 1 个不在数据库，数据库中有 58 个不在索引。
- `/Users/yoyi/.codex/sessions/` 目录层级较深，浅层扫描会误判为空。依据：`find ... -maxdepth 3` 得到 0 个文件，`find ... -maxdepth 8` 得到 231 个 JSONL 文件。
- `.codex-global-state.json` 主要是桌面端和侧边栏状态，不能当作会话权威来源。依据：顶层字段包括窗口位置、项目顺序、活跃工作区、输入历史、侧边栏状态等。
- `memories/` 混有人工记忆、摘要、`.omx` 日志和 `.git`，只能当参考资料入口，不能当作事实权威。依据：目录枚举显示 `MEMORY.md`、`raw_memories.md`、`memory_summary.md`、`rollout_summaries/`、`.omx/logs/`、`.git/` 同时存在。

可用主线：

- 第一版索引应以 `state_5.sqlite` 的 `threads` 表为主表，并用 `threads.rollout_path` 定位原始 JSONL 会话文件。依据：`threads` 有 289 条记录，`rollout_path` 覆盖 289/289，且 289 个路径全部存在。

## 读取范围

本轮只读了任务包允许的数据源：

- `/Users/yoyi/.codex/session_index.jsonl`
- `/Users/yoyi/.codex/sessions/`
- `/Users/yoyi/.codex/state_5.sqlite`
- `/Users/yoyi/.codex/.codex-global-state.json`
- `/Users/yoyi/.codex/skills/`
- `/Users/yoyi/.codex/plugins/`
- `/Users/yoyi/.codex/memories/`
- `/Users/yoyi/workspace/product-line/`

本轮没有读取或打印 `auth.json`、`.env`、密钥、令牌。依据：执行命令只针对上述路径、SQLite 结构和统计；没有访问授权文件路径。

## 数据源盘点

### `/Users/yoyi/.codex/state_5.sqlite`

用途判断：

- 可作为第一版会话索引主数据源。
- 可提供线程、项目目录、原始会话文件路径、归档状态、模型、权限策略、子任务关系和动态工具信息。

依据：

- 文件存在，大小约 6004736 字节。
- 表列表：`threads`、`thread_dynamic_tools`、`thread_spawn_edges`、`jobs`、`stage1_outputs`、`backfill_state`、`agent_jobs`、`agent_job_items`、`remote_control_enrollments`、`_sqlx_migrations`。
- 记录数：`threads` 289，`thread_dynamic_tools` 227，`thread_spawn_edges` 186，`jobs` 14，`stage1_outputs` 5，`backfill_state` 1，`agent_jobs` 0，`agent_job_items` 0，`remote_control_enrollments` 0。

`threads` 字段：

- `id`
- `rollout_path`
- `created_at`
- `updated_at`
- `source`
- `model_provider`
- `cwd`
- `title`
- `sandbox_policy`
- `approval_mode`
- `tokens_used`
- `has_user_event`
- `archived`
- `archived_at`
- `git_sha`
- `git_branch`
- `git_origin_url`
- `cli_version`
- `first_user_message`
- `agent_nickname`
- `agent_role`
- `memory_mode`
- `model`
- `reasoning_effort`
- `agent_path`
- `created_at_ms`
- `updated_at_ms`
- `thread_source`
- `preview`

`threads` 覆盖情况：

- 总数 289。
- `cwd` 覆盖 289/289。
- `rollout_path` 覆盖 289/289。
- `title` 覆盖 289/289。
- `first_user_message` 覆盖 289/289。
- `preview` 覆盖 288/289。
- `model` 覆盖 289/289。
- `agent_path` 覆盖 0/289。
- `git_sha` 覆盖 80/289。
- `archived=0` 有 231 条，`archived=1` 有 58 条。

`threads` 统计：

- `model_provider=ai` 有 289 条。
- `memory_mode=enabled` 有 289 条。
- `thread_source`：`subagent` 153，`user` 41，空值 95。
- `cwd` 排名前几项：`/Users/yoyi/Desktop/kt-erp` 106，`/Users/yoyi/gamework` 56，`/Users/yoyi/Documents/Codex/2026-05-09/gihub-mattpocock-skills` 45，`/Users/yoyi/gameai/crazytown` 17，`/Users/yoyi/Documents/Codex/game-harness` 17。

可靠字段：

- `id`：线程标识。依据：主键字段，且可与会话文件名和会话索引对照。
- `rollout_path`：原始会话文件路径。依据：289/289 覆盖，289/289 文件存在。
- `cwd`：线程工作目录。依据：289/289 覆盖。
- `created_at`、`updated_at`、`created_at_ms`、`updated_at_ms`：排序和更新时间字段。依据：字段存在且覆盖率高。
- `archived`、`archived_at`：归档状态。依据：字段存在，统计可分 231/58。
- `thread_source`：粗分来源。依据：聚合值可读，分为 `subagent`、`user`、空值。

只能参考的字段：

- `source`：不能直接展示为分类。依据：字段里混有 `vscode`、`cli` 和大量 JSON 形态的子任务来源，直接聚合会产生噪声。
- `title`、`first_user_message`、`preview`：可用于展示摘要，但不能当作事实权威。依据：它们是文本摘要或首条用户消息，可能含敏感内容，也可能不是最终状态。
- `sandbox_policy`、`approval_mode`：可展示当时权限环境，但不能推断当前权限。依据：记录的是线程创建或运行时状态，不代表当前工作台可执行权限。
- `git_sha`、`git_branch`、`git_origin_url`：覆盖不完整。依据：`git_sha` 仅 80/289 覆盖。

风险：

- SQLite 是 Codex 内部状态库，结构可能随版本变化。索引器必须做表存在、字段存在和读取失败降级。
- `first_user_message`、`preview`、`source` 可能包含用户输入或业务信息，第一版界面不应默认大段展示。
- 不允许写回该数据库。依据：任务包禁止修改 `/Users/yoyi/.codex`。

### `/Users/yoyi/.codex/sessions/`

用途判断：

- 可作为原始会话正文来源。
- 第一版可用来按线程打开原始文件、统计消息类型、建立会话详情页。

依据：

- 目录存在。
- `maxdepth 8` 扫描得到 231 个文件。
- 231 个文件总行数 162574。
- 文件扩展名都是 `.jsonl`。
- 时间范围从 `2026-04-28T06:19:23.947Z` 到 `2026-05-27T00:09:13.483Z`。
- 采样 JSONL 顶层字段稳定为 `timestamp`、`type`、`payload`。
- 采样类型计数：`response_item` 23661，`event_msg` 17063，`turn_context` 1252，`session_meta` 277，`compacted` 2。

字段：

- 顶层：`timestamp`、`type`、`payload`。
- `payload` 采样见到的字段包括：`type`、`role`、`content`、`message`、`cwd`、`model`、`sandbox_policy`、`approval_policy`、`turn_id`、`call_id`、`name`、`arguments`、`output`、`stdout`、`stderr`、`exit_code`、`summary`、`memory_citation`、`encrypted_content` 等。

可靠字段：

- `timestamp`：事件时间。
- `type`：事件类型。
- `payload.cwd`、`payload.model`、`payload.sandbox_policy`、`payload.approval_policy`：上下文事件里可参考，但以 SQLite `threads` 作为线程级聚合主来源。

只能参考或需谨慎的字段：

- `payload.content`、`payload.message`、`payload.output`、`payload.stdout`、`payload.stderr`：可能包含业务正文、命令输出、路径、用户输入，不应无过滤地展示。
- `payload.encrypted_content`：不要解析，不要展示。
- `payload.memory_citation`：可作为记忆引用线索，但不是项目事实权威。

风险：

- 文件数量和数据库线程数量不一致。依据：`sessions/` 有 231 个文件，`threads` 有 289 条；另有归档路径。
- 目录层级会按年月日展开，不能用浅层扫描。
- 会话正文可能包含敏感输入，第一版只能做只读、按需打开和脱敏摘要。

### `/Users/yoyi/.codex/session_index.jsonl`

用途判断：

- 可作为轻量线程列表和兼容数据源。
- 不适合作为第一版权威索引。

依据：

- 文件存在，267 行。
- 解析后字段只有 `id`、`thread_name`、`updated_at`。
- 去重线程 ID 为 232。
- 与 SQLite 对比：索引中有 1 个 ID 不在 SQLite，SQLite 中有 58 个 ID 不在索引。

字段：

- `id`
- `thread_name`
- `updated_at`

可靠字段：

- `id`、`updated_at` 可辅助快速列表。

只能参考的字段：

- `thread_name` 只能作为显示名参考。依据：没有项目目录、会话路径、权限、归档、模型等字段。

风险：

- 覆盖不完整。
- 无法表达归档线程、项目归属、原始会话路径和子任务关系。

### `/Users/yoyi/.codex/.codex-global-state.json`

用途判断：

- 可作为桌面端状态和项目偏好来源。
- 不能作为会话权威来源。
- 不能把侧边栏显示当作可靠索引。

依据：

- 顶层字段包括：`electron-persisted-atom-state`、`electron-main-window-bounds`、`electron-saved-workspace-roots`、`project-order`、`active-workspace-roots`、`projectless-thread-ids`、`thread-workspace-root-hints`、`pinned-thread-ids` 等。
- `electron-saved-workspace-roots` 数量 9。
- `project-order` 数量 9。
- `active-workspace-roots` 数量 1。
- `projectless-thread-ids` 数量 22。
- `thread-workspace-root-hints` 数量 22。
- `pinned-thread-ids` 数量 0。
- `prompt-history` 有 66 个桶。

字段：

- 项目相关：`electron-saved-workspace-roots`、`project-order`、`active-workspace-roots`
- 线程提示相关：`projectless-thread-ids`、`thread-workspace-root-hints`、`pinned-thread-ids`
- 桌面状态相关：窗口尺寸、侧边栏折叠、模型提示、输入历史等

可靠字段：

- `electron-saved-workspace-roots` 和 `project-order` 可用于项目列表候选。依据：字段明确保存工作区根目录和顺序。
- `active-workspace-roots` 可用于“最近活跃”提示。依据：字段存在且数量为 1。

只能参考的字段：

- `thread-workspace-root-hints`：只能辅助项目归属。依据：仅 22 条，覆盖远低于 `threads.cwd` 的 289 条。
- `projectless-thread-ids`：只能辅助理解侧边栏或项目归属异常。
- `prompt-history`：禁止作为索引正文来源。依据：它是输入历史，可能含敏感内容，且不是会话事实。

风险：

- 这是 UI 状态，不是稳定数据契约。
- 包含输入历史，默认不应展示。
- 用户已明确侧边栏不能当可靠数据源；项目 README 也记录“不能把侧边栏当作索引依据”。

### `/Users/yoyi/.codex/skills/`

用途判断：

- 可作为本地个人和系统 skill 清单来源。
- 可给工作台展示可用 skill、路径和说明摘要。

依据：

- 目录存在。
- 找到 7 个 `SKILL.md`。
- 文件包括 `.system/imagegen`、`.system/openai-docs`、`.system/plugin-creator`、`.system/skill-creator`、`.system/skill-installer`、`neat-freak`、`playwright`。
- 每个 `SKILL.md` 均有标题和描述字段。

字段或可提取项：

- 路径
- skill 名称，来自目录名或标题
- 标题，来自一级标题
- 描述，来自 `description:` 行
- 来源类型：`.system` 或用户目录
- 文件大小

可靠字段：

- 路径、目录名、文件存在性。

只能参考的字段：

- 标题和描述。依据：来自自然语言文档，可能被用户修改，不是强类型配置。

风险：

- skill 文档可能很长，不应在索引列表中全文载入。
- `.system` 和用户安装 skill 应分开显示，避免把系统内置和用户扩展混为一类。

### `/Users/yoyi/.codex/plugins/`

用途判断：

- 可作为插件缓存、插件 manifest 和插件内 skill 清单来源。

依据：

- `plugins/cache/` 下有 11 个 `.codex-plugin/plugin.json`。
- manifest 名称和版本包括：`browser` 26.519.41501，`chrome` 0.1.7，`computer-use` 1.0.799，`game-studio` 0.1.0，`github` 0.1.0，`google-drive` 0.1.0，`hyperframes` 0.1.0，`superpowers` 5.0.7，`documents` 26.521.10419，`presentations` 26.521.10419，`spreadsheets` 26.521.10419。
- 插件内找到 43 个 `skills/*/SKILL.md`。

manifest 字段：

- 常见字段：`name`、`version`、`description`、`author`、`homepage`、`interface`、`keywords`、`license`、`repository`、`skills`
- 部分字段：`mcpServers`、`apps`

可靠字段：

- manifest 路径、`name`、`version`、是否存在 `mcpServers`、是否存在 `apps`。
- 插件内 `skills/*/SKILL.md` 文件路径。

只能参考的字段：

- manifest 里的 `skills` 字段。依据：本轮读取到 manifest 中 `skills` 计数为 0，但目录下实际有 43 个 skill 文件，说明不能只靠 manifest 的 `skills` 字段列 skill。

风险：

- `plugins/cache` 是缓存路径，版本和目录哈希可能变化。
- 插件可能包含应用和 MCP 配置，第一版只做只读展示，不自动启用、不自动安装。

### `/Users/yoyi/.codex/memories/`

用途判断：

- 可作为记忆和历史摘要入口。
- 不可作为项目当前事实权威。

依据：

- 核心文件：`MEMORY.md` 324 行，`memory_summary.md` 126 行，`raw_memories.md` 490 行，`extensions/ad_hoc/instructions.md` 13 行。
- `rollout_summaries/` 有 5 个摘要文件。
- `.omx/logs/` 有 3 个 JSONL 日志文件。
- `.omx` 日志字段包括 `_ts`、`event`、`native_session_id`、`session_id`、`thread_id`、`turn_id`、`input_preview`、`output_preview` 等。
- 目录内存在 `.git/`。

字段或可提取项：

- Markdown 文件路径、行数、更新时间。
- rollout 摘要文件名。
- `.omx` 日志的时间、事件类型、线程 ID、输入输出预览。

可靠字段：

- 文件路径、文件存在性、行数。

只能参考的字段：

- `MEMORY.md`、`raw_memories.md`、`memory_summary.md` 正文。
- `.omx/logs/*` 的 `input_preview`、`output_preview`。

风险：

- 记忆可能过期、冲突或被人工整理过。
- `.omx` 属于另一套工作流痕迹，不应混入 Codex 会话权威索引。
- 不应全文展示记忆或预览，避免泄露历史上下文。

## 第一版字段建议

线程索引最小字段：

- `thread_id`：来自 `threads.id`。
- `title`：来自 `threads.title`，显示用。
- `cwd`：来自 `threads.cwd`，项目归属主字段。
- `rollout_path`：来自 `threads.rollout_path`，打开原始会话用。
- `created_at_ms`、`updated_at_ms`：优先使用毫秒字段；缺失时降级到 `created_at`、`updated_at`。
- `archived`、`archived_at`：来自 `threads`。
- `thread_source`：来自 `threads.thread_source`。
- `model`、`model_provider`、`reasoning_effort`：来自 `threads`，展示用。
- `sandbox_policy`、`approval_mode`：来自 `threads`，展示当时环境用。
- `tokens_used`、`has_user_event`：来自 `threads`，统计用。
- `preview`：来自 `threads.preview`，默认折叠或截断展示。

项目索引最小字段：

- `project_root`：主来源 `threads.cwd` 聚合；辅助来源 `electron-saved-workspace-roots`。
- `thread_count`：按 `threads.cwd` 聚合。
- `latest_updated_at_ms`：按 `threads.cwd` 聚合最大更新时间。
- `active_hint`：来自 `.codex-global-state.json.active-workspace-roots`，只能参考。
- `project_order_hint`：来自 `.codex-global-state.json.project-order`，只能参考。

skill 索引最小字段：

- `skill_id`：路径派生。
- `source_type`：`system`、`user`、`plugin`。
- `path`：`SKILL.md` 绝对路径。
- `title`：一级标题。
- `description`：`description:` 行。
- `plugin_name`、`plugin_version`：插件内 skill 才有。

plugin 索引最小字段：

- `plugin_name`：manifest `name`。
- `plugin_version`：manifest `version`。
- `manifest_path`
- `skill_paths`：目录扫描 `skills/*/SKILL.md`。
- `has_mcp_servers`
- `has_apps`

## 禁止或默认不展示

- 不读取或展示 `auth.json`、`.env`、密钥、令牌。
- 不写 `/Users/yoyi/.codex`。
- 不把 `.codex-global-state.json` 的侧边栏状态当作会话归属权威。
- 不默认展示 `prompt-history`。
- 不默认展示 JSONL 里的 `payload.content`、`payload.message`、`payload.stdout`、`payload.stderr`、`payload.output`。
- 不解析或展示 `encrypted_content`。

## 残留不确定

- SQLite schema 是否会随 Codex 版本变化：不确定。依据：这是内部状态库，没有在本任务内发现稳定公开契约。
- `created_at` 和 `created_at_ms` 的单位关系：未做逐条换算验证。第一版可优先使用毫秒字段，缺失时降级。
- `source` JSON 的完整类型集合：未逐条解析成结构化类型。依据：聚合结果显示类型复杂，建议索引内核单独做解析器。
- 归档会话是否都在 `/Users/yoyi/.codex/archived_sessions/`：本轮只通过 `threads.rollout_path` 证明 289 个路径存在，没有全盘枚举未入库的归档文件。

