# 任务包:存储降级修复——boundary_reviews/audit_events DB 领先勘察+补桥 v1

日期:2026-07-16 · 档位:**轻档**(只读勘察+漏桥补接;fail-safe/判决体/安全闸零碰;重 seed 恢复=包外用户在场窗口) · 执行者:执行线 · 背景:观察期实际已断——真机降级 json_only 自 07-14 夜起,总指导 07-16 巡检才发现(巡检失职另记账)。

## 现象(真机审计原文·live store 只读可核)

1. `07-14 21:55:56 storage_mode_degraded_json_only`:`db_json_reconciliation_not_green:supervisor_boundary_reviews:db_leading`
2. `07-14 22:32:02` 与 `07-16 15:29:42` 再降:`db_json_reconciliation_not_green:workflow_audit_events:db_leading=[]:js…`(**reason 串在审计里被截断,先取全文**)
3. 现态:App 运行于 json_only,功能正常、数据无损;每次启动对账再降。

## A·勘察(先回传结论再动修——先核步)

1. `supervisor_boundary_reviews` 表 DB 领先根因:boundary review 的**全部写点清单**(global_supervisor_review store 的 boundary 写路径),07-14 21:55 前后哪一笔写造成 DB 有 JSON 无;对照 M5-C 接桥宣称的覆盖面,指出漏桥点或双写不同笔的位置。
2. `workflow_audit_events` 的完整不绿 reason(`db_leading=[]` 却不绿——是 js_leading 一侧还是判据对空列表的误判?)——把启动对账的完整输出捞出来贴原文。

## B·修(勘察结论核复后)

- 漏桥写点→照 M5-B/C 显式桥模式接(DB delta+审计同笔→JSON 投影);
- 若是对账**判据 bug**(如空列表误判不绿)→修判据,但判据属敏感面:案发测试先行、fail-safe 语义零碰、修法先回传再落。

## C·交付

案发测试(复现 DB 领先→修后启动对账绿)+ 全量基线只增不减(976/45 起)+ shape gate 三数(基线 13/5/5)+ fmt 仅历史三;不 commit;10 项回传(第 7 项三数必含)。重 seed 恢复 db_primary=包外,修好后总指导约用户在场窗口执行。
