# 站 3b 首发(attempt-1):被 S1 执行层合一闸安全拦下 · 复盘

日期:2026-07-12 · 结论:**安全面全对,授权判定漏认主管路径——已修**(commit 前状态)

## 账本时序(用户真机回贴原文)

```
supervisor:workflow-users-yoyi-documents-mario-test-default:1783869038503625000
supervisor_temporary_codex_home：主管临时 CODEX_HOME 已创建；auth.json 仅为到 ~/.codex/auth.json 的符号链接。
supervisor_session_launcher：主管会话已启动；后续工具调用会落入同一账本。
supervisor_temporary_codex_home：主管临时 CODEX_HOME 已清理。
fresh_task_session_bound：已通过 C1 建会话并精确绑定本 work item；历史 native_thread_id 不会复用。
fresh_task_session_abandoned：worker 派发失败：real_execution_gate_blocked:blocked_waiting_authorization:permission envelope or authorization matrix is incomplete（guard_reasons: audit_ref_missing,authorization_scope_missing,user_confirmation_required）
control_core_dispatch_worker：denied: real_execution_gate_blocked:...(同上)
supervisor_temporary_codex_home：主管临时 CODEX_HOME 已创建/已清理（第二段）
supervisor_worker_return / supervisor_session_launcher：主管请求用户决定（waiting_user）
```

## 判读

1. **安全面逐项正确**:临时 CODEX_HOME 建/清成对;全新任务会话绑定(C1);派发失败后 `fresh_task_session_abandoned` 即时清理绑定;主管不硬闯、停 waiting_user 请用户决定。失败路径像 v3-v6 一样是**证据**,不是事故。
2. **根因**(commands.rs S1 执行层合一闸):`authorization_complete: path_lock_hit`——只认固定测试项目 path-lock,主管授权的 3b 只读派发不可见 → `blocked_waiting_authorization`。
3. guard_reasons 三条(audit_ref/authorization_scope/user_confirmation)是**搭车诊断文案**:B 线安全子集 guard 本就把这三条排除在 guard_blocked 之外(option A),实际 block 源于 authorization_complete=false。
4. **意外收获(实证)**:guard 安全子集对「read-only 沙箱 + 空写根」的请求**放行**(观察到的 reasons 仅授权三条)——3b 只读信封的执行安全检查首跑即过。

## 修复(attempt-2 前落地)

- 新增 `real_execution_authorization_complete(project_root, write_roots, supervisor_authorized)`:
  `测试项目 path-lock ∨ (主管授权派发 ∧ 3b 项目 ∧ 零写根)`;判决体 `decide_real_execution_command` 一字不动。
- 经典线喂 3b(supervisor_authorized=false)仍拒;其它项目主管授权也拒;案发测试
  `station3b_real_execution_authorization_complete_scoping` 六断言钉死刻度。
- 验证:cargo test --lib **871/0/43**;fmt 仅剩 3 历史漂移。

## 残留物(惰性,无需清理)

- attempt-1 物化的 1 条 `authorized-prepared-dispatch:planned-task-supervisor-pilot-5…`(state=prepared)
  挂在已耗授权段下;派发唯一性过滤按 plan_authorization_id 圈定,新一轮全新授权不受影响。
- 同工作流另有 45 条 B2 时代 `prepared` 旧账(director/codex-dev/validation/review 四角色),
  `has_inflight_dispatch` 只数 `running`(现 0 条),不挡道。
- mario test 项目目录零变化(基线=pre-launch-baseline.txt,7 文件 SHA-256;attempt-1 根本没起 worker)。
