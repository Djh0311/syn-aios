# Evidence: Stage H / H5-Level-B2 Task Package And Delegation v1

日期：2026-06-08

## 结论

H5-Level-B2 任务包已创建，结论为：

```text
accepted_as_h5_level_b2_task_package_created_and_ready_for_reused_dev_thread_delegation
```

本轮只接受为：

- H5-Level-B2 `mario test` 受控写入型真实项目工作流派发任务包已创建。
- B2 执行点、目标项目、目标 worker session、sandbox、allowed write path、prompt summary/ref/marker、readback、runtime log、audit、evidence、handoff 和停止条件已冻结。
- 后续可优先复用已有 H5-Level-B1 开发线程执行 B2，而不是为每个 probe 新建一次性线程。

本轮不接受为：

- H5-Level-B2 已执行。
- H5 通用项目工作流真实派发产品化完成。
- H5 product command 正式化完成。
- H3-B new-session 成功。
- H4-Level-B 真实失败 / 超时探针完成。
- 阶段 H 完成。

## 任务包

新增任务包：

```text
tasks/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md
```

## 冻结的执行范围

```text
project_label: mario test
project_root: /Users/yoyi/Documents/mario test
project_id: project:users-yoyi-documents-mario-test
workflow_id: workflow:users-yoyi-documents-mario-test:default
target_node_id: workflow:users-yoyi-documents-mario-test:default:node:codex-dev
target_session_id: 019e798a-ac37-7771-b982-e38084fcd22e
adapter_id: codex-local
operation: resume
sandbox: workspace-write
allowed_project_write_path: /Users/yoyi/Documents/mario test/.workbench/h5-b2/real-dispatch-write-probe.md
readback_marker: H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08
```

## 关键边界

- B2 允许真实 Codex 执行并允许最小项目写入，但只允许写 `.workbench/h5-b2/real-dispatch-write-probe.md` 或同目录必要探针元数据。
- B2 不允许修改 `index.html`、`styles.css`、`game.js`、`README.md`；这些核心文件必须记录执行前后 hash 并保持一致。
- B2 不授权读取 auth/token/secret/`.env`/keychain/OAuth/provider credential。
- B2 不授权读取完整 transcript 或 rollout。
- B2 不授权 `new_session`、自动重试、stop / kill / restart、planned adapters 真实接入或 provider/model verification。
- B2 即使成功，也只能接受为单项目 workspace-write 真实派发 probe 完成，不能冒充 H5 通用产品化或阶段 H 完成。

## 多线程复用策略

用户明确要求不要“一个对话用完就扔”。因此 B2 派发策略调整为：

- 优先复用 H5-Level-B1 `mario test` 真实派发开发线程。
- 主管线只做任务冻结、派发、等待、复核和权威入口同步。
- 开发线负责执行任务包、写 execution evidence/handoff、回交。
- 如后续需要验证线或回收线，优先复用既有验证 / 回收职责线程；仅在线程职责不匹配、不可恢复或历史上下文会误导时才新建。

## 本轮未执行

本轮没有执行：

- 真实 `codex exec`
- 真实 `codex exec resume`
- prompt 发送
- `/Users/yoyi/.codex` 读写
- `mario test` 项目写入
- 产品代码改动

## 下一步

1. 同步权威入口。
2. 读取并复用 H5-Level-B1 开发线程。
3. 将 B2 任务包派发给复用开发线，要求高思考程度并回交 evidence / handoff。
4. 主管线等待开发线完成后做独立复核。
