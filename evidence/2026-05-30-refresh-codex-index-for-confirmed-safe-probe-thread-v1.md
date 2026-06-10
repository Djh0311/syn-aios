# 刷新 Codex 索引以包含确认 safe probe 测试会话 v1 证据

## 结论先说

薄弱点：

- 刷新前桌面壳使用的 `codex-index.json` 查不到目标 thread。依据：刷新前用 `jq` 按 `thread_id=019e7389-349a-7f02-aa31-a4a90b24e865` 查询，输出为空。
- 当前真实 Codex 线程数继续变化，本轮刷新后索引线程数为 324。这个数字只能代表本轮生成时状态。
- 本任务只补索引前置条件，没有派发 safe probe，也没有证明 safe probe 下一步一定成功。

可用结果：

- 已写入 `product-line/prototypes/index-kernel/codex-index.json`。
- 目标 thread 已进入当前索引。
- 目标 thread 的 `project_root` 是 `/private/tmp/codex-control-probe-v2`。
- 目标 thread 的 `rollout_exists=true`。
- 没有读取完整 transcript。
- 没有读取授权、密钥、`.env`。
- 没有写 `/Users/yoyi/.codex`。
- 没有执行 `codex exec resume`。
- 没有发送 safe probe。

## 任务对象

- thread id：`019e7389-349a-7f02-aa31-a4a90b24e865`
- 测试会话名：`请只回复这一句：CONTROL_PROBE_OK_2026_05_29`
- 测试 cwd：`/private/tmp/codex-control-probe-v2`

## 执行步骤

刷新前只读检查当前索引：

```bash
jq --arg id '019e7389-349a-7f02-aa31-a4a90b24e865' '.threads[]? | select(.thread_id==$id) | {thread_id,title,project_root,rollout_path,rollout_exists,thread_source,model,model_provider,has_user_event,warnings}' product-line/prototypes/index-kernel/codex-index.json
```

结果：输出为空，说明刷新前旧索引缺少目标 thread。

刷新索引：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --pretty
```

结果摘要：

```json
{"memory_count": 11, "output": "/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json", "plugin_count": 11, "project_count": 33, "rollout_checked": 324, "rollout_existing": 324, "skill_count": 51, "thread_count": 324, "warning_count": 0}
```

刷新后只读检查目标 thread：

```bash
jq --arg id '019e7389-349a-7f02-aa31-a4a90b24e865' '.threads[]? | select(.thread_id==$id) | {thread_id,title,project_root,rollout_path,rollout_exists,thread_source,model,model_provider,has_user_event,warnings}' product-line/prototypes/index-kernel/codex-index.json
```

结果：

```json
{
  "thread_id": "019e7389-349a-7f02-aa31-a4a90b24e865",
  "title": "请只回复这一句：CONTROL_PROBE_OK_2026_05_29",
  "project_root": "/private/tmp/codex-control-probe-v2",
  "rollout_path": "/Users/yoyi/.codex/sessions/2026/05/29/rollout-2026-05-29T19-40-32-019e7389-349a-7f02-aa31-a4a90b24e865.jsonl",
  "rollout_exists": true,
  "thread_source": "user",
  "model": "gpt-5.5",
  "model_provider": "ai",
  "has_user_event": false,
  "warnings": []
}
```

结构校验：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

结果：

```text
validation_ok
```

warning 汇总：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json --warning-summary
```

结果：

```json
{"entrypoints_truncated": 4, "harness_candidates_truncated": 1, "missing_entrypoints": 6, "missing_manifest": 12, "missing_readme": 12, "missing_version": 14, "project_root_missing": 2, "title_truncated": 86, "weak_harness_signal": 1}
```

## 安全边界

没有执行：

- `codex exec resume`
- safe probe 派发
- harness

没有读取：

- 完整 transcript
- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容

没有写：

- `/Users/yoyi/.codex`
- 真实 workflow state

补充验证：

- 刷新前 `/Users/yoyi/.codex` 目录 mtime：`1780074121`
- 刷新后 `/Users/yoyi/.codex` 目录 mtime：`1780074121`

mtime 不变只能作为辅助证据；主要依据是本轮只运行索引生成器，写入目标限于工作区 `codex-index.json`、本 evidence 和 handoff。

## 是否建议进入下一轮 safe probe 派发

建议可以进入下一轮任务包的回收和派发前检查，但不是本任务直接派发。

依据：

- 派发前阻塞点“绑定 thread 不在索引内”已经消除。
- 目标 thread 在索引内。
- rollout 存在。

仍需总指导回收接受后，再进入 `tasks/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md`。
