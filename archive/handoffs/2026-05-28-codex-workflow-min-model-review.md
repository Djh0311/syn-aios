# Codex 项目级工作流最小数据模型总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-codex-workflow-min-model.md`
- 开发线：信息架构线
- Evidence：`product-line/evidence/2026-05-28-codex-workflow-min-model.md`
- Handoff：`product-line/handoffs/2026-05-28-codex-workflow-min-model-result.md`
- Decision：`product-line/decisions/2026-05-28-codex-workflow-min-model.md`

## 结论

接受为“阶段 3 最小工作流数据模型”。

不接受为代码实现，不接受为可编辑自动化工作流完成，不接受为多 agent 接入完成。

依据：

- 模型采用 agent-agnostic schema，当前实例只填 Codex，符合高扩展开发规则。
- 模型分清了只读索引派生层、本地工作台事实层、未来智能建议层。
- 模型明确 LM 是未来智能核心，不是事实核心。
- 模型给出了 `workflow_version`、`source_kind`、`permission_level`、`agent_type`、`adapter_id`、`artifact_type`、`node_type`、`edge_type`、`model_policy` 等扩展字段。
- 模型没有把 OpenClaw / VS Code / OpenCode / Claude Code 写成已可用能力。
- 模型把 Codex 专用文件夹式 harness 建模为 `ProjectCapability` 和 `HarnessResource`，没有继续当单个脚本候选处理。
- 模型明确当前 `harness_candidates` 是文件级候选，不能说已经完整支持文件夹式 Codex harness。

## 先说薄弱点

- 这轮只定义模型，没有实现代码。依据：任务允许写入只有 `evidence/`、`handoffs/`、`decisions/`。
- 本地工作台状态存储位置还没定。依据：handoff 明确 JSON / SQLite 和路径仍不确定。
- 文件夹式 harness 的 manifest 规范还没定。依据：handoff 明确 manifest、扫描深度、安全边界仍不确定。
- ReviewRecord 从 review 文件解析还是先本地登记还没定。依据：handoff 明确这是不确定项。
- 模型偏抽象，后续实现需要严格控制第一版范围，避免直接跳到复杂画布、多 agent 或 LM 调度。

## 接受内容

接受以下模型对象：

- `WorkspaceProject`
- `AgentAdapter`
- `Workflow`
- `WorkflowNode`
- `WorkflowEdge`
- `ActorRole`
- `WorkItem`
- `Artifact`
- `ReviewRecord`
- `ProjectCapability`
- `HarnessResource`
- `AuditEvent`

接受以下分层：

- 只读索引派生层：从 `codex-index.json` 派生项目、会话、handoff、evidence、harness 候选。
- 本地工作台事实层：保存 workflow、node、edge、state、review、audit、project capability。
- 未来智能建议层：LM 可以提出建议，但不能直接成为事实状态。

接受以下当前 Codex-only 映射：

- `agent_type = codex`
- `adapter_id = codex-local`
- `provider = local-codex-index`
- `source_kind = codex_index | workspace_state | project_file | user_input | derived`
- `permission_level = read_only`
- `workflow_version = 1`
- `model_policy = none`

## Harness 判断

接受关于 harness 的判断：

- Codex 专用 harness 是项目能力和验证资源。
- 文件夹形式 harness 应以文件夹根作为 `HarnessResource.root_path`。
- 文件夹内部脚本、配置、README、manifest 应作为 `entrypoints` 或 `Artifact`。
- 与 Codex 的关系通过 `agent_type=codex`、`adapter_id=codex-local`、`capabilities` 表达。
- 当前索引缺文件夹根、manifest、版本、来源仓库、功能说明、适配 agent / adapter、命令入口语义、最近验证状态。

因此下一步应优先派索引内核线补文件夹式 harness 扫描。

## 安全和范围判断

接受当前安全边界。

依据：

- 没有写 `/Users/yoyi/.codex`。
- 没有改真实 Codex 状态库。
- 没有改前端、Tauri、Rust 或索引内核代码。
- 没有读取或展示 auth、env、密钥、令牌、授权文件内容。
- 没有读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 没有自动运行 harness。
- 没有接入非 Codex agent。
- 没有把 LM 建议当作事实状态。

## 当前状态

这条任务从“待派发”改为“已回收”。

当前可以说：

- 阶段 3 最小工作流模型已定。
- 高扩展开发规则已落到工作流模型。
- 文件夹式 Codex harness 已有建模口径。

仍不能说：

- 工作流模型已经实现到前端。
- 本地状态存储已经落地。
- 文件夹式 harness 索引已经实现。
- 可编辑自动化工作流已经完成。

## 下一步

下一步派给索引内核线：补文件夹式 Codex harness 扫描。

理由：

- 用户明确指出当前 harness 基本是 Codex 专用且以文件夹形式存在。
- 当前模型已经把 harness 定义成 `ProjectCapability` / `HarnessResource`。
- 当前索引只有文件级 `harness_candidates`，会漏掉文件夹级能力。

后续索引内核任务完成后，再派桌面应用线接入最小工作流模型和新的 harness 资源字段。
