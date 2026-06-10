# Stage H / H0 Safety Boundary And Task Package Freeze v1

日期：2026-06-07

状态：已完成文档冻结，并已通过全局主管复核。  
用途：冻结阶段 H 真实 `codex-local` 执行的安全边界、任务顺序、授权条件、测试项目原则、证据要求和 UI / Tauri 验收边界。H0 是文档 / 任务包冻结，不是产品实现。

## 1. 权威依据

本任务包依据：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- E5 Level A / Level B、G1、G2、G5 的 task / evidence / handoff。

## 2. H0 接受范围

H0 可接受为：

- 阶段 H 真实执行总边界已冻结。
- H1-H7 的前置关系已冻结。
- 后续任务读写 `/Users/yoyi/.codex` 的授权条件已冻结。
- H2 真实 resume 和 H3 真实 send / 新会话的执行前授权条件已冻结。
- 测试项目选择原则已冻结：默认先用隔离测试项目，不能默认使用 `mario test` 或真实业务项目。
- allowed write roots、denied paths、secret deny list、no full transcript 边界已冻结。
- prompt preview、task memory packet、permission dialog、runtime log、audit、readback、failure reason、duplicate guard 的最低要求已冻结。
- UI 显示边界和真实 Tauri 验收要求已冻结。
- Codex 多线程协作仅作为架构参考，不能照搬为工作台事实模型。

H0 不接受为：

- H1 / H2 / H3 / H4 / H5 / H6 / H7 已完成。
- 通用真实 send / resume 产品化完成。
- 真实 `codex exec` / `codex exec resume` 已执行。
- planned adapters 真实接入完成。
- provider credential / model verification 完成。
- G3 全量真实 Tauri 截图验收补齐。
- 自动重试、自动恢复、取消 / kill 策略或复杂业务自动编排完成。

## 3. 阶段 H 总边界

阶段 H 只产品化 `codex-local` 真实自动化工作流，不接入 Claude Code / OpenClaw / OpenCode / OpenCode-like 等 planned adapters 的真实执行。

真实执行必须经过：

```text
项目 / 方案授权 / 任务包
-> 任务记忆包
-> prompt preview
-> permission envelope / 用户确认
-> codex-local runner
-> continuation record
-> runtime log
-> audit event
-> readback result
-> worker report / failure reason
-> 项目主管过程确认
-> observation / candidate / formal memory 状态机
-> 全局主管复核
```

默认禁止：

- 未经具体任务包和用户确认执行真实 `codex exec` / `codex exec resume`。
- 未经具体任务包授权读写 `/Users/yoyi/.codex`。
- 读取 auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 读取完整 transcript / rollout 作为普通开发证据。
- 将 readback unavailable / failed / timed out 显示为真实 0 条结果。
- 让 UI、Markdown、日志、worker report、LLM summary 或 Codex 多线程关系绕过控制核心。
- 使用 `--dangerously-bypass-approvals-and-sandbox`。
- 静默自动重试、静默扩大写入范围或默认继承 secret / provider credential。

## 4. 后续任务授权矩阵

| 任务 | 前置 | 是否允许真实 Codex | 是否允许读写 `/Users/yoyi/.codex` | 冻结边界 |
| --- | --- | --- | --- | --- |
| H1 CodexLocalRunner 架构和数据契约 | H0 主管复核后 | 不允许 | 不允许 | 只做类型、guard、runner 契约、单测；不能真实执行 |
| H2 通用真实 resume 产品化 | H1 完成并复核 | 允许，但必须由 H2 任务包逐项授权 | 允许，但仅限 H2 任务包列明的必要 session / index / metadata 范围 | 必须列出测试项目、target session、cwd、sandbox、allowed write roots、prompt summary、readback plan、回滚和 evidence |
| H3 通用真实 send / 新会话产品化 | H1 完成，建议 H2 先完成 | 允许，但必须由 H3 任务包逐项授权 | 允许，但仅限 H3 任务包列明的新会话创建 / 绑定必要范围 | 必须绑定项目、角色、任务包和授权范围；不得开放自由聊天式裸控制器 |
| H4 readback / failure / timeout / cancel / duplicate guard | H2/H3 至少一条真实路径有证据 | 默认不执行；如需真实复现必须单独授权 | 默认不读写；如需 readback metadata 必须单独授权 | cancel / stop 不能默认为 kill；自动重试必须先 preview 和用户确认 |
| H5 项目工作流真实派发集成 | H2/H3/H4 的最小安全链路完成 | 允许，但只能通过产品化 runner 和任务包授权触发 | 允许，但只能继承 H2/H3/H4 已冻结的最小范围 | prepared dispatch -> confirmation -> real dispatch -> readback -> report 必须可追溯 |
| H6 真实执行 UI 产品化和 Tauri 验收 | H2-H5 对应能力有事实记录 | 不应新增裸执行能力 | 不应新增 `.codex` 访问范围 | 只展示产品化状态；必须真实 Tauri 截图或明确 incomplete |
| H7 H 阶段最终验收和冻结 | H1-H6 完成或明确 deferred | 不允许新增真实执行 | 不允许新增 `.codex` 读写 | 只做矩阵、证据、deferred freeze 和 H-to-I handoff |

## 5. H2 / H3 执行前授权条件

H2 或 H3 任务包在任何真实执行前必须逐项列明并获得明确授权：

- 操作类型：`resume`、`send` 或 `new_session`。
- 测试项目：默认隔离测试项目；不能默认 `mario test`，不能默认真实业务项目。
- 目标 session 或新 session 创建规则。
- project root、cwd、sandbox、timeout。
- allowed write roots 和 denied paths。
- `/Users/yoyi/.codex` 允许读取 / 写入的最小范围。
- secret deny list 和 no full transcript 边界。
- prompt summary、完整 prompt 保存策略和是否允许发送。
- task memory packet included / excluded / review materials。
- permission dialog 必须展示的用户可理解说明。
- continuation record、runtime log、audit event、readback result 和 evidence 路径。
- failure reason、timeout、cancel / stop、duplicate guard 和回滚 / 降级策略。

未满足上述任一项时，必须保持未完成 / 阻断，不能冒领真实执行完成。

## 6. 测试项目选择原则

默认原则：

- 先用隔离测试项目，路径和写入范围必须由 H2/H3 任务包再次确认。
- `mario test` 只保留为 E5 Level B 历史健康探针证据，不能默认复用。
- 真实业务项目必须有更高一级明确授权、备份 / 回滚说明和独立风险确认。

测试项目必须满足：

- 可被安全备份或可安全丢弃。
- allowed write roots 明确且足够窄。
- 不包含 auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 任务目标可用文件 hash、runtime log、audit 和 readback 交叉验证。

## 7. 路径边界

允许写入根必须由后续任务包逐项声明，默认只允许：

- 工作台项目内的 H 任务 evidence / handoff / task 文档。
- 隔离测试项目中任务包明确列出的路径。
- 工作台自有 continuation / runtime log / audit / workflow state 记录，且必须由对应产品代码路径写入。

默认禁止路径 / 内容：

- `/Users/yoyi/.codex`，除非 H2/H3/H4/H5 任务包逐项授权。
- auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 完整 transcript / rollout。
- 用户未授权的真实业务项目。
- 任意 provider credential store。
- 任意 shell 临时脚本绕过产品 guard 的执行路径。

## 8. UI 显示边界确认

本任务是否改前端：

- [x] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [ ] 改读模型摘要或状态显示。
- [ ] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- H0 文档中对后续 UI 要求的边界描述。

本任务禁止显示：

- 任何产品 UI 新文案或新按钮。
- “真实执行已完成”“已发送”“已 resume”“Codex 已收到任务”“凭据 / 模型已验证”等越界文案。

中间版本范围：

- 本轮必须落地：文档冻结。
- 本轮只做读模型 / 摘要：无。
- 本轮后置：H6 真实执行 UI 产品化和真实 Tauri 验收。

验收：

- 类型检查：不需要，本任务不改产品代码。
- 离线交互测试：不需要，本任务不改产品代码。
- 构建：不需要，本任务不改产品代码。
- 真实窗口 / 截图验收：不需要，本任务不改 UI；后续 H6 或任何 UI 任务必须按规则执行。

## 9. 验收 / 扫描

H0 完成时必须扫描：

- 过度声明：通用真实 send / resume 完成声明、H1 / H2 / H3 完成声明、planned adapters 接入完成声明、provider credential 验证完成声明。
- 入口方向：`CURRENT.md` 和 `tasks/README.md` 必须指向 H0 已完成或待主管复核，下一步 H1，不能直接 H2/H3。

不跑：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test`

原因：H0 不改产品代码。

## 10. 回交

本任务产物：

- `tasks/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md`
- `evidence/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md`
- `handoffs/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1-result.md`

下一步建议：

- H0 已通过全局主管复核。
- 当前进入 H1 CodexLocalRunner 架构和数据契约。
- 不直接进入 H2/H3 真实执行。
