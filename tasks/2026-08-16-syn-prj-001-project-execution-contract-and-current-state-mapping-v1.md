# SYN-PRJ-001: 项目执行合同与现状映射

日期: 2026-08-16
阶段: M5 (Stage-14)
状态: CURRENT / LEAF
优先级: HIGH

## 目标

冻结 M5 全链对象自身 ID、canonical orchestration/correlation identity、WorkflowRunId 分配、PreparedAttempt 状态机、owner、restart points、两类 user decision、Grant 签发 / readback / revoke / dispatch 顺序与 principal、完整 grant 字段、分型 report kind / trusted actor、两类 gateway、ProjectSummary query port 和旧对象 mapping。

## 范围

### 1. Orchestration Identity 合同

全链稳定的编排身份标识：
- correlation_id: 关联 ID，贯穿整个编排生命周期
- orchestration_id: 编排实例 ID，唯一标识一次编排
- workflow_run_id: 工作流运行 ID

### 2. PreparedAttempt 状态机

状态: Prepared -> Runnable -> Running -> Completed/Failed/Cancelled

### 3. ExecutionGrantGateway 与 ConversationCapabilityGateway 分离

- ExecutionGrantGateway: 控制副作用命令
- ConversationCapabilityGateway: 控制只读访问和提案提交

### 4. Grant 签发顺序（严格）

1. AuthorizationDecision (用户确认)
2. 创建 Run / WorkItem + worker RoleSession binding
3. 创建 PreparedAttempt (稳定 AttemptId, 不可执行)
4. Mint attempt-scoped ExecutionGrant
5. Grant 持久化 + readback 通过
6. 创建 Dispatch
7. 把 Attempt 推进到 Runnable
8. 经 outbox 启动

### 5. WorkerReport 分型

- Execution: 真实执行后的回程报告
- Manual: 手动粘贴的离线报告
- Offline: 完全离线的手动输入

### 6. ProjectSummary Query Port

最小、只读、不可反写的项目摘要

## 当前代码映射

| M5 对象 | 现有代码位置 | 状态 |
|---------|-------------|------|
| ExecutionGrant | mcp/execution_grant.rs | 已存在，需扩展 |
| WorkerReport | worker_report.rs | 已存在，需分型 |
| PlanAuthorization | plan_authorization_store.rs | 已存在 |
| RoleSession | m3_role_session.rs | 已存在 |
| Dispatch | workflow_run_dispatch_entrypoints.rs | 已存在 |
| ProjectSummary | 不存在 | 需新建 |
| PreparedAttempt | 不存在 | 需新建 |
| OrchestrationIdentity | 不存在 | 需新建 |

## 验收标准

1. 所有对象 ID 类型冻结，不再使用裸 String
2. Grant 签发顺序文档化并有测试覆盖
3. PreparedAttempt 状态机实现并有单元测试
4. ExecutionGrantGateway 和 ConversationCapabilityGateway 接口定义
5. ProjectSummaryQueryPort 接口定义
6. WorkerReport 分型完成

## 不做

- 不实现真实执行
- 不退役旧 command/store
- 不重做 UI 布局
- 不接入真实 Codex/Runner
