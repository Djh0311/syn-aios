# SYN-DAT-001 迁移清单 v1

**创建日期**: 2026-08-03
**状态**: FROZEN_V1
**依赖**: syn-dat-001-mechanism-contract-v1.md

## 1. 迁移概述

本文档基于 M1 合同和 SYN-DAT-001 机制合同，为 M2 阶段提供逐域迁移清单。每个 domain 的迁移状态、下一步、持有项和验证要求。

## 2. Reference Slice 迁移清单

### 2.1 Workflow Domain (reference_slice_id: workflow-state-sidecar)

**迁移状态**: READY_FOR_DAT-003
**下一步**: DAT-003 (首个 vertical slice)
**持有项**: HOLD-DB-JSON-RUNTIME-TRUTH, HOLD-UNKNOWN-QUARANTINE-STORE

**迁移清单**:
- [ ] 创建 command_receipts 表 (schema version: 1)
- [ ] 创建 events 表 (schema version: 1)
- [ ] 创建 audit_records 表 (schema version: 1)
- [ ] 创建 outbox_items 表 (schema version: 1)
- [ ] 创建 current_snapshots 表 (schema version: 1)
- [ ] 创建 projection_checkpoints 表 (schema version: 1)
- [ ] 实现 UoW 协调器
- [ ] 实现 CommandReceiptRepository
- [ ] 实现 EventLedgerRepository
- [ ] 实现 AuditLedgerRepository
- [ ] 实现 OutboxRepository
- [ ] 实现 CurrentSnapshotRepository
- [ ] 实现 ProjectionCheckpointRepository
- [ ] 实现 workflow_state aggregate
- [ ] 实现 update_work_item_state command
- [ ] 实现 workflow projector
- [ ] 创建 temp DB 测试
- [ ] 运行 cargo check --lib (non-test build)
- [ ] 运行 cargo test --lib (unit tests)
- [ ] 验证 atomicity guarantee
- [ ] 验证 single writer rule
- [ ] 验证 receipt persistence

**验证要求**:
1. 所有表 schema 与 SYN-DAT-001 一致
2. 所有状态机转换正确
3. 所有唯一约束和外键约束正确
4. 所有索引正确创建
5. UoW atomicity 测试通过
6. Single writer rule 测试通过
7. Receipt persistence 测试通过

### 2.2 Conversation Domain

**迁移状态**: HOLD
**下一步**: M3 (RoleSession 实现)
**持有项**: HOLD-CROSS-SCOPE-ROLE-MAPPING, HOLD-RAW-TRANSCRIPT-RETENTION

**迁移清单**:
- [ ] 定义 RoleSession aggregate
- [ ] 定义 Turn command
- [ ] 定义 conversation transport adapter
- [ ] 定义 scrubbed transcript projection
- [ ] 创建 temp DB 测试
- [ ] 验证 conversation 事务边界

**验证要求**:
1. RoleSession 与 SYN-DAT-001 机制合同一致
2. Turn command 与 CommandReceipt 状态机一致
3. Scrubbed transcript projection 与 AuditRecord 一致
4. Conversation 事务边界正确

### 2.3 Memory Domain

**迁移状态**: HOLD
**下一步**: M7 (Memory governance 实现)
**持有项**: HOLD-MEMORY-PROMOTION-POLICY, HOLD-RAW-TRANSCRIPT-RETENTION

**迁移清单**:
- [ ] 定义 MemoryCandidate aggregate
- [ ] 定义 Capture/Observation commands
- [ ] 定义 memory governance adapter
- [ ] 定义 scrubbed memory projection
- [ ] 创建 temp DB 测试
- [ ] 验证 memory 事务边界

**验证要求**:
1. MemoryCandidate 与 SYN-DAT-001 机制合同一致
2. Capture/Observation commands 与 CommandReceipt 状态机一致
3. Scrubbed memory projection 与 AuditRecord 一致
4. Memory 事务边界正确

### 2.4 Knowledge Domain

**迁移状态**: HOLD
**下一步**: M8 (Knowledge adapter 实现)
**持有项**: HOLD-PATH-REALPATH-SYMLINK, HOLD-OBJECT-EXTERNAL-URI

**迁移清单**:
- [ ] 定义 KnowledgeIndex aggregate
- [ ] 定义 FileOperation commands
- [ ] 定义 knowledge adapter
- [ ] 定义 rebuildable knowledge projection
- [ ] 创建 temp DB 测试
- [ ] 验证 knowledge 事务边界

**验证要求**:
1. KnowledgeIndex 与 SYN-DAT-001 机制合同一致
2. FileOperation commands 与 CommandReceipt 状态机一致
3. Rebuildable knowledge projection 与 CurrentSnapshot 一致
4. Knowledge 事务边界正确

## 3. 公共 Ports 迁移清单

### 3.1 Repository Ports

**迁移状态**: READY_FOR_DAT-002
**下一步**: DAT-002 (Additive schema + repository ports)

**迁移清单**:
- [ ] CommandReceiptRepository port
- [ ] EventLedgerRepository port
- [ ] AuditLedgerRepository port
- [ ] OutboxRepository port
- [ ] CurrentSnapshotRepository port
- [ ] ProjectionCheckpointRepository port
- [ ] UnitOfWork port
- [ ] Projector port

### 3.2 Schema Ports

**迁移状态**: READY_FOR_DAT-002
**下一步**: DAT-002 (Additive schema + repository ports)

**迁移清单**:
- [ ] CommandReceipt schema
- [ ] WorkbenchEventEnvelope schema
- [ ] AuditRecord schema
- [ ] OutboxItem schema
- [ ] OutboxLease schema
- [ ] CurrentSnapshot schema
- [ ] ProjectionCheckpoint schema
- [ ] UnknownQuarantineRef schema

### 3.3 Receipt Ports

**迁移状态**: READY_FOR_DAT-002
**下一步**: DAT-002 (Additive schema + repository ports)

**迁移清单**:
- [ ] CommandReceipt generation
- [ ] CommandReceipt persistence
- [ ] CommandReceipt recovery
- [ ] CommandReceipt audit

## 4. 禁止字段迁移清单

### 4.1 Payload Storage

**迁移状态**: FROZEN
**验证状态**: 已在 SYN-DAT-001 机制合同中冻结

**禁止字段**:
- raw_transcript
- prompt_content
- tool_output
- secret_value
- credential_token
- provider_response_full
- stdout_content
- stderr_content

### 4.2 Scrub Rules

**迁移状态**: FROZEN
**验证状态**: 已在 SYN-DAT-001 机制合同中冻结

**Scrub 触发器**:
- sensitive_material_detected
- raw_content_in_event
- credential_in_payload
- secret_in_audit

**Scrub 动作**:
- REPLACE_WITH_REFERENCE
- OMIT_FIELD
- REDACT_CONTENT

## 5. M1 残留项迁移清单

### 5.1 Grant 校验

**迁移状态**: HOLD
**下一步**: DAT-002/003 (建真 grant mint/load/verify)
**持有项**: HOLD-EXECUTION-GRANT-PERSISTENCE

**迁移清单**:
- [ ] 创建 grant store
- [ ] 实现 grant mint
- [ ] 实现 grant load
- [ ] 实现 grant verify
- [ ] 创建 temp DB 测试
- [ ] 验证 grant 与 CommandReceipt 一致

### 5.2 FND-006 场景 3/4

**迁移状态**: HOLD
**下一步**: DAT-008 (隔离 App 验收)
**持有项**: HOLD-UNKNOWN-QUARANTINE-STORE

**迁移清单**:
- [ ] 创建 fake runner 夹具
- [ ] 验证 伪造 report 全链运行时
- [ ] 验证 伪造 grant 全链运行时
- [ ] 验证 quarantine 语义

### 5.3 FND-006 场景 5

**迁移状态**: HOLD
**下一步**: DAT-008 (隔离 App 验收) 或 M3 (supervisor 会话机制)
**持有项**: HOLD-CROSS-SCOPE-ROLE-MAPPING

**迁移清单**:
- [ ] 创建 supervisor 会话夹具
- [ ] 验证 Station 3b 写入拒绝运行时
- [ ] 验证 supervisor 会话语义

### 5.4 sqlite_production_preflight 稳定失败

**迁移状态**: HOLD
**下一步**: DAT-002 期间定性修复

**迁移清单**:
- [ ] 分析 preflight 失败原因
- [ ] 修复 preflight 逻辑
- [ ] 验证 preflight 正确拦截

### 5.5 进程夹具族环境性失败

**迁移状态**: HOLD
**下一步**: DAT-002 期间并案排查

**迁移清单**:
- [ ] 分析 codex_local_runner 失败原因
- [ ] 分析 obsidian 失败原因
- [ ] 分析 manual_relay 失败原因
- [ ] 修复环境性问题
- [ ] 验证所有进程夹具稳定

### 5.6 code-map advisory

**迁移状态**: HOLD
**下一步**: 首个 DAT 提交批顺手处理

**迁移清单**:
- [ ] 更新 code-map
- [ ] 添加新模块能力映射
- [ ] 修复 index.json invalid domain path

## 6. 迁移验证矩阵

### 6.1 验证层级

| 层级 | 必须证明 | 不能声称 |
|---|---|---|
| Contract/schema lint | owner、FK、状态、禁止字段、migration 顺序一致 | repository 已正确实现 |
| Unit/property | UoW、幂等、scrub、lease、projector 确定性 | 生产入口全接入 |
| Temp SQLite/fixture | rollback、crash point、parity、quarantine、重建 | live store 已迁移 |
| Non-test build | production path 可构建 | App 行为正确 |
| Isolated Tauri | scratch store 冷启动/强退/重启/恢复可见 | 真实数据、provider 或发布通过 |
| 经授权 live migration | 精确 domain 的真实 before/after/parity/rollback | 其他 domain 或全工作台已切换 |

### 6.2 关键机械断言

1. commit 前任一点失败全部回滚
2. commit 后重试不重复外部动作
3. 投影失败有 durable receipt
4. raw JSON 默认不进入产品 DTO
5. 旧/new count、key、canonical hash 可解释

## 7. 迁移退出门

全部满足才允许将 M3 设为 current：

- [ ] 同一具名 reference slice / domain 完整通过 UoW、denial audit、current snapshot、outbox、projector、shadow、parity、recovery
- [ ] 公共 ports、schema、receipt 和禁止字段冻结，所有消费方版本可追踪
- [ ] 每个已触及 domain 有 exact migration state，其余明确 `not-migrated / HOLD`
- [ ] 隔离 App 崩溃 / 重启证据通过，结论未越级到真实 store
- [ ] 旧数据未被物理删除，rollback / export 可执行
- [ ] CURRENT 回写实际完成、证据、HOLD 和下一阶段
- [ ] 用户显式激活 M3 前不得自动进入角色会话实现

## 8. 迁移清单冻结声明

本文档冻结了 M2 阶段迁移清单。所有迁移状态、下一步、持有项和验证要求已冻结，可供 DAT-002—008 实现使用。本文档不授权生产 schema 修改、真实数据迁移或产品代码变更。

**冻结日期**: 2026-08-03
**冻结者**: M2 执行线
**验证状态**: 静态清单冻结，待 DAT-002 实现验证
