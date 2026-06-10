# 任务包：Codex 受控真实会话写入探针 v2

## 所属开发线

Codex 会话线。

## 背景

无副作用探针 v1 已发现 `codex exec [PROMPT]`、`codex exec --json`、`--output-last-message` 和 `resume` 候选入口，但没有真实创建会话、发送 prompt、等待结果或读回新回复。

用户已批准一次受控测试写入。该批准只用于本任务内的无业务测试会话，不代表可以触碰现有真实业务会话或读取授权文件。

依据：

- `product-line/handoffs/2026-05-29-codex-session-control-probe-v1-review.md`：下一步应先做受控真实会话写入探针 v2。
- `product-line/evidence/2026-05-29-codex-session-control-probe-v1.md`：CLI 候选入口存在，但写入能力 blocked。
- 用户当前明确回复“批准”。

## 目标

验证最小真实闭环：

- `codex exec [PROMPT]` 是否能创建一次测试会话。
- `codex exec --json` 是否输出机器可读事件。
- `--output-last-message` 是否能拿到最终回复。
- 新会话是否能在 Codex 本地状态或 sessions 中被发现。
- 新会话是否能用 transcript reader 读回。
- 本次真实写入影响了哪些 Codex 文件或状态。

## 测试 prompt

必须使用无业务内容：

```text
请只回复这一句：CONTROL_PROBE_OK_2026_05_29
```

不得包含项目业务、密钥、个人信息或真实任务。

## 允许读取

- `product-line/README.md`
- `product-line/STAGE_PLAN.md`
- `product-line/tasks/README.md`
- `product-line/DEV_LINES.md`
- `product-line/handoffs/2026-05-29-codex-session-control-probe-v1-review.md`
- `product-line/prototypes/index-kernel/transcript_reader.py`
- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/codex-index.json`
- `/Users/yoyi/.codex/state_5.sqlite` 的只读统计和 thread 元数据
- `/Users/yoyi/.codex/sessions/` 的只读文件清单和新增测试会话文件
- `/Users/yoyi/.codex/archived_sessions/` 的只读文件清单

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 授权文件内容
- 密钥文件内容

## 允许写入

允许 Codex CLI 因本次受控测试自然写入自己的会话状态和会话文件。

允许写入项目内：

- `product-line/evidence/2026-05-29-codex-controlled-real-session-write-probe-v2.md`
- `product-line/handoffs/2026-05-29-codex-controlled-real-session-write-probe-v2-result.md`

允许写入临时目录：

- `/tmp/codex-control-probe-v2/`
- `/tmp/codex-control-probe-v2-last-message.txt`
- `/tmp/codex-control-probe-v2-events.jsonl`
- `/tmp/codex-control-probe-v2-transcript.json`
- `/tmp/codex-control-probe-v2-index.json`

## 禁止事项

- 不向现有真实业务会话发送测试消息。
- 不运行 `codex resume <真实业务会话> <prompt>`。
- 不运行 `codex fork <真实业务会话> <prompt>`。
- 不删除、迁移、归档、重命名任何 Codex 会话。
- 不读取 `auth.json`、`.env`、授权文件、密钥文件。
- 不把完整真实业务会话正文写入仓库。
- 不写项目业务目录。
- 不改 Tauri / React 前端。
- 不运行 harness。
- 不保存 API key、token 或供应商配置。

## 建议执行步骤

1. 记录执行前 `.codex` 线程数、最新 rollout 路径、sessions 文件数，只输出统计。
2. 在 `/tmp/codex-control-probe-v2/` 作为 cwd 运行：

```bash
codex exec --json --output-last-message /tmp/codex-control-probe-v2-last-message.txt "请只回复这一句：CONTROL_PROBE_OK_2026_05_29"
```

3. 保存 stdout 事件流到 `/tmp/codex-control-probe-v2-events.jsonl`。
4. 记录退出码、最终回复文件是否存在、是否包含目标文本。
5. 记录执行后 `.codex` 线程数和新增 rollout 候选，只输出统计和路径。
6. 用 `build_index.py --codex-home /Users/yoyi/.codex --output /tmp/codex-control-probe-v2-index.json` 生成临时索引。
7. 在临时索引中定位本次测试会话。
8. 用 `transcript_reader.py` 对本次测试会话输出 `/tmp/codex-control-probe-v2-transcript.json`。
9. evidence / handoff 只记录统计、路径和是否读回目标文本，不贴完整 transcript。

如果第 2 步失败，不要继续强行 resume 或 fork。记录失败即可。

## 验收标准

- 明确记录是否运行了真实 `codex exec`。
- 明确记录是否创建了新会话。
- 明确记录是否写入新的 rollout/session 文件。
- 明确记录 `--json` 是否输出机器可读事件。
- 明确记录 `--output-last-message` 是否写出最终回复。
- 明确记录 transcript reader 是否能读回本次测试会话。
- 不碰现有真实业务会话。
- 不读取授权、密钥、`.env`。
- evidence / handoff 不包含完整 transcript。

## 必须回传

1. 薄弱点先说。
2. 运行了什么命令。
3. 是否创建新会话。
4. 新增或变化的 Codex 文件/状态统计。
5. `--json` 是否可用。
6. `--output-last-message` 是否可用。
7. transcript reader 是否读回目标文本。
8. 写了哪些文件。
9. 是否读取授权或密钥。
10. 是否适合进入工作流编排运行模型。
