# Codex 会话全文读取 v1 总指导回收意见

## 结论

接受。

接受为“单个 Codex 会话全文 transcript 读取 v1 已完成”。

不接受为“全部 Codex 会话格式已覆盖”，不接受为“工作台内可直接和 Codex 会话对话”，不接受为“Codex 会话创建/恢复/发送能力已验证”。

## 薄弱点

- 真实样本只验证了 1 个线程，不能证明所有历史会话格式都覆盖。
- 未知事件只保留 metadata 和 warning，不做语义完备解析。
- 疑似敏感内容当前只打 warning，不自动清洗普通正文。
- 真实 transcript 输出到 `/tmp`，不在仓库内；这符合任务包安全边界，但也意味着回收只按统计和代码验证，不复查完整正文。

## 接受依据

- 新增入口 `product-line/prototypes/index-kernel/transcript_reader.py`，能按索引内 `thread_id` 找到 `rollout_path` 并读取单个 JSONL。
- 输出结构包含 `thread_id`、`rollout_path`、`project_path`、`title`、`events`、`summary`、`warnings`、`source_stats`。
- 事件结构包含 `event_id`、`timestamp`、`event_type`、`role`、`turn_id`、`call_id`、`tool_name`、`text`、`arguments`、`output`、`stdout`、`stderr`、`exit_code`、`metadata`、`warnings`。
- 测试覆盖正常消息、工具调用、工具结果、命令输出、加密字段省略、坏 JSON 行、未知事件、非索引 thread、路径越界、缺文件、权限拒绝分支、CLI stdout 不打印正文、默认索引不含全文字段。
- evidence 只记录真实样本统计，没有贴完整正文、工具输出正文或命令输出正文。
- 默认 `codex-index.json` 检查通过，且没有加入 `events`、`transcript`、`text`、`arguments`、`output`、`stdout`、`stderr`、`exit_code` 等全文字段。

## 本轮验证

已复跑：

```bash
python3 -m unittest product-line/prototypes/index-kernel/tests/test_transcript_reader.py
```

结果：12 tests OK。

已复跑：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

结果：`validation_ok`。

没有复跑完整 discover；回传记录为 38 个通过。

## 写入边界

开发线回传的写入文件：

- `product-line/prototypes/index-kernel/transcript_reader.py`
- `product-line/prototypes/index-kernel/tests/test_transcript_reader.py`
- `product-line/evidence/2026-05-29-codex-session-full-transcript-v1.md`
- `product-line/handoffs/2026-05-29-codex-session-full-transcript-v1-result.md`

总指导本轮新增：

- `product-line/handoffs/2026-05-29-codex-session-full-transcript-v1-review.md`

## 安全边界

接受原因：

- 没有写 `/Users/yoyi/.codex`。
- 没有改 Codex 状态库。
- 没有把真实完整 transcript 写入仓库。
- 没有启动 Codex CLI。
- 没有创建新 Codex 会话。
- 没有发送消息给 Codex。

注意：任务回传说明曾列出 `/Users/yoyi/.codex/auth.json` 路径但未读内容。后续任务仍禁止读取授权文件内容。

## 对当前阶段的影响

可以进入下一步“Codex 会话控制能力探针 v1”。

下一步不应直接做假对话 UI。必须先确认 Codex 会话创建、恢复、发送、等待回复和读取结果的可行入口。

桌面应用线后续展示 transcript 时必须：

- 默认折叠长正文。
- 默认折叠工具输出和命令输出。
- 显示 sensitive warning。
- 不自动展示 encrypted content。
- 不把全部会话正文预加载进首页或全局索引。
