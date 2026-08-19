# F2C01R02 四个 v1 空数组约束补执行

本文件是 stage-16 第三轮单阻断项返修的四段报告正本。不构成 stage-16 关闭。独立复核 verdict 须由总指导另派，范围只限本阻断项 + 回归。

完整请求/响应 JSON 另存：

- 修复前：`docs/harness/reports/F2C01R02-before-pairs.json`（探针 `/tmp/f2c01r02-before-1787154585`）
- 修复后：`docs/harness/reports/F2C01R02-after-pairs.json`（探针 `/tmp/f2c01r02-after-1787156542`）

## Harness

本轮由总指导 kickoff「F2C01R02 返修 Kickoff」明确开始。已立唯一 current leaf `F2C01R02`，authorization 保持 `{"schemaVersion":1,"authorized":false}`。叶在完成标准全部真实通过后原子归档到 `docs/harness/done/2026-08/F2C01R02-v1-empty-array-enforcement.md`，当前无 current leaf。未改 stage-15、未关闭 stage-16、未 push、未设 `SYN_R4_ACCEPTANCE_PROFILE`、未写 syn-shell。

写面限于预声明路径：桥源码、合同两行错误码 + 样例注记、fixture +2 BEHAVIOR case、plan / current-state / stage-16 / audit / 本报告与 pairs JSON。`register_for_state`、`m6_org_*`、`commands.rs`、AppState 可见性、`manifest.v1.json` 均为零 diff。

后续事项（不属本轮）：syn-shell 类型镜像仍钉旧合同 SHA `d10f00447f980ede6381981e7270a5de854198ad4c9c6cb945abf0a252a1f3a9`；本叶改了合同两行错误码后该 SHA 将变化，须由总指导另派壳侧同步，本轮不许动 syn-shell。

## 产品

`dispatch_register_stable_member` 在派发 `register_for_state` 之前，对 v1 四个数组 `scope_assignments` / `role_assignments` / `capability_permission_refs` / `contact_bindings` 做显式空校验。任一非空即 fail-closed 返回已登记码 `F2_FORBIDDEN_AUTHORITY_INPUT`；被拒写不进入核心，因此不留幂等记录。未新登记错误码：capability 是权限面、contact_bindings 是联系授权面，与既有两个数组同码一致。

合同只把那两行的错误列从 “not in the proven success domain” 改为 `F2_FORBIDDEN_AUTHORITY_INPUT`，并在 register-params 样例下补一句注记。`v1 requires []` 正文未改，这是补执行不是改约。

fixture 增加 `CF-F2-NEG-019` / `CF-F2-NEG-020`，分别由 `f2c01r02_nonempty_capability_permission_refs_are_forbidden` 与 `f2c01r02_nonempty_contact_bindings_are_forbidden` 断言；测试体含 case id。

## 证据

覆盖统计（`node docs/contracts/fixtures/f2-bridge-001/coverage-audit.cjs`，exit 0）：

```json
{
  "cases": 28,
  "behavior": 25,
  "document": 3,
  "covered_with_precise_assertion": 28,
  "required_keys_only_does_not_count": true,
  "missing": [],
  "percent": "100.0%"
}
```

定向测试：`CARGO_TARGET_DIR=/tmp/f2c01r02-target-b33038e cargo test --lib f2c01 --offline -- --test-threads=1` exit 0，**19 passed / 0 failed / 0 ignored**（原 17，本轮 +2）。

`cargo check --offline` exit 0，rustc 汇总 **888 warnings，F2 新增 0**。

`rustfmt --edition 2021 --check` 该 rs 文件 exit 0；`git diff --check` exit 0。

### 修复前真进程反例（HEAD 二进制，未改码）

二进制 `/tmp/f2-shell-core-b33038e/debug/codex-governance-workbench` sha256 `39d544ca8dccf0c8c8c6e5101a5e76a790d01d1d6ec560ce72cd48a41f30c351`。全新空根 `/tmp/f2c01r02-before-1787154585/local.codex.governance.workbench`。两反例均被接受并落库。

**capability 非空 — 请求**

```json
{
  "schema_version": "syn.f2.shell-core-bridge.request.v1",
  "request_id": "probe:register-cap",
  "method": "organization.register_stable_member",
  "params": {
    "member_id": "member_f2c01r02_before_cap",
    "display_name_ref": "display-name:member_f2c01r02_before_cap",
    "identity_evidence": {
      "kind": "EXPLICIT_IDENTITY_CONTRACT",
      "contract_kind": "syn.m6.org.stable-member-identity/v1",
      "identity_contract_ref": "identity-contract:member_f2c01r02_before_cap",
      "source_record_ref": "identity-source:member_f2c01r02_before_cap",
      "source_revision": 1,
      "observed_at": 1700000000000,
      "explicit_human_command": true
    },
    "scope_assignments": [],
    "role_assignments": [],
    "capability_permission_refs": [{
      "ref_id": "capability:research",
      "subject_member_id": "member_f2c01r02_before_cap",
      "kind": "capability",
      "source": "policy-owner:fixture",
      "revision": 1,
      "observed_at": 1700000000000,
      "directory_is_authority": false,
      "read_only": true
    }],
    "memory_refs": [],
    "contact_bindings": [],
    "idempotency_key": "register-f2c01r02-before-cap"
  },
  "external_refs": [
    {"kind": "thread_id", "value": "shell-thread-opaque"},
    {"kind": "desktop_id", "value": "shell-desktop-opaque"},
    {"kind": "pairing_id", "value": "shell-pairing-opaque"}
  ],
  "deadline_unix_ms": 1787154590540
}
```

**capability 非空 — 响应（修复前：成功并持久化）**

```json
{
  "schema_version": "syn.f2.shell-core-bridge.response.v1",
  "request_id": "probe:register-cap",
  "method": "organization.register_stable_member",
  "ok": true,
  "code": "F2_OK",
  "result": {
    "result_kind": "stable_member_registration",
    "payload": {
      "disposition": "REGISTERED",
      "member": {
        "member_id": "member_f2c01r02_before_cap",
        "membership_lifecycle": "ESTABLISHED",
        "scope_assignments": [],
        "role_assignments": [],
        "capability_permission_refs": [{
          "ref_id": "capability:research",
          "subject_member_id": "member_f2c01r02_before_cap",
          "kind": "capability",
          "source": "policy-owner:fixture",
          "revision": 1,
          "observed_at": 1700000000000,
          "directory_is_authority": false,
          "read_only": true
        }],
        "availability_ref": null,
        "contact_binding_refs": [],
        "contact_bindings": [],
        "memory_refs": [],
        "promoted_from": null,
        "display_name_ref": "display-name:member_f2c01r02_before_cap",
        "identity_contract_ref": "identity-contract:member_f2c01r02_before_cap",
        "identity_source_record_ref": "identity-source:member_f2c01r02_before_cap",
        "identity_source_revision": 1,
        "revision": 1,
        "created_at": 1787154587531,
        "deactivated_at": null
      },
      "quarantine": null,
      "replayed": false,
      "directory_is_authority": false
    }
  },
  "receipt": {
    "idempotency_key": "register-f2c01r02-before-cap",
    "replayed": false,
    "external_refs": [
      {"kind": "thread_id", "value": "shell-thread-opaque"},
      {"kind": "desktop_id", "value": "shell-desktop-opaque"},
      {"kind": "pairing_id", "value": "shell-pairing-opaque"}
    ]
  }
}
```

**contact_bindings 非空 — 请求**

```json
{
  "schema_version": "syn.f2.shell-core-bridge.request.v1",
  "request_id": "probe:register-contact",
  "method": "organization.register_stable_member",
  "params": {
    "member_id": "member_f2c01r02_before_contact",
    "display_name_ref": "display-name:member_f2c01r02_before_contact",
    "identity_evidence": {
      "kind": "EXPLICIT_IDENTITY_CONTRACT",
      "contract_kind": "syn.m6.org.stable-member-identity/v1",
      "identity_contract_ref": "identity-contract:member_f2c01r02_before_contact",
      "source_record_ref": "identity-source:member_f2c01r02_before_contact",
      "source_revision": 1,
      "observed_at": 1700000000000,
      "explicit_human_command": true
    },
    "scope_assignments": [],
    "role_assignments": [],
    "capability_permission_refs": [],
    "memory_refs": [],
    "contact_bindings": [{
      "binding_ref": "contact-binding:member_f2c01r02_before_contact",
      "to_role_ref": "role:secretary",
      "to_recipient_ref": "actor:member_f2c01r02_before_contact/recipient",
      "source": "syn.fixture.explicit-contact-binding/v1",
      "revision": 1,
      "observed_at": 1700000000000
    }],
    "idempotency_key": "register-f2c01r02-before-contact"
  },
  "external_refs": [
    {"kind": "thread_id", "value": "shell-thread-opaque"},
    {"kind": "desktop_id", "value": "shell-desktop-opaque"},
    {"kind": "pairing_id", "value": "shell-pairing-opaque"}
  ],
  "deadline_unix_ms": 1787154590540
}
```

**contact_bindings 非空 — 响应（修复前：成功并持久化）**

```json
{
  "schema_version": "syn.f2.shell-core-bridge.response.v1",
  "request_id": "probe:register-contact",
  "method": "organization.register_stable_member",
  "ok": true,
  "code": "F2_OK",
  "result": {
    "result_kind": "stable_member_registration",
    "payload": {
      "disposition": "REGISTERED",
      "member": {
        "member_id": "member_f2c01r02_before_contact",
        "membership_lifecycle": "ESTABLISHED",
        "scope_assignments": [],
        "role_assignments": [],
        "capability_permission_refs": [],
        "availability_ref": null,
        "contact_binding_refs": ["contact-binding:member_f2c01r02_before_contact"],
        "contact_bindings": [{
          "binding_ref": "contact-binding:member_f2c01r02_before_contact",
          "to_role_ref": "role:secretary",
          "to_recipient_ref": "actor:member_f2c01r02_before_contact/recipient",
          "source": "syn.fixture.explicit-contact-binding/v1",
          "revision": 1,
          "observed_at": 1700000000000
        }],
        "memory_refs": [],
        "promoted_from": null,
        "display_name_ref": "display-name:member_f2c01r02_before_contact",
        "identity_contract_ref": "identity-contract:member_f2c01r02_before_contact",
        "identity_source_record_ref": "identity-source:member_f2c01r02_before_contact",
        "identity_source_revision": 1,
        "revision": 1,
        "created_at": 1787154587886,
        "deactivated_at": null
      },
      "quarantine": null,
      "replayed": false,
      "directory_is_authority": false
    }
  },
  "receipt": {
    "idempotency_key": "register-f2c01r02-before-contact",
    "replayed": false,
    "external_refs": [
      {"kind": "thread_id", "value": "shell-thread-opaque"},
      {"kind": "desktop_id", "value": "shell-desktop-opaque"},
      {"kind": "pairing_id", "value": "shell-pairing-opaque"}
    ]
  }
}
```

修复前 sqlite：`m6_stable_member_identities` 两行（`member_f2c01r02_before_cap`、`member_f2c01r02_before_contact`）；`m6_member_directory_command_receipts` 两行。进程 exit 0，stderr 空。

### 修复后真进程反例（同形状，新二进制）

二进制 `/tmp/f2c01r02-target-b33038e/debug/codex-governance-workbench` sha256 `93c50d7fc454190d2ac90ebc7116dae5385a3818a780ab876e2846c8e5efed25`。全新空根 `/tmp/f2c01r02-after-1787156542/local.codex.governance.workbench`。

**capability 非空 — 响应（修复后）**

```json
{
  "schema_version": "syn.f2.shell-core-bridge.response.v1",
  "request_id": "probe:register-cap",
  "method": "organization.register_stable_member",
  "ok": false,
  "code": "F2_FORBIDDEN_AUTHORITY_INPUT",
  "error": {
    "code": "F2_FORBIDDEN_AUTHORITY_INPUT",
    "message": "v1 capability_permission_refs must be empty"
  },
  "receipt": {
    "idempotency_key": null,
    "replayed": false,
    "external_refs": [
      {"kind": "thread_id", "value": "shell-thread-opaque"},
      {"kind": "desktop_id", "value": "shell-desktop-opaque"},
      {"kind": "pairing_id", "value": "shell-pairing-opaque"}
    ]
  }
}
```

拒绝后 `organization.sqlite` **不存在**。

**contact_bindings 非空 — 响应（修复后）**

```json
{
  "schema_version": "syn.f2.shell-core-bridge.response.v1",
  "request_id": "probe:register-contact",
  "method": "organization.register_stable_member",
  "ok": false,
  "code": "F2_FORBIDDEN_AUTHORITY_INPUT",
  "error": {
    "code": "F2_FORBIDDEN_AUTHORITY_INPUT",
    "message": "v1 contact_bindings must be empty"
  },
  "receipt": {
    "idempotency_key": null,
    "replayed": false,
    "external_refs": [
      {"kind": "thread_id", "value": "shell-thread-opaque"},
      {"kind": "desktop_id", "value": "shell-desktop-opaque"},
      {"kind": "pairing_id", "value": "shell-pairing-opaque"}
    ]
  }
}
```

再检 sqlite 仍不存在。随后同进程：`secretary_status` / `global_supervisor_status` 均为 `F2_OK`；合法空四数组 register `F2_OK` / `REGISTERED` / `replayed=false`；同键重放 `receipt.replayed=true`。落库后只有 `member_f2c01r02_after_legal` / `register-f2c01r02-after-legal`。进程 exit 0，stderr 空。完整成功路径 JSON 见 after-pairs 文件。

## 载体

精确路径 git 提交（无 push）。提交信息写明 enforce v1 empty-array constraint the contract already states; close the silent write-surface widening found by the F2 verdict，并带 Co-authored-by trailer。SHA 记在本轮用户回报中。工作副本与 `/tmp` 探针不是发布；offline `cargo check`/`cargo test` 与真进程 `__syn_bridge` 是本阻断项的运行证据，不构成真实日用或阶段关闭。
