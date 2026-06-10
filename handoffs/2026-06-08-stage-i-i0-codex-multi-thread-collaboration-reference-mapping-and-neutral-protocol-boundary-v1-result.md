# Handoff: Stage I / I0 Codex Multi-thread Collaboration Reference Mapping And Neutral Protocol Boundary v1

日期：2026-06-08

## 回交结论

I0 已完成，结论为 `accepted`。

接受范围：

- Codex 多线程协作参考映射完成。
- adopt / reject / defer 冻结完成。
- 阶段 I 中立协议对象草案完成。
- 后续 I1-I2 合并 checkpoint 推进节奏确认完成。
- 权威入口从 I0 推进到 I1-I2 合并 checkpoint。

不接受范围：

- I1 / I2 产品代码完成。
- WorkerAdapter / RunUnit 类型实现完成。
- 真实多 agent 编排完成。
- 通用自由 Codex 控制台完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 新的真实 Codex 执行授权。

## 新增记录

- `tasks/2026-06-08-stage-i-i0-codex-multi-thread-collaboration-reference-mapping-and-neutral-protocol-boundary-v1.md`
- `evidence/2026-06-08-stage-i-i0-codex-multi-thread-collaboration-reference-mapping-and-neutral-protocol-boundary-v1.md`
- `handoffs/2026-06-08-stage-i-i0-codex-multi-thread-collaboration-reference-mapping-and-neutral-protocol-boundary-v1-result.md`

## 权威入口同步

已同步：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`

同步后当前下一步：

```text
I1-I2：WorkerAdapter / WorkThread / RunUnit 中立模型 + DispatchRequest / PermissionEnvelope / WorkerHandoff 协议
```

## 验证

I0 不改产品代码，不重跑 `npm` / `cargo`。

本轮做了：

- H-I plan I0-I6 范围只读核对。
- H7 evidence / handoff 和当前入口只读核对。
- 旧 I0 / H7 入口口径扫描。
- I0 任务包、evidence、handoff 和入口文档同步。

## 边界

I0 产品路径没有新增产品代码，没有发送 prompt，没有启动 Tauri / GUI / 截图，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。

过程偏差：收尾扫描时一条 shell 命令误把 Markdown 反引号放进双引号，触发 command substitution，导致 `codex exec` / `codex exec resume` 被空 stdin 调起。输出显示 `No prompt provided via stdin`，并且打开 `/Users/yoyi/.codex/state_5.sqlite` 时因 readonly database 失败。该偏差不是产品代码路径，也没有成功写入工作台产品数据；但本轮不能再严格声称“完全没有触发 Codex 命令 / 完全没有触碰 `.codex`”。

## 下一步

进入 `I1-I2` 合并 checkpoint。

默认边界：

- 不执行新的真实 Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不接 planned adapters 真实执行。
- 不读取 provider credential。
- 如涉及 UI，必须按 UI 显示边界方案规划和验收。
