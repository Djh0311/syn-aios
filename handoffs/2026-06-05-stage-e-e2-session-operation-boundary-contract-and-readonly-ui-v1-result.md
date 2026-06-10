# Handoff：Stage E / E2 Session Operation Boundary Contract And Readonly UI v1

日期：2026-06-05

## 本轮完成

E2 已完成：工作台现在有统一的会话操作边界读模型，可在 `WorkbenchSnapshot.session_operations[]` 中表达：

- `send_message`
- `stop`
- `restart`
- `resume`
- `export`
- `delete`
- `favorite`

每个操作按 adapter 派生。`codex-local` 有七类边界，planned adapters 也有同名边界但全部保持不可执行 / 计划中 / 破坏性阻断。智能体页只在既有“智能体”入口内显示只读“会话操作边界”面板；未新增入口、tab、右侧顶级入口、输入框或操作按钮。

秘书只读模型新增会话操作边界风险和查看建议，但不会生成发送、停止、重启、resume、导出、删除或收藏 action proposal。

## 接受范围

接受为：

- 阶段 E / E2 会话操作边界契约完成。
- 七类会话操作的权限、审计、数据写入、UI 和后续真实执行条件已逐项定义。
- 智能体页可以安全解释这些操作当前不可执行或 planned。
- planned adapters 继续不可执行。

不接受为：

- 会话中心真实发消息完成。
- 通用 `codex exec resume` / stop / restart 完成。
- 会话导出、删除、收藏完成。
- Claude Code / OpenClaw / OpenCode 真实接入完成。
- 外部模型或凭据管理完成。
- 运行日志、自动重试、取消恢复或运维诊断完成。
- 真实 worker / Codex 执行完成。
- 阶段 G 真实 Tauri 全面验收完成。

## 验证结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`：`offline interaction tests passed: 10`
- `npm run build`：通过，保留既有 Vite chunk size warning
- `cargo test --lib session_operation`：1 passed
- `cargo test --lib adapter_descriptor`：2 passed
- `cargo test --lib agent_adapter`：2 passed
- `cargo test --lib`：222 passed，1 ignored
- `rustfmt --check src/types.rs src/lib.rs src/commands.rs`

禁止文案扫描有 1 个既有命中：

- `prototypes/productized-desktop-shell/src/views/CanvasView.tsx:243` 的“已停止实验运行。”

该命中属于独立实验画布运行提示，不是 E2 新增，不表示会话中心 stop 已实现；本轮未改画布运行逻辑。

## 手动测试清单

在应用里测试：

1. 打开桌面壳，进入“智能体”页面。
2. 确认页面仍是会话中心，不出现新的一级入口、右侧顶级入口或项目页 tab。
3. 在“适配器能力”附近找到“会话操作边界”面板。
4. 确认 Codex 下显示发消息、停止、重启、resume、导出、删除、收藏七项。
5. 确认发消息和 resume 显示“需要后续任务”，停止 / 重启显示“当前不可执行”，导出 / 收藏显示“计划中”，删除显示“破坏性阻断”。
6. 确认 Claude Code、OpenClaw、OpenCode、OpenCode-like 仍显示 planned / 不可执行状态。
7. 确认面板里没有消息输入框，也没有发消息、停止、重启、resume、导出、删除或收藏按钮。
8. 点击现有“重新读取”和“定位 rollout”辅助动作时，确认它们仍是原来的只读 / 定位动作，不被解释成七类会话操作。
9. 打开右侧“秘书只读摘要”，确认能看到会话操作边界提醒；确认秘书没有给出发送、停止、删除等执行提案。

真实窗口 / 截图验收未完成；上述清单需要后续真实 Tauri 验收切片补证据。

## 当前权威入口

- `CURRENT.md`
- `AUTHORITY.md`
- `README.md`
- `STAGE_PLAN.md`
- `tasks/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- 本任务包：`tasks/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`
- Evidence：`evidence/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`

## 后续建议

建议后续 E3 二选一：

- E3A：真实发送 / resume 方案设计，只写方案、权限、审计、readback、失败恢复和安全边界，不直接执行。
- E3B：模型 / 凭据只读状态深化，定义设置 / 管理入口、安全摘要、不可见 secret 边界和 provider 不可用状态。

下一步仍不能直接接 Claude Code / OpenClaw / OpenCode，也不能把 planned descriptor 或 E2 operation boundary 改成可执行能力。

## 边界声明

本轮没有读写 `/Users/yoyi/.codex`，没有读取 auth/token/`.env`/keychain/OAuth/provider credential/完整 transcript，没有执行 `codex exec` 或 `codex exec resume`，没有调用外部 agent 或模型 provider，没有新增 store 或迁移数据库，没有修改 workflow state JSON，没有启动真实 worker / workflow machine / MCP canvas run。
