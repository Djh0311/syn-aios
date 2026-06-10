# 任务包：Codex 会话控制能力探针 v1

## 所属开发线

Codex 会话线。

## 背景

当前工作台主线已纠偏为 Codex 会话管理和 Codex 工作流编排。上一轮已完成单个 Codex 会话 transcript 读取 v1，但还不能证明工作台可以创建、恢复、发送消息、等待回复或读取新回复。

依据：

- `product-line/handoffs/2026-05-29-codex-session-full-transcript-v1-review.md`：已接受 transcript 读取 v1，但明确不接受为会话创建/恢复/发送能力已验证。
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`：下一步必须先做 Codex 会话控制能力探针，不能直接做假对话 UI。
- `product-line/STAGE_PLAN.md`：阶段 3 需要能从工作台直接向指定 Codex 会话发送消息，或在能力探针通过后创建新 Codex 会话。
- 用户目标是工作台能像当前多 Codex 对话协作一样，让总指导分发任务，执行会话开发，完成后反馈给总指导。

## 已知

- 当前能按 `thread_id` 读取单个会话 transcript。
- 当前索引能提供线程、项目、rollout 路径和部分元数据。
- Codex CLI / Codex App 可能有 resume、headless 或其它入口，但当前项目没有验证。
- Codex++ 采用外部 launcher + CDP 注入增强 Codex App，但这不等于我们应该直接照搬。

## 未知

- 当前本机 `codex` 命令是否可用。
- `codex` 是否支持稳定的非交互创建会话。
- `codex` 是否支持 resume 指定会话并发送 prompt。
- `codex` 是否能返回可机器读取的执行结果。
- `codex` 是否会写 `/Users/yoyi/.codex`，以及写入是否可接受。
- 是否存在官方或更稳定的本地 API / SDK / MCP 入口。
- Tauri 工作台后续应该调用 CLI、SDK、CDP，还是只做受控外部进程编排。

本任务目标是确认这些未知，不允许把不确定能力包装成完成。

## 目标

做 Codex 会话控制能力探针，并输出能力矩阵。

至少判断：

- 能否发现本机 Codex CLI / App 版本和帮助信息。
- 能否列出或识别可用的会话相关命令。
- 能否新建会话。
- 能否 resume 既有会话。
- 能否向会话发送一条受控 prompt。
- 能否等待执行完成并拿到回复。
- 能否用已完成的 transcript 读取器读回新回复。
- 哪些动作会写 Codex 状态库或会话文件。
- 哪些动作需要用户确认或必须禁止自动执行。

## 探针原则

先做无副作用探针：

- `which codex`
- `codex --version`
- `codex --help`
- `codex <subcommand> --help`
- 如存在官方文档或本地帮助，优先依据本地帮助。

只有无副作用探针能证明存在安全入口时，才允许做受控真实会话探针。

受控真实会话探针必须满足：

- prompt 内容必须是无业务内容的测试句。
- 输出目录只能在 `/tmp` 或工作台自己的状态目录。
- 不读取或打印密钥、授权文件、`.env`。
- 不写项目业务目录。
- 不修改既有真实业务会话。
- 不向用户正在工作的真实会话发送测试消息。
- 如果会写 Codex 自己的会话文件，必须在 evidence 里说明写入了什么类型的文件；如果不能确认，就不要执行真实写入。

## 建议探针方式

先实现一个独立探针脚本，避免直接接入桌面 UI：

- `product-line/prototypes/index-kernel/codex_session_control_probe.py`
- `product-line/prototypes/index-kernel/tests/test_codex_session_control_probe.py`

脚本输出 JSON 能力矩阵，例如：

```json
{
  "codex_cli": {
    "available": true,
    "path": "...",
    "version": "...",
    "help_checked": true
  },
  "capabilities": {
    "create_session": "supported|unsupported|unknown|blocked",
    "resume_session": "supported|unsupported|unknown|blocked",
    "send_prompt": "supported|unsupported|unknown|blocked",
    "wait_for_result": "supported|unsupported|unknown|blocked",
    "read_back_with_transcript": "supported|unsupported|unknown|blocked"
  },
  "evidence": [],
  "warnings": [],
  "blocked_reasons": []
}
```

如果实际实现不用 Python，也必须说明原因。

## 允许读取

- `product-line/README.md`
- `product-line/STAGE_PLAN.md`
- `product-line/tasks/README.md`
- `product-line/DEV_LINES.md`
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/handoffs/2026-05-29-codex-session-full-transcript-v1-review.md`
- `product-line/prototypes/index-kernel/transcript_reader.py`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/prototypes/index-kernel/`
- 本地 `codex` 命令的版本和帮助输出

允许读取真实 Codex 会话的边界：

- 只读索引内 thread 元数据和 transcript 统计。
- 不读取完整真实正文到 evidence 或 handoff。
- 不读取授权文件内容。

## 允许写入

- `product-line/prototypes/index-kernel/codex_session_control_probe.py`
- `product-line/prototypes/index-kernel/tests/test_codex_session_control_probe.py`
- `product-line/evidence/2026-05-29-codex-session-control-probe-v1.md`
- `product-line/handoffs/2026-05-29-codex-session-control-probe-v1-result.md`

如必须写临时探针输出：

- `/tmp/codex-session-control-probe-*.json`
- `/tmp/codex-session-control-probe-*`

## 禁止事项

- 不写 `/Users/yoyi/.codex`，除非用户在本任务执行中明确批准一个受控测试写入；没有明确批准时只能做无副作用探针。
- 不改 Codex 状态库。
- 不删除、迁移、归档、重命名任何 Codex 会话。
- 不读取 `auth.json`、`.env`、授权文件、密钥文件。
- 不向现有真实业务会话发送测试消息。
- 不把完整真实会话正文写入仓库。
- 不启动桌面 UI 实现。
- 不改 Tauri / React 前端。
- 不运行 harness。
- 不把 CDP 注入作为默认方案。
- 不保存 API key、token 或供应商配置。

## 验收标准

- 输出一份能力矩阵。
- 明确记录每项能力的状态：`supported`、`unsupported`、`unknown` 或 `blocked`。
- 每个 `supported` 必须有可复查依据。
- 每个 `blocked` 必须写清阻塞原因。
- 如果只完成无副作用探针，也可以接受为 blocked / unknown 结果，但不能写成支持。
- 如果执行了受控真实会话探针，必须说明：
  - 执行了什么命令。
  - 是否创建了新会话。
  - 是否写了 Codex 会话文件。
  - 如何清理或为什么不清理。
  - 如何用 transcript 读取器读回结果。
- 测试必须覆盖能力矩阵解析、命令缺失、帮助输出无法识别、blocked 状态。
- evidence / handoff 不包含完整真实会话正文、密钥或授权内容。

## 建议测试

至少新增这些测试：

1. 没有 `codex` 命令时输出 `codex_cli.available=false`。
2. 帮助输出不含会话控制线索时，相关能力为 `unknown` 或 `unsupported`。
3. 帮助输出含 resume / exec / prompt 等线索时，能提取候选能力，但不直接标记真实 supported。
4. 真实执行未获授权时，真实写入类能力标记为 `blocked`。
5. 输出 JSON schema 稳定。
6. 不把 help 全文中的敏感样式内容原样写入 evidence。

## 必须回传

开发线回传必须包含：

1. 薄弱点先说。
2. 做了什么。
3. 改了哪些文件。
4. 新增了哪些测试。
5. 能力矩阵。
6. 哪些能力有依据支持，依据是什么。
7. 哪些能力是 unknown / blocked，原因是什么。
8. 是否执行了真实会话创建、resume 或发送；如果执行，是否有用户明确批准。
9. 是否写了 `/Users/yoyi/.codex` 或 Codex 状态库。
10. 是否适合进入“Codex 工作流编排运行模型 v1”。

## 总指导回收动作

总指导回收时必须判断：

- 接受
- 需要修改
- 暂停
- 废弃

并特别检查：

- 是否把 unknown 包装成 supported。
- 是否绕过用户确认写 Codex 状态。
- 是否向真实业务会话发送了测试消息。
- 是否足以支撑下一步自动编排。
