# 共享主管 Conversation Binding 阶段语义与失败收口返工 v1

日期：2026-07-23  
状态：离线实现与定向验证完成；真实 App、真实 store 和生产验收仍未授权

## 目标

纠正前序建立链把失败过度折叠为四阶段的口径，并把已建立 binding 的启动失败收口为可验证、失败关闭的状态机。

本包只允许修改本地源码、测试夹具和临时目录。禁止启动真实 App，禁止读取或修改真实 store，禁止 stage/commit；不将任何离线结果推导为真实 App 首句根因或产品验收通过。

## 当前阶段表

| 阶段 | 唯一语义 | receipt 可以说什么 | 不能说什么 |
| --- | --- | --- | --- |
| `binding_construct` | context、身份、run 或 binding 形状不成立 | binding 未准备好 | 已持久化或已启动 |
| `binding_store_prepare` | workflow/sidecar/repository/lock/load 等 durable write 前准备失败 | 存储未准备完成 | DB delta 已提交 |
| `binding_persist_db` | DB-primary delta 未提交 | 主存储未写入 | JSON 已投影 |
| `binding_project_json` | DB-primary 已领先而 JSON projection 未完成 | 兼容投影未完成 | JSON 已一致 |
| `binding_activate` | 宿主观察到 thread 后，binding 激活失败 | binding 未激活，工具继续关闭 | transport 是否已经启动、binding 是否已终结 |
| `transport_start` | transport 启动返回失败，且 Failed 终结已写入并从 sidecar 复核 | transport 未启动，binding 已安全收口 | 更多运行时细节 |
| `binding_terminate` | transport/activation 后的终结写入或终结复核失败 | 终结未确认，工具继续关闭 | binding 已终结或 lifecycle=`Failed` |

`binding_activate` 是前端 allowlist 的固定枚举；未知阶段和未知 thrown error 一律降为通用安全文案，不能进入共享状态。

## 最小实现合同

- 建立 writer 把 store 准备、DB 持久化、JSON projection 和 binding 构造冲突分开映射；测试注入仅编译于 `cfg(test)`。
- activation 失败不再误标为 `binding_persist_db`。无论它发生在 existing-thread 前置激活还是 host receipt 的 thread 观察激活，均先尝试持久化 `Failed`，成功后才返回 `binding_activate`。
- transport start 返回错误后，命令先写入 `Failed`，再从 durable JSON sidecar 读取 lifecycle 复核；任一步失败只返回 `binding_terminate`。
- 终结尝试前设置进程内 fail-closed guard。它只关闭该共享 run 的 `tools/list` 和 `tools/call`，不写入或伪造 lifecycle；成功的 durable terminal lifecycle 可自行承担关闭，写入或复核未确认时 guard 保留，新的 durable binding 也会清理旧 guard。
- 安全 receipt 一律不带 conversation/thread/attempt/reply/tool/proposal/chain/worker 结果，也不带路径、argv、stderr、环境、原始用户文本或内部错误串。

## 定向离线验收

| 注入 | 阶段/receipt | DB/JSON lifecycle 断言 | MCP 工具面 |
| --- | --- | --- | --- |
| store 准备失败 | `binding_store_prepare` | 无新增 binding | `tools/list` 空，`tools/call` 拒绝 |
| activation 失败 | `binding_activate` | 两端均为 `Failed` | 两者关闭 |
| transport 启动失败 | `transport_start` | 两端均为 `Failed` | 两者关闭 |
| termination 失败 | `binding_terminate` | 两端保留原 `Active`，不伪称 Failed | 两者仍关闭 |

完整命令与结果见 [验证证据](../evidence/2026-07-23-shared-supervisor-conversation-binding-phase-semantics-and-failure-closure-rework-verification-v1.md)。其中 DB/JSON 断言全部使用临时 fixture；没有使用真实副本或真实 store。

## 明确不声称

- 不声称真实 App、真实对话、真实 binding、reply、proposal、card、chain 或 worker 已成功。
- 不声称真实 App 首句失败根因已定位或修复。
- 不声称 DB-primary JSON projection 历史不一致已消失；`binding_project_json` 仍是明确失败面。
- 不声称 shape gate 全绿。历史 shape debt 单列记录，不能被本次定向通过掩盖。

## 与前序包的关系

[建立链离线诊断与最小修复 v1](2026-07-23-shared-supervisor-conversation-binding-establishment-repair-package-v1.md) 和其验证证据保留为历史部分记录：其私有副本 `25/0 → 26/1 Starting` 观察仍有效，但“四阶段已完整覆盖”和“transport failure 已终结”的表述已被本包替代。

## 下一步（需新授权）

若要继续，只能另开真实 App 验收包：先冻结 build、进程与真实 store，再按既有停止合同采集单次首句的安全 `binding_stage` 与已授权计数。失败不得自动重试或发送第二、第三句。
