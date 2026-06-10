# Stage J / J2-B B2 Execution Bridge Level A Evidence v1

日期：2026-06-09

状态：B2 execution bridge Level A 已完成并通过主管线 fresh verify；未执行真实 Codex；未启动 B2 env-gated 真实探针。

## 本轮目标

在 B1 read-only 真实 `resume` 探针已收口后，补齐 B2 隔离项目 workspace-write 探针的产品桥：

- 使用 J2 run unit 作为源头。
- 使用 `codex_control` source。
- 进入统一 `real_execution_product_command` family。
- 支持 `new_session` Phase B 路径。
- 默认测试只用 fake runner，不发送真实 prompt，不读写 `/Users/yoyi/.codex`。
- 真实执行只保留为 ignored / env-gated harness，等待主管线二次启动。

## 本轮完成

- 新增 / 确认 `ProjectWorkflowAutomationJ2BB2Input` / `ProjectWorkflowAutomationJ2BB2Output`。
- 新增 / 确认 Tauri command `run_project_workflow_automation_j2_b_b2`。
- 新增 J2-B B2 bridge：`run_project_workflow_automation_j2_b_b2_at` / `run_project_workflow_automation_j2_b_b2_with_runner`。
- B2 bridge 串联：
  - `preview_real_execution_product_command_at(source_kind="codex_control")`
  - `prepare_real_execution_product_command_at`
  - `record_real_execution_product_command_decision_at(confirmed_by="user", allowed_once=true)`
  - `run_real_execution_product_command_phase_a_at`
  - `run_real_execution_product_command_new_session_phase_b_with_runner`
- B2 bridge 固定隔离项目、workflow、node、`codex-local`、`new_session`、`workspace-write`、prompt summary/ref/hash、allowed write path。
- B2 bridge 将 `allowed_write_roots` 收窄为 `.workbench/stage-j/j2-b`，而不是整个隔离项目根。
- B2 authorization 使用 `H3RealNewSessionAuthorizationMatrix`，并限制为 Stage J / J2-B 隔离项目 workspace-write probe。
- B2 workflow audit event 记录的 `allowed_write_roots` 同步使用窄写根，避免 evidence / audit 口径分裂。
- 默认 fake-runner 测试验证 Product Command attempt、continuation attempt、runtime log、audit/readback refs、run unit refs 可追溯。
- 新增 ignored / env-gated B2 real harness：默认 `cargo test --lib` 不触发真实 Codex。
- B2 real harness 在执行前只预创建窄写根目录，避免 `--add-dir` 指向不存在路径；不会预创建 allowed write file。
- B2 real harness 新增全项目文件 manifest before / after 对比；除 allowed write path 外，不允许新增或修改任何项目文件。

## 关键代码证据

- B2 product bridge 入口和 runner 注入：`prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs:658`
- B2 preview / prepare / user decision / Phase A / new-session Phase B 串联：`prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs:753`
- B2 `codex_control` request 固定为 `operation_id="new_session"`、`sandbox=workspace-write`、`allowed_write_roots=[.workbench/stage-j/j2-b]`：`prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs:1447`
- B2 new-session authorization matrix：`prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs:1510`
- B2 workflow audit event 写入窄 `allowed_write_roots`：`prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs:1928`
- B2 fake-runner 成功测试：`prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs:2595`
- wrong prompt hash / non-user confirmation 写入前阻断测试：`prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs:2684`
- B2 ignored / env-gated real harness：`prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs:3004`
- B2 real harness 全项目 manifest 后验：`prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs:2299`
- B2 real harness 预创建窄写根目录：`prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs:3047`
- B2 Tauri command：`prototypes/productized-desktop-shell/src-tauri/src/commands.rs:240`
- B2 command 已挂入 Tauri invoke handler：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs:13682`

## Fresh Verify

- `cargo test --lib project_workflow_automation`：11 passed / 2 ignored。
- `cargo test --lib real_execution_command`：33 passed / 3 ignored。
- `cargo test --lib session_continuation`：17 passed / 4 ignored。
- `cargo fmt -- --check`：通过。
- `cargo test --lib`：313 passed / 10 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：13 passed。
- `npm run build`：通过；保留既有 Vite chunk size warning。

说明：Rust 测试仍有既有 `JsonRpcError::invalid_params` dead code warning；本轮未处理，和 B2 bridge 无关。

## 扫描 / 边界分类

- `j2_b_b2_real_isolated_project_workflow_new_session_probe_requires_env_authorization` 标记为 ignored，且要求 `J2_B_B2_REAL_EXECUTION_AUTHORIZED=1`、项目 root、allowed write path、marker、run parent env 全部匹配后才会执行。
- `run_project_workflow_automation_j2_b_b2` 只在 Rust command/type/handler 中出现；普通前端未接按钮或 TS wrapper。
- 默认测试使用 fake runner；`prompt_sent=true` / `real_codex_executed=true` 仅表示 fake Phase B output 模拟真实执行结果，用于验证 sidecar / refs / gate，不代表本轮真实 Codex 已执行。
- Prompt body 只作为 Phase B runtime input；fake-runner 测试断言 product command sidecar 不包含 canonical prompt body。
- B2 allowed write root 当前为隔离项目内 `.workbench/stage-j/j2-b` 目录；真正成功条件仍要求真实执行后 baseline `README.md` / `project-notes.md` hash 不变，且全项目文件 manifest 除 allowed write path 外完全一致。

## 边界确认

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/.env/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Browser/Chrome/Tauri/Vite dev/screenshot。
- 未同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。
- 真实 B2 env-gated harness 未运行。

## 不能声明

- 不能声明 B2 workspace-write 真实探针完成。
- 不能声明隔离项目 allowed write path 已由真实 Codex 写入。
- 不能声明 baseline hash 已完成真实 before / after 验收。
- 不能声明 worker report candidate / C5 / process fact observation 已真实回收。
- 不能声明 J3 memory capture bus 已完成。
- 不能声明 J2-B 整体完成或 Stage J 完成。

## 下一步

1. 将本 evidence / handoff 交给长期只读复核线审查 B2 Level A 实现。
2. 若无 P0/P1，由主管线决定是否启动 B2 env-gated 真实探针。
3. B2 若执行成功，必须新增单独真实执行 evidence / handoff，并复核 baseline hash、allowed write path、readback marker、prompt body 未持久化和 `.codex` 边界。
4. B2 收口后进入 J3 记忆捕获总线，不跳过 J3。
