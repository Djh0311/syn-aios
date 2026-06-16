# Stage L / L1 K3-B1 Blocked Recovery Product Path Handoff v1

日期：2026-06-16

状态：实现与本地验证已完成；独立复核线 Aquinas 复审 `STATUS: CLEAR_WITH_P2`。提交前停止，未 `git add` / `git commit`。

## 当前状态

K3-B1 已在产品层显示为 `blocked_by_safety_review_again`。项目工作流右侧面板新增恢复卡片，普通用户可看到：

- 为什么挡：真实 Codex resume 会向外部服务发送项目/session 派生 prompt，并写入 Codex 本地状态。
- 能做什么：查看冻结 exact command 后手动运行并回交、重新授权申请、等待更窄本地执行桥。
- 不能做什么：不能自动重试、不能进入 K3-B2、不能把手动回交自动当成功。

## K3-B1 / K3-B2 结论

- K3-B1：仍 blocked；L1 没有启动 retry。
- K3-B2：仍阻断；只有后续主管线明确接受 `manual_recovery_accepted` 或等价 accepted 状态，才可准备 L2。
- 手动回交：只进入 `manual_recovery_needs_review`，不自动成功。
- 重新授权：只进入 `pending_renewed_risk_approval`，不继承旧授权、不执行。
- 更窄本地桥：仅为后续设计候选，不在 L1 实现可执行桥。

## 用户可选恢复路径

1. 用户手动 exact command 回交：UI 已展示 K3-B1 冻结 env/test 命令、执行目录、prompt ref/hash 和手动执行边界；用户回交 stdout / stderr / exit code / run dir / last message / sidecar refs / runtime log refs / audit refs / readback status / result_count / hashes / user statement，由主管线复核。
2. 用户重新明确批准风险：另开独立执行任务包和安全审查，L1 不直接执行。
3. 更窄本地执行桥设计：另开设计任务包，不能绕过安全审查。

## 已验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 15`。
- `npm run build`：通过，保留既有 Vite chunk size warning。
- `cargo fmt -- --check`：通过。
- `cargo test --lib k3_b1_recovery`：5 passed。
- `cargo test --lib workflow_audit`：3 passed。
- `cargo test --lib page_read_model`：7 passed。
- `cargo test --lib runtime_log_store`：1 passed。
- `cargo test --lib memory_capture_bus`：7 passed。
- `cargo test --lib real_execution_command`：36 passed / 7 ignored。
- `cargo test --lib project_workflow_automation`：15 passed / 4 ignored。
- `cargo test --lib`：508 passed / 21 ignored。
- `node scripts/harness/workbench-shape-gate.js --mode check`：0 errors / 0 warnings。
- `git diff --check`：通过。
- `node scripts/harness/capability-scan.js --target .`：PASS 7 / WARN 10 / FAIL 0。
- `node scripts/harness/guard-state-files.js --target .`：PASS 19，envFiles 为空。

## 边界

本包没有执行真实 `codex exec` / `codex exec resume`，没有发送 K3-B1 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/.env/keychain/OAuth/provider credential/full transcript/rollout/prompt body，没有启动 K3-B1 retry / K3-B2，没有自动写 FormalMemory。

浏览器验证尝试说明：Vite dev server 在提权后可启动；Playwright bundled browser 缺失，系统 Chrome headless 在当前环境 SIGABRT，真实浏览器验证未完成。UI 主文案和按钮路径由 offline React render 测试覆盖；此项应由复核线判断是否记 P2 residual risk。

独立复核：Aquinas 初审返回 `STATUS: FINDINGS`，P1 为手动回交未展示 actual exact command / 命令引用；已通过 `manual_exact_command` 契约、UI 展示和测试修复。P2 为扫描未覆盖 untracked 文件；已改为显式 modified / untracked 文件列表扫描。复审结论为 `STATUS: CLEAR_WITH_P2`，唯一保留项为真实浏览器验证环境阻断的 P2 residual risk。复核记录见 `evidence/2026-06-10-stage-l-l1-k3-b1-blocked-recovery-product-path-review-aquinas-v1.md`。

## 下一步建议

1. 主管线决定是否接受 Aquinas `CLEAR_WITH_P2` 的浏览器验证 residual risk。
2. 若接受，由主管线决定是否更新 `CURRENT.md` 并提交。
3. 若用户后续选择手动回交或重新授权真实执行，必须另开任务包；不得把 L1 当成 K3-B1 retry 成功或 K3-B2 可开始。
