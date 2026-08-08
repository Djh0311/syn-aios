---
contract_id: syn-dat-001b-live-manifest-preflight-v1
version: 1
status: FROZEN_V1
evidence_level: STATIC_OPENING_ONLY
schema_authority: m2_transaction_foundation_authority
dependencies: ["syn-dat-001-mechanism-contract-v1"]
hold_refs: ["HOLD-DB-JSON-RUNTIME-TRUTH", "HOLD-UNKNOWN-QUARANTINE-STORE"]
---

# SYN-DAT-001B: 只读 Live-Manifest Preflight v1

## 合同概述

本文档定义 DAT-007 逐域真实切换的前置条件：只读 live-manifest preflight。它列出 exact roots、数据等级、只读方法、允许保留的 value-free count/key/hash、敏感 material 停止路线和零 mutation 证明。

## 1. Preflight 范围

本文档覆盖 workflow domain 的 live-manifest preflight，因为 workflow domain 是 reference_slice_id `workflow-state-sidecar` 的宿主 domain。

## 2. Exact Roots

### 2.1 Workflow Domain Roots

| Root | Physical Path | Data Level | Read-Only Method |
|---|---|---|---|
| `workflow-state.v0.json` | `$HOME/.syn/workflow-state.v0.json` | INTERNAL_DOMAIN_STATE | `std::fs::read_to_string` |
| `workflow-state.v0.lock` | `$HOME/.syn/workflow-state.v0.lock` | INTERNAL_METADATA | `std::fs::metadata` |
| `workbench.sqlite` | `$HOME/.syn/workbench.sqlite` | INTERNAL_DOMAIN_STATE | `rusqlite::Connection::open_with_flags(OPEN_READ_ONLY)` |

### 2.2 Workflow Domain Data Level Classification

| Data Level | Description | Disposition |
|---|---|---|
| `INTERNAL_DOMAIN_STATE` | 核心业务状态，如 workflow state、work items | KEEP, shadow/parity |
| `INTERNAL_METADATA` | 元数据，如 lock files、migration records | KEEP, no cutover |
| `RESTRICTED_AUDIT` | 审计记录，如 workflow audit events | KEEP, shadow/parity |
| `RESTRICTED_EXECUTION_AUDIT` | 执行审计，如 runtime logs | KEEP, shadow/parity |
| `RESTRICTED_LEGACY_RUNTIME_STATE` | 遗留运行时状态，如 execution attempts | HOLD, no cutover |

## 3. Read-Only Methods

### 3.1 JSON Sidecar Read-Only

```json json-sidecar-read-only
{
  "method": "std::fs::read_to_string",
  "path": "$HOME/.syn/workflow-state.v0.json",
  "flags": "none",
  "mutation_proof": "read_to_string does not modify file",
  "concurrency": "safe for concurrent reads",
  "error_handling": "file_not_found returns None, parse_error returns Err"
}
```

### 3.2 SQLite Read-Only

```json sqlite-read-only
{
  "method": "rusqlite::Connection::open_with_flags(OPEN_READ_ONLY)",
  "path": "$HOME/.syn/workbench.sqlite",
  "flags": "SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_FULLMUTEX",
  "pragma": "PRAGMA query_only = ON",
  "mutation_proof": "connection opened with read-only flag",
  "concurrency": "safe for concurrent reads with FULLMUTEX",
  "error_handling": "file_not_found returns Err, locked returns Err"
}
```

### 3.3 Lock File Read-Only

```json lock-file-read-only
{
  "method": "std::fs::metadata",
  "path": "$HOME/.syn/workflow-state.v0.lock",
  "flags": "none",
  "mutation_proof": "metadata does not modify file",
  "concurrency": "safe for concurrent reads",
  "error_handling": "file_not_found returns None"
}
```

## 4. Value-Free Count/Key/Hash

### 4.1 允许保留的 Value-Free 数据

| Type | Example | Preservation |
|---|---|---|
| `count` | work_item_count: 13 | 允许保留 |
| `key` | workflow_id: "wf-001" | 允许保留 |
| `hash` | record_hash: "sha256:abc..." | 允许保留 |
| `revision` | revision: 5 | 允许保留 |
| `watermark` | source_watermark: "evt-001" | 允许保留 |

### 4.2 禁止保留的 Value 数据

| Type | Example | Disposition |
|---|---|---|
| `raw_content` | work_item_state_json: "{...}" | SCRUB_AND_STOP_BEFORE_ORDINARY_STORE |
| `secret` | api_token: "sk-..." | SCRUB_AND_STOP_BEFORE_ORDINARY_STORE |
| `credential` | password: "..." | SCRUB_AND_STOP_BEFORE_ORDINARY_STORE |
| `transcript` | full_transcript: "..." | SCRUB_AND_STOP_BEFORE_ORDINARY_STORE |
| `prompt` | prompt_body: "..." | SCRUB_AND_STOP_BEFORE_ORDINARY_STORE |

## 5. 敏感 Material 停止路线

### 5.1 敏感字段检测

```json sensitive-field-detection
{
  "field_patterns": [
    ".*token.*",
    ".*secret.*",
    ".*password.*",
    ".*credential.*",
    ".*api_key.*",
    ".*auth.*"
  ],
  "action": "SCRUB_AND_STOP_BEFORE_ORDINARY_STORE",
  "preservation": "reference_only_no_original_values"
}
```

### 5.2 停止路线

```json sensitive-material-stop-route
{
  "detection_point": "before_persistence",
  "action": "replace_with_hash_reference",
  "audit": "scrub_result_recorded_in_audit",
  "recovery": "requires_manual_reclassification"
}
```

## 6. 零 Mutation 证明

### 6.1 Read-Only 操作证明

| Operation | Mutation | Proof |
|---|---|---|
| `std::fs::read_to_string` | None | 函数签名只接受 &Path，返回 Result<String> |
| `rusqlite::Connection::open_with_flags(OPEN_READ_ONLY)` | None | SQLite flags 包含 SQLITE_OPEN_READ_ONLY |
| `std::fs::metadata` | None | 函数签名只接受 &Path，返回 Result<Metadata> |

### 6.2 并发安全证明

| Operation | Concurrency | Proof |
|---|---|---|
| JSON read | Safe | 多个读者可以同时读取同一文件 |
| SQLite read-only | Safe | SQLITE_OPEN_FULLMUTEX 确保线程安全 |
| Metadata read | Safe | 多个读者可以同时读取同一文件 |

### 6.3 Error Handling 证明

| Error | Handling | Recovery |
|---|---|---|
| File not found | Return None | Skip domain, log warning |
| Parse error | Return Err | Skip domain, log error |
| Permission denied | Return Err | Skip domain, log error |
| SQLite locked | Return Err | Retry with backoff |

## 7. Preflight 验证步骤

### 7.1 验证 JSON Sidecar

1. 检查文件是否存在
2. 读取文件内容
3. 解析 JSON
4. 验证 schema version
5. 验证 required fields
6. 检查敏感字段
7. 记录 count/key/hash

### 7.2 验证 SQLite

1. 检查文件是否存在
2. 以 read-only 模式打开
3. 验证 schema version
4. 查询 table counts
5. 查询 sample records
6. 检查敏感字段
7. 记录 count/key/hash

### 7.3 验证 Lock File

1. 检查文件是否存在
2. 读取 metadata
3. 验证 lock status
4. 记录 lock information

## 8. Preflight 输出

### 8.1 Preflight Report

```json preflight-report
{
  "domain": "workflow",
  "reference_slice_id": "workflow-state-sidecar",
  "status": "passed|failed|warning",
  "roots": {
    "json_sidecar": {
      "path": "$HOME/.syn/workflow-state.v0.json",
      "exists": true,
      "read_only_method": "std::fs::read_to_string",
      "mutation_proof": "read_to_string does not modify file",
      "count": 13,
      "key": "workflow_id",
      "hash": "sha256:abc..."
    },
    "sqlite": {
      "path": "$HOME/.syn/workbench.sqlite",
      "exists": true,
      "read_only_method": "rusqlite::Connection::open_with_flags(OPEN_READ_ONLY)",
      "mutation_proof": "connection opened with read-only flag",
      "count": 68,
      "key": "table_name",
      "hash": "sha256:def..."
    },
    "lock_file": {
      "path": "$HOME/.syn/workflow-state.v0.lock",
      "exists": false,
      "read_only_method": "std::fs::metadata",
      "mutation_proof": "metadata does not modify file"
    }
  },
  "sensitive_fields": [],
  "warnings": [],
  "zero_mutation_proof": "all operations are read-only"
}
```

## 9. Preflight 冻结声明

本文档冻结了 workflow domain 的 live-manifest preflight。所有 exact roots、数据等级、只读方法、value-free count/key/hash、敏感 material 停止路线和零 mutation 证明已冻结，可供 DAT-007 实现使用。本文档不授权生产 schema 修改、真实数据迁移或产品代码变更。

**冻结日期**: 2026-08-03
**冻结者**: M2 执行线
**验证状态**: 静态 preflight 冻结，待 DAT-007 实现验证
