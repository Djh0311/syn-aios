# Codex 数据盘点线回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-codex-data-inventory.md`
- 开发线：Codex 数据盘点线
- 回传 evidence：`product-line/evidence/2026-05-27-codex-local-data-inventory.md`
- 回传 handoff：`product-line/handoffs/2026-05-27-codex-index-kernel-handoff.md`

## 结论

接受。

## 先说薄弱点

- `session_index.jsonl` 的行数会继续变化，不能把单次行数当固定事实。回收复核时它是 270 行，但 evidence 记录盘点时是 267 行。这个变化不影响核心结论，因为去重线程数仍是 232，SQLite 线程数仍是 289。
- `state_5.sqlite` 是 Codex 内部状态库，没有稳定公开契约。索引内核线必须做 schema 检查和降级。
- `threads.source` 没有完成结构化解析，只能留给索引内核线单独处理。
- 归档会话是否存在未入库文件，本轮没有全量证明。

## 接受依据

- evidence 覆盖了任务包要求的数据源：`session_index.jsonl`、`sessions/`、`state_5.sqlite`、`.codex-global-state.json`、`skills/`、`plugins/`、`memories/`、`product-line/`。
- evidence 明确禁止读取或展示 `auth.json`、`.env`、密钥、令牌，并记录本轮没有读取这些文件。
- evidence 明确第一版主索引应使用 `state_5.sqlite.threads`，并说明 `rollout_path` 覆盖 289/289 且路径存在。
- 回收复核确认：`threads` 总数 289，`cwd` 覆盖 289，`rollout_path` 覆盖 289。
- 回收复核确认：`session_index.jsonl` 去重线程 232，SQLite 线程 289，索引中 1 个不在 SQLite，SQLite 中 58 个不在索引。这个结果支撑“session_index 不能做权威”的结论。
- handoff 给出了索引内核线的读取顺序、字段清单、输出结构建议、安全规则和最小验收。

## 当前生效结论

- 第一版以 `state_5.sqlite.threads` 为会话主索引。
- `threads.cwd` 是项目归属主字段。
- `threads.rollout_path` 是原始会话文件入口。
- `.codex-global-state.json` 只做项目列表、顺序、活跃工作区提示，不做会话权威。
- `session_index.jsonl` 只做轻量补充或兼容检查。
- 会话正文、命令输出、输入历史、记忆正文默认不展示。
- `skills/`、`plugins/cache/`、`memories/` 作为独立资料入口。
- 索引器必须只读，不写 `/Users/yoyi/.codex`。

## 派生任务

- 新增索引内核线任务包：`product-line/tasks/2026-05-27-codex-index-kernel.md`

## 状态

已回收，接受进入下一阶段输入。
