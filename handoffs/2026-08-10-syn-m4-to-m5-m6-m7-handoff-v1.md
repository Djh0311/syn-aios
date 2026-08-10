# Syn M4 收口至 M5 / M6 / M7 指引 v1

日期：2026-08-10<br>
状态：`GUIDANCE_ONLY / DOWNSTREAM_HOLD_PENDING_M4_REACCEPTANCE / NOT_EXECUTION_AUTHORITY`

本文件保留 M4 已进入主线的 typed ref/event 交接素材和下游 HOLD 边界，但“M4 已完成”的旧结论已被 2026-08-11 独立总线复核撤回。M4C01–M4C10 与 `stage-06` 的历史归档仍成立；下游在 M4 修正再验收前不得把本交接当作完成前置。当前入口见 `docs/plans/2026-08-11-syn-m4-independent-remediation-and-reacceptance-plan-v1.md`。

本文件不激活 M5、M6、M7 或任何真实数据、模型、connector、远端与发布工作。

## 5 分钟接手顺序

1. 读当前用户指令、根/仓库 `AGENTS.md`、`docs/harness/plan.md`，确认没有活动 stage 或 leaf。
2. 先读 `docs/current-state.md`、`docs/harness/reports/M4-independent-bus-review-2026-08-11.md` 和当前 M4 修正计划；再把 `docs/harness/reports/M4C10-mainline-integration-and-acceptance.md` 作为当时的机械与隔离证据读取，不沿用其整阶段完成结论。
3. 读 `docs/contracts/m4-secretary-attention-daily-resolution-v1.md` 的唯一 `m4-resolution-v1` JSON，机器消费必须拒绝缺失、重复或格式漂移。
4. 需要实现下游时，再读对应 M5/M6/M7 独立计划，并由新的用户指令、stage、唯一 leaf 和授权建立施工入口；本交接和 M4 历史授权都不授予实现权。

## 已可依赖的 M4 主线事实

- 普通产品 Secretary 使用后端解析的持久 RoleSession、PersonalScope、daily channel 与权限快照；renderer、固定项目 cwd 和路由不拥有身份。
- M4 单写 Inbox/OpenLoop/Decision projection、Notification、Reminder、显式 PersonalAction、DailyBrief/Report、scheduler、协调 receipt/event/audit/checkpoint；协调状态不反写 source owner。
- Source admission 是 source-first、版本化、可回源和 fail closed 的。不同 owner 不合并；unknown/sensitive/stale/unjoinable 输入 quarantine；OpenLoop 不自动克隆 Todo。
- 首页消费 typed read model；每项带 source、owner、reason、last change、status 和 opaque deep link。模型不可用时 deterministic brief/report 继续工作；空事件窗口 agent/model 均为 0。
- 五类 legacy read path 只保留 compatibility read-only。当前普通产品没有 legacy tuple adapter，因此 inventory 全部 quarantine；旧面没有协调写，也未物理删除。
- C09 只证明 synthetic fixture + fake model + isolated debug App 的首启/强退/重启，以及 task-local 结构化 receipt 所声明的可见交互；截图像素不可携带复核，也不证明真实日常使用或发布。

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
- 最终机械基线：M1/M3/M4 合同 exact；M4 Rust 98/98；完整主机权限 Rust 1639/0/45；typecheck、44-entrypoint offline、build、launcher syntax、定向 rustfmt 和 C09 三份 receipt 复核通过。
- 未进入：真实个人资料、真实用户项目写入、真实模型/provider、真实 Codex 消息、真实账号/凭据/connector、网络外部写入、真实迁移、push/merge/rebase、部署、发布、M5–M10 产品实现。
- 接手时若需要扩大上述边界、source owner 不唯一、revision/watermark 无法精确绑定、协调动作会反写 owner、事件含正文/secret，或空事件会触发模型，停在事实并回到新任务包裁决。

当前停止点是 M4 修正计划已建立、尚无活动 stage/leaf/授权；由新的 M4 修正开发主管在专门任务中只读接管并等待用户授权，没有自动下一包。
