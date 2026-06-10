# 任务包：刷新 Codex 索引以包含确认 safe probe 测试会话 v1

## 任务名

刷新 Codex 索引以包含确认 safe probe 测试会话 v1。

## 所属开发线

索引内核线 / Codex 会话线。

验证线回收只读结果。

## 当前判断

真实 workflow state 已经写入 active binding，work item 也已经是 `ready_to_dispatch`。

但还不能直接派发 safe probe。

原因：

- 绑定 thread id `019e7389-349a-7f02-aa31-a4a90b24e865` 当前不在桌面壳使用的 `codex-index.json`。
- 当前后端派发代码会拒绝“不在索引内”的绑定会话。

依据：

- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-prepare-real-workflow-state-for-safe-probe-multiline-v1-review.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 薄弱点

- 当前绑定是用户确认的测试会话，但静态索引还是旧的。
- 如果不刷新索引，下一轮 safe probe 会在派发前被拒绝。
- 刷新索引会读取 `/Users/yoyi/.codex` 的元数据，但不能读取授权、密钥、`.env` 或完整 transcript。
- 本任务只补索引前置条件，不能顺手执行派发。

## 目标

让桌面壳使用的当前 `codex-index.json` 能找到确认测试 thread：

- thread id：`019e7389-349a-7f02-aa31-a4a90b24e865`
- 测试会话名：`请只回复这一句：CONTROL_PROBE_OK_2026_05_29`
- 测试 cwd：`/private/tmp/codex-control-probe-v2`
- rollout：应存在

完成后，验证线只读确认：

- `codex-index.json` 中能找到该 thread。
- 该 thread 的 rollout 存在。
- 该 thread 仍被识别为测试会话，不是业务会话。
- 没有读取完整 transcript。
- 没有写 `/Users/yoyi/.codex`。

## 非目标

- 不执行 `codex exec resume`。
- 不发送 safe probe。
- 不写真实 workflow state。
- 不写 `/Users/yoyi/.codex`。
- 不读取完整 transcript。
- 不读取 `auth.json`、`.env`、密钥、token 或授权文件内容。
- 不运行 harness。
- 不删除、移动、归档 Codex 会话。
- 不修改业务项目。

## 允许读取

允许读取项目内：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-prepare-real-workflow-state-for-safe-probe-multiline-v1-review.md`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/build_index.py`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/tests/`

允许读取 `/Users/yoyi/.codex` 的非敏感元数据：

- thread 元数据。
- session 索引元数据。
- rollout 文件路径和存在性。
- 必要的会话摘要字段。

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文
- 与确认测试会话无关的业务会话正文

## 允许写入

允许写入：

- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-refresh-codex-index-for-confirmed-safe-probe-thread-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-refresh-codex-index-for-confirmed-safe-probe-thread-v1-result.md`

如果需要临时输出，允许写：

- `/private/tmp/codex-index-refresh-safe-probe-v1/`

## 禁止事项

- 禁止执行 `codex exec resume`。
- 禁止发送 safe probe。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止读取授权、密钥或 `.env`。
- 禁止读取完整 transcript。
- 禁止运行 harness。
- 禁止把业务会话误标为测试会话。
- 禁止修改真实 workflow state。

## 实施要求

执行顺序：

1. 只读确认当前 `codex-index.json` 是否缺少目标 thread。
2. 运行索引刷新逻辑或生成新索引，使当前 `codex-index.json` 包含目标 thread。
3. 只读验证目标 thread 在索引中存在。
4. 只读验证 rollout 存在。
5. 输出 evidence 和 handoff。

如果刷新失败：

- 不手工编造索引条目。
- 写明失败原因。
- 写明是否读取过敏感文件。
- 写明是否写过 `codex-index.json`。

## 验收标准

必须满足：

- `codex-index.json` 中能按 thread id 找到 `019e7389-349a-7f02-aa31-a4a90b24e865`。
- 该 thread 的 cwd 或 project root 指向 `/private/tmp/codex-control-probe-v2`。
- 该 thread 的 rollout 路径存在。
- 没有读取完整 transcript。
- 没有读取授权、密钥、`.env`。
- 没有写 `/Users/yoyi/.codex`。
- 没有执行 `codex exec resume`。
- 没有发送 safe probe。

建议验证命令：

```bash
python3 /Users/yoyi/workspace/product-line/prototypes/index-kernel/build_index.py --check /Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json
```

如改动索引内核代码，必须运行相关 Python 单元测试。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 做了什么。
3. 是否写了 `codex-index.json`。
4. 是否写了 `/Users/yoyi/.codex`。
5. 是否读取授权、密钥、`.env`。
6. 是否读取完整 transcript。
7. 目标 thread 是否已进入当前索引。
8. rollout 是否存在。
9. 新增 evidence / handoff。
10. 验证命令和结果。
11. 是否建议进入下一轮 safe probe 派发。

## 总指导回收动作

总指导回收时必须判断：

- 接受。
- 需要修改。
- 暂停。
- 废弃。

只有接受后，才能进入：

- `tasks/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md`
