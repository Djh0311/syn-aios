# Stage K / K3-B0 Real Workflow Execution Bridge And Harness Handoff v1

日期：2026-06-10

结论：`accepted_with_p2`。

K3-B0 已可接受为 K3-B1 / K3-B2 真实执行前置 bridge / harness 完成；不接受为 K3-B1 / K3-B2 已执行，不接受为 K3-Level-B 完成，不接受为 K3 或 Stage K 完成。

## 本轮完成

- K3-B 专用 bridge / harness 已落在 `project_workflow_automation.rs`。
- B1 / B2 frozen refs 已接入统一 `real_execution_product_command` preview / prepare / decision / Phase A / Phase B。
- K3-B1 / K3-B2 no-op 测试覆盖 frozen refs、permission envelope、task memory packet、readback marker、duplicate guard、hash / manifest boundary 和 unknown readback。
- K3-B1 / K3-B2 ignored + env-gated real execution entry 已存在，默认不运行。
- Tauri command wrapper 已加非空 `runtime_prompt_body` 阻断，防止普通 invoke 直接进入真实 harness。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 主管线验证

- `cargo test --lib project_workflow_automation`：15 passed / 4 ignored。
- `cargo test --lib real_execution_command`：36 passed / 7 ignored。
- `cargo test --lib memory_capture`：7 passed。
- `cargo test --lib runtime_log`：6 passed。
- `cargo test --lib worker_protocol`：8 passed。
- `cargo test --lib`：331 passed / 16 ignored。
- `cargo fmt -- --check`：通过。

保留 warning：`mcp/protocol.rs invalid_params is never used`，为既有 warning。

## 扫描结论

- 未发现新增裸 `Command::new("codex")` 产品路径。
- 普通前端 `App.tsx`、`views`、`components` 未接 K3-B wrapper / button。
- 敏感词 / prompt 命中为既有 guard、deny-list、测试、边界文案和 runtime-only 类型；K3-B0 没有持久化 prompt body。
- readback unknown 状态仍保持 `result_count=null`；candidate / observation 未自动写 FormalMemory。

## 边界确认

K3-B0 产品代码路径没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 启动 Tauri / Browser / Chrome / Vite preview / 截图工具。
- 新增前端执行按钮。
- 自动写 FormalMemory。

过程偏差：

- 主管线收尾扫描误把 Markdown 反引号放进 shell 双引号，触发了 `codex exec` / `codex exec resume` 命令替换。
- 输出显示 `Reading prompt from stdin... No prompt provided via stdin.`，并且访问 `/Users/yoyi/.codex/state_5.sqlite` 时因 readonly database 初始化失败。
- 这不是 K3-B0 产品代码路径，不作为 K3-B1 / K3-B2 执行证据；但本轮不能再严格声称“完全没有执行 Codex 命令 / 完全没有触碰 `.codex`”。

## P2

K3-B2 真实执行前建议加强 allowed write path 的内容 / marker / hash 断言。当前 ignored real entry 会打印 allowed path hash，并断言外部 manifest 不变和 allowed path 存在；但 B2 执行任务包应把 allowed file 的内容证明、hash 和 marker 作为验收项写入 evidence。

## 下一步

下一步应写 K3-B1 真实执行任务包，而不是直接执行：

- 目标：`/Users/yoyi/Documents/mario test`
- session：`019e798a-ac37-7771-b982-e38084fcd22e`
- operation：`resume`
- sandbox：`read-only`
- marker：`K3_B1_MARIO_TEST_WORKFLOW_READ_ONLY_OK_2026_06_10`

K3-B1 会真实写入 `/Users/yoyi/.codex`，必须在任务包中再次冻结授权、prompt hash、项目核心文件 hash、runtime log / audit / readback 路径和停止条件。
