# Kickoff：S1B-H2-R4F `preflight_home` 受控私有 home fail-closed 闭环修复 v2

完整阅读并严格执行：

`tasks/2026-07-22-s1b-h2-r4f-preflight-home-repair-package-v1.md`

R4F 已实证：一条新首句 canonical recorded 后，在 runner 前止于安全 `preflight_home` 事实；第二句、R4E 工具线和 Pending 卡均未到达。该 family 不是 auth/config/permission 的单一已证根因。

本轮先在 App/Workbench/dev/Codex/MCP、registry 和全部 store holder 为空的关闭现场，对当前 workflow/run 派生的单一 controlled resident home 做一次流式、只读的固定元数据检查。只允许回传 base/active/config/metadata 的存在、类型和 owner-only；MCP config 的 `expected/exact_legacy/drift/malformed/unreadable`；run/workflow/generation match boolean；auth 的 `is_symlink/targets_default_auth/default_source_regular_file`。不得输出内容、路径、完整 identity、symlink target、原始错误或认证资料，不得读取 auth 正文。

必须以该现场分类和源码执行顺序唯一锁定最早 leaf，再在任务包列出的两个 Rust 文件内为同一 leaf 建红灯、完成最小修复并跑离线闸。不得只补分类后把真实 leaf 留给 R4G；无法复现时以 `BLOCKED_PREFLIGHT_HOME_STATE_NOT_REPRODUCIBLE` 停止，不猜测、不另出中间诊断包。

保留所有 owner-only、认证符号链接、精确 MCP 配置、generation/thread 身份和未知 drift 的 fail-closed 保护。只有产品自己生成、可无歧义恢复且 fixture 证明安全的状态才可修复。

不得启动或构建真实 App，不得操作真实 store、runner 或认证资料；除上述一次 fixed-output 元数据检查外，不得读取 controlled private home，任何情况下都不得写入、复制或导出它。不得改 H2 单工具预批准、MCP/transport、watchdog、invalid-resume、进程清理、M5、安全闸或固定测试项目。若现场已漂移或无法唯一锁定具体可修 leaf，按 `BLOCKED_PREFLIGHT_HOME_STATE_NOT_REPRODUCIBLE` 停止，不猜测、不重发、不现场修复。

代码与离线闸通过即停。真实 R4G 只做最终验收，必须另包、另授权、重新 Gate 0、新 binary 与新 message identity；不得再承担 home leaf 或工具线诊断。
