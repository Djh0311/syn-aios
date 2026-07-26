# Kickoff：S1B-H2-R3 canonical → 主管注入/回复链只读诊断 v1

- 日期：2026-07-22
- 状态：等待用户转发后执行
- 权威任务包：`tasks/2026-07-22-s1b-h2-r3-canonical-to-supervisor-injection-diagnosis-package-v1.md`

## 可直接执行的 kickoff

执行 `S1B-H2-R3 canonical → 主管注入/回复链只读诊断 v1`。

先完整阅读并严格遵守：

`/Users/yoyi/workspace/product-line/tasks/2026-07-22-s1b-h2-r3-canonical-to-supervisor-injection-diagnosis-package-v1.md`

已知现场结论不是“消息没送到”：用户三次发送分别形成三条 canonical record，计数 `recorded/injected/replied = 8/3/3 → 11/3/3`，但三次均无 injected、无主管自然回复、无工具、无卡、无 chain。不要继续重发，也不要把 `+3` 再判成产品重复落账。

本轮只读取证，不启动 App，不发送任何消息，不构建，不写真实 store，不改代码。先确认现场仍关闭并重算冻结 hash；然后锁定三条 message_id，把每条 `recorded → turn prepared → process registration → thread.started/binding → runner exit → injected → reply` 与 resident session、canonical lifecycle audit、进程登记和私有 runner 产物逐一关联。

必须给出最早失败边界，并裁决为：A 单一代码根因、B 首次根因加后续 fail-closed、C 外部条件、或 D 现有证据不足。关键结论至少要有两类证据；证据不足就明确写 `NEEDS_SAFE_INTERNAL_DIAGNOSTIC`，不允许猜。

私有 stderr、用户原文、auth/token、CODEX_HOME 正文不得写进仓库；仓库只保留 hash、时间、identity、受控错误家族和脱敏证据。若发现 holder/残留，不自行 kill，按 `BLOCKED_LIVE_HOLDER` 停止；若相关脏项漂移，按 `BLOCKED_DIRTY_OVERLAP` 停止。

成功出口不是修码或 live 通过，而是：三条消息证据矩阵完整、根因裁决成立、形成新的诊断 evidence，并根据裁决另出一份可执行的最小修复/现场恢复/安全诊断任务包及 kickoff。更新 `CURRENT.md`，catch log 只在发现新拦截时 EOF 追加。全程不 stage、不 commit。

