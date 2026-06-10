# Evidence: Stage K / K0 Scope, Permission, And Acceptance Matrix Freeze v1

日期：2026-06-10

状态：已完成。复核线只读审查无 P0/P1；P2 已补“真实执行点字段工作表”和“候选测试项目登记表”。K0 是 Stage K 的文档 / 只读冻结任务，不改产品代码，不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`。

## 1. 本轮目标

把 Stage K 从“开发计划”推进到可执行的 K0 任务包，冻结下一阶段日常可用工作台产品化的范围、边界和验收矩阵。

Stage K 目标：

```text
把 Stage J 的受控产品化 checkpoint 推进为日常可用工作台。
```

K0 重点冻结：

- K1-K6 任务顺序。
- K2/K3/K5 真实执行授权条件。
- 测试项目矩阵。
- prompt / transcript / `.codex` / secret 边界。
- 记忆捕获策略。
- UI 普通层 / 详情层 / 开发者层分层。
- 多会话协作分线职责。
- checkpoint 文档同步规则。
- 候选测试项目登记字段：project_id、session、baseline hash、allowed roots、denied paths、readback marker。
- 真实执行点字段工作表：execution id、operation、adapter、project、session、sandbox、prompt hash、memory packet、permission envelope、readback、runtime log、audit、baseline、dirty worktree、rollback、user confirmation。

## 2. 新增 / 更新文件

新增：

- `docs/plans/2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md`
- `tasks/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1.md`
- `evidence/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1.md`
- `handoffs/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1-result.md`

未修改产品代码。

## 3. K0 冻结内容摘要

### 3.1 阶段范围

Stage K 做：

- 智能体页日常可用。
- 通用 Codex `resume` / `new session` 产品入口。
- 项目工作流真实派发闭环。
- 运行中 / 待办 / 失败恢复。
- 记忆捕获、候选确认和任务记忆注入体验。
- 真实 Tauri dogfood。

Stage K 不做：

- 不接 planned adapters 的真实执行。
- 不做 provider credential store 或 model verification。
- 不开放任意目录无限制读写。
- 不让 agent 自治批准高风险权限。
- 不自动写正式记忆。
- 不无确认自动 retry / stop / restart。

### 3.2 任务顺序

- K0：范围、权限和验收矩阵冻结。
- K1：智能体对话页日常可用重构。
- K2：通用 Codex `resume` / `new session` 产品入口。
- K3：项目工作流真实派发闭环。
- K4：记忆捕获、候选确认和任务记忆注入体验。
- K5：运行中、待办、失败恢复和操作控制。
- K6：真实 Tauri dogfood 和验收收口。

### 3.3 测试项目

- `mario test`：`/Users/yoyi/Documents/mario test`，历史真实探针参考；K0 不继承旧授权。
- 工作台自身项目：`/Users/yoyi/workspace/product-line`，dogfood；K0 不授权真实执行。
- Stage K 隔离测试项目：`/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project`，后续 K2/K3 可创建；K0 不创建。
- 真实业务项目：默认禁止，必须用户单独指定。

### 3.4 真实执行授权

任何 K2-K5 真实执行点必须重新列明：

- 操作类型。
- adapter。
- project / cwd / workflow / run unit / task package / memory packet。
- target session 或 new session 规则。
- sandbox / timeout / allowed write roots / denied paths。
- `/Users/yoyi/.codex` 最小读写范围。
- prompt summary / hash / runtime body 策略。
- readback / runtime log / audit / evidence / rollback。
- 用户确认方式。

### 3.5 UI 信息层级

普通用户层只显示：

- 项目、对话、输入、发送前确认、执行状态、结果摘要、待确认事项、记忆候选摘要。

详情层可显示：

- 写入范围、记忆包、权限原因、readback 人话解释、worker report 摘要、evidence / handoff 链接。

设置 / 开发者层才显示：

- Product Command ids、runtime log refs、audit refs、sidecar path、store revision、readback enum、adapter / provider raw boundary、legacy 状态。

## 4. 验证记录

本轮执行的是文档 / 任务包冻结，不跑产品测试。

已执行扫描：

```text
rg -n "Stage K .*已完成|K0 .*已完成|授权直接执行新的真实|授权直接读写 /Users/yoyi/.codex|通用自由 Codex 控制台已开放|任意目录无限制" \
  docs/plans/2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md \
  tasks/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1.md
```

结果分类：

- `Stage K 已完成` 命中只出现在 K0 “不接受为”或验收清单中。
- `通用自由 Codex 控制台已开放` 命中只出现在 K0 “不接受为”中。
- `任意目录无限制执行已开放` 命中只出现在 K0 “不接受为”中。
- `授权直接执行新的真实` 和 `授权直接读写 /Users/yoyi/.codex` 命中只出现在 Stage K 计划状态说明中，且语义为“不授权”。

未执行：

- 未执行 `npm run typecheck`。
- 未执行 `npm run test:offline-interaction`。
- 未执行 `npm run build`。
- 未执行 Rust 测试。
- 未启动 Tauri / Browser / Chrome。
- 未截图。

原因：

- K0 只写计划 / 任务包 / evidence / handoff，不改产品代码，不涉及 UI 实际渲染或后端执行。

## 5. 复核状态

已派复核线只读审查：

- 目标线程：既有复核线 `019eabfc-7e22-70b3-860e-8017c46919f4`
- 要求：不改文件、不启动 GUI、不执行真实 Codex、不读写 `/Users/yoyi/.codex`
- 审查范围：Stage K 计划、K0 边界、K0 是否足够支撑 K1-K6

复核线回交结论：

- P0：无。
- P1：无。
- P2：K0 任务包需要把若干矩阵从“类别”补成“可验收字段”，尤其是 project_root / project_id / session_id、allowed / denied path、baseline hash、prompt hash/ref、readback marker、`.codex` 副作用范围、dirty worktree、stop/restart proposal 等。

主管线已修补：

- `tasks/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1.md` 新增 `5.1 候选测试项目登记表`。
- `tasks/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1.md` 新增 `5.2 真实执行点字段工作表`。

## 6. 边界确认

本轮没有：

- 修改产品代码。
- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout。
- 启动 Tauri / Browser / Chrome。
- 创建隔离测试项目。
- 修改 workflow state JSON。
- 新增 store / sidecar / DB migration。

## 7. 结论

K0 可接受为：

- Stage K 范围、权限、测试项目、UI 信息层级、多会话分工、checkpoint 同步规则和 K1-K6 验收矩阵冻结完成。
- K0 本身不授权真实执行，不授权读写 `/Users/yoyi/.codex`，不开发产品代码。

```text
accepted
```

允许进入：

- K1 UI 线：智能体对话页日常可用重构。
- K2 Execution 线：通用 Codex `resume` / `new session` 产品入口任务包准备。

K2 的任何真实执行仍需单独任务包和执行点授权。
