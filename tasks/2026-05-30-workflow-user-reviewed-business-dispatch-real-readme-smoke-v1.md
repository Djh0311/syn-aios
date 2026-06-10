# 任务包：用户审核业务派发真实 README 极小验证 v1

## 任务名

用户审核业务派发真实 README 极小验证 v1。

## 所属开发线

桌面应用线 / Codex 会话线。

总指导线回收。

## 当前判断

用户审核业务派发代码修正已完成，可以进入真实极小验证候选阶段。

依据：

- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-fix-v1-review.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-user-reviewed-business-dispatch-fix-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-fix-v1-result.md`

大白话：

代码已经准备好，现在需要做一次很小的真实派发，验证工作台能不能真的通过用户审核业务指令改一个允许范围内的文件。

## 薄弱点

- 这一步会真实执行 `codex exec resume`，会写 `/Users/yoyi/.codex`。
- 这一步会修改测试项目 README。
- 如果真实 Codex 会话环境和离线测试不同，仍可能失败。
- 这一步只验证极小 README 修改，不证明复杂业务自动编排。

## 目标

通过桌面壳用户审核业务派发路径，对测试项目 README 做一个极小、可复核的修改：

- 目标文件：`/Users/yoyi/codex-workflow-mario-test/README.md`
- 目标修改：追加一行 `Workflow dispatch smoke passed.`
- 派发方式：`prompt_kind = user_reviewed_instruction`
- 执行目录：`/Users/yoyi`
- 沙箱：`workspace-write`
- 允许写入根目录：`/Users/yoyi/codex-workflow-mario-test`

## 非目标

- 不改 `index.html`、`styles.css`、`game.js`。
- 不改 `/Users/yoyi/gameai/agent world`。
- 不读取完整 transcript。
- 不读取 `auth.json`、`.env`、密钥、token、授权文件。
- 不运行 harness。
- 不联网安装依赖。
- 不做复杂业务自动编排。

## 必须先获得用户明确批准

执行前必须让用户明确同意：

- 执行真实 `codex exec resume`。
- 写 `/Users/yoyi/.codex`。
- 修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
- 写真实 workflow state。

没有明确批准，只能做只读前置检查，不得派发。

## 允许读取

允许读取：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/codex-workflow-mario-test/README.md`
- 真实 workflow state 的必要结构：
  - `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文

## 允许写入

用户明确批准后，允许写：

- `/Users/yoyi/codex-workflow-mario-test/README.md`
- `/Users/yoyi/.codex`，通过 `codex exec resume`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1-result.md`

## 禁止事项

- 禁止未获用户确认就执行 `codex exec resume`。
- 禁止修改允许范围外文件。
- 禁止读取敏感文件和完整 transcript。
- 禁止运行 harness。
- 禁止联网安装依赖。
- 禁止使用 `--dangerously-bypass-approvals-and-sandbox`。
- 禁止把 README 极小验证说成复杂业务自动编排完成。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号文本时必须使用单引号或 `rg -F`。

## 建议用户审核业务指令

```json
{
  "instruction_id": "user-reviewed-instruction:readme-smoke-v1",
  "summary": "向测试项目 README 追加一行 smoke 标记。",
  "objective": "验证桌面壳用户审核业务派发能真实修改允许范围内的文件，并完成回收。",
  "execution_cwd": "/Users/yoyi",
  "sandbox_mode": "workspace-write",
  "allowed_write_roots": [
    "/Users/yoyi/codex-workflow-mario-test"
  ],
  "allowed_reads": [
    "/Users/yoyi/codex-workflow-mario-test/README.md"
  ],
  "allowed_writes": [
    "/Users/yoyi/codex-workflow-mario-test/README.md"
  ],
  "forbidden_actions": [
    "不读取 auth.json、.env、密钥、token、授权文件",
    "不读取完整 transcript",
    "不修改 index.html、styles.css、game.js",
    "不修改 /Users/yoyi/gameai/agent world",
    "不运行 harness",
    "不联网安装依赖"
  ],
  "timeout_seconds": 300,
  "max_retries": 0,
  "required_return": [
    "薄弱点",
    "是否修改 README",
    "是否修改允许范围外文件",
    "是否读取敏感文件",
    "最终文件摘要",
    "自检结果"
  ]
}
```

## 验收标准

必须满足：

- README 追加目标行。
- 不修改允许范围外文件。
- 真实 workflow state 写入 dispatch / control / attempt / audit。
- 不保存完整 transcript。
- 不读取敏感文件。
- 最终回传清楚说明执行结果。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 是否获得用户明确批准。
3. 是否执行真实 `codex exec resume`。
4. 是否写 `/Users/yoyi/.codex`。
5. 是否写真实 workflow state。
6. 是否修改 README。
7. 是否修改允许范围外文件。
8. 是否读取敏感文件或完整 transcript。
9. workflow state 写入字段类型和 audit id。
10. 新增 evidence / handoff。
11. 验证命令和结果。
