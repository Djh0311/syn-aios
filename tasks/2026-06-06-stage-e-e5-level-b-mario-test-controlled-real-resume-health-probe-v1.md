# Task Package：Stage E / E5 Level B Mario Test Controlled Real Resume Health Probe v1

状态：已完成。  
用途：在 E5 Level A 之后，对 `mario test` 做一次最小、受控、真实 `codex-local` resume 健康探针，验证真实 prompt 发送、真实 `codex exec resume`、真实 readback 和证据回收链路。  
执行方式：先写明目标 session、cwd、prompt、允许写入范围、回滚和证据；只有用户再次明确批准本任务包的执行预览后，才允许执行真实 `codex exec resume`。

完成记录：

- Evidence：`evidence/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`
- Handoff：`handoffs/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1-result.md`
- 结论：`accepted_as_minimal_real_resume_health_probe`
- 结果：真实 `codex exec resume` exit code `0`，last message 返回 `E5_LEVEL_B_MARIO_TEST_DIRECTOR_RESUME_OK_2026_06_06`，四个 mario test 项目文件 hash 前后一致。

## 0. 先说薄弱点

- 这是 E5 Level B，风险高于 E5 Level A：会执行真实 `codex exec resume`，会写 `/Users/yoyi/.codex`。
- 用户已说“做 Level B，项目选择为 mario test”，但 `mario test` 历史上存在两个相关项目：
  - `/Users/yoyi/Documents/mario test`：四角色真实 demo 项目。
  - `/Users/yoyi/codex-workflow-mario-test`：早期 README smoke 测试项目。
- 本任务包默认把用户说的 `mario test` 解释为 `/Users/yoyi/Documents/mario test`。如果用户实际指的是 `/Users/yoyi/codex-workflow-mario-test`，必须停下重写任务包。
- `/Users/yoyi/Documents/mario test` 历史上已经跑过真实四角色闭环，但 E5 Level B 不能复用历史成功当作当前验收；必须有本轮新证据。
- 这次只做健康探针，不做项目开发，不修改 mario demo 文件，不启动四角色完整工作流，不进入 F1 / G1。

## 1. 已知事实 / 未知 / 假设

已知事实：

- E5 Level A 已完成：`tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`。
- E7 已完成，阶段 E 总结论为 `accepted_with_deferred_items`；E5 Level B 仍 deferred，但允许在用户明确授权后单独执行。
- `/Users/yoyi/Documents/mario test` 的四角色绑定历史记录存在：
  - project id：`project:users-yoyi-documents-mario-test`
  - workflow id：`workflow:users-yoyi-documents-mario-test:default`
  - 总指导 node：`workflow:users-yoyi-documents-mario-test:default:node:director`
  - 总指导 native thread id：`019e798a-6ce5-76c3-b8ee-33bd0fda841f`
  - project root / cwd：`/Users/yoyi/Documents/mario test`
- 历史 evidence 记录过 runner stdin 未关闭导致 `codex exec resume` 超时；后续已修复 runner 写入 prompt 后关闭 stdin。

未知：

- 当前 Codex 原生会话 `019e798a-6ce5-76c3-b8ee-33bd0fda841f` 是否仍可正常 resume。
- 当前 `/Users/yoyi/.codex` 是否允许本轮真实写入。
- 当前 `codex` CLI 版本行为是否与历史记录一致。
- 真实 readback 是否能稳定拿到固定标记。

本任务采用的假设：

- 本轮目标是最小健康探针，不修改 `/Users/yoyi/Documents/mario test`。
- 使用“总指导”会话作为目标 session，因为它是 `/Users/yoyi/Documents/mario test` 的入口角色。
- 本轮不读取完整 transcript，不读取 auth/token/`.env`/keychain/OAuth/provider credential。
- 本轮不写 workflow state，除非执行者先停下并重新取得用户对 workflow state 写入的明确批准。

## 2. 执行预览

本轮默认执行对象：

```text
project_label: mario test
project_root: /Users/yoyi/Documents/mario test
workflow_id: workflow:users-yoyi-documents-mario-test:default
node_id: workflow:users-yoyi-documents-mario-test:default:node:director
session_title: 总指导
native_thread_id: 019e798a-6ce5-76c3-b8ee-33bd0fda841f
adapter_id: codex-local
operation: resume
```

本轮 prompt 必须是以下精确内容，不得临时扩写：

```text
你正在参与 E5 Level B 真实 resume 健康探针，项目为 /Users/yoyi/Documents/mario test。
请只回复一行：
E5_LEVEL_B_MARIO_TEST_DIRECTOR_RESUME_OK_2026_06_06
不要读取、列出或修改任何文件。不要运行命令。不要创建计划。不要调用工具。
```

建议命令语义：

```text
codex exec -C "/Users/yoyi/Documents/mario test" --sandbox read-only resume --skip-git-repo-check --json --output-last-message <tmp-last-message-path> 019e798a-6ce5-76c3-b8ee-33bd0fda841f
```

执行要求：

- prompt 必须通过 stdin 传入，不拼接进 shell 字符串。
- 写完 prompt 后必须关闭 stdin，避免历史 `Reading prompt from stdin...` 卡住问题。
- `--sandbox` 优先使用 `read-only`，因为本轮不授权项目文件写入。
- `--output-last-message` 必须写到 `/tmp` 或工作区 evidence 临时目录，回收时只保存固定标记和摘要，不保存完整 transcript。

## 3. 明确授权范围

如用户批准执行，本轮允许：

- 执行真实 `codex exec resume`。
- 向 native thread `019e798a-6ce5-76c3-b8ee-33bd0fda841f` 发送上文精确 prompt。
- 通过真实 resume 写 `/Users/yoyi/.codex` 的 Codex 原生会话状态。
- 在 `/tmp` 或 `evidence/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1/` 下保存 stdout / stderr 摘要、last message、hash 和命令结果。
- 读取 `/Users/yoyi/Documents/mario test` 的文件列表 / hash 用于确认没有项目文件被修改。

本轮不允许：

- 不修改 `/Users/yoyi/Documents/mario test` 下任何文件。
- 不写 workflow state。
- 不创建新的 work item / dispatch / workflow machine run。
- 不启动四角色完整工作流。
- 不向开发线、验证线、回收线发送 prompt。
- 不读取完整 transcript。
- 不读取 auth、token、`.env`、keychain、OAuth、provider credential 或密钥文件。
- 不调用 Claude Code / OpenClaw / OpenCode / OpenCode-like。
- 不调用外部模型 provider。
- 不把本轮健康探针解释成真实会话控制器完成、自动重试完成、runtime log 完成或阶段 G 验收完成。

## 4. 执行前检查

执行前必须完成：

1. 用户明确确认本任务包执行预览，尤其是：
   - project_root：`/Users/yoyi/Documents/mario test`
   - native_thread_id：`019e798a-6ce5-76c3-b8ee-33bd0fda841f`
   - prompt 固定标记：`E5_LEVEL_B_MARIO_TEST_DIRECTOR_RESUME_OK_2026_06_06`
   - 允许写 `/Users/yoyi/.codex`
   - 不允许改项目文件
2. 记录执行前项目文件 hash：
   - `/Users/yoyi/Documents/mario test/index.html`
   - `/Users/yoyi/Documents/mario test/styles.css`
   - `/Users/yoyi/Documents/mario test/game.js`
   - `/Users/yoyi/Documents/mario test/README.md`
3. 确认不会使用 shell 双引号包住含反引号文本。
4. 确认不会用历史 evidence 替代本轮真实执行证据。

## 5. 验收

成功验收必须同时满足：

- `codex exec resume` 进程正常退出。
- last message 包含：

```text
E5_LEVEL_B_MARIO_TEST_DIRECTOR_RESUME_OK_2026_06_06
```

- 记录本轮确实发送了真实 prompt。
- 记录本轮确实写了 `/Users/yoyi/.codex`。
- `/Users/yoyi/Documents/mario test` 的四个项目文件 hash 前后一致。
- 没有写 workflow state。
- 没有读取完整 transcript、auth、token、`.env`、secret、keychain、OAuth 或 provider credential。
- 新增 evidence / handoff 明确 Level B 健康探针完成范围和不接受范围。

失败验收：

- 如果 resume 超时、exit nonzero、last message 缺少固定标记或 hash 变化，必须标为 `needs_changes`。
- 如果项目文件被修改，必须停止并回传；除非用户明确要求，否则只给出差异和回滚建议，不自动回滚。
- 如果误读敏感文件或完整 transcript，必须标为过程偏差并停下。

## 6. Evidence / Handoff 要求

执行后新增：

- `evidence/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`
- `handoffs/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1-result.md`

Evidence 必须写清：

- 是否获得用户最终批准。
- 实际 command argv 摘要，不能保存 secret。
- target project / workflow / node / native thread。
- prompt 固定标记。
- 是否执行真实 `codex exec resume`。
- 是否写 `/Users/yoyi/.codex`。
- 是否修改 `/Users/yoyi/Documents/mario test`，用 hash 前后对比证明。
- readback 是否包含固定标记。
- stdout / stderr / exit code / timeout 摘要。
- 是否读取完整 transcript 或敏感文件。
- 本轮不接受为什么。

Handoff 必须写清：

- Level B 健康探针是否通过。
- 是否可以把 E5 Level B 接受为“最小真实 resume 健康探针完成”。
- 仍不能接受为哪些能力完成。
- 如果失败，下一步修补建议。

## 7. Stop 条件

遇到以下情况必须停下：

- 用户未明确批准本任务包执行预览。
- 用户没有确认 `mario test` 指 `/Users/yoyi/Documents/mario test`。
- 需要改 prompt。
- 需要修改项目文件。
- 需要写 workflow state。
- 需要读取完整 transcript。
- 需要读取 auth/token/`.env`/secret/keychain/OAuth/provider credential。
- 需要执行除目标 native thread 外的其他 Codex session。
- `codex exec resume` 需要提升到比本任务包更大的写入范围。

## 8. 回收口径

完成后可接受为：

- E5 Level B 最小真实 `codex-local` resume 健康探针完成。
- 指定 mario test 总指导 session 收到真实 prompt 并返回固定标记。
- 真实 readback 最小链路完成。
- 本轮证明确实会写 `/Users/yoyi/.codex`，且没有修改项目文件。

完成后不接受为：

- 通用真实 send / resume 产品化完成。
- 项目工作流自动派发完成。
- 四角色工作流重新验证完成。
- runtime log / diagnostics 完成。
- 自动重试完成。
- planned adapters 真实接入完成。
- provider credential / model verification 完成。
- 阶段 G 真实 Tauri 验收完成。
- 中间版本最终验收完成。
