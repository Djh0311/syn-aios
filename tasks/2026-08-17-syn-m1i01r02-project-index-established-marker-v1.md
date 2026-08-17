# SYN-M1I01R02 project_index 已建立标记纠正

日期: 2026-08-17
阶段: stage-14 仍开；唯一 current leaf 保持 M5R07
状态: 独立复核阻塞 `253a03e` P1 后的窄纠正包

`253a03e` 把已建立判定绑在 `m1/` 目录。整目录删除后被当成从未建立并允许静默重建。本包在 `m1/` 外增加 project_index 自有 established marker。

## 授权边界

- 不 reset / stash / clean / `git add -A` / push / merge / rebase / deploy / release。
- 不改 M2–M6、stage lifecycle、authorization、壳文档、已有 WIP。
- 不声称 M3O01 已解阻。
- 不把登记/mint 装进 AppState、renderer 或 Tauri command。

## 验证

- `cargo check --lib --offline`
- `cargo test --lib --offline -- m1_project_index`
- `git diff --check`
