# 四角色工作流机器 Handoff

## 薄弱点

- 代码路径已实现，但还没有真实跑 mario test 四会话闭环。
- 这不是并发调度，也不是复杂失败自愈；失败会停机并落账。
- 真实验收仍需要单独确认，因为会执行 `codex exec resume`、写 `/Users/yoyi/.codex`、写真实 workflow state，并允许开发线改 `/Users/yoyi/Documents/mario test`。

## 完成内容

实现了最小工作流机器：

`总指导 -> 开发线 -> 验证线 -> 回收线 -> 总指导结论 -> 下一轮 / 最终接受`

新增后端命令：

- `run_workflow_machine`

新增记录：

- `workflow_machine_runs[]`

新增 UI：

- 当前工作项卡片内的“工作流机器 / 总指导循环闭环”
- “启动闭环”确认按钮

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，4 个测试。
- 指定 Cargo 缓存后 `cargo test --offline`：通过，68 passed，1 ignored。
- `npm run build`：通过。
- `build_index.py --check codex-index.json`：`validation_ok`。

默认 `cargo test --offline` 仍会因本机默认 Cargo 缓存版本不匹配失败：

- 锁文件需要 `serde_json 1.0.150`
- 默认离线缓存只有 `1.0.149`

## 边界

- 未执行真实 `codex exec` / `codex exec resume`。
- 未写 `/Users/yoyi/.codex`。
- 未写真实 workflow state。
- 未修改 `/Users/yoyi/Documents/mario test`。
- 未读取敏感文件或完整 transcript。

## 下一步

可以进入真实全流程验收：

- 工作项：`workflow:users-yoyi-documents-mario-test:default:create-mario-demo-v1`
- 项目：`/Users/yoyi/Documents/mario test`
- 目标：用四角色工作流机器闭环完成马里奥 demo。

执行前必须再次明确确认真实运行边界。
