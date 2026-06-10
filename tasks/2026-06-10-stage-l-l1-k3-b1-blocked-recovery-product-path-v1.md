# Stage L / L1 K3-B1 Blocked Recovery Product Path v1

日期：2026-06-10

状态：待执行。本文是 Stage L 的 L1 任务包，用于把 K3-B1 `blocked_by_safety_review_again` 转成用户可理解、可恢复、可审计的产品路径。L1 默认不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不启动 K3-B1 retry，不启动 K3-B2。

## 0. 全局主管理解

已知事实：

- Stage K 已冻结为 `accepted_with_deferred_items`，不是严格无缺口完成。
- Stage L / L0 已完成，结论为 `accepted`，已冻结 L1-L6 顺序和权限矩阵。
- K3-B1 已执行过一次，但失败分类为 `failed_classified_codex_state_readonly`。
- K3-B1 retry 申请再次被安全审查拒绝，结论为 `blocked_by_safety_review_again`。
- 审查拒绝理由是：真实 Codex resume 会发送项目 / session 派生 prompt 到外部服务，并写入 `/Users/yoyi/.codex`。
- 审查明确禁止 workaround、indirect execution 或 policy circumvention。
- K3-B1 成功或等价替代路径通过之前，K3-B2 不得开始。

本任务的核心判断：

```text
L1 要解决“用户和工作台如何理解、恢复、回收这个 blocked 状态”，不是直接再执行一次 K3-B1。
```

## 1. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-stage-l-post-k-deferred-closure-and-daily-use-hardening-plan-v1.md`
- `tasks/2026-06-10-stage-l-l0-post-k-deferred-closure-scope-permission-and-acceptance-freeze-v1.md`
- `evidence/2026-06-10-stage-l-l0-post-k-deferred-closure-scope-permission-and-acceptance-freeze-v1.md`
- `handoffs/2026-06-10-stage-l-l0-post-k-deferred-closure-scope-permission-and-acceptance-freeze-v1-result.md`
- `evidence/2026-06-10-stage-k-k3-b1-retry-safety-review-rejection-v1.md`
- `handoffs/2026-06-10-stage-k-k3-b1-retry-safety-review-rejection-v1-result.md`
- `tasks/2026-06-10-stage-k-k3-b1-1-codex-state-permission-and-retry-gate-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

## 2. 目标

L1 必须完成：

- 把 K3-B1 当前状态显示为 `blocked_by_safety_review_again`，不能显示成失败可自动重试、已成功、已完成或可进入 K3-B2。
- 给用户解释清楚阻断原因：真实 Codex resume 会向外部服务发送 prompt，并写入 `/Users/yoyi/.codex`。
- 给出三个合法恢复选项：用户手动 exact command 回交、用户重新明确批准风险后另行申请真实执行、更窄的本地执行桥设计。
- 建立用户手动回交的产品路径：回交 stdout / stderr / exit code / run dir / last message / sidecar refs / hash 结果，由主管线复核后才能改变 K3-B1 状态。
- 建立重新批准的产品路径：只能创建“待重新授权 / 待安全审查”状态，不能在 L1 默认执行。
- 建立更窄本地执行桥的设计约束：只能作为后续任务包候选，不能绕过安全审查。
- 补 runtime log / audit / readback / memory capture 边界：blocked 和 recovery decision 可记录，真实 prompt、secret、完整 transcript 和 `.codex` 内容不能进入普通记录。
- 在 UI 中把该状态展示成普通用户能理解的恢复卡片，而不是开发者错误码堆叠。

## 3. 不做项

L1 不做：

- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不发送 K3-B1 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 不启动 K3-B1 retry。
- 不启动 K3-B2。
- 不通过 Browser / Chrome / shell / test harness 间接实现同一真实执行结果。
- 不把用户手动回交的文本自动当成成功事实。
- 不自动写 FormalMemory。
- 不新增 planned adapters 真实接入。
- 不做 provider credential / model verification。
- 不做手机端 UI。

## 4. 恢复选项冻结

### 4.1 选项 A：用户手动 exact command 回交

产品含义：

- 工作台显示 exact command、风险说明和需要回交的字段。
- 用户离开工作台，在自己明确控制的终端环境中运行命令。
- 用户把结果回填给工作台或交给主管线。
- 工作台只记录“用户提交了回交材料”，不自动认定 K3-B1 成功。

必须回交字段：

- `stdout_summary`
- `stderr_summary`
- `exit_code`
- `run_dir`
- `last_message`
- `sidecar_refs`
- `runtime_log_refs`
- `audit_refs`
- `readback_status`
- `result_count`
- `project_file_hashes_before_after`
- `user_statement`

验收要求：

- `result_count=null` 必须显示为未知 / 不可用，不能显示为 0 条。
- `exit_code=0` 仍不等于自动成功，必须结合 last message、marker、hash 和主管复核。
- 手动回交材料里如包含 secret / token / 完整 transcript / `.codex` 原始内容，必须阻断或要求脱敏。

### 4.2 选项 B：用户重新明确批准风险后再次申请真实执行

产品含义：

- UI 可展示“重新申请真实执行”的风险说明和准备清单。
- 用户必须明确看到：该路径会发送项目 / session 派生 prompt 到外部服务，并写入 `/Users/yoyi/.codex`。
- L1 只能产出 `pending_renewed_risk_approval` 或 `ready_for_separate_execution_request` 状态。
- 真正执行必须另开 L1-B 或 L2 前置执行任务包，并再次冻结 execution point。

禁止：

- 禁止把 L1 的“重新批准”按钮直接接到真实 runner。
- 禁止使用已有授权自动继承。
- 禁止审查拒绝后换壳执行同一命令。

### 4.3 选项 C：更窄本地执行桥设计

产品含义：

- 只允许设计，不允许在 L1 直接实现可执行桥。
- 目标是降低高风险外发和 `.codex` 写入范围，而不是绕过安全审查。
- 后续如要实现，必须单独任务包说明 bridge 边界、用户本机确认、allowed roots、denied paths、prompt hash、readback、audit、rollback 和安全审查要求。

最低设计要求：

- 不能读取 secret、token、完整 transcript、rollout、provider credential。
- 不能隐藏 prompt 外发事实。
- 不能自动批准真实执行。
- 不能把用户手动运行代理成“工作台已安全执行”。

## 5. 产品状态模型

L1 至少要能表达以下状态：

- `blocked_by_safety_review_again`
- `manual_recovery_available`
- `manual_recovery_submitted`
- `manual_recovery_needs_review`
- `manual_recovery_rejected`
- `manual_recovery_accepted`
- `pending_renewed_risk_approval`
- `renewed_execution_request_rejected`
- `narrow_bridge_design_required`

状态约束：

- `blocked_by_safety_review_again` 不能解锁 K3-B2。
- `manual_recovery_submitted` 不能自动解锁 K3-B2。
- 只有主管线复核后明确写入 `manual_recovery_accepted` 或等价 accepted 状态，才允许准备 L2。
- `pending_renewed_risk_approval` 不等于已授权执行。
- `renewed_execution_request_rejected` 必须保持 blocked，不允许自动换路径重试。

## 6. 后端 / 数据边界

实现时优先复用现有事实源：

- Product Command / real execution command 记录。
- runtime log store。
- audit events。
- workflow state 中的 run unit / attempt / readback 引用。
- memory capture event / observation / candidate 既有链路。

如确实需要新增工作台自有 sidecar，必须满足：

- 文件只写在 product-line 工作台自有状态目录或任务包明确允许的 workspace 写入范围。
- schema 最小化，只保存 recovery 状态、用户选择、脱敏摘要、refs 和 hash。
- 不保存 prompt body。
- 不保存 secret、token、完整 transcript、`.codex` 原始内容、rollout 或 provider credential。
- corrupt JSON / revision conflict 不覆盖原文件。
- duplicate submission 不重复生成 accepted 状态。

禁止：

- 禁止把 `/Users/yoyi/.codex` 当作 L1 读源。
- 禁止通过测试或 helper 隐式执行 `codex exec` / `codex exec resume`。
- 禁止把旧 K3-B1 retry command 标记为成功。

## 7. UI 显示边界确认

L1 涉及 UI，必须遵守 `docs/workbench-frontend-display-boundary-v1.md` 和 `docs/plans/task-package-ui-display-boundary-rule-v1.md`。

普通用户主层应该显示：

- 当前状态：K3-B1 被安全审查再次阻断。
- 为什么阻断：会向外部服务发送 prompt，并写入 Codex 本地状态。
- 现在能做什么：手动运行并回交、重新授权申请、等待更窄本地执行桥。
- 哪些事还不能做：不能自动重试，不能进入 K3-B2，不能把失败当完成。
- 下一步需要谁处理：用户、全局主管、或后续执行任务包。

开发者 / 详情层可以显示：

- `execution_point_id`
- frozen prompt hash
- project root
- session id
- exact command
- expected marker
- rejected safety reason
- audit / runtime / readback refs

开发者 / 详情层不得默认铺开：

- prompt body。
- full transcript。
- secret、token、`.env`、keychain、OAuth、provider credential。
- raw `.codex` 路径内容。
- 大段内部 JSON。

UI 禁止文案：

- `K3-B1 retry 成功`
- `K3-B2 可开始`
- `自动重试已启用`
- `已完成真实恢复`
- `读回 0 条`
- `安全审查已绕过`
- `已获得通用真实执行授权`

## 8. 运行日志 / 审计 / readback 边界

runtime log 必须记录：

- blocked 状态可见。
- 用户选择了哪个恢复路径。
- 是否提交了手动回交材料。
- 是否等待主管线复核。
- 是否仍然阻断 K3-B2。

audit 必须记录：

- 谁提交了 recovery decision。
- 提交时间。
- 选择的恢复路径。
- 风险说明是否被确认。
- 主管线复核结论。

readback 必须遵守：

- 未执行真实 Codex 时，readback status 不能显示为成功。
- readback unavailable / failed / not attempted 的 `result_count` 必须是 `null`，UI 显示未知 / 不可用。
- 用户手动回交 last message 需要单独标记为 user-submitted evidence，不能伪装成系统自动 readback。

## 9. 记忆层边界

允许：

- 记录“用户选择了某个 recovery 路径”的 capture event。
- 记录“K3-B1 因安全审查被阻断”的 observation。
- 生成候选记忆，供用户确认。

禁止：

- 自动写 FormalMemory。
- 把手动回交内容自动当正式事实。
- 把 blocked 状态改写为成功经验。
- 保存 prompt body、secret、token、完整 transcript 或 `.codex` 原始内容。

候选记忆建议文案必须含有不确定性：

```text
K3-B1 retry 曾因真实 Codex resume 的外发和 .codex 写入风险被安全审查再次阻断；后续恢复需要用户手动回交、重新授权申请或更窄本地执行桥。
```

## 10. 分线职责

全局主管线：

- 审核 L1 任务包执行是否越界。
- 最终决定 L1 是否 accepted、blocked 或 accepted_with_deferred_items。
- 不把手动回交材料自动升级为成功。

执行 / runner 线：

- 只处理 recovery 状态、product command 边界、runtime log、audit、readback 状态。
- 不调用真实 Codex。
- 不读写 `/Users/yoyi/.codex`。

工作流线：

- 处理 K3-B1 / K3-B2 前置关系、run unit 状态和 blocked gate。
- 不解锁 K3-B2，除非主管线接受等价恢复。

UI / Tauri 线：

- 做普通用户可理解的 blocked recovery 卡片和详情层。
- 不把开发者信息铺进主界面。
- 不新增真实执行按钮。

记忆线：

- 接入 capture / observation / candidate 边界。
- 不自动正式化。

复核线：

- 只读复核 P0/P1/P2、安全、UI 信息层级、架构边界、测试和 evidence。
- 不写产品代码，不替主管线做最终接受。

## 11. 建议实施切片

为避免拆得过细，L1 可作为一个任务包内的四个切片执行：

1. L1-A：blocked recovery read model 和状态契约。
2. L1-B：UI blocked recovery 卡片、手动回交入口和重新授权说明。
3. L1-C：runtime / audit / readback / memory capture 边界接入。
4. L1-D：证据、handoff、扫描和主管线接受复核。

如任何切片需要真实 `codex exec` / `codex exec resume`，必须停止并拆出独立执行任务包。

## 12. 验证要求

若本任务改产品代码，至少运行：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- 相关 Rust 单测，例如 recovery / product command / runtime log / audit / memory capture / workflow gate 测试。
- `cargo fmt -- --check` 或明确列出 `rustfmt --check` 文件。

必须扫描：

- `K3-B1 retry 成功`
- `K3-B2 可开始`
- `自动重试已启用`
- `安全审查已绕过`
- `result_count: 0`
- `codex exec`
- `codex exec resume`
- `/Users/yoyi/.codex`

扫描命中必须分类。命中可以来自历史证据、任务包、禁止项、测试 fixture 或 guard，但不能来自 L1 新增真实执行路径。

## 13. 接受标准

L1 可接受为：

- K3-B1 blocked 状态在产品层可理解、可恢复、可审计。
- 用户能看到阻断原因、风险和合法恢复选项。
- 手动回交路径已定义，并能保持待复核状态。
- 重新授权路径只进入待批准 / 待审查状态，不执行。
- 更窄本地执行桥仅作为后续设计候选。
- runtime log / audit / readback / memory capture 边界明确。
- K3-B2 gate 仍然被阻断。
- 没有真实 Codex 执行、prompt 发送或 `.codex` 读写。

L1 不接受为：

- K3-B1 retry 成功。
- K3-B2 可开始。
- 真实 retry / stop / restart / resume 已实现。
- 通用真实执行恢复策略完成。
- 安全审查可以绕过。
- 用户手动回交自动被接受。
- FormalMemory 自动写入。
- Stage L 完成。

## 14. evidence / handoff 要求

完成后必须新增：

- `evidence/2026-06-10-stage-l-l1-k3-b1-blocked-recovery-product-path-v1.md`
- `handoffs/2026-06-10-stage-l-l1-k3-b1-blocked-recovery-product-path-v1-result.md`

evidence 必须包含：

- 实际改动范围。
- 是否改产品代码。
- 是否执行真实 Codex。
- 是否读写 `/Users/yoyi/.codex`。
- UI 显示边界确认。
- runtime / audit / readback / memory capture 边界确认。
- 测试和扫描结果。
- P0/P1/P2 复核结论。

handoff 必须包含：

- 当前状态。
- K3-B1 是否仍 blocked。
- K3-B2 是否仍阻断。
- 用户可选恢复路径。
- 下一步建议。

## 15. L1 后续

L1 完成后：

- 如果没有手动回交 accepted，也没有新的真实执行 accepted，下一步仍不能进入 L2。
- 如果用户手动回交被主管线接受，可以准备 L2：K3-B2 isolated workspace-write execution closure。
- 如果用户重新批准真实执行，只能另开独立执行任务包并重新通过安全审查。
- 如果更窄本地执行桥成为首选，必须先写桥设计任务包，不得直接实现执行。
