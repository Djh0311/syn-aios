# 任务包：Codex 工作流编排运行模型 v1

## 所属开发线

Codex 会话线。

协作开发线：总指导线、索引内核线。

说明：本任务主责是把 Codex 会话控制和 transcript 读回串成最小运行闭环，不是桌面 UI 任务。

## 背景

当前阶段已纠偏：产品主线不是任务包管理器，而是 Codex 会话管理和 Codex 工作流编排。

已完成的前置能力：

- 单个 Codex 会话 transcript 可按 `thread_id` 读取。
- 受控 `codex exec` 能创建无业务测试会话。
- `--json` 能输出机器可读事件。
- `--output-last-message` 能写出最终回复。
- 临时索引能发现新会话。
- transcript reader 能读回新会话。

依据：

- `product-line/handoffs/2026-05-29-codex-session-full-transcript-v1-review.md`
- `product-line/handoffs/2026-05-29-codex-session-control-probe-v1-review.md`
- `product-line/handoffs/2026-05-29-codex-controlled-real-session-write-probe-v2-review.md`
- `product-line/tasks/README.md`

## 已知、未知和假设

已知：

- `codex exec --skip-git-repo-check --json --output-last-message <file> <prompt>` 已在受控测试里跑通。
- 新建测试会话会写 `/Users/yoyi/.codex/state_5.sqlite` 和 `/Users/yoyi/.codex/sessions/.../rollout-*.jsonl`。
- 当前 transcript reader 能读回新会话，但仍会对加密内容做省略。

未知：

- `codex resume <session_id> <prompt>` 是否适合稳定多轮编排。
- 多个 Codex 会话并发运行时是否会互相污染输出或状态。
- 长任务执行时 `--json` 事件类型是否仍然只有最小 4 类。
- Codex CLI 的插件和 MCP warning 是否会影响长时间运行。

假设：

- v1 先不用 resume，不碰已有业务会话，只用新建无业务测试会话验证编排模型。
- v1 先不做桌面 UI，只做可被桌面壳后续调用的本地运行模型和最小原型。
- 任务包文件在产品里只作为内部协议、审计和导出物，不作为主界面中心。

## 目标

实现并验证一个最小 Codex 工作流运行模型：

- 总指导节点能生成一份结构化任务指令。
- 执行节点能接收任务指令并通过新 Codex 测试会话执行。
- 编排器能等待执行节点完成。
- 编排器能用 transcript reader 读回执行节点结果。
- 总指导节点能基于执行结果生成回收意见。
- 运行过程能留下结构化状态、事件、会话引用和产物引用。

大白话目标：证明工作台以后可以让“总指导”和“开发线会话”自动接力跑一小段流程，而不是靠用户手动复制粘贴。

## 非目标

- 不做桌面 UI。
- 不做画布编辑器。
- 不做真实业务任务。
- 不派发给已有真实业务会话。
- 不验证 resume。
- 不验证 fork。
- 不做并发调度。
- 不运行 harness。
- 不生成真实业务任务包。
- 不改 Codex 授权、配置或状态结构。
- 不接入非 Codex agent。
- 不做个人知识库。

## 最小运行场景

使用无业务测试目标，避免污染真实项目：

```text
总指导测试目标：请让执行线完成一个无业务控制探针。执行线只需返回 WORKER_DONE_2026_05_29。
```

建议最小节点：

- `director_plan`：生成结构化任务指令。
- `worker_run`：创建新的 Codex 测试会话并执行任务。
- `worker_readback`：用临时索引和 transcript reader 读回执行会话。
- `director_review`：生成回收意见。
- `runtime_summary`：输出本次运行摘要。

建议最小状态流：

```text
planned -> dispatched -> running -> reported -> recovered -> accepted
```

## 建议实现

建议新增独立原型目录：

```text
product-line/prototypes/codex-workflow-runtime/
```

建议新增文件：

- `README.md`
- `workflow_runtime.py`
- `tests/test_workflow_runtime.py`

建议 `workflow_runtime.py` 支持两种模式：

1. `--dry-run`
   - 不运行 Codex CLI。
   - 只生成 workflow run 计划和状态流。
   - 用于无授权时验证模型结构。

2. `--real-codex-probe`
   - 需要用户明确批准后才能执行。
   - 只运行无业务测试 prompt。
   - 只新建测试会话。
   - 不 resume、不 fork、不碰已有业务会话。

建议输出临时运行文件：

```text
/tmp/codex-workflow-runtime-v1/run.json
/tmp/codex-workflow-runtime-v1/worker-events.jsonl
/tmp/codex-workflow-runtime-v1/worker-last-message.txt
/tmp/codex-workflow-runtime-v1/worker-transcript.json
/tmp/codex-workflow-runtime-v1/index.json
/tmp/codex-workflow-runtime-v1/director-review.json
```

建议 `run.json` 至少包含：

- `run_id`
- `created_at_ms`
- `workflow_version`
- `goal`
- `nodes`
- `edges`
- `events`
- `sessions`
- `artifacts`
- `warnings`
- `status`

建议节点结构至少包含：

- `node_id`
- `node_type`
- `role`
- `status`
- `started_at_ms`
- `ended_at_ms`
- `input_ref`
- `output_ref`
- `session_ref`
- `warnings`

建议事件结构至少包含：

- `event_id`
- `event_type`
- `node_id`
- `timestamp_ms`
- `summary`
- `artifact_refs`

## 真实 Codex 写入规则

即使本任务包已经写好，执行 `--real-codex-probe` 前仍必须再次获得用户明确批准。

批准语义必须明确到类似：

```text
批准执行 Codex 工作流运行模型 v1 的真实无业务探针
```

不能把“可以”“继续”“批准”这类单独词语自动理解为允许真实写入。

没有明确批准时，只允许：

- 写原型代码。
- 写测试。
- 跑 dry-run。
- 读取允许范围内的项目文档和临时文件。

## 允许读取

项目内：

- `product-line/tasks/README.md`
- `product-line/handoffs/2026-05-29-codex-session-full-transcript-v1-review.md`
- `product-line/handoffs/2026-05-29-codex-session-control-probe-v1-review.md`
- `product-line/handoffs/2026-05-29-codex-controlled-real-session-write-probe-v2-review.md`
- `product-line/prototypes/index-kernel/transcript_reader.py`
- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/codex-index.json`

真实探针获批后允许只读：

- `/Users/yoyi/.codex/state_5.sqlite` 的统计和线程元数据。
- `/Users/yoyi/.codex/sessions/` 的新增测试会话文件。
- `/Users/yoyi/.codex/archived_sessions/` 的只读文件清单。

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 授权文件内容
- 密钥文件内容
- 与本任务无关的业务会话正文

## 允许写入

允许写入项目内：

- `product-line/prototypes/codex-workflow-runtime/`
- `product-line/evidence/2026-05-29-codex-workflow-orchestration-runtime-model-v1.md`
- `product-line/handoffs/2026-05-29-codex-workflow-orchestration-runtime-model-v1-result.md`

允许写入临时目录：

- `/tmp/codex-workflow-runtime-v1/`

真实探针明确获批后，允许 Codex CLI 因新建无业务测试会话自然写入：

- `/Users/yoyi/.codex/state_5.sqlite`
- `/Users/yoyi/.codex/sessions/.../rollout-*.jsonl`

## 禁止事项

- 不向现有真实业务会话发送 prompt。
- 不运行 `codex resume <真实业务会话> <prompt>`。
- 不运行 `codex fork <真实业务会话> <prompt>`。
- 不删除、迁移、归档、重命名任何 Codex 会话。
- 不读取 `auth.json`、`.env`、授权文件、密钥文件。
- 不把完整业务会话正文写入仓库。
- 不把完整临时 transcript 写入 evidence / handoff。
- 不改 Tauri / React 前端。
- 不运行 harness。
- 不生成真实业务任务包文件。
- 不把任务包管理器作为主流程中心。

## 验收标准

dry-run 验收：

- 能生成结构化 `run.json`。
- 能表达节点、边、状态流、事件、会话引用占位和产物引用占位。
- 能在不运行 Codex CLI 的情况下通过测试。

真实无业务探针验收，必须在用户明确批准后才执行：

- 能创建执行线测试会话。
- 能把结构化任务指令发送给执行线测试会话。
- 能等待执行线测试会话完成。
- 能用 `--output-last-message` 拿到执行结果。
- 能用临时索引和 transcript reader 读回执行线测试会话。
- 能生成总指导回收意见。
- 能输出一份 run summary，说明每个节点状态。
- evidence / handoff 只记录统计、路径、状态和短摘要，不贴完整 transcript。

安全验收：

- 未读 `auth.json`、`.env`、授权文件或密钥文件。
- 未碰已有真实业务会话。
- 未运行 resume 或 fork。
- 未删除、迁移、归档、重命名会话。
- 未运行 harness。

## 必须回传

1. 薄弱点先说。
2. 是否只做了 dry-run，还是执行了真实无业务探针。
3. 如果执行真实探针，用户明确批准语句是什么。
4. 运行了哪些命令。
5. 创建了哪些新测试会话。
6. 工作流状态流是否完整走到 `accepted`。
7. transcript reader 是否读回执行线结果。
8. 总指导回收意见是否生成。
9. 写了哪些文件。
10. 是否读取授权、密钥或业务会话正文。
11. 是否适合进入桌面壳会话管理 UI。

## 总指导回收重点

回收时重点看：

- 有没有又偏回任务包管理器。
- 有没有未经明确批准就真实写入 Codex。
- 有没有触碰已有业务会话。
- 运行模型是否足够简单，能不能直接映射到后续桌面 UI。
- `run.json` 是否能表达“总指导派发、执行线执行、总指导回收”的最小闭环。
- 失败时是否能停在明确状态，而不是继续乱跑。
