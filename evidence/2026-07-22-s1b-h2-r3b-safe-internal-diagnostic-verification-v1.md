# S1B-H2-R3B 安全内部诊断闭环验证 v1

日期：2026-07-22（+0800）  
任务包：`tasks/2026-07-22-s1b-h2-r3b-safe-internal-diagnostic-package-v1.md`  
前置：`evidence/2026-07-22-s1b-h2-r3-canonical-to-supervisor-injection-diagnosis-v1.md`

状态：**代码与离线验证完成；未启动真实 App、未写真实 store，R4 现场验证尚未授权。**

## 结论

R3 的 `NEEDS_SAFE_INTERNAL_DIAGNOSTIC` 缺口已在既有 resident session 模块内收口：一条用户消息已经 canonical-recorded、但 resident consult 在 injected 前最终失败时，入口以既有 Batch 2 写路尽力追加一条 message-scoped 内部 audit diagnostic。它只记录闭合的结构化 `stage` / `stable_error_family` 与既有安全关联字段；原始错误只留在私有失败 envelope，绝不序列化。

诊断写失败被刻意忽略：已完成的业务 recorded 结果与既有 `message_recorded_supervisor_incomplete` 人话保持不变，不会重读、重试、rebase 或再跑 consult。相同 client/message 的 replay 不会增加诊断；不同 client 的独立动作仍分别记录。诊断 `target_ref` 不含 workflow id，因此不会进入用户 workflow ledger/read model。

## Gate 0：冻结与脏项

- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；staged 集为空。
- R3B 指定的八个冻结 SHA-256 均与开工前记录精确一致；本包仅在允许的 resident session 源/测试面实施，随后新增本 evidence 与最小 CURRENT 状态回写。
- 工作树原本已有不属于本包的脏项，未 reset、clean、stash、覆盖或归属它们；未触发 `BLOCKED_DIRTY_OVERLAP`。

## 最小实现与安全边界

- 私有 `SupervisorResidentConsultFailure` 在直接调用点携带结构化 stage；闭合 family 覆盖 preflight（reap/session/executable/plan/home/facts）、runner（output init/spawn/prepared lifecycle/registry/thread binding/terminal）和保守 `unknown`。新增 diagnostic 不按错误文本 substring 推断 family；既有 invalid-resume 检测保持原样。
- 已知 stdout/stdin、child wait/exit、turn completion 与 runner audit terminal 均直接归为 `runner_terminal`；thread-start/binding 归为 `thread_binding`。只有无法从真实调用点归属的泛型 runner failure 才保守为 `unknown`。
- submit 入口只在 recorded 已成功、consult 最终失败的既有分支中尝试一次追加。成功 injected/reply、watchdog 技术重试、invalid-resume 的单次轮转与进程组清理路径没有改变。
- 使用原有 `write_m5b_batch2_workflow_state`。没有新增 Tauri command、sidecar、MCP server、消息运输路或日志落盘面；H2 单工具批准、read-only/approval/sandbox 与 M5 写路均未修改。
- 失败注入只在 `cfg(test)` 的当前测试线程中生效；非测试构建的正常入口直接调用既有 Batch 2 writer，不保留可注入 writer 的生产路径。
- `event_id` 是既有 `audit_events` 的必需、确定性存储主键（Batch 2 DB/JSON 投影以它对账），不是原始错误 payload 或新的 transport/schema 面。

无原文 schema 示例：

```text
event_id = supervisor-resident-delivery-diagnostic:<stable-id>
event_type = supervisor_resident_delivery_diagnostic_recorded
target_ref = supervisor-resident-delivery-diagnostic:<message-id>
project_id, workflow_id, message_id, run_id
generation / thread_id（仅已知时）
stage = preflight | runner | unknown
stable_error_family = <closed enum member>
created_at
```

诊断对象不含用户正文、原始 `Err`、stderr、路径、命令行、token、auth、`CODEX_HOME` 或 MCP 参数。测试对对象字段作精确白名单断言，并证明受控失败标记不会进入 canonical JSON 或 workflow ledger。

## 先红后绿与定向验证

| 检查 | 结果 |
| --- | --- |
| 新夹具先红：`cargo test --offline --lib s1b_h2_delivery_diagnostic_ -- --nocapture` | 实现前 4 个失败断言均确认 diagnostic 数为 0，准确复现可观测性缺口。 |
| 新夹具转绿（同命令） | **5 passed, 0 failed**：preflight、output-init、prepared lifecycle、同 client 幂等/不同 client 分立、Batch 2 append 失败。 |
| `cargo test --offline --lib s1b_ -- --nocapture` | **32 passed, 0 failed, 1 ignored**；忽略项是既有授权 live 用例。常规隔离环境对受控子进程 PID 可见性受限，使用受控系统进程可见性复跑后全绿；没有启动真实 App。 |
| `cargo test --offline --lib s1_resident_submit_proposal_ -- --nocapture` | **3 passed, 0 failed**。 |
| `cargo test --offline --lib m5b -- --nocapture` | **10 passed, 0 failed**。 |
| `cargo test --offline --lib m5c -- --nocapture` | **5 passed, 0 failed**。 |
| `cargo test --offline --lib m5f1 -- --nocapture` | **3 passed, 0 failed**。 |
| 用户 read-model 隐私回归 | `s1b_h2_private_tool_arguments_do_not_enter_the_ordinary_supervisor_read_model`：**1 passed**。 |
| `cargo check --offline --lib`、`rustfmt --edition 2021 --check` | 均通过；仅有既有编译 warnings。 |
| 前端 `npm run typecheck`、`npm run test:offline-interaction` | 均通过。 |

## Shape、私密与变更审计

- Shape baseline 与 check 的 finding 完全相同：`13 error / 5 warning / 5 info`，共 23 项；**零净增**。check 的非零退出是既有 shape 债务，不把它表述为绝对全绿。
- `git diff --check` 通过；staged 仍为空。
- 静态审计确认诊断 JSON 构造只使用上列安全字段和既有 audit 主键；`raw_detail` 不在该构造中。测试还覆盖 diagnostic append 失败不产生 partial event、不重跑 consult。
- 全部 fixture 是离线临时状态与受控子进程；未读取或写入真实 workflow-state、DB/WAL/SHM、`.codex`、测试项目文件或私有 runner 正文。

## 未执行的真实现场项与 R4 前置条件

本包**不证明** H2 真实 App 的消息输送、主管自然回复、MCP handler、工具结果同 thread 或 PendingUserConfirmation 落卡。它也没有发送首句/第二句、启动 App、操作真实 store、点击卡、启动 chain 或派发 worker。

R4 必须另出现场包、另获用户在场授权，并至少先完成新的 Gate 0：App/holder 空、当前源码与 debug binary 冻结、真实 JSON/DB/WAL/SHM 及 proposal/Pending/chain/generation/thread/registry/项目 hash 基线。现场仅在首句确实 canonical-recorded 且主管链正常回复时才可继续第二句；验收仅限一张目标匹配 Pending 卡、工具结果和主管回复同 thread、刷新不重复、chain 与项目不变，绝不批准卡或启动 chain。若再出现 recorded 后未完成，R4 应读取本 diagnostic 的稳定 family，不得从用户面或私密原文猜测根因。
