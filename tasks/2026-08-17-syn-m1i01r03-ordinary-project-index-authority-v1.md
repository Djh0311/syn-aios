# SYN-M1I01R03 普通 AppState 的 M1 登记/读权威

日期: 2026-08-17
阶段: stage-14 仍开；唯一 current leaf 保持 M5R07
状态: 独立验收发现普通登记仍是测试专用后的窄纠正包

总线判定：M1 已有持久 UUID registry 原语，但普通登记只存在于测试 `M1ProjectIndexRegistrar`。普通 `AppState` 只打开读句柄，从未建立时为 `None`。M3 / M5 因此仍被挡住。

## 授权边界

- 主工作树有用户 WIP：不 reset / stash / clean / `git add -A` / `git add .` / `commit -a`。
- 不改 M3 / M5 / M6 源、renderer、Tauri command、壳文档、stage-14、M5R07 leaf、authorization、stage lifecycle。
- 不改写冻结 M1–M4 正文 / hash / schema。
- 不给 M1 任何 ActorId、RoleSession、permission、scope、identity 或 M3 所有权。
- M3 还不消费本端口；M3 角色会话动作保持 fail closed。
- 不声称 M1 / M3 已解阻、M5 已通过、stage 已关闭或 M6 已激活。

## 产品结果

普通产品 `AppState` 安装服务器-only M1 权威边界：可显式登记并读取 canonical `M1ProjectId`。登记必须显式、服务器内部、带类型。不得从 legacy index / root / path / locator / cwd / scratch / UI / M5 helper / 启动自动登记。不得外露原始 registry。空 / 未安装 / 缺失走稳定码 `m1_project_index_unavailable`。已登记精确别名签发 `project:<uuid>`，原子持久化，普通 `AppState` 重建后解析同一 id。边界是 `Result`。

## 本包精确路径

- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `docs/contracts/m1-project-index-ordinary-authority-v1.md`
- `tasks/2026-08-17-syn-m1i01r03-ordinary-project-index-authority-v1.md`
- `docs/harness/reports/M1I01R03-ordinary-project-index-authority.md`
- `docs/harness/unfinished/M1I01R03-ordinary-project-index-authority.md`

仅当新 `AppState` 字段迫使测试字面量无法编译时，才最小改现有 fixture。

## 验证

- `git diff --check`
- `cargo test --lib --offline -- m1_project_index -- --test-threads=1`
- `cargo check --lib --offline`

不 push / merge / rebase / deploy / release。
