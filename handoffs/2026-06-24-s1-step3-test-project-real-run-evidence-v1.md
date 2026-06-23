# 回交：S1-③ 测试项目真跑端到端「合并闸 runtime 真生效」· 执行线 → 主导线 v1

日期：2026-06-24　性质：**真跑 codex 进固定测试项目 = 高危#1 轻档** + S1（高危#3）runtime 端到端验证　任务包：`tasks/2026-06-24-s1-step3-test-project-real-run-verification-v1.md`　上游：S1 实现 commit `d0e1e03`

## 0. 一句话结论

**4 条 runtime 证据全成立**：① 真 codex 经 `execute_project_workflow_node_at`（S1 合并闸所在层）真跑、completed/exit0；② 非测试 root 被闸在起 runner 前运行时拦截、零 codex；③ codex 只写测试目录、无外溢；④ dispatch completed/exit0。**执行线未 commit。**

## 1. 关键前情：任务包假设落空 → 你已拍板 option A

任务包 §1 要"经 `execute_project_workflow_node` 产品路径真跑、证明真走 A 强闸"。但核实物发现：**现成 `#[ignore]` 真跑测试全部绕过 S1 闸**——`real_run_full_dispatch_resume`(lib.rs) 直调 `execute_workflow_node_dispatch_for_index_at`（闸之后那层），`real_run_workflow_node_adapter` 更低（适配器层）；而过 `execute_project_workflow_node_at`（S1 闸所在）的测试全是 stub runner。**S1 闸从没被真 codex 验过**，且 §3「不起链/不改码」+ §4「测试数不变」凑不齐一条现成路。

回报主导线 → **你拍板 option A**：加只读·验证用 `#[ignore]` 测试，让真 codex 强制走 `execute_project_workflow_node_at`（过闸）。

## 2. 做了什么（仅加验证测试，S1 闸/沙箱/实现一字不碰）

lib.rs 新增 **2 个 `#[ignore]` 验证测试**（净增 +173 行、纯新增）：

- **`s1_step3_real_run_through_gate`**（证据①③④）：bootstrap 工作流 → 建任务包（objective 指示 codex 写 `s1-step3-proof.txt` + 本次 token）→ ready → 绑真会话 `019ed9f7-…`（仍在、可 resume）→ 真 `RealWorkflowNodeCodexRunner` → 调 **`execute_project_workflow_node_at`（过 S1 闸）**。闸不放行会在起 runner 前 Err，故 completed 即证明过了 `decide_real_execution_command`（authorized）。
- **`s1_step3_nontest_root_blocked_before_runner`**（证据②）：project_root 换非测试路径走同函数，runner 用 **panic-stub**（被调即 panic = 即便闸有 bug 也绝不真起 codex 进非测试）→ 断言返回 `real_execution_gate_blocked`、panic-stub 零触发。

两测试均 `#[ignore]`：`cargo test --lib` 的 **passed 仍 580 不变**（ignored 25→27），手动 `-- --ignored` 才跑。指令链已核：`TaskDraftRequest.objective` → artifact.brief → fields.goals → objective → `prompt_preview`（commands.rs:2243）→ codex 真 prompt。

## 3. 四条 runtime 证据

### ① 真走 A 强闸 + path-lock 命中才放行
`cargo test --lib s1_step3_real_run_through_gate -- --ignored --nocapture`（真跑 123s）：
```
[S1_STEP3] state=completed exit=Some(0) summary=Some("已在当前项目根目录创建 `s1-step3-proof.txt`，内容为 `S1-step3 gate real-run ok 1782239044226`。")
[S1_STEP3] proof_path=/Users/yoyi/codex-workflow-mario-test/s1-step3-proof.txt content="S1-step3 gate real-run ok 1782239044226\n"
```
经 `execute_project_workflow_node_at`（含 S1 闸）走通 = 真 codex 过了 `decide_real_execution_command`（authorized·path-lock 命中）；proof 含本次 token = 确是本次产物。

### ② 非测试 root 运行时被拦、零 codex
`cargo test --lib s1_step3_nontest_root_blocked_before_runner -- --ignored --nocapture`：
```
[S1_STEP3_NONTEST] blocked_error=real_execution_gate_blocked:blocked_waiting_authorization:permission envelope or authorization matrix is incomplete（guard_reasons: audit_ref_missing,authorization_scope_missing,user_confirmation_required）
test ... ok
```
拦截由 **path-lock miss → authorization_complete=false** 驱动（铁律）；guard_reasons 里那 3 个授权 reason 正是 option A 排除的（不计 guard_blocked）。**panic-stub 零触发 = runner 零调用 = 没起 codex**；非测试路径未被创建。

### ③ 沙箱只动测试目录（决定性：codex 自身改动清单）
本次 resume 的 codex 会话日志，其全部 `patch_apply_end` 改动路径：
```
/Users/yoyi/codex-workflow-mario-test/s1-step3-proof.txt        （本次）
/Users/yoyi/codex-workflow-mario-test/workflow-fulldispatch-proof.txt （上次·会话历史）
```
**codex 全部文件写入都在测试项目内、测试项目外零写。** 旁证：`$HOME` 顶层 / `~/Documents` / `~/Desktop`(codex 可归因) / `/tmp`(非测试 fixture) 本次无 codex 新文件；`~/.codex` 只动 codex 自身状态库（state/goals/logs/process_manager），**auth.json 时间 Jun 3、凭据未碰**。

> **透明记录**：扫描发现 `~/Desktop/kt-erp/.git/objects/` 478 个对象比 marker 新（02:24:43）。已查证**与 codex 无关**：kt-erp 工作树/源码零改、最新 commit 仍 16:08（无新提交）、只动 `.git` 内部对象 = 外部进程（IDE/git）的环境活动；codex 沙箱 `workspace-write` 锁死测试项目根本写不到它，且 codex 会话改动清单铁证没碰它。非 S1/沙箱问题，未追该外部线。

### ④ completed / exit 0
`result.dispatch.state == "completed"`、`exit_code == Some(0)`（见①输出）。

### 回归
`cargo test --lib` = **580 passed / 0 failed / 27 ignored**（passed 不变、两新测试 #[ignore]）；`git diff --check` 干净；lib.rs +173 纯新增。

## 4. 待办 / flags（主导线决断）

1. **两测试是 `#[ignore]`** — 常规 CI 不跑（守 §4「580 passed 不变」）。建议：**②`s1_step3_nontest_root_blocked_before_runner` 安全·快·是铁律的产品路径回归**，可考虑去掉 `#[ignore]` 作常驻保护（会让 passed 580→581，你定）；①含真 codex 必须留 `#[ignore]`。
2. **proof 文件留在测试项目** — `/Users/yoyi/codex-workflow-mario-test/s1-step3-proof.txt`（连同此前 chain/fulldispatch proof）；测试项目 git 可 `git clean` 清。
3. **加了测试代码** — 本是 §3「不改任何代码」死线，经你 option A 明确放宽给"验证测试"；S1 闸/沙箱/`command_plan_for`/判决体一字未动（git diff 只 lib.rs）。

## 5. 主导线审证据 + 收口指引

- **核证据**：重跑 `cargo test --lib s1_step3_real_run_through_gate -- --ignored --nocapture`（应 completed/exit0 + proof 含 token）；`s1_step3_nontest_root_blocked_before_runner -- --ignored`（应 blocked、不 panic）；`cargo test --lib`（580/27）。
- **核 diff**：`git diff` 只 lib.rs（2 个 `#[ignore]` 测试）；S1 闸/沙箱/实现 0-diff。
- **收口**：执行线不 commit；②③ 全过 → 你回写 `CURRENT.md`「S1 整体完成（实现审过 d0e1e03 + 测试项目真跑验过）」+ commit（带验证测试）→ 接 S2。
