# SYN-M3O01R01 AppState 权威槽位边界

日期: 2026-08-17
阶段: stage-14 仍开；唯一 current leaf 保持 M5R07
状态: 独立验收拒绝 `8b39d2b` 后的窄纠正包

被拒绝 candidate：`8b39d2b0f8a19b15085f369babf8da5eb29770f9`

`8b39d2b` 的窄合同缺陷：未安装权威只在测试里构造错误值，没有经过真实 `AppState` 槽位边界返回稳定码 `m3_project_role_session_authority_unavailable`。

## 授权边界

- 主工作树有用户 WIP：不 reset / stash / clean / `git add -A` / `git add .` / `commit -a`。
- 不新增 Tauri command、renderer 接线、原始 repository 外露。
- 不改 M5 / M6 源、lifecycle、authorization、壳文档、`linux-schema.json`。
- 不伪造 `ProjectId`。已安装端口对每一个 claim 继续 fail closed，直到存在合法权威源。
- 不声称 M3 已解阻、M5 已通过、stage 已关闭或 M6 已激活。

## 产品结果

普通产品 `AppState` 仍安装服务器-only 权威。验收 / 遗留组合保持未安装。未安装槽位必须经过服务器-only accessor / 槽位边界返回 `m3_project_role_session_authority_unavailable`。定向测试覆盖真实 `AppState` 边界，而不是手造错误值。

## 本包精确路径

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/m3_project_role_session_authority.rs`
- `tasks/2026-08-17-syn-m3o01r01-project-role-session-authority-slot-v1.md`
- `docs/contracts/m3-project-role-session-authority-slot-boundary-v1.md`
- `docs/harness/reports/M3O01R01-project-role-session-authority-slot.md`
- `docs/harness/unfinished/M3O01R01-project-role-session-authority-slot.md`

## 验证

- `cargo check --lib --offline`
- 定向 `cargo test --lib --offline -- m3_project_role_session_authority`
- `git diff --check`

不 push / merge / rebase / deploy / release。
