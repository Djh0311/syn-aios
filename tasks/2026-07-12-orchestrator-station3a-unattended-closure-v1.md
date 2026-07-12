# 实现任务包：站 3a 无人值守收口与最终发射前检查 v1（可派）

日期：2026-07-12  
状态：`HISTORICAL__SUPERSEDED`。本包只覆盖控制核心桥之前的发射前检查，现已被 `tasks/2026-07-12-orchestrator-station3a-supervisor-action-control-core-bridge-v1.md` 与 v7 完成证据取代；其中 `approval_policy = "never"`、旧 42 字节 proof 和 `READY_FOR_UNLOCKED_UI` 都不是当前实现或当前状态。  
上承：`docs/plans/2026-07-11-orchestrator-fast-path-five-stations-plan-v1.md` 站 3a、`docs/plans/2026-07-11-supervisor-contract-v1-draft.md`、`tasks/2026-07-11-orchestrator-station2-contract-switch-pilot-v1.md`。  
执行方式：总指导负责边界与最终复核；执行线只按本包工作，不 commit。用户希望使用 `5.6-terra-极高`，但当前子任务接口不暴露模型选择参数；不得伪称已设置该模型，按最高审查强度执行。

## 0. 当前现场

- 固定测试项目：`/Users/yoyi/codex-workflow-mario-test`。真实项目一律不碰。
- 证明文件：`station3a-supervisor-write-proof.txt`，目标精确字节为 `station3a supervisor worker write verified`，42 字节、无末尾换行。
- 已完成过一次真实 worker 写入，但旧账本因授权编号没有继承而把成功结果记成失败；此后真实 UI 发射又依次暴露了长 ID 截断碰撞、模型抄错 work item ID、Codex `auto_review` 把工作台正本授权误判为不可信材料。
- 当前主树有大量用户既有未提交修改；必须保留并只碰本包列出的 3a 文件。最新 debug `.app` 已构建，但最终真实发射因 Mac 锁屏暂停。

## 1. 无人值守期间可以做什么

1. 只读审查当前 3a 增量，重点核对授权、写根、prepared dispatch 和临时 `CODEX_HOME` 边界。
2. 对发现的确定性缺陷做最小修复，并补对应回归测试；若只是偏好或未来 3b 设计问题，只记录，不扩写。
3. 跑自动化验证、检查测试项目证明文件与 git 基线、构建最新 debug `.app`。
4. 写一份简短 evidence，明确“自动检查通过到哪一步”和“仍需解锁后的真实 UI 发射”。

## 2. 无人值守期间禁止做什么

- 不操作任何非测试真实项目，不开始站 3b。
- 不新增或修改 `/Users/yoyi/codex-workflow-mario-test` 中任何文件；只能读取证明文件和 `git status`。
- 不用 shell、直接 MCP 或手改 sidecar 冒充最终主管发射；真实发射只能从工作台 UI 的“允许并开始”进入。
- 不写用户全局 `~/.codex`，不读取或复制凭据；临时 `auth.json` 仍只能是符号链接。
- 不再放宽 path-lock、沙箱、任务包、授权段或 `allowed_write` 强闸。
- 不碰研究文档、视觉原型、`.claude/`、`.playwright-cli/` 和其它用户未提交文件。
- 不更新 `CURRENT.md`，不 `git add`，不 commit，不 push。

## 3. 必查实现点

### 3.1 fail-closed 写根

- `h5_project_dispatch_bridge.rs` 对缺失或空 `allowed_write` 必须拒绝；不得回退到 `project_root`。
- 主管仍以 `--sandbox read-only` 启动；worker 仅能在任务包精确写根内使用 `workspace-write`。

### 3.2 授权在副作用前成立

- `dispatch_worker` 启动 worker 前必须同时匹配 project/workflow/node/work item/authorization/allowed_write。
- 执行记录必须继承同一 `plan_authorization_id` 和 authorized check；不能先跑完再发现账本不一致。

### 3.3 ID 唯一性与正本恢复

- 站 3a planned task ID 必须基于完整 authorization hash，不能因 `stable_id` 截断导致不同方案碰撞。
- 模型抄错长 work item ID 时，只允许从同一项目、工作流、节点、授权段的**唯一** `authorized_prepared_auto_dispatch` 恢复正本；0 个或多个候选必须拒绝。
- 恢复后 worker 账本必须记录正本 work item ID，并留下 canonicalized warning；不得把任意错误 ID 映射到别的任务。

### 3.4 临时主管审批边界

- 临时 `CODEX_HOME/config.toml` 仅含 `approval_policy = "never"` 和 `supervisor_orchestrator` MCP；不得含其它 MCP 或用户配置。
- `never` 只消除已经由工作台确认后的重复 Codex 外层复审；工作台自己的 active authorization、prepared dispatch、任务包、精确写根、固定测试项目 path-lock 必须全部保留。
- 此配置只接受为站 3a 固定测试项目试点；不得据此解锁站 3b 或真实项目。

## 4. 允许修改的文件

- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_session_launcher.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `docs/plans/2026-07-11-supervisor-contract-v1-draft.md`
- 本包对应的 `evidence/2026-07-12-orchestrator-station3a-unattended-closure-v1.md`

确需修改列表外文件时，停止并向总指导报告，不自行扩大范围。

## 5. 验证顺序

先定点，后全量；不得用自报代替命令结果。

1. `cargo test --lib mcp::supervisor_orchestrator::tests --quiet`
2. `cargo test --lib station3a_ --quiet`
3. `cargo test --lib s3_director_dispatch_integration_stub --quiet`
4. `cargo test --lib --quiet`
5. `npm run typecheck`
6. `npm run test:offline-interaction`
7. `cargo check --offline`
8. `cargo fmt --check`：只允许既有 `codex_db.rs`、`codex_local_runner.rs`、`mcp/storage.rs` 漂移；新增文件不得漂移。
9. `git diff --check`
10. 测试项目只读核对：证明文件 42 字节、内容精确、无末尾换行；`git status --short` 相对既有基线只多该证明文件。
11. 构建：`../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug --bundles app`。

若全量验证耗时或遇既有失败，保留原始失败文本并区分“本包回归”与“仓库基线”。

## 6. 回交格式

回交必须包含：

- 实际修改文件；若零修改，明确写零修改。
- 发现的问题按 P0/P1/P2 列出，附文件与行号；没有问题就写“未发现确定性缺陷”。
- 每条验证命令的真实结果。
- 证明文件的字节证据与测试项目 `git status`。
- 最新 debug `.app` 路径与构建结果。
- 最终状态只能是：`READY_FOR_UNLOCKED_UI` 或 `BLOCKED`。

## 7. 不接受为完成

- 只跑单测、不审授权路径。
- 把已有证明文件当成本次主管闭环已通过。
- 把 `approval_policy = "never"` 说成全局关闭审批。
- 没有真实账本中的 worker + read_worker_report + `final_mark: pass` 就宣布 3a 完成。
- 触碰真实项目、提交代码或清理用户既有脏文件。
