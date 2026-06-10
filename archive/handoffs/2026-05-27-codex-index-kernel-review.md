# Codex 只读索引内核回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-codex-index-kernel.md`
- 开发线：索引内核线
- 原型脚本：`product-line/prototypes/index-kernel/build_index.py`
- 输出索引：`product-line/prototypes/index-kernel/codex-index.json`
- 回传 evidence：`product-line/evidence/2026-05-27-codex-index-kernel.md`
- 回传 handoff：`product-line/handoffs/2026-05-27-codex-index-kernel-result.md`

## 结论

接受为只读索引内核原型。

不直接进入桌面应用主实现。需要先派验证线补坏 schema、缺字段、缺文件、坏 manifest 的夹具测试。

## 先说薄弱点

- 上游盘点的 289 条线程已经过期，当前真实 SQLite 线程数是 290。索引内核没有硬套旧数字，这一点处理正确。
- 当前验证只覆盖真实环境和索引结构，没有覆盖坏 schema、缺字段、缺 rollout 文件、坏 plugin manifest 等异常环境。
- 索引 JSON 仍包含项目路径、会话路径、线程短标题、模型和 token 统计，属于本机工作上下文。后续界面默认展示必须继续收紧。
- `threads.source` 仍未结构化解析，这不阻塞当前原型，但不能在后续界面里直接展示原始值。

## 复核结果

- `python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json` 输出 `validation_ok`。
- `python3 -m py_compile product-line/prototypes/index-kernel/build_index.py` 通过。
- 脚本使用 `file:/Users/yoyi/.codex/state_5.sqlite?mode=ro` 打开 SQLite。
- 脚本执行 `PRAGMA query_only = ON`，索引中记录 `query_only_enabled=true`。
- 索引顶层字段包含：`generated_at`、`warnings`、`threads`、`projects`、`skills`、`plugins`、`memories`、`source_stats`。
- 线程数：290。
- 项目数：30。
- skills 总数：50，其中本地非插件 7，插件内 43。
- plugins 数：11。
- memories 元数据入口数：11。
- rollout 文件存在率：290/290。
- `session_index.jsonl` 标记为 `auxiliary_thread_list`。
- `.codex-global-state.json` 标记为 `ui_state_and_project_hint_source`，且 `used_to_override_thread_cwd=false`。
- `thread_source` 没有超出 `user/subagent/unknown`。
- rollout 缺失数为 0。
- 标题长度已收紧，没有超过截断上限。

## 当前生效结论

- `codex-index.json` 可以作为桌面应用线的只读样例输入。
- `build_index.py` 可以作为索引内核原型，但在进入产品化前必须补验证线夹具测试。
- 第一版继续坚持只读，不写 `/Users/yoyi/.codex`。
- `threads.cwd` 仍是项目归属权威来源。
- `.codex-global-state.json` 只能作为 UI 状态和项目提示来源。

## 派生任务

- 新增验证线任务包：`product-line/tasks/2026-05-27-index-kernel-validation.md`

## 状态

已回收，接受为原型；进入验证补测。
