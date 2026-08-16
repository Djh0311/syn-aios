# SYN-M5R07 最窄验收修正

日期: 2026-08-17
阶段: stage-14 / leaf M5R07
状态: AWAITING_INDEPENDENT_ACCEPTANCE

2026-08-17 返修：消费 M3 RoleSession 与服务器解析的 project id；前端不得选择/扩大 allowed command；正式 UI 走完执行/receipt/report/review/result/summary；摘要 ACL 服务端派生；source refs 可解析；隔离 receipt 由后端状态派生。

写域例外（仅此两文件，不得扩大）：
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib_read_model_boundary_tests.rs`

原因：`AppState` 增加 `m5_store_path` 后，既有测试字面量缺字段无法编译（E0063）。这两处只补 `m5_store_path: None`，不改 command 语义或读模型边界。不 closeout，不激活 M6。
