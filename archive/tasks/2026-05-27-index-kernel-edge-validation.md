# 任务包：索引内核边界异常补测

## 所属开发线

验证线。

## 背景

索引内核 hardening 已完成并回收。当前仍未覆盖 SQLite 损坏文件、权限拒绝、符号链接绕过、超大 JSONL 等边界风险。为避免这些风险拖进桌面应用线，需要在现有验证线里补测，不新增开发线。

依据：

- `product-line/handoffs/2026-05-27-index-kernel-hardening-review.md`
- `product-line/evidence/2026-05-27-index-kernel-hardening.md`
- `product-line/handoffs/2026-05-27-index-kernel-hardening-result.md`

## 目标

- 增加索引内核边界异常测试。
- 覆盖 SQLite 损坏文件。
- 尽量覆盖权限拒绝场景；如果本机权限模型无法稳定模拟，要说明原因。
- 覆盖 rollout_path 符号链接绕过或确认现有 `is_relative_to(resolve)` 能阻断。
- 覆盖大文件或大 JSONL 不被默认读取正文。
- 输出 evidence 和 handoff。

## 允许读取

- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/build_index.py`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-27-index-kernel-hardening.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-27-index-kernel-hardening-result.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-27-index-kernel-hardening-review.md`

## 允许写入

- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/tests/`
- `/Users/yoyi/workspace/product-line/evidence/`
- `/Users/yoyi/workspace/product-line/handoffs/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 `state_5.sqlite`。
- 不读取或打印 `auth.json`、`.env`、密钥、令牌。
- 不把测试夹具放进 `/Users/yoyi/.codex`。
- 不把会话正文、命令输出、工具输出、输入历史或记忆正文加入索引。

## 验收标准

- 有可运行测试命令。
- 原 12 个测试仍通过。
- 新增边界测试通过，或明确说明无法稳定模拟的原因。
- 不依赖网络。
- 不读取或写入真实 `/Users/yoyi/.codex`。
- 输出一份 evidence 和 handoff。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增了哪些测试
4. 哪些边界场景通过
5. 哪些场景无法稳定模拟
6. 是否建议修改索引内核
7. 风险和下一步建议
