# 总指导回收意见：第一条用户审核极小业务试跑指令设计 v1

## 回收对象

- 任务包：`/Users/yoyi/workspace/product-line/tasks/2026-05-30-first-user-reviewed-tiny-business-instruction-design-v1.md`
- Evidence：`/Users/yoyi/workspace/product-line/evidence/2026-05-30-first-user-reviewed-tiny-business-instruction-design-v1.md`
- Handoff：`/Users/yoyi/workspace/product-line/handoffs/2026-05-30-first-user-reviewed-tiny-business-instruction-design-v1-result.md`

## 结论

接受为“候选指令设计完成”。

不接受为“用户已批准执行”，也不接受为“真实业务试跑已开始”。

## 薄弱点

- 目标目录 `/Users/yoyi/gameai/agent world` 存在，但顶层条目数为 0；只读体检的业务价值有限。
- 当前 active binding 仍是测试会话，且带有 `confirmed_test_session_not_business_session` warning，不应直接用于业务试跑。
- 候选指令未写入真实 workflow state。
- 没有用户确认前，不能把候选指令当成已批准任务。

## 接受内容

接受以下事实：

- 已产出候选指令。
- 候选只允许只读体检目录元数据和顶层文件/目录名称。
- 候选禁止读取 `.env`、`auth.json`、密钥、token、授权文件或完整 transcript。
- 候选禁止写业务文件。
- 候选禁止执行 `codex exec`、`codex exec resume`、harness。
- 候选设置超时 600 秒、最大重试 0。
- 遇到敏感文件、读取正文、写入、执行命令、绑定会话需求时必须停止并回传。

## 复核依据

总指导只读复核：

- `/Users/yoyi/gameai/agent world` 存在。
- `/Users/yoyi/gameai/agent world` 是目录。
- 顶层条目数：0。
- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- 真实 workflow state 中仍可见 `confirmed_test_session_not_business_session` warning。

## 当前可以说

- 第一条极小业务试跑的候选指令已经设计出来。
- 这条候选适合作为“只读目录体检”的保守试跑模板。

## 当前不能说

- 不能说用户已经批准执行。
- 不能说真实业务自动编排已经开始。
- 不能说当前测试会话可以直接作为业务会话使用。
- 不能说候选已写入真实 workflow state。

## 需要用户确认

1. 是否接受“只读项目目录体检”作为第一条极小业务试跑。
2. 是否先重新绑定一个 cwd 为 `/Users/yoyi/gameai/agent world` 的业务 Codex 会话。
3. 如果后续目录中出现 `package.json`、`README.md` 等非敏感 manifest，是否允许读取正文。
4. 是否允许把候选指令写入真实 workflow state 的 `workflow_execution_controls[]`。

## 下一步

等待用户确认以上四个问题。

没有确认前，不进入真实派发。
