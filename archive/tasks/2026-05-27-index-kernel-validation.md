# 任务包：索引内核异常夹具验证

## 所属开发线

验证线。

## 背景

索引内核原型已回收并接受，但当前验证只覆盖真实环境和输出结构，还没有覆盖坏 schema、缺字段、缺文件和坏 manifest。第一版要稳定，必须证明索引器遇到 Codex 内部结构变化时会降级，而不是崩掉或误写文件。

依据：

- `product-line/handoffs/2026-05-27-codex-index-kernel-review.md`
- `product-line/evidence/2026-05-27-codex-index-kernel.md`
- `product-line/handoffs/2026-05-27-codex-index-kernel-result.md`

## 目标

- 为 `build_index.py` 补异常夹具验证。
- 覆盖 SQLite 缺表、缺字段、rollout 文件缺失、坏 plugin manifest、坏 session_index JSONL。
- 验证索引器只读，不写 `/Users/yoyi/.codex`。
- 输出验证 evidence 和下一步修复建议。

## 允许读取

- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/build_index.py`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-27-codex-index-kernel.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-27-codex-index-kernel-result.md`

## 允许写入

- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/tests/`
- `/Users/yoyi/workspace/product-line/evidence/`
- `/Users/yoyi/workspace/product-line/handoffs/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 `state_5.sqlite`。
- 不用真实授权文件做测试夹具。
- 不读取或打印 `auth.json`、`.env`、密钥、令牌。
- 不把测试夹具放进 `/Users/yoyi/.codex`。

## 建议验证项

- SQLite 文件不存在。
- SQLite 存在但没有 `threads` 表。
- `threads` 表缺少非关键字段。
- `threads` 表缺少 `id` 字段。
- `rollout_path` 指向不存在的文件。
- `rollout_path` 指向允许目录外文件。
- `session_index.jsonl` 含坏 JSON 行。
- plugin manifest JSON 损坏。
- skill 文件编码异常或读取失败。

## 验收标准

- 有可运行验证命令。
- 验证命令不依赖网络。
- 验证命令不读取真实授权文件。
- 失败场景能产生 warning 或明确错误码，不静默吞掉。
- 不对 `/Users/yoyi/.codex` 产生写入。
- 输出一份 evidence 和一份 handoff。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增了哪些测试或夹具
4. 哪些异常场景通过
5. 哪些异常场景暴露了问题
6. 是否建议修改索引内核
7. 风险和下一步建议
