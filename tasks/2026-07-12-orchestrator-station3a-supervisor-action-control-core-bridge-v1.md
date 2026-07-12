# 实现任务包：站 3a 主管动作意图 → Syn 控制核心 → worker 适配器桥 v1

日期：2026-07-12

收口状态：`COMPLETED__REAL_V7_PASS__RISK_CLEANED__READY_FOR_3B`（2026-07-12）。实现、自动化验证和固定测试项目 v7 UI 真跑均已完成；v7 后三轮独立复核发现的 binding 身份碰撞、首轮迁移漏同步 dispatch 引用、历史引用误改绑风险及相关账本一致性 P1 已在 3b 前清理；3b 未启动。完成证据以 `evidence/2026-07-12-orchestrator-station3a-control-core-bridge-v1.md` 为准。

性质：**架构纠偏任务包；以下保留实施时冻结的边界与验收，不再作为待派任务。**

执行方式：总指导负责边界、协议复核与最终核实物；执行线按本包实现，不 commit。
上承：

- 最终蓝图：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`
- 中间版本权威：`docs/middleware-version-development-plan-v1.md`，尤其“修订后的执行边界”第 1、2、5、7 条
- 系统架构：`docs/workbench-system-architecture-v1.md`，尤其控制核心、应用服务层、适配器层和 Codex 多线程约束
- 当前提案：`docs/plans/2026-07-11-supervisor-orchestrator-mode-proposal-v1.md`
- 当前主管契约：`docs/plans/2026-07-11-supervisor-contract-v1-draft.md`
- 旧站 3a 包：`tasks/2026-07-12-orchestrator-station3a-unattended-closure-v1.md`

## 0. 为什么必须改

蓝图要求：

```text
项目主管作出编排判断
→ 输出结构化动作意图
→ Syn 控制核心按已批准方案校验
→ 应用服务调用 AgentAdapter / 既有受控派发入口
→ worker 执行
→ 结果和证据回到账本与项目主管
```

当前站 3a 实际实现为：

```text
项目主管
→ 直接调用有副作用的 dispatch_worker MCP 工具
→ MCP 内部校验并直接启动 worker
```

真实试点已经证明这条传输通路不可靠：主管工具调用在到达 Syn 账本前被 Codex 外层取消，账本没有 worker、没有 `final_mark`，但末条消息把系统取消写成了 `user cancelled`；用户明确没有点击取消。

因此，本包不推翻“LM 编排、状态机守门”，只纠正执行权所在的位置：

> **项目主管可以决定下一步做什么，但不能直接持有启动 worker 的钥匙；它只提交动作提议，真正执行权属于 Syn 控制核心。**

## 1. 本包目标

完成一个最小可用的主管动作循环：

1. 主管每一步输出一份严格结构化的 `SupervisorActionProposalV1`。
2. 模型输出只是不可信提议，不携带任何权限。
3. Syn 将提议绑定到当前真实 `run_id`、项目、工作流、授权快照和账本修订版。
4. 控制核心逐项校验后，通过现有应用服务 / Codex adapter 执行。
5. 执行结果写入权威账本，形成 `SupervisorActionResultV1`。
6. 非终态结果再次交给主管判断下一步，直到终标、等待用户或触发预算上限。
7. 主管的副作用动作不再依赖 Codex MCP 工具审批是否放行。

完成后只能说明：**站 3a 新控制路径具备自动化验证条件**。在固定测试项目完成一次新的真实 UI 发射前，不得宣布站 3a PASS，更不得解锁真实项目站 3b。

## 2. 已拍设计

### 2.1 智能和权力分开

- 项目主管 LM 负责：拆解、判断、选择动作、解释理由、阅读结果、决定继续或终止。
- Syn 控制核心负责：身份绑定、授权、任务包、精确读写根、工具清单、配额、幂等、状态机、审计和失败分类。
- AgentAdapter 负责：接收控制核心已经批准的受控命令，启动 / 续发 / 读取 worker，并返回结果。
- worker 负责：只在自己的衰减权限包内执行任务并结构化回交。
- 账本负责：保存真正发生过什么；模型自述不能覆盖账本。

### 2.2 站 3a 采用宿主驱动的分步动作循环

试点首选路径：

```text
Syn 启动主管一步
→ 主管在 last_message 输出一个动作提议 JSON
→ 主管进程结束
→ Syn 严格解析并校验
→ 控制核心执行或拒绝
→ 结果落账
→ Syn 带着结果和精简账本快照启动主管下一步
```

约束：

- 同一逻辑主管运行始终使用同一个工作台 `run_id`。
- 不把 Codex thread id 当成工作台主键，也不要求站 3a 先实现持久 provider 对话。
- 每一步最多产生一个动作；禁止模型一次输出一串待执行动作。
- Syn 必须在每一步之间重新检查授权是否仍 active、是否过期、是否被撤销、workflow revision 是否变化。
- 达到最大动作数、总时长、worker 数或追问预算时，必须停止并形成诚实状态。
- 只读 MCP 可暂时保留用于受控读取；任何能创建、续发、终标、写账本或影响 worker 生命周期的 MCP 工具都不能再作为主管执行通路。

不采用“改名后的 `submit_action` MCP 在同一次工具调用里照样直接启动 worker”。这仍把真实副作用绑在 Codex 工具调用能否通过外层审查上，没有解决本次实证问题。

### 2.3 临时 CODEX_HOME 继续保留，但不再承担权限绕过

- 临时 `CODEX_HOME`、`0700` 目录、`0600` 配置和 `auth.json → ~/.codex/auth.json` 符号链接方案保持不变；不得复制凭据。
- 主管继续使用 `read-only` 沙箱，不能自己写项目。
- 新路径不得依赖 `approval_policy = "never"` 才能工作；副作用已经移出 Codex 工具调用后，应删除这一临时绕过配置，或证明剩余只读工具仍有不可替代需求并单独上报总指导。
- 不得恢复 `--ignore-user-config`，不得把用户全局 Codex 配置复制进临时 home。

## 3. 协议冻结

### 3.1 模型输出：`SupervisorActionProposalV1`

主管只能输出一个 JSON 对象，不得在 JSON 前后夹带自然语言：

```json
{
  "schema_version": "supervisor_action_proposal.v1",
  "kind": "dispatch_worker",
  "target": {
    "node_id": "node-id",
    "work_item_id": "work-item-id"
  },
  "reason": "为什么现在应执行这个动作",
  "expected_result": "主管下一步需要看到什么证据"
}
```

模型提议中**不得出现**：

- `project_root`、`allowed_read`、`allowed_write`。
- `authorization_id`、权限等级、沙箱等级。
- shell argv、可执行文件路径、环境变量、凭据路径。
- `approved=true`、`bypass`、`full_access` 等自我授权字段。
- 由模型指定的正式 `action_id`、账本 revision 或审计结论。

这些字段只能由 Syn 从当前运行上下文和权威正本派生。模型不得通过“把批准信息再说一遍”获得权力。

### 3.2 首批动作类型

首批只允许以下判别联合；每种类型都有独立严格字段，不接受自由形态 `payload`：

- `dispatch_worker`：目标只能是当前任务包中尚可派发的 `node_id + work_item_id`。
- `inspect_worker`：读取同一运行下一个已登记 worker 的权威状态和结构化回交。
- `follow_up_worker`：向同一运行下已登记 worker 发送一次受预算约束的追问。
- `wait_worker`：等待同一运行下的 worker，超时值由 Syn 限幅。
- `finalize`：提议 `pass / needs_rework / blocked`，控制核心必须根据账本证据判断是否允许落正式终标。
- `report_user`：形成用户可见报告，但不能写成用户决定、用户确认或用户取消。
- `request_user_decision`：仅在越权、范围变化、不可逆风险或关键方向问题时创建待用户决定，不执行所请求的动作。

任何未知 `kind`、多余字段、字段类型错误或目标缺失都必须 `protocol_invalid`，零副作用。

### 3.3 Syn 生成的权威记录

解析成功后，由 Syn 生成 `SupervisorActionRecordV1`，至少绑定：

- 工作台生成的 `action_id` 和幂等键。
- `run_id`、project id、workflow id、node / work item / worker id。
- active authorization id 与 authorization snapshot hash。
- workflow revision / task package fingerprint。
- 动作种类、主管理由和收到时间。
- 校验结果、执行状态、adapter id、dispatch / readback / audit refs。

执行结果形成 `SupervisorActionResultV1`，状态至少区分：

- `completed`
- `denied_scope`
- `authorization_stale`
- `protocol_invalid`
- `quota_exceeded`
- `adapter_failed`
- `transport_failed`
- `waiting_worker`
- `waiting_user`

主管只能读取这些结果，不能修改它们。

## 4. 控制核心执行顺序

每个动作必须按以下顺序，不能先执行后补审计：

1. 确认主管 `run_id` 存在且仍为 active。
2. 从 run 绑定中取得 project / workflow / authorization；不采信模型路径。
3. 检查 authorization active、未过期、未撤销，且 workflow revision 未漂移。
4. 检查该动作种类在本单 capability allowlist 中。
5. 对 `dispatch_worker` 唯一定位同授权的 `authorized_prepared_auto_dispatch`。
6. 从任务包正本派生 allowed read / write、工具、检查和停止条件；不读取模型提供的权限字段。
7. 检查节点、work item、worker、项目和授权全链一致。
8. 检查并发、追问、动作次数、运行时长和重复派发配额。
9. 先写 action reservation / accepted audit，再调用既有受控派发入口。
10. adapter 返回后写 completed / failed result、worker 绑定和证据引用。
11. 崩溃恢复时根据幂等键返回旧结果或安全收口，不能重复启动 worker。

站 3a 不要求一次性重建通用 AgentAdapter 插件框架。可以把现有 Codex 真实派发入口视为首个 adapter 实现，但必须通过一个清楚的控制核心应用服务调用；不得从协议解析器、launcher 或 MCP handler 直接 spawn worker。

## 5. “用户取消”必须有真凭据

只有同时满足以下条件，状态才允许写成 `user_cancelled`：

- 用户在 Syn UI 上触发明确取消命令；
- 后端收到该命令并生成用户动作编号；
- 账本存在 `user_cancel_requested` 事件、时间和目标 run；
- 停止结果可追溯到这条事件。

以下情况一律不能归因给用户：

- Codex 外层工具审批自动取消。
- provider / runner 中断。
- 主管进程退出、超时或 stderr 报错。
- JSON 解析失败。
- 应用关闭或进程收割。

这些情况分别进入 `transport_failed`、`adapter_failed`、`protocol_invalid`、`timed_out` 或 `system_stopped`。用户可见文案必须说明“系统没有执行”，不能说“你取消了”。

## 6. 允许修改的文件

预计允许面：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/supervisor_action_protocol.rs`
- 新增 `prototypes/productized-desktop-shell/src-tauri/src/supervisor_action_controller.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_session_launcher.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `docs/plans/2026-07-11-supervisor-orchestrator-mode-proposal-v1.md`
- `docs/plans/2026-07-11-supervisor-contract-v1-draft.md`
- 本包对应的新 evidence 文件

仅当需要把新的诚实失败状态显示到现有主管结果区域时，才允许最小修改：

- `prototypes/productized-desktop-shell/src/lib/types/workflow.ts`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectJiaobanPanel.tsx`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`
- `prototypes/productized-desktop-shell/tests/jiaoban-supervisor-pilot-switch.test.tsx`

确需修改列表外文件，先停下向总指导说明原因。不得顺手重构现有 worker 协议读模型、会话中心、记忆层、研究文档或 UI 布局。

## 7. 明确不做

- 不运行非固定测试项目，不解锁站 3b。
- 不增加主管 shell、文件写、凭据、浏览器或系统管理员权限。
- 不采用每派一个 worker 都让用户确认一次的流程。
- 不让模型生成 `allowed_write` 后再由后端“核对”；权限直接从正本派生。
- 不通过修改工具名称、提示词或 `approval_policy` 继续赌 Codex 外层审查。
- 不把项目主管降级成确定性状态机；动作选择仍由 LM 决定。
- 不实现跨项目自动接力、后台长期自治、模型自动路由或可见执行对话产品化。
- 不改临时 `CODEX_HOME + auth.json` 符号链接的已确认凭据方案。
- 不清理用户既有脏工作树，不 `git add`、不 commit、不 push。

## 8. 验收测试

### 8.1 协议和零副作用

- 合法动作逐种解析成功。
- 未知 kind、多余字段、缺字段、非 JSON、JSON 前后夹文本全部拒绝。
- 提议包含 `allowed_write`、`project_root`、authorization 或绕过字段时拒绝。
- `protocol_invalid` 不产生 reservation、worker、final mark 或用户决定。

### 8.2 授权与权限衰减

- 控制核心只从 active authorization + task package + prepared dispatch 派生权限。
- 过期 / 撤销授权、revision 漂移、跨项目 target、错误 node/work item、重复候选都在 adapter 前拒绝。
- worker 权限必须是用户批准范围的子集；不能扩大读写根、工具或运行预算。
- `finalize: pass` 在缺 worker report / readback / 必需证据时拒绝或降为 `needs_rework`。

### 8.3 幂等和恢复

- 相同 proposal 在同一账本 revision 重放，不重复启动 worker。
- reservation 后崩溃能恢复为同一 action，不能生成第二个 dispatch。
- 主管下一步只拿到前一步的权威结果和必要账本摘要，不能拿模型自造结果。

### 8.4 取消归因

- 没有 `user_cancel_requested` 账本事件时，任何取消都不能显示 `user_cancelled`。
- provider 中止、外层工具取消、进程超时和应用停止分别得到诚实状态。
- 真实用户点击取消时，动作编号、run 和最终停止状态能串起来。

### 8.5 集成闭环

用 fake supervisor 输出动作提议、fake adapter 回结果，至少跑通：

```text
dispatch_worker
→ waiting / inspect_worker
→ 读取结构化 worker report
→ finalize(pass)
→ report_user
```

并断言：

- 主管工具清单中不存在可直接启动 / 续发 / 终标 worker 的副作用 MCP 工具。
- worker 启动来自控制核心应用服务，不来自 MCP handler。
- 每一步都有 action record、guard result、adapter result 和 audit refs。
- 经典状态机路径未被改动，试点开关默认仍关闭。

### 8.6 仓库验证

至少执行：

1. 新协议与 controller 定点测试。
2. `cargo test --lib mcp::supervisor_orchestrator::tests --quiet`
3. `cargo test --lib station3a_ --quiet`
4. `cargo test --lib s3_director_dispatch_integration_stub --quiet`
5. `cargo test --lib --quiet`
6. `npm run typecheck`
7. `npm run test:offline-interaction`
8. `cargo check --offline`
9. `cargo fmt --check`；只允许已知既有三处漂移，新增块不得漂移。
10. `git diff --check`

实现回交状态只能是：

- `READY_FOR_FIXED_PROJECT_REAL_RUN`
- `BLOCKED`

固定测试项目的新真实 UI 发射仍作为单独一步，由用户明确批准后进行。真实发射必须使用新的证明文件名，不能复用旧 42 字节文件冒充本次结果。

## 9. 真实发射通过标准

新路径在固定测试项目中必须同时证明：

- 账本出现主管 action proposal、控制核心 guard、adapter dispatch 和 worker 记录。
- worker 确实执行任务包允许的精确写入。
- 主管读取到结构化回交并基于证据终标。
- `final_mark: pass` 和用户报告均落账。
- 没有副作用 `dispatch_worker` MCP 调用。
- 没有虚假的 `user cancelled`。
- 临时 `CODEX_HOME` 按原方案创建和清理，`auth.json` 始终只为符号链接。

以上全部成立后，站 3a 才能标记 PASS。该结果只证明固定测试项目路径，不证明其它真实项目可安全运行。

## 10. 不接受为完成

- 只把 `dispatch_worker` 改名为 `submit_action`，内部仍由 MCP handler 直接启动 worker。
- 靠提示词写“用户已经批准”解决权限。
- 继续要求模型复制 project root、authorization 或 allowed_write。
- 只有单测，没有控制核心闭环集成测试。
- 没有用户取消事件，却把系统中止写成用户取消。
- 旧证明文件、旧 worker 或旧账本被当作本次新路径证据。
- 为了做站 3a 顺手建设完整通用 adapter 平台、可见对话 UI 或跨项目自治。
- 未经用户单独批准就修改安全闸或进行新的真实发射。
