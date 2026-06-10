# 任务包：工作流状态收口真实派发复测 v1

## 任务名

工作流状态收口真实派发复测 v1。

## 所属开发线

桌面应用线 / Codex 会话线 / 工作流状态线。

总指导线回收。

## 当前判断

状态收口代码已修，存量真实 workflow state 旧账也已修，需要做一次新的真实极小派发复测。

依据：

- `handoffs/2026-05-30-workflow-node-state-closure-fix-v1-review.md`
- `handoffs/2026-05-30-workflow-state-real-readme-smoke-node-closure-fix-v1-review.md`
- `evidence/2026-05-30-workflow-state-real-readme-smoke-node-closure-fix-v1.md`

大白话：

代码修好了，旧账也改干净了。现在要再跑一次很小的真实派发，确认新代码不会再把 codex-dev 节点挂在“执行中”。

## 薄弱点

- 这一步会真实执行 `codex exec resume`，会写 `/Users/yoyi/.codex`。
- 这一步会修改测试项目 README。
- 这一步会写真实 workflow state。
- 这仍然只是极小复测，不证明复杂业务自动编排完成。

## 目标

通过用户审核业务派发路径，对测试项目 README 追加第二条复测标记：

- 目标文件：`/Users/yoyi/codex-workflow-mario-test/README.md`
- 目标追加行：`Workflow dispatch state closure retest passed.`
- 派发方式：`prompt_kind = user_reviewed_instruction`
- 执行目录：`/Users/yoyi`
- 沙箱：`workspace-write`
- 允许写入根目录：`/Users/yoyi/codex-workflow-mario-test`

核心验收：

- README 追加目标行。
- work item 进入 `ready_for_review`。
- 实际派发节点 codex-dev 不再是 `running`，应收口为 `ready_for_review`。
- 真实 workflow state 写 dispatch / execution control / execution attempt / audit。

## 非目标

- 不修改 `index.html`、`styles.css`、`game.js`。
- 不修改 `/Users/yoyi/gameai/agent world`。
- 不读取完整 transcript。
- 不读取 `auth.json`、`.env`、密钥、token、授权文件。
- 不运行 harness。
- 不联网安装依赖。
- 不做复杂业务自动编排。

## 前置要求

执行前必须确认：

- `/Users/yoyi/codex-workflow-mario-test/README.md` 存在。
- 目标行 `Workflow dispatch state closure retest passed.` 尚不存在；如果已存在，停止并回传，不重复追加。
- 已有可用 binding thread：`019e7738-5e29-74e0-a22f-5c2481b64c38`。
- thread 在 `codex-index.json` 中，`project_root=/Users/yoyi/codex-workflow-mario-test`，rollout 存在。
- 真实 workflow state 中对应 retest work item 已准备为 `ready_to_dispatch`；如果不存在，需要先停止并回传需要准备 state，不得擅自写。

## 必须先获得用户明确批准

执行真实派发前必须让用户明确同意：

- 执行真实 `codex exec resume`。
- 写 `/Users/yoyi/.codex`。
- 修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
- 写真实 workflow state。

没有明确批准，只能做只读前置检查，不得派发。

## 允许读取

允许读取：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-node-state-closure-fix-v1-review.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-state-real-readme-smoke-node-closure-fix-v1-review.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/codex-workflow-mario-test/README.md`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`
- 真实 workflow state 的必要结构：
  - `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文
- rollout JSONL 正文

## 允许写入

用户明确批准后，允许写：

- `/Users/yoyi/codex-workflow-mario-test/README.md`
- `/Users/yoyi/.codex`，通过 `codex exec resume`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-state-closure-real-dispatch-retest-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v1-result.md`

允许更新：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`

## 禁止事项

- 禁止未获用户确认就执行 `codex exec resume`。
- 禁止修改允许范围外文件。
- 禁止读取敏感文件和完整 transcript。
- 禁止运行 harness。
- 禁止联网安装依赖。
- 禁止使用 `--dangerously-bypass-approvals-and-sandbox`。
- 禁止把复测说成复杂业务自动编排完成。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号文本时必须使用单引号或 `rg -F`。

## 建议用户审核业务指令

```json
{
  "instruction_id": "user-reviewed-instruction:state-closure-retest-v1",
  "summary": "向测试项目 README 追加一行状态收口复测标记。",
  "objective": "验证修复后的工作流状态收口路径在真实派发后不会让 codex-dev 节点残留 running。",
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

## 需要准备的 workflow state

如果执行前发现不存在 retest work item，不要直接派发。

需要另一步准备：

- 新 work item id 建议：`workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest`
- assigned_role_id：`codex-dev`
- current_node_id：director 或符合当前状态机的待派发节点
- state：`ready_to_dispatch`
- active binding 指向 thread `019e7738-5e29-74e0-a22f-5c2481b64c38`

准备 state 本身会写真实 workflow state，也需要单独批准。

## 验收标准

必须满足：

- README 追加目标行。
- `index.html`、`styles.css`、`game.js` hash 不变。
- completed dispatch `exit_code=0`。
- work item state 为 `ready_for_review`。
- work item current node 为 review。
- codex-dev node state 不是 `running`，应为 `ready_for_review`。
- 写入 execution control / execution attempt。
- 写入 audit event。
- 不保存完整 transcript。
- 不读取敏感文件。

## 建议验证

只读复核：

```bash
rg -n -F 'Workflow dispatch state closure retest passed.' /Users/yoyi/codex-workflow-mario-test/README.md
shasum -a 256 /Users/yoyi/codex-workflow-mario-test/README.md /Users/yoyi/codex-workflow-mario-test/index.html /Users/yoyi/codex-workflow-mario-test/styles.css /Users/yoyi/codex-workflow-mario-test/game.js
/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json
rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md
```

workflow state 摘要复核：

- retest work item state。
- retest dispatch state / exit code。
- codex-dev node state。
- execution control / attempt。
- audit event。

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
9. completed 后 codex-dev node 是否仍为 `running`。
10. workflow state 写入字段类型和 audit id。
11. 新增 evidence / handoff。
12. 验证命令和结果。

## 总指导回收重点

总指导回收时必须判断：

- 新代码路径是否真实避免 codex-dev 残留 `running`。
- 是否只修改了允许范围内 README。
- 是否没有读取敏感文件或完整 transcript。
- 是否仍然不能把本轮说成复杂业务自动编排完成。
