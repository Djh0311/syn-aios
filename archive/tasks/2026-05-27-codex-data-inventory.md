# 任务包：Codex 本地数据盘点

## 所属开发线

Codex 数据盘点线。

## 背景

第一版要先治理 Codex，不先统筹所有 agent。依据：用户明确说先做接入 Codex 的版本，先把 Codex 管理好。

## 目标

- 只读盘点 `/Users/yoyi/.codex` 下可用于工作台索引的数据源。
- 明确会话、项目、skills、plugins、memories、状态库、日志分别在哪里。
- 标出每类数据的可靠性和风险。
- 给索引内核线输出字段建议。

## 允许读取

- `/Users/yoyi/.codex/session_index.jsonl`
- `/Users/yoyi/.codex/sessions/`
- `/Users/yoyi/.codex/state_5.sqlite`
- `/Users/yoyi/.codex/.codex-global-state.json`
- `/Users/yoyi/.codex/skills/`
- `/Users/yoyi/.codex/plugins/`
- `/Users/yoyi/.codex/memories/`
- `/Users/yoyi/workspace/product-line/`

## 允许写入

- `/Users/yoyi/workspace/product-line/evidence/`
- `/Users/yoyi/workspace/product-line/handoffs/`

## 禁止事项

- 不改 `/Users/yoyi/.codex`。
- 不移动、删除、格式化任何 Codex 文件。
- 不读取或打印 `auth.json`、`.env`、密钥、令牌。
- 不把 Codex 侧边栏显示当作可靠数据源。

## 验收标准

- 输出一份 evidence，列出每个数据源的路径、字段、用途、风险。
- 输出一份 handoff，给索引内核线说明该读哪些字段。
- 所有结论都要写依据。
- 明确哪些字段只能参考，不能作为权威。

## 必须回传

1. 做了什么
2. 读了哪些文件或目录
3. 新增了哪些 evidence / handoff
4. 哪些数据源可用于第一版
5. 哪些数据源不可靠或禁止使用
6. 风险和下一步建议
