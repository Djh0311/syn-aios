# Stage K Architecture Calibration Plan v1

日期：2026-06-10

状态：Stage K 插入式架构校准计划，已通过 K2.5 任务包执行并收口为 `accepted`。本文不改变 Stage K 原目标，仍以“自由操控 Codex + 自动化工作流 + 记忆层记录”为最终交付目标。本文用于在 K2 已完成、K3 开始前增加架构校准 gate，避免继续把 K3/K4/K5 建在 probe、legacy 或半收敛路径上。

本文不是执行任务包，不授权新的真实 `codex exec` / `codex exec resume`，不授权直接读写 `/Users/yoyi/.codex`，不替代 `CURRENT.md`、`STAGE_PLAN.md`、`tasks/README.md` 或 evidence / handoff 的完成事实。

## 1. 校准结论

K2 已证明通用 `codex-local` `resume` / `new_session` 产品入口可行，但当前底层仍同时存在：

- Product Command 主路径。
- K2 / J2 授权探针和固定 fixture 常量。
- legacy workflow dispatch / workflow machine blocked wrapper 和内部 helper。
- MCP canvas 实验 runner 的潜在真实 Codex spawn 代码。
- transcript viewer / readback / memory capture 之间需要继续隔离的读写边界。
- memory capture / observation / candidate / formal memory 之间的跨 sidecar 半事务风险。

因此，K3 项目工作流真实自动化编排开始前，必须先完成 K2.5 架构校准。

## 2. 不变目标

Stage K 目标不变：

- 用户能在工作台中选择项目、选择或创建 Codex 对话，直接输入任务。
- 用户确认后，工作台通过统一 Product Command 调用 `codex-local`。
- Codex 执行结果进入运行队列、runtime log、audit、readback 和用户可读摘要。
- 项目工作流能把用户目标拆成 run units 并按授权派发。
- 工作台操作、Codex 结果、worker report 和 final review 能进入 observation / memory candidate。
- 正式记忆仍必须经过确认、版本、来源和审计。

## 3. 校准边界

K2.5 做：

- 收敛命令面分类。
- 把固定 probe 常量迁为 fixture / test-only 语义。
- 明确唯一真实执行主路径。
- 封存或迁移 MCP canvas 真实 runner 旁路。
- 增加 workspace-write 真实文件变更证明设计。
- 增加跨 sidecar 一致性扫描 / 修复设计。
- 增加 Stage K 架构验收扫描 gate。

K2.5 不做：

- 不执行新的真实 Codex。
- 不发送新的 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不接 planned adapters。
- 不做 provider credential / model verification。
- 不把普通浏览器 smoke 当作 Tauri 验收。
- 不把 candidate / observation 自动写成 FormalMemory。

## 4. Gate 拆分

### K2.5-A：Command Surface Calibration

目标：

- 对 Tauri command surface 做分类：`product_current`、`legacy_blocked`、`developer_settings_only`、`read_viewer_only`、`test_probe_only`、`sealed_experiment`。
- 普通 UI 不再依赖 legacy action branch 作为日常执行路径。
- `canvas_start_run` / `canvas_tick_run` 继续保持 blocked，MCP canvas runner 不允许成为产品执行旁路。

验收：

- 命令分类表进入 evidence。
- 扫描裸 `Command::new("codex")`、legacy wrapper、MCP runner、frontend legacy wrapper。
- 无新增真实执行。

### K2.5-B：Product Command Generic Intent Calibration

目标：

- 把 K2 / J2 固定 execution point 抽象为通用链路：

```text
WorkbenchUserCommand
-> CodexExecutionIntent
-> ProductCommandRequest
-> Phase A / Phase B
```

- mario test / isolated project / fixed session / canonical prompt 只能作为 fixture 或测试执行点，不再作为普通产品逻辑入口。

验收：

- 新增或确认通用 intent / request builder。
- K2/J2 常量有明确 fixture-only 命名或隔离边界。
- 普通 UI 只提交结构化 intent，不拼 CLI。

### K2.5-C：Workflow Dispatch Calibration

目标：

- K3 run unit 真实派发必须走 Product Command。
- J2-B bridge 只能保留为历史 / fixture / compatibility，不作为 K3 主路径。
- run unit、worker handoff、readback、runtime log、audit、memory capture 要能通过 ref 链接起来。

验收：

- K3 开发前有明确 `RunUnit -> ProductCommand -> WorkerReport -> ProcessFact -> MemoryCapture` 链路。
- legacy dispatch / workflow machine 不作为普通 K3 执行入口。

### K2.5-D：Memory Consistency Calibration

目标：

- 增加跨 sidecar 一致性检查设计，覆盖 ProductCommand、Continuation、RuntimeLog、CaptureEvent、Observation、MemoryCandidate、FormalMemory。
- 能识别 orphan candidate、observation missing link、formal memory missing candidate adoption link、runtime log missing attempt、capture event missing downstream record。

验收：

- 新增 consistency finding taxonomy。
- 损坏 / 缺链路 / 半写状态不会被 UI 解释成完整成功。
- 不自动修正式记忆，只生成 finding / proposal。

### K2.5-E：Workspace Write Proof Calibration

目标：

- workspace-write 不再只靠 `sandbox != read-only` 推断项目写入。
- allowed roots 执行前后必须有 manifest 或 hash diff。
- forbidden roots / baseline files 必须保持可证明不变。

验收：

- `writes_project_files` 来源从推断升级为证据摘要。
- read-only 执行必须证明核心文件 hash 不变。
- workspace-write 执行必须证明只写 allowed paths。

### K2.5-F：Architecture Acceptance Gate

目标：

- 在 K3/K4/K5/K6 前统一跑架构扫描。

扫描项：

- 裸 `Command::new("codex")` 是否只存在于批准 runner 或 sealed experiment。
- legacy command 是否仍在普通产品路径被调用。
- prompt body 是否被普通 sidecar / runtime log / memory 持久化。
- `.codex` / secret / full transcript / rollout 是否被错误进入记忆。
- readback unavailable / failed / timed_out 是否仍保持 `result_count=null`。
- candidate / observation / knowledge hit 是否被误当 FormalMemory。
- 跨 sidecar orphan / partial write 是否有 finding。

## 5. 分线职责

全局主管线：

- 冻结 K2.5 目标、边界、验收和回收口径。
- 维护入口文档只在 checkpoint 开始、完成、阻断时同步。
- 审查开发线是否绕过 Product Command、runtime log、audit、readback、memory candidate。

Execution 线：

- 处理 command surface、Product Command generic intent、runner write proof。
- 不接 planned adapters。
- 不执行真实 Codex，除非另有执行点授权。

Workflow 线：

- 处理 K3 之前的 `RunUnit -> ProductCommand` 主路径设计。
- 不继续把 J2-B bridge 当普通执行入口。

Memory 线：

- 处理跨 sidecar consistency scanner / finding taxonomy。
- 不自动写 FormalMemory。

Validation 线：

- 做架构扫描、越界扫描、fixture-only 扫描、误导文案扫描。
- 真实 Tauri / 截图仍留到 K6 或独立授权任务。

## 6. 开发顺序

推荐顺序：

1. 创建并执行 `K2.5 Architecture Calibration` 任务包。
2. 完成 K2.5-A / K2.5-F 的命令面扫描和分类。
3. 完成 K2.5-B / K2.5-C 的通用 intent 和 K3 主路径校准。
4. 完成 K2.5-D / K2.5-E 的 memory consistency 和 write proof 校准。
5. 通过 K2.5 acceptance gate 后再进入 K3。

## 7. 完成口径

K2.5 可接受为：

- Stage K 架构主路径完成校准。
- K3 可在不继承 probe / legacy / MCP 旁路风险的前提下继续。
- Product Command 仍是唯一真实执行归口。
- 工作流真实派发和记忆捕获有一致的 ref 链路和校验 gate。

K2.5 不接受为：

- K3 已完成。
- K4 记忆捕获体验已完成。
- K5 failure / retry / stop / restart 已完成。
- K6 真实 Tauri dogfood 已完成。
- 任意项目无限制自由控制台。
- planned adapters 真实接入。
- 自动 FormalMemory 写入。

## 8. 当前下一步

K2.5 已完成。当前下一步恢复为：

```text
进入 K3 项目工作流真实自动化编排产品化。
```
