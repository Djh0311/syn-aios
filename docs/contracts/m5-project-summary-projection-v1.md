# M5 ProjectSummary 投影合同 v1

- 版本：v1（2026-08-16）
- 状态：**FROZEN（M5R06 冻结）**
- 关系：补充 M1 ProjectSummary export；**不改 M1–M4 正文与 hash**。

## 规则

- Summary 最小、source-backed、带 version + watermark、可确定性重建、只读、不可反写。
- 只经 `ProjectSummaryQueryPort`；每次按 consumer RoleSession、scope、policy 判权。
- 落后 watermark 返回 stale，不悄悄当新。
- 跨项目、无权限、过期 consumer 拒绝。
- source refs 可回权威对象 id，摘要不得复制原文、transcript 或 secret。
- 查询与重建不得写回项目 owner 状态。
