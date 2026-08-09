# Syn M3 收口至 M4 / M5 指引 v1

日期：2026-08-10<br>
状态：`GUIDANCE_ONLY / M3_COMPLETED / NOT_EXECUTION_AUTHORITY`

这是一份 5 分钟接手指引，不是 M4、M5、M6+ 的激活、实现或授权文件。M3C08 内容提交为 `fa8e392`，M3 状态为 `COMPLETED / MAINLINE / STAGE-05 CLOSED`；当前没有活动 stage 或 leaf。与本次状态回写同批的终态控制提交执行并归档 M3C08 `done` 与 stage-05 `close-stage`，不在此猜测该控制提交 hash。

## 5 分钟接手顺序

1. 读取当前用户指令、`AGENTS.md` 与 `docs/harness/plan.md`，先确认 Harness 没有活动 stage 或 leaf。
2. 读取 `docs/current-state.md`，确认 M3 为 `COMPLETED / MAINLINE / STAGE-05 CLOSED`，M3C08 内容提交为 `fa8e392`。
3. 读取 stage-05 / M3C08 的 done 归档以及 `docs/harness/reports/M3C08-mainline-integration-and-acceptance.md` 的退出矩阵、receipt SHA-256、未进入边界和 P0/P1/P2。
4. 如需理解已实现的 M3 边界，读取 `docs/contracts/m3-role-session-turn-handoff-resolution-v1.md` 与 M3C01–M3C07 archive leaf / M3C07 report；不要把 archive 直接当作新的执行授权。

## 已可依赖的 M3 事实

- RoleSession、Turn、ProviderHandle、可重建 ConversationContext、server-owned read model 和显式 Handoff 已由 M3C01–M3C06 进入主线。
- role、scope、对象、channel 与 permission 由服务器解析；existing thread 的 owner / scope / Station 3b guard 在 provider spawn 前生效。
- M3 repository / schema 只接收受限 shadow / provenance 输入；raw transcript、前端 cache 与不确定 thread 不成为真源。
- transport 是 start / continue / poll / stop / resume adapter；fake provider 覆盖 restart 与不重复 effect 的隔离语义。
- Agent Center 与 Jiaoban cache 已退为显示 fallback；server read model 才是恢复入口。
- M3C07 的桌面范围只覆盖 Agent / Jiaoban synthetic host、debug build、isolated profile 与 fake provider。截图在 Codex 主任务 `019fe53e-c4c2-7ab0-a965-0e231075df57` 线程内；仓库内只有 6 份 launcher JSON receipt。

## 迁移与回切红线

- 可回切的是 UI / read fallback 或新 projection；必须保留 provenance、receipt、export / manifest。
- 不得删除或放宽 M1 thread-owner、scope、Station 3b guard；不得重放 provider effect、恢复跨项目 bypass，或把 cache 提升为事实 owner。
- M3C07 isolated child 的 global invoke allowlist 会拒绝 legacy Agent / Jiaoban transport；这是 synthetic acceptance 的 fail-closed gate，不是普通模式的旧路退役。
- 真实 provider、真实 Codex 消息、真实用户项目 / 账号、凭据、connector、网络、部署、发布和真实数据迁移均未进入。

## M4 / M5 的进入条件

M4 与 M5 在本文件中均为 `PLANNED / NOT_ACTIVE`。M3 已完成 RoleSession / Turn / ProviderHandle / Handoff 持久恢复、spawn 前 fail-closed、Handoff 幂等回源、frontend cache 非 owner、最小上下文边界、隔离证据及 current-state 回写等退出条件；M3C08 内容提交为 `fa8e392`，但这不激活任何下游实现。

本交接不授予 M4/M5 实现权。任何 M4、M5 或 M6+ 实现都需要新的明确用户指令、匹配的新 Harness stage、唯一 leaf 和授权；不得由计划、提交、报告或本指引自动推导。

## 已完成状态与接手回报

- 主线回归通过：M1 四合同和 `29085cc` diff exact；`m3c07_` exit 0、11/11，`m3c0` exit 0、123/123；最终主机权限 `--lib` exit 0、1524 通过 / 0 失败 / 45 忽略、72.83s。启动器纠偏后主线程再次直接复跑 typecheck、offline interaction、launcher check 与 build，均 exit 0；offline 实际 39 entrypoint、摘要 15，build 306 modules、955ms，仅有既有 `>500k` chunk warning。受限 sandbox 的初次 1520 / 4 / 45 红灯、3 个 source-string collision、1 个 PID `lstart` EPERM、脚本消歧与 host exact rerun 均见 M3C08 验收报告。
- M3C08 内容提交为 `fa8e392`；终态控制提交执行并归档 M3C08 `done` 与 stage-05 `close-stage`，不猜测该控制提交 hash，也不声称 push、merge 或 release。这不授予 M4/M5 实现权。
- 若出现失败、范围漂移或缺失证据，只报告事实、影响、现有回切状态和所需新决定；不要自行扩大范围。
- M3 / stage-05 已收口，当前没有活动工程任务。接手人应明确报告读取入口、实际结果、改动位置、验证材料、未进入边界和下一步所需的用户授权。
