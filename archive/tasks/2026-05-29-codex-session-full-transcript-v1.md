# 任务包：Codex 会话全文读取 v1

## 所属开发线

Codex 会话线。

## 背景

当前阶段已纠偏：工作台主线不是任务包管理器，而是 Codex 会话管理和 Codex 工作流编排。

依据：

- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`：当前第一优先级是正确读取和管理 Codex 内容，包括会话完整正文、工具调用、工具结果和时间线。
- `product-line/STAGE_PLAN.md`：阶段 1 需要读取选中 Codex 会话的完整正文、工具调用、工具结果和时间线；阶段 3 需要在工作流中查看会话角色和会话正文。
- `product-line/evidence/2026-05-27-codex-local-data-inventory.md`：`state_5.sqlite` 的 `threads.rollout_path` 可定位原始 JSONL 会话文件；`/Users/yoyi/.codex/sessions/` 是原始会话正文来源。
- 用户明确要求：Codex 对话的每一个内容都要能读取，但当前可以先做协调各个对话的工作，最重要的是把工作流跑起来。

## 已知

- 当前索引内核已经能读取线程元数据和 `rollout_path`。
- 原始会话 JSONL 顶层字段采样为 `timestamp`、`type`、`payload`。
- 采样类型包含 `response_item`、`event_msg`、`turn_context`、`session_meta`、`compacted`。
- `payload` 可能包含正文、工具调用、工具结果、命令输出、错误输出、压缩摘要、加密内容引用等。

## 未知

- 所有真实 JSONL 事件类型是否都已覆盖。
- 当前 Codex 版本是否新增了未见过的 payload 结构。
- 会话正文里是否包含密钥、授权、`.env` 内容或其他敏感信息。
- 长会话全文解析的性能上限。

本任务不能靠猜补齐这些未知，必须用 warning 和测试体现。

## 目标

实现“按用户选择的会话读取完整 transcript”的最小能力。

具体目标：

- 基于现有索引里的线程 ID / `rollout_path`，读取单个会话 JSONL。
- 输出结构化 transcript，不把全文塞进全局 `codex-index.json`。
- 保留原始事件顺序和时间线。
- 区分用户消息、assistant 消息、工具调用、工具结果、命令输出、系统/上下文事件、压缩事件、未知事件。
- 对未知事件保留可诊断 metadata，但不要丢弃。
- 对 `payload.encrypted_content` 只标记存在，不解析、不展示。
- 增加 transcript 读取测试夹具。
- 生成 evidence 和 handoff。

## 建议实现位置

优先放在索引内核原型旁边，作为 Codex 会话线的独立模块，不要直接混进全局索引生成逻辑：

- `product-line/prototypes/index-kernel/`

可新增：

- `transcript_reader.py`
- `tests/test_transcript_reader.py`

也可以在 `build_index.py` 中新增只读命令入口，但不要让默认 `build_index.py` 生成全量正文。

建议 CLI 形态：

```bash
python3 product-line/prototypes/index-kernel/transcript_reader.py \
  --index product-line/prototypes/index-kernel/codex-index.json \
  --thread-id <thread-id> \
  --output /tmp/codex-transcript.json
```

如果实际实现选择不同入口，必须在 evidence 里说明原因。

## 输出结构建议

输出 JSON 顶层建议包含：

- `thread_id`
- `rollout_path`
- `project_path`
- `title`
- `created_at_ms`
- `updated_at_ms`
- `events`
- `summary`
- `warnings`
- `source_stats`

`events[]` 建议包含：

- `event_id`
- `timestamp`
- `event_type`
- `actor`
- `role`
- `turn_id`
- `call_id`
- `tool_name`
- `text`
- `arguments`
- `output`
- `exit_code`
- `metadata`
- `warnings`

注意：字段可以按真实数据调整，但必须解释调整依据。

## 敏感信息策略

本任务允许读取会话正文，但不允许无边界输出敏感内容。

最低要求：

- 不读取 `auth.json`。
- 不读取 `.env` 文件。
- 不读取 Codex 授权文件。
- 不打印完整真实 transcript 到终端、evidence 或 handoff。
- 测试可以使用人工夹具正文。
- 真实样本验证只允许输出计数、类型、长度、warning、脱敏片段或哈希，不输出完整正文。
- 对看起来像密钥、token、API key、Authorization header 的字段，输出时必须打 `sensitive_like_content` warning。
- `payload.encrypted_content` 不解析、不展示。

如果为了验证必须读取一个真实会话文件，只能读取用户选择或索引中明确指定的一条会话，并且回传里只能写统计结果。

## 允许读取

- `product-line/README.md`
- `product-line/STAGE_PLAN.md`
- `product-line/tasks/README.md`
- `product-line/DEV_LINES.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/evidence/2026-05-27-codex-local-data-inventory.md`
- `product-line/prototypes/index-kernel/`
- `product-line/prototypes/index-kernel/codex-index.json`
- 通过 `codex-index.json` 或测试参数指向的单个 `rollout_path`

允许读取真实 rollout 的边界：

- 只读。
- 只针对一个或少量明确 thread 的 JSONL。
- 不把完整正文写进 evidence、handoff 或任务队列。
- 不扫全量正文作为默认动作。

## 允许写入

- `product-line/prototypes/index-kernel/transcript_reader.py`
- `product-line/prototypes/index-kernel/tests/test_transcript_reader.py`
- 如需共享类型或工具函数，可修改 `product-line/prototypes/index-kernel/build_index.py`，但必须保持默认索引输出不包含全文正文。
- `product-line/evidence/2026-05-29-codex-session-full-transcript-v1.md`
- `product-line/handoffs/2026-05-29-codex-session-full-transcript-v1-result.md`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改 Codex 状态库。
- 不移动、删除、归档或重命名任何 Codex 会话文件。
- 不读取 `auth.json`、`.env`、授权文件、密钥文件。
- 不把完整真实会话正文写入仓库。
- 不把完整真实会话正文打印到终端、evidence、handoff。
- 不改桌面应用 UI。
- 不启动 Codex CLI。
- 不创建新 Codex 会话。
- 不发送消息给 Codex。
- 不运行 harness。
- 不把 transcript 全文加入默认 `codex-index.json`。

## 验收标准

- 有可运行的 transcript 读取入口。
- 给定一个索引内 `thread_id`，能找到 `rollout_path` 并读取对应 JSONL。
- 输出保留事件顺序。
- 输出能区分至少这些类别：
  - 用户消息
  - assistant 消息
  - 工具调用
  - 工具结果
  - 命令 stdout / stderr / exit code
  - turn context
  - session meta
  - compacted
  - unknown
- 未知事件不崩溃，并进入 `unknown` 或带 warning 的结构。
- 损坏 JSONL 行不导致整条会话读取失败，必须记录 warning。
- 缺文件、权限拒绝、非索引 thread、路径越界都有测试。
- `payload.encrypted_content` 不解析、不展示。
- 默认 `codex-index.json` 不新增会话正文、工具输出、命令输出字段。
- 测试通过。
- evidence / handoff 不包含完整真实会话正文。

## 建议测试

至少新增这些测试：

1. 正常 transcript 夹具能解析用户消息、assistant 消息、工具调用和工具结果。
2. 命令输出能区分 stdout、stderr、exit_code。
3. `payload.encrypted_content` 只标记 warning，不输出内容。
4. 损坏 JSONL 行产生 warning，但其余行继续解析。
5. 未知事件类型保留 metadata 并产生 warning。
6. 非索引 thread 拒绝。
7. `rollout_path` 越界拒绝。
8. 缺失 rollout 文件给出明确错误。
9. 默认索引生成不包含全文正文。

如果权限拒绝在当前系统不好稳定模拟，测试里可以保留 skip 分支，但必须说明。

## 必须回传

开发线回传必须包含：

1. 薄弱点先说。
2. 做了什么。
3. 改了哪些文件。
4. 新增了哪些测试。
5. transcript 输出结构是什么。
6. 真实会话验证是否做了；如果做了，读取了哪个 thread，输出了哪些统计，不得贴完整正文。
7. 如何证明没有写 `/Users/yoyi/.codex` 或 Codex 状态库。
8. 如何证明默认 `codex-index.json` 没有加入全文正文。
9. 哪些事件类型仍不确定。
10. 是否适合进入下一步“Codex 会话控制能力探针 v1”。

## 总指导回收动作

总指导回收时必须判断：

- 接受
- 需要修改
- 暂停
- 废弃

并特别检查：

- 是否真的按会话读取全文，而不是只读摘要。
- 是否没有把真实全文写进 evidence / handoff / `codex-index.json`。
- 是否没有写 Codex 状态库。
- 是否足以支撑后续工作流回收会话结果。
