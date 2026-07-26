# Kickoff：知识库生产写路构建阻断修复 v1

日期：2026-07-21  
状态：**已执行完成；本文仅保留原开工指令，当前结果见 `evidence/2026-07-21-knowledge-vault-production-write-path-build-blocker-verification-v1.md`，下一步见 `CURRENT.md`。**  
发起：总指导 → 执行线  
完整合同：`tasks/2026-07-21-knowledge-vault-production-write-path-build-blocker-package-v1.md`  
基线：`e9ad7f3`；执行线不 commit

## 先读

1. `AGENTS.md`
2. 完整任务包（逐字读完）
3. `src-tauri/src/knowledge_vault.rs:215-252`
4. `src-tauri/src/workflow_state_store.rs:69-145`
5. `src-tauri/src/workflow_db_primary_wiring.rs:1-75`

## 你要修的唯一问题

`knowledge_vault.rs:251` 调用了 test-only：

```rust
crate::workflow_state_store::atomic_write(...)
```

所以 `cargo test` 绿、non-test `cargo check/build` 报 E0425，真实 App 无法构建。

首选修法已经拍定：知识库审计归低频 Batch 2，改接：

```rust
crate::write_m5b_batch2_workflow_state(
    workflow_state_path,
    "knowledge_vault_audit",
    &value,
)?;
```

## 开工顺序

1. 先确认 `git status` 与 HEAD=`e9ad7f3`；若已有他人改动，保护它们并报告。
2. 只替换知识库生产 writer，立即跑 `cargo check --lib`，先证明 privacy/non-test 编译成立。
3. 把知识库测试的假 workflow-state 改成合法 `workflow_state_v0` fixture，锁事件与 revision。
4. 在 M5B 测试册追加一条窄 DB-primary Batch 2 audit 测试：DB/JSON 各一、revision +1、restart reconcile green。
5. 跑完整验收并写 evidence + 新 decision 纠偏旧“bridge 私有不可达/模式默认关”挂账。
6. 按任务包 10 项回传；不 stage、不 commit。

## 只准写四个文件

- `prototypes/productized-desktop-shell/src-tauri/src/knowledge_vault.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_storage_mode_m5b_tests.rs`
- `evidence/2026-07-21-knowledge-vault-production-write-path-build-blocker-verification-v1.md`
- `decisions/2026-07-21-knowledge-vault-audit-production-write-path-v1.md`

## 三条硬禁

1. **不许**删除 `workflow_state_store.rs` 的 `#[cfg(test)]`，不许新增 production raw atomic write。
2. **不许**改 M5 bridge/repository/storage-mode 生产文件、schema/CAS、安全闸或沙箱；若首选接法需要这些，停下报总指导。
3. **不许**启动真实 App、碰现场 workflow-state/DB/vault、继续 H2、顺手清 warnings 或改前端。

## 最低验收

- `cargo check --lib`：必须 exit 0。
- `cargo test --lib`：至少 1030/0/44，新增测试只增不减。
- M5-B/M5-C/M5-F1 定向：全绿。
- `cargo-tauri build --debug`：exit 0，只构建不启动。
- typecheck、离线交互、shape 13/5/5、`git diff --check`：不回归。
- 静态核：知识库零 `atomic_write` 引用；test helper 仍有 `#[cfg(test)]`。
- `git diff --name-only`：仅四个允许文件。

## 停止条件

privacy 不通、必须改生产桥、DB/JSON 分叉、revision 多增、reconcile 不绿、需碰真实数据，任一出现立即停，不自由发挥。

历史停止条件：修复核收前，S1B-H2 保持暂停。当前构建阻断已清，但 H2 live 仍需按 `CURRENT.md` 重冻结修后 binary 后另行续验。
