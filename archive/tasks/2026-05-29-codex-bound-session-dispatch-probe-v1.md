# 任务包：Codex 绑定会话受控派发探针 v1

## 所属开发线

Codex 会话线。

## 关联口径来源

- `product-line/decisions/2026-05-29-codex-session-plan-retained-workflow-first.md`
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `product-line/tasks/2026-05-29-desktop-shell-workflow-node-session-binding-v1.md`
- `product-line/evidence/2026-05-29-codex-controlled-real-session-write-probe-v2.md`

## 后续验证

本任务完成后，由总指导决定是否另派验证线。验证线不是本任务的共同执行线。

## 背景

桌面应用线已经把工作流节点和已有 Codex 会话的绑定关系接上。下一步不能直接做自动执行 UI，因为还不知道 `codex resume <session_id> <prompt>` 能不能稳定把指令发进一个已存在会话，并等待结果、读回 transcript。

本任务只做会话线能力探针：验证“已绑定会话能否被受控派发”。它不接桌面 UI，不自动跑真实工作流。

依据：

- 新建测试会话 v2 已证明 `codex exec --skip-git-repo-check --json --output-last-message` 可创建无业务测试会话并读回。
- 但 v2 明确没有验证 `resume`、多轮对话、已有业务会话派发。
- 工作流节点绑定 v1 只建立对象关系，没有发送消息。

## 薄弱点

- 本任务如果真实执行，会写 `/Users/yoyi/.codex`。依据：`codex resume` 会向目标会话追加消息和结果。
- 不能拿真实业务会话做测试。依据：测试 prompt 会污染会话正文。
- 即使无业务测试通过，也不能直接证明长任务开发稳定。依据：短 prompt 不覆盖工具调用、权限确认、长时间运行、失败重试。
- `codex resume` 的会话 id 和桌面会话 id 是否长期稳定仍未知。

## 已知、未知和假设

已知：

- 本机 `codex exec` 新建无业务测试会话已跑通。
- transcript reader 能按新 `thread_id` 读回测试会话。
- 工作流节点绑定状态里可以保存 `native_thread_id`。

未知：

- `codex resume <thread_id> <prompt>` 是否能非交互运行。
- `codex resume` 是否支持 `--json` 和 `--output-last-message`。
- `codex exec resume <thread_id> <prompt>` 与 `codex resume <thread_id> <prompt>` 哪个更适合。
- resume 后 transcript reader 能否稳定看到第二轮消息和结果。
- resume 对不同 cwd、信任目录、git repo 检查的要求是什么。

假设：

- v1 只用无业务测试会话。
- v1 不碰已有真实业务会话。
- v1 可先用 dry-run / help 探针确认命令形态。
- 真实 resume 探针必须再次获得用户精确批准。

## 目标

输出一份能力矩阵，判断“绑定会话受控派发”是否可用于下一步工作流自动执行原型。

至少判断：

1. 是否能找到可用 resume 命令形态。
2. 是否能对无业务测试会话发送第二条 prompt。
3. 是否能等待 resume 完成。
4. 是否能拿到最终回复。
5. 是否能用 transcript reader 读回第二轮内容。
6. 是否能区分第一次新建会话和第二次 resume 追加内容。
7. 是否能记录写入了哪些 Codex 状态或 rollout 文件。

大白话目标：

先确认“已经绑定的会话能不能继续收到工作流派发指令”。确认不了，就不能继续做自动执行。

## 非目标

- 不接桌面 UI。
- 不改 Tauri / React。
- 不运行真实业务工作流。
- 不向真实业务会话发送消息。
- 不运行 harness。
- 不做并发调度。
- 不做失败重试策略实现。
- 不删除、移动、归档 Codex 会话。
- 不读取 `auth.json`、`.env`、密钥或授权文件。
- 不把完整 transcript 写入仓库。

## 探针分两段

### 第一段：无副作用探针

允许执行：

- `codex resume --help`
- `codex exec resume --help`
- `codex --help`
- 只读检查已有 v2 测试会话的元数据和路径。

输出：

- 候选命令形态。
- 是否支持 `--json`。
- 是否支持 `--output-last-message`。
- 是否支持 `--skip-git-repo-check`。
- 真实写入是否仍 blocked。

### 第二段：真实无业务 resume 探针

只有在用户再次精确批准后才能执行。

批准语句必须类似：

```text
批准执行 Codex 绑定会话受控派发探针 v1 的真实无业务 resume
```

不能把下面这些当成授权：

- 可以
- 继续
- 批准
- 开始
- 下一步
- 写

真实探针只允许使用无业务测试会话。

优先使用 v2 生成的测试会话：

```text
019e7389-349a-7f02-aa31-a4a90b24e865
```

如果该会话不可用，可以先新建一个新的无业务测试会话，但必须写明原因，并仍然只用测试 prompt。

## 测试 prompt

第二轮 resume prompt 必须使用无业务内容：

```text
请只回复这一句：BOUND_SESSION_DISPATCH_OK_2026_05_29
```

不得包含真实项目业务、用户隐私、密钥、授权信息或开发任务。

## 建议执行方式

真实探针获批后，建议先尝试最可能稳定的形态。具体命令以本机 help 为准，不允许猜了就跑。

候选之一：

```bash
codex resume 019e7389-349a-7f02-aa31-a4a90b24e865 --json --output-last-message /tmp/codex-bound-session-dispatch-v1-last-message.txt "请只回复这一句：BOUND_SESSION_DISPATCH_OK_2026_05_29"
```

候选之二：

```bash
codex exec resume 019e7389-349a-7f02-aa31-a4a90b24e865 --json --output-last-message /tmp/codex-bound-session-dispatch-v1-last-message.txt "请只回复这一句：BOUND_SESSION_DISPATCH_OK_2026_05_29"
```

如果 help 显示参数顺序不同，按 help 调整，并在 evidence 写明依据。

输出临时文件建议：

- `/tmp/codex-bound-session-dispatch-v1/events.jsonl`
- `/tmp/codex-bound-session-dispatch-v1/last-message.txt`
- `/tmp/codex-bound-session-dispatch-v1/index.json`
- `/tmp/codex-bound-session-dispatch-v1/transcript.json`
- `/tmp/codex-bound-session-dispatch-v1/result.json`

## 能力矩阵

最终必须输出：

```json
{
  "resume_command_shape": "supported|unsupported|unknown|blocked",
  "resume_json_events": "supported|unsupported|unknown|blocked",
  "resume_output_last_message": "supported|unsupported|unknown|blocked",
  "resume_wait_for_result": "supported|unsupported|unknown|blocked",
  "read_back_second_turn": "supported|unsupported|unknown|blocked",
  "safe_for_workflow_dispatch_v1": "yes|no|unknown"
}
```

每个 `supported` 必须有依据。每个 `blocked` 必须写阻塞原因。

## 允许读取

允许读取项目内：

- `product-line/decisions/2026-05-29-codex-session-plan-retained-workflow-first.md`
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `product-line/tasks/2026-05-29-desktop-shell-workflow-node-session-binding-v1.md`
- `product-line/evidence/2026-05-29-codex-controlled-real-session-write-probe-v2.md`
- `product-line/prototypes/index-kernel/transcript_reader.py`
- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/codex-index.json`

真实探针获批后允许只读：

- `/Users/yoyi/.codex/state_5.sqlite` 的统计和线程元数据。
- `/Users/yoyi/.codex/sessions/` 的文件清单和本任务测试会话 JSONL。
- `/Users/yoyi/.codex/archived_sessions/` 的文件清单。

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 授权文件内容
- 密钥文件内容
- 与本任务无关的真实业务会话正文

## 允许写入

无副作用探针阶段允许写入：

- `product-line/evidence/2026-05-29-codex-bound-session-dispatch-probe-v1.md`
- `product-line/handoffs/2026-05-29-codex-bound-session-dispatch-probe-v1-result.md`
- `/tmp/codex-bound-session-dispatch-v1/`

真实 resume 探针获批后，允许 Codex CLI 因无业务测试自然写入自己的测试会话状态和 rollout 文件。

不允许写入：

- 项目业务目录。
- Tauri / React 前端。
- 工作台真实 workflow state。

## 禁止事项

- 禁止在未获精确批准时运行真实 resume。
- 禁止向真实业务会话发送测试消息。
- 禁止运行 `codex fork`。
- 禁止删除、迁移、归档、重命名 Codex 会话。
- 禁止读取 `auth.json`、`.env`、授权文件、密钥文件。
- 禁止把完整 transcript、完整事件流或完整 session JSONL 写入仓库。
- 禁止改 Tauri / React 前端。
- 禁止写工作台真实 workflow state。
- 禁止运行 harness。
- 禁止把 dry-run 或 help 探针写成真实支持。

## 验收标准

无论是否获得真实 resume 探针批准，都必须：

- 输出能力矩阵。
- 明确哪些能力 supported、blocked、unknown。
- 写清是否执行了真实 resume。
- 写清是否写了 `/Users/yoyi/.codex`。
- 写清是否触碰真实业务会话。
- 不泄露完整 transcript、密钥或授权内容。

如果执行真实 resume，还必须：

- 记录命令形态和依据。
- 记录退出码。
- 记录事件流是否生成。
- 记录最终回复是否命中目标文本。
- 记录 transcript reader 是否读回第二轮目标文本。
- 记录 Codex 本地状态或 rollout 文件变化。
- 说明是否适合进入“工作流节点派发 Codex 指令 v1”。

## 建议测试

如新增脚本，至少覆盖：

- help 输出解析。
- 未获批准时真实 resume blocked。
- 命令失败时不继续读回。
- 能力矩阵状态稳定。
- 不把完整 transcript 写入 evidence。

如果只写 evidence / handoff，不新增脚本，也必须说明原因。

## 必须回传

回传时必须说明：

1. 薄弱点。
2. 做了什么。
3. 是否只做无副作用探针，还是执行了真实 resume。
4. 如果执行真实 resume，用户精确批准语句是什么。
5. 能力矩阵。
6. 写了哪些文件。
7. 是否写了 `/Users/yoyi/.codex`。
8. 是否触碰真实业务会话。
9. 是否读取授权、密钥、`.env`。
10. 是否适合下一步进入“工作流节点派发 Codex 指令 v1”。

## 总指导回收重点

回收时重点看：

- 是否绕过精确批准运行了真实 resume。
- 是否污染了真实业务会话。
- 是否把 unknown 包装成 supported。
- 是否能证明第二轮消息可读回。
- 是否足以支撑下一步把工作流节点和真实派发连接起来。

