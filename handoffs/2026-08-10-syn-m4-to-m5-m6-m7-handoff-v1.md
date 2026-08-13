# Syn M4 收口至 M5 / M6 / M7 指引 v1

日期：2026-08-10<br>
状态：`GUIDANCE_ONLY / M4R07_V2_PRODUCT_CHAIN_PASS / STAGE_07_CLOSEOUT_PENDING / NOT_EXECUTION_AUTHORITY`

本文件保留 M4 已进入主线的 typed ref/event 交接素材和下游 HOLD 边界。2026-08-11 独立总线复核的五项 P1 已由 M4R01–M4R06 修正，M4R07 v2 后端/普通产品链为 12/12 PASS；但 `stage-07` closeout / lifecycle 尚未完成，本交接仍不是 M5–M7 的执行前置或授权。当前入口见 `docs/plans/2026-08-11-syn-m4-independent-remediation-and-reacceptance-plan-v1.md`。

本文件不激活 M5、M6、M7 或任何真实数据、模型、connector、远端与发布工作。

## 5 分钟接手顺序

1. 读当前用户指令、根/仓库 `AGENTS.md`、`docs/harness/plan.md` 和当前 M4R07 leaf，确认 `stage-07` 仍在 lifecycle closeout，不误写成已关闭。
2. 先读 `docs/current-state.md`、当前 M4 修正计划、M4R07 v2 portable receipt 与 v2 manifest；把 `docs/harness/reports/M4-independent-bus-review-2026-08-11.md` 当作修正前基线，把 M4C10 报告当作更早的机械与隔离证据。
3. 读 `docs/contracts/m4-secretary-attention-daily-resolution-v1.md` 的唯一 `m4-resolution-v1` JSON，机器消费必须拒绝缺失、重复或格式漂移。
4. 需要实现下游时，再读对应 M5/M6/M7 独立计划，并由新的用户指令、stage、唯一 leaf 和授权建立施工入口；本交接和 M4 历史授权都不授予实现权。

## 已可依赖的 M4 主线事实

- 普通产品 Secretary 使用后端解析的持久 RoleSession、PersonalScope、daily channel 与权限快照；renderer、固定项目 cwd 和路由不拥有身份。
- M4 单写 Inbox/OpenLoop/Decision projection、Notification、Reminder、显式 PersonalAction、DailyBrief/Report、scheduler、协调 receipt/event/audit/checkpoint；协调状态不反写 source owner。
- Source admission 是 source-first、版本化、可回源和 fail closed 的。不同 owner 不合并；unknown/sensitive/stale/unjoinable 输入 quarantine；OpenLoop 不自动克隆 Todo。
- 首页消费 typed read model；每项带 source、owner、reason、last change 和 status。M4R04 已接注册 owner 精确回源，M4R05 已接复用 M3 会话真源的持续对话与重启恢复；模型不可用时 deterministic brief/report 继续工作，空事件窗口 agent/model 均为 0。
- M4R02 已接普通产品来源与个人对象组合，M4R03 已接 snoozed OpenLoop / Reminder 服务端到期恢复；M4R06 已让五类 legacy read path 经实际 server-owned reader 形成 parity/quarantine 和受守卫 fallback，旧面没有协调写，也未物理删除。
- M4R07 v2 receipt 固定 12 次并实际完成 12 次。第 8 次普通 `recovery_timer` 真实等待 98 秒并完成后端恢复验证；UI / Computer Use / PNG / attestation 明确 `NOT_EXECUTED / NOT_APPLICABLE`。这既不是视觉失败，也不是视觉 PASS。

## 给 M5：ProjectSummary source 接口

M5 继续唯一拥有 `ProjectSummary` 与项目事实。M4 只在 M5 合同和 owner 可用后消费 typed source ref；当前缺失必须保持 `UNAVAILABLE / HOLD`，不能由 M4 扫项目文件或复制项目事实补齐。

M5 adapter 交给 M4 的 admission 至少要提供：

- `source_owner_ref`、`scope_ref`、`source_type`、`canonical_source_object_id`；
- 严格递增的 unsigned-64 `source_revision`、`source_event_id`、`source_owner_watermark`；
- `occurred_at_utc`、`source_status_code`、deterministic `attention_signals`、可空 `due_at_utc`；
- `source_link`（仅 `INTERNAL_ROUTE` / `HANDOFF_REF` / `OWNER_COMMAND_REF`）、`sensitivity`、`scrubbed_summary_ref`、`payload_hash`。

项目状态改变仍只能走 M5/source owner command。M4 只保存 scrubbed receipt ref；acknowledge、snooze、close 或 carry-over 都不等于项目事项完成。

## 给 M6：Global Supervisor consult 接口

M3 继续唯一拥有 Handoff。M4 已有 request、`UNAVAILABLE / PENDING / RETURNED / FAILED` receipt 处理和重启恢复；普通产品 adapter 目前固定返回 `M6_RECIPIENT_UNAVAILABLE`。

M6 将来只需实现成功 consult recipient/port，并通过 M3 Handoff 返回 scrubbed typed receipt。M6 意见不直接改 M4 coordination 或任何项目；M4 也不把 pending/returned receipt 当授权或 source-owner 完成。替换 unavailable adapter 时不得放宽 RoleSession、scope、permission、correlation、idempotency 与 source-owner 回写边界。

## 给 M7：Daily 事件接口

M4 已持久发出以下两个 source-backed event，公共 envelope 使用 `event-audit-outbox-v1.WorkbenchEventEnvelope`，sensitivity 固定为 `SCRUBBED_INTERNAL_REF_ONLY`：

- `DailyWindowClosed` / `syn.m4.daily-window-closed/v1`：`scope_ref`、`daily_window_id`、`iana_timezone`、`local_date`、`window_start_utc`、`window_end_utc`、`scope_source_watermark`、`projector_version`、`closed_at_utc`。
- `DailyReportVersioned` / `syn.m4.daily-report-versioned/v1`：`scope_ref`、`daily_window_id`、`daily_report_id`、`report_version`、`report_ref`、`supersedes_report_ref`、`scope_source_watermark`、`projector_version`、`generated_at_utc`。

M7 按合同 idempotency key 消费，并只创建 M7-owned artifact/annotation ref；join key 是 `daily_window_id + report_version`。event payload 不含 report body 或 memory candidate。M7 不原位修改 M4 report/attention/coordination row，M4 也不把日报或关注提升为 FormalMemory、PersonalFact、个人模型或 Skill。

## 回切、证据与停止线

- 回切只可选择受守卫 legacy read-only 展示，或关闭 M4 ingestion/scheduler/read projection；保留 M1/M3 guards、M4 coordination/event/audit/receipt/quarantine/report version，不重放 effect、不删除旧面。
- 当前产品链完成标记：M4R07 portable receipt 为 v2 PASS、12/12，v2 manifest 精确绑定 portable receipt SHA 与 `launch_8_ui_validation` canonical SHA；`stage-07` 尚未关闭。
- 未进入：真实个人资料、真实用户项目写入、真实模型/provider、真实 Codex 消息、真实账号/凭据/connector、网络外部写入、真实迁移、push/merge/rebase、部署、发布、M5–M10 产品实现。
- 接手时若需要扩大上述边界、source owner 不唯一、revision/watermark 无法精确绑定、协调动作会反写 owner、事件含正文/secret，或空事件会触发模型，停在事实并回到新任务包裁决。

当前停止点是 M4R07 v2 产品链 PASS，文档、独立复核与 `stage-07` Harness 生命周期仍待收口。M5–M10 未激活；真实资料/provider/connector/远端/发布未验，没有自动下一包。
