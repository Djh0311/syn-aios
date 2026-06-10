# Stage K / K3-B1 Mario Test Workflow Read-Only Real Resume Run Handoff v1

日期：2026-06-10

结论：K3-B1 已执行但失败分类，状态为 `failed_classified_codex_state_readonly`。

## 结果

K3-B1 没有通过验收。产品路径确实走到了真实 Phase B runner：

- `runner_call_allowed=true`
- `prompt_sent=true`
- `real_codex_executed=true`
- `writes_codex_home=true`
- `writes_project_files=false`

但 Codex 原生状态库在当前执行环境中不可写：

```text
/Users/yoyi/.codex/state_5.sqlite
attempt to write a readonly database
```

因此 runner exit code 为 `1`，readback 为 `readback_failed`，`result_count=null`，last-message 文件未生成，`cargo test` 最终 exit code 为 `101`。

## 运行目录

```text
/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/runs/k3-b1-run-1781062927271794000
```

关键文件：

- `workflow-state.v0.json`
- `real-execution-product-commands.v1.json`
- `session-continuations.v1.json`
- `runtime-logs.v1.json`

缺失文件：

```text
k3-b1-mario-test-workflow-read-only-last-message-2026-06-10t03-00-03z.json
```

## 项目文件

`/Users/yoyi/Documents/mario test` 四个核心文件 hash 前后一致：

```text
f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf  index.html
6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f  styles.css
814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd  game.js
02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5  README.md
```

## 非沙箱重跑

主管线按规则申请非沙箱重跑同一条 exact ignored test，但安全审查拒绝，理由是该动作会向真实 Codex 路径发送 prompt / 项目派生数据并写 `~/.codex`。主管线没有绕过该拒绝。

## 不可声称

- 不可声称 K3-B1 已完成。
- 不可声称 readback marker 命中。
- 不可声称 K3-Level-B 完成。
- 不可声称 K3-B2 可以开始。
- 不可声称 Stage K 完成。

## 下一步

下一步必须写并执行 K3-B1.1 环境 / 权限 / retry gate，而不是直接重跑或进入 K3-B2。
