# Evidence：Stage E / E5 Level B Mario Test Controlled Real Resume Health Probe v1

日期：2026-06-06

## 1. 结论

E5 Level B mario test 最小真实 resume 健康探针已完成，结论为：

```text
accepted_as_minimal_real_resume_health_probe
```

接受为：

- 用户已在本轮明确批准执行 `tasks/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`。
- 对 `/Users/yoyi/Documents/mario test` 的“总指导” native thread `019e798a-6ce5-76c3-b8ee-33bd0fda841f` 执行了一次真实 `codex exec resume`。
- 固定健康探针 prompt 已通过 stdin 写入，并在写入后关闭 stdin。
- `codex exec resume` 正常退出，exit code 为 `0`。
- `last-message.txt` 包含固定标记：`E5_LEVEL_B_MARIO_TEST_DIRECTOR_RESUME_OK_2026_06_06`。
- 本轮真实 resume 写入了 `/Users/yoyi/.codex`；这是用户对本任务包授权的 Level B 范围内副作用。
- `/Users/yoyi/Documents/mario test` 四个项目文件 hash 前后一致，没有新增项目根目录文件。

不接受为：

- 通用真实 send / resume 产品化完成。
- 会话中心自由发消息完成。
- 项目工作流自动派发完成。
- 四角色工作流重新验证完成。
- runtime log / diagnostics 完成。
- 自动重试完成。
- planned adapters 真实接入完成。
- provider credential / model verification 完成。
- 阶段 G 真实 Tauri 验收完成。
- 中间版本最终验收完成。

## 2. 授权记录

用户本轮明确指令：

```text
/Users/yoyi/workspace/product-line/tasks/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md批准执行 E5 Level B mario test 健康探针
```

该授权覆盖任务包内的目标项目、native thread、固定 prompt、真实 `codex exec resume`、`/Users/yoyi/.codex` 写入和 evidence 保存。未授权修改项目文件、写 workflow state、读取完整 transcript、读取 auth/token/`.env`/secret/keychain/OAuth/provider credential、启动四角色完整工作流或执行其他 session。

## 3. 执行对象

```text
project_label: mario test
project_root: /Users/yoyi/Documents/mario test
workflow_id: workflow:users-yoyi-documents-mario-test:default
node_id: workflow:users-yoyi-documents-mario-test:default:node:director
session_title: 总指导
native_thread_id: 019e798a-6ce5-76c3-b8ee-33bd0fda841f
adapter_id: codex-local
operation: resume
sandbox: read-only
```

固定 prompt：

```text
你正在参与 E5 Level B 真实 resume 健康探针，项目为 /Users/yoyi/Documents/mario test。
请只回复一行：
E5_LEVEL_B_MARIO_TEST_DIRECTOR_RESUME_OK_2026_06_06
不要读取、列出或修改任何文件。不要运行命令。不要创建计划。不要调用工具。
```

## 4. Command argv 摘要

实际通过 Node runner 使用 argv 数组启动，不把 prompt 拼进 shell 命令：

```text
codex exec -C /Users/yoyi/Documents/mario test --sandbox read-only resume --skip-git-repo-check --json --output-last-message <evidence-dir>/last-message.txt 019e798a-6ce5-76c3-b8ee-33bd0fda841f
```

runner 证据：

- `evidence/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1/run-resume-health-probe.mjs`

原始执行结果摘要：

- `evidence/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1/command-result.json`
- `started_at`: `2026-06-06T07:21:24.202Z`
- `finished_at`: `2026-06-06T07:21:57.714Z`
- `exit_code`: `0`
- `signal`: `null`
- `timeout_ms`: `300000`

## 5. Readback

`last-message.txt`：

```text
E5_LEVEL_B_MARIO_TEST_DIRECTOR_RESUME_OK_2026_06_06
```

`codex-stdout.jsonl` 摘要：

```text
thread.started: 019e798a-6ce5-76c3-b8ee-33bd0fda841f
turn.started
agent_message: E5_LEVEL_B_MARIO_TEST_DIRECTOR_RESUME_OK_2026_06_06
turn.completed
```

stdout usage 摘要：

```text
input_tokens: 452097
cached_input_tokens: 268544
output_tokens: 5052
reasoning_output_tokens: 452
```

说明：执行者没有手工读取完整 transcript / rollout；真实 Codex CLI resume 会使用自身会话上下文。本轮保存的是 stdout 事件摘要、stderr 摘要、last message 和命令结果，不保存完整 transcript。

## 6. stderr 摘要

stderr 包含：

- `Reading prompt from stdin...`
- remote plugin catalog sync authentication warning。
- MCP process group termination / shutdown warning。
- curated plugin sync timeout warning，路径包含 `/Users/yoyi/.codex/.tmp/plugins-clone-...`。

解释：

- 这些是真实 Codex CLI 启动 / 关闭过程中的 warning，不影响 exit code。
- `.codex/.tmp` 写入属于本轮已授权的真实 Codex resume 副作用。
- 没有发现项目文件修改证据。
- 本轮没有读取 auth/token/`.env`/secret/keychain/OAuth/provider credential 内容。

## 7. 项目文件 hash

执行前：

| file | sha256 |
| --- | --- |
| `/Users/yoyi/Documents/mario test/index.html` | `f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf` |
| `/Users/yoyi/Documents/mario test/styles.css` | `6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f` |
| `/Users/yoyi/Documents/mario test/game.js` | `814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd` |
| `/Users/yoyi/Documents/mario test/README.md` | `02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5` |

执行后：

| file | sha256 |
| --- | --- |
| `/Users/yoyi/Documents/mario test/index.html` | `f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf` |
| `/Users/yoyi/Documents/mario test/styles.css` | `6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f` |
| `/Users/yoyi/Documents/mario test/game.js` | `814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd` |
| `/Users/yoyi/Documents/mario test/README.md` | `02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5` |

项目根目录文件列表执行后仍为：

```text
/Users/yoyi/Documents/mario test/game.js
/Users/yoyi/Documents/mario test/index.html
/Users/yoyi/Documents/mario test/styles.css
/Users/yoyi/Documents/mario test/README.md
```

结论：项目文件未修改，未新增项目根目录文件。

## 8. Evidence 文件 hash

| file | sha256 |
| --- | --- |
| `last-message.txt` | `409a339da23a4ac17528b78860b24dadf2fe180eb14fb9333c8edcfb47bd8a50` |
| `codex-stdout.jsonl` | `8ed584faed2bd0ccfabc3cb8ee7cecf37f996b540f84ab6526a79b20697775a0` |
| `codex-stderr.txt` | `40f8f62dd19ab61ff072cebe6e793629bb2ceb03ab94fe6dc3e61cb519b08f74` |
| `command-result.json` | `e8d0f93d93df2ba8cd7b87a2666d735cac6de04525ea7c85ed1a871e0d4c268e` |

## 9. 边界确认

本轮做了：

- 执行真实 `codex exec resume`。
- 发送固定健康探针 prompt。
- 写 `/Users/yoyi/.codex`，由真实 Codex resume 产生。
- 保存 last message、stdout JSONL、stderr、command result 和本 evidence。
- 读取项目根目录文件列表和四个指定项目文件 hash，用于证明项目未修改。

本轮没有做：

- 没有修改 `/Users/yoyi/Documents/mario test` 项目文件。
- 没有写 workflow state。
- 没有创建 work item / dispatch / workflow machine run。
- 没有启动四角色完整工作流。
- 没有向开发线、验证线、回收线发送 prompt。
- 没有手工读取完整 transcript / rollout。
- 没有读取 auth、token、`.env`、secret、keychain、OAuth、provider credential 或密钥文件内容。
- 没有调用 Claude Code / OpenClaw / OpenCode / OpenCode-like。
- 没有把本轮健康探针解释成通用会话控制器完成。

## 10. 后续建议

可以把本轮接受为“E5 Level B 最小真实 `codex-local` resume 健康探针完成”。

后续如果要把真实 send / resume 产品化，仍需单独任务包，至少补：

- 工作台自有 continuation store 与真实 Codex result 的正式绑定。
- runtime log / diagnostics 归入 G1 / G2。
- 真实 Tauri UI 和截图归入 G3。
- 失败、超时、readback unavailable、自动重试和权限恢复的产品策略。

## 11. 文档同步和扫描

完成真实探针证据后，又同步了当前入口和阶段计划：

- `CURRENT.md`
- `AUTHORITY.md`
- `README.md`
- `STAGE_PLAN.md`
- `tasks/README.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`

同步口径：

- E5 Level B mario test 健康探针已完成。
- F1 可作为下一步拆任务包，但 F1 尚未执行。
- G1 runtime log、G2 diagnostics、G3 真实 Tauri 验收仍未执行。
- Level B 只接受为指定 mario test 总指导 session 的最小真实 resume 健康探针。
- 后续任何新的真实 `codex exec resume`、真实 prompt 发送、真实 readback 或读写 `/Users/yoyi/.codex` 仍必须另行取得用户明确授权。

收尾扫描：

- 已扫描当前入口、阶段计划、任务包和 evidence / handoff，确认没有继续把本任务保留在审批前或真实 resume 未执行的当前事实状态。
- 已扫描是否误写成“通用真实 send / resume 产品化完成 / 真实会话控制器完成 / 自动重试完成 / runtime log 完成 / F1 已执行 / 阶段 G 验收完成”。命中内容均为“不接受为 … 完成”或后续仍未执行边界，不是完成声明。

本次文档同步没有重新执行真实 Codex，没有再次读写 `/Users/yoyi/.codex`，没有修改 `/Users/yoyi/Documents/mario test`，没有写 workflow state。
