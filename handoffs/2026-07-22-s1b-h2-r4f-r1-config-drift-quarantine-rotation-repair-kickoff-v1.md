# Kickoff：S1B-H2-R4F-R1 `config_drift` 隔离换代修复 v1

完整阅读并严格执行：

`tasks/2026-07-22-s1b-h2-r4f-r1-config-drift-quarantine-rotation-repair-package-v1.md`

已证最早 leaf 是 `config_drift`。不得继续寻找或猜测历史命令，也不得迁移、覆盖未知配置。只在目录/文件 owner-only、非 symlink，metadata 与当前 run/workflow/旧 generation 精确匹配，auth 链接精确有效，且动作当下重新分类仍为 `config_drift` 时，才可把旧 active home 整体原样隔离归档，generation `+1`，以当前精确配置重建 active home、重建事实并将当前消息作为 initial 仅执行一次。

换代必须由 typed leaf 驱动，不得匹配错误字符串。malformed/unreadable/missing、类型/权限异常、metadata 或 auth 异常全部继续 fail-closed；不得 rename、建 staging 或跑 runner。创建失败恢复旧 active，未知配置字节不得改变。

代码只准修改任务包列出的两个 Rust 文件。不得改 H2 单工具预批准、sandbox/read-only、MCP transport、watchdog、invalid-resume、进程清理、M5、安全闸其他分支或固定测试项目。

先红后绿，完成任务包列出的定向与聚合离线闸后立即停止。不得构建/启动真实 App，不得运行真实 Codex/MCP，不得读写真实 store/private home/auth，不得发送消息或点卡。

若必须扩大源码面或放宽任何安全前置条件，按 `BLOCKED_SCOPE_EXPANSION` 停止。离线收口后只回传十项；R4G 最终真实验收另包、另授权。
