# Stage J / J2-B B1 Execution Bridge Evidence v1

日期：2026-06-09

状态：B1 execution bridge 已完成并通过主管线 fresh verify；未执行真实 Codex。

## 本轮完成

- 新增 `ProjectWorkflowAutomationJ2BB1Input` / `ProjectWorkflowAutomationJ2BB1Output`。
- 新增 Tauri command `run_project_workflow_automation_j2_b_b1`。
- 新增 J2-B B1 bridge：校验冻结字段后串联 `preview_real_execution_product_command_at(source_kind="codex_control") -> prepare_real_execution_product_command_at -> record_real_execution_product_command_decision_at(confirmed_by="user", allowed_once=true) -> run_real_execution_product_command_phase_a_at -> run_real_execution_product_command_phase_b_with_runner`。
- B1 bridge 固定 `mario test` / 指定 workflow / 指定 codex-dev node / 指定 session / `resume` / `read-only` / B1 prompt summary/ref/hash。
- B1 bridge 使用 B1 canonical prompt body 作为 Phase B runtime input；默认测试确认 prompt body 不进入 product command sidecar。
- read-only sandbox 允许 `allowed_write_roots=[]`；非 read-only sandbox 仍要求显式 allowed write root。
- 主管线补了一个小修：`ProjectWorkflowAutomationJ2BB1Output.audit_refs` 顶层也包含新追加的 J2-B B1 workflow audit event id。

## 关键文件

- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`

## Fresh Verify

- `cargo test --lib project_workflow_automation`：8 passed。
- `cargo test --lib real_execution_command`：33 passed / 3 ignored。
- `cargo test --lib session_continuation`：17 passed / 4 ignored。
- `cargo test --lib codex_local_runner`：11 passed。
- `cargo fmt -- --check`：通过。
- `cargo test --lib`：310 passed / 8 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：13 passed。
- `npm run build`：通过；保留既有 Vite chunk size warning。

## 扫描分类

- `J2-B B1 已执行`、`真实 Codex 自动多角色闭环完成`、`Stage J 完成` 等命中均为禁止项或历史文档边界说明，不是新增完成态声明。
- `run_project_workflow_automation_j2_b_b1` 命中集中在 command / type / bridge / tests。
- `Command::new("codex")` 命中为既有 runner；B1 默认测试使用 fake runner，不触发真实进程。
- `/Users/yoyi/.codex` 命中为 denied paths、历史 runner/fixture/边界文案；本轮未读写该路径。
- `J2_B_B1_CANONICAL_PROMPT` 只作为 Phase B runtime input 和测试断言出现；默认测试确认 product command sidecar 不包含 canonical prompt body。

## B1 Before Baseline

主管线只读计算了 `/Users/yoyi/Documents/mario test` 核心文件 hash，用于后续真实 B1 before/after 对比：

- `index.html`：`f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf`
- `styles.css`：`6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f`
- `game.js`：`814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd`
- `README.md`：`02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5`

## 边界确认

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/.env/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Browser/Chrome/Tauri/Vite dev/screenshot。
- 未同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。

## 不能声明

- 不能声明 B1 真实执行完成。
- 不能声明真实 prompt 已发送。
- 不能声明 B1 readback marker 已读回。
- 不能声明 worker report candidate / C5 / observation 真实回收完成。
- 不能声明 J2-B、J3 或 Stage J 完成。
