# 刷新 Codex 索引以包含确认 safe probe 测试会话 v1 交接

## 状态

任务完成，可进入验证线只读回收。

## 做了什么

- 只读确认刷新前 `codex-index.json` 缺少目标 thread。
- 运行索引生成器刷新 `product-line/prototypes/index-kernel/codex-index.json`。
- 只读确认目标 thread 已进入索引。
- 只读确认目标 thread 的 rollout 存在。
- 写入 evidence 和 handoff。

## 是否写了 `codex-index.json`

是。

写入文件：

- `product-line/prototypes/index-kernel/codex-index.json`

刷新结果摘要：

- 线程数：324
- 项目数：33
- skills 数：51
- plugins 数：11
- rollout：324/324 存在

## 是否写了 `/Users/yoyi/.codex`

否。

本轮没有写 `/Users/yoyi/.codex`，也没有修改真实 Codex 状态库。

## 是否读取授权、密钥、`.env`

否。

没有读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥
- token
- 授权文件内容

## 是否读取完整 transcript

否。

本轮只通过索引元数据验证 `rollout_path` 和 `rollout_exists`，没有打开目标 rollout 文件正文。

## 目标 thread 是否已进入当前索引

是。

目标 thread：

- `019e7389-349a-7f02-aa31-a4a90b24e865`

索引中记录：

- `title`：`请只回复这一句：CONTROL_PROBE_OK_2026_05_29`
- `project_root`：`/private/tmp/codex-control-probe-v2`
- `thread_source`：`user`
- `model`：`gpt-5.5`
- `model_provider`：`ai`
- `warnings`：空

## rollout 是否存在

是。

索引中记录：

- `rollout_path`：`/Users/yoyi/.codex/sessions/2026/05/29/rollout-2026-05-29T19-40-32-019e7389-349a-7f02-aa31-a4a90b24e865.jsonl`
- `rollout_exists`：`true`

## 新增 evidence / handoff

- `product-line/evidence/2026-05-30-refresh-codex-index-for-confirmed-safe-probe-thread-v1.md`
- `product-line/handoffs/2026-05-30-refresh-codex-index-for-confirmed-safe-probe-thread-v1-result.md`

## 验证命令和结果

刷新前查目标 thread：

```bash
jq --arg id '019e7389-349a-7f02-aa31-a4a90b24e865' '.threads[]? | select(.thread_id==$id) | {thread_id,title,project_root,rollout_path,rollout_exists,thread_source,model,model_provider,has_user_event,warnings}' product-line/prototypes/index-kernel/codex-index.json
```

结果：输出为空。

刷新索引：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --pretty
```

结果：成功写入 `codex-index.json`。

刷新后查目标 thread：

```bash
jq --arg id '019e7389-349a-7f02-aa31-a4a90b24e865' '.threads[]? | select(.thread_id==$id) | {thread_id,title,project_root,rollout_path,rollout_exists,thread_source,model,model_provider,has_user_event,warnings}' product-line/prototypes/index-kernel/codex-index.json
```

结果：目标 thread 存在，`project_root=/private/tmp/codex-control-probe-v2`，`rollout_exists=true`。

结构校验：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

结果：

```text
validation_ok
```

## 是否建议进入下一轮 safe probe 派发

建议可以进入下一轮派发任务的回收前置检查。

但本轮没有派发 safe probe，也没有执行 `codex exec resume`。是否进入派发，应由总指导回收接受后决定。

## 风险

- 当前索引刷新只证明目标 thread 已进入静态索引，不证明 safe probe 派发一定成功。
- 当前索引 warning 汇总仍有 harness / project context 类 warning，和目标 thread 无直接关系，但桌面壳仍应展示。
