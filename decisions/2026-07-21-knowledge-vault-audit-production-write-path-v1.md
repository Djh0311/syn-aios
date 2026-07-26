# 决策：知识库 audit 接入 Batch 2 生产写路 v1

日期：2026-07-21  
依据：`tasks/2026-07-21-knowledge-vault-production-write-path-build-blocker-package-v1.md`  
验证：`evidence/2026-07-21-knowledge-vault-production-write-path-build-blocker-verification-v1.md`

## 决定

知识库的 `knowledge_vault_note_created`、`knowledge_vault_note_user_edited`、
`knowledge_vault_note_ai_written` audit 均归为低频 Batch 2 写入。调用点固定使用：

```rust
crate::write_m5b_batch2_workflow_state(
    workflow_state_path,
    "knowledge_vault_audit",
    &value,
)?;
```

不暴露或复用 `workflow_state_store::atomic_write` 作为生产接口；该 helper 继续是
`#[cfg(test)]` test fixture。知识库不直接绕过 Batch 2 调用 validated JSON writer，
也不实现自己的 DB-primary fallback、retry 或 rebase。

## 纠偏 2026-07-20 决策

`decisions/2026-07-20-knowledge-vault-first-slice-v1.md` 的“挂账”中，下列判断已被
源码与本次 non-test/DB-primary 实证推翻：

- Batch bridge 对 `knowledge_vault` 子模块私有不可达；
- test-only atomic writer 可作为知识库生产写路；
- 因此知识库 audit 在 DB-primary 下不能进入投影。

crate-root Batch 2 helper 对子模块可见，且它已是明确 opt-in 的低频写路：DB-primary
先记录 repository delta 再投影 JSON；JSON-only 保持 validated writer；Blocked
JSON-only 保持既有 degradation audit 与 CAS 语义。本次新增测试锁定知识库 audit 的
DB/JSON 各一、revision `+1` 和 restart reconcile green。

## 未变更边界

07-20 决策中的 vault 自有目录、md 文件真相、路径锁、权限闸、AI source_summary
要求、受限渲染器与五个 Tauri command 均不变。此次只修复 workflow-state audit 的
生产写入路径与其测试 fixture。

## 后续

后续任何新增知识库 audit 仍使用固定 phase `knowledge_vault_audit`，避免无界 phase
词表。所有含 Rust production 路径的后续包应保留 `cargo check --lib` 或等价 non-test
build 闸，避免 `cargo test` 的 test cfg 再次掩盖生产构建错误。
