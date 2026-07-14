# 任务包:后端 UI 配套三件——系统状态/审计账本/follow-up 命令面 v1

日期:2026-07-15 · 档位:**轻档**(只加只读读模型命令+勘察;安全闸/解封面/存储语义零碰) · 执行者:执行线 · 背景:前端按设计定稿施工(前端总包并行),三处 UI 需要后端数据/命令面。

## A·系统状态读模型(首页「系统状态」区块+顶栏健康点)

新 `#[tauri::command]`(只读):返回 `{storage_mode: "db_primary"|"json_only", storage_healthy: bool, observation_day: u32(观察期第N天·自 storage-mode 配置或首条 initialized 审计推), last_degradation: Option<{at_ms, reason_human}>, recent_catches: Vec<{at_ms, summary}>(最近拦截·可先空实现留形状), gate_summary: Option<String>(如"mario 写解封·仅此项目")}`。数据源=storage_mode 健康缓存/审计流,**零新写点**。命令名与返回形状回传注明,前端好接。

## B·审计账本读模型(新「审计账本页」的数据)

新只读命令:分页+按类过滤的统一审计流(主 store audit_events+各 sidecar 审计聚合视图),`{page, page_size, kind_filter?} → {total, items: Vec<{at_ms, source(store名), event_type, human_summary(有则用现有人话字段,无则 event_type), target_ref, raw_json}> }`。只读聚合,不新增表/不改写路;db_primary 下从 DB 读、json_only 从 JSON 读(照 reconcile 现成读法复用)。

## C·follow-up 回话命令面勘察+补缺(卡住态乙型「直接回它一句」)

勘察:现有 worker follow-up 通道(supervisor orchestrator follow-up/manual relay 续话)有没有**前端可调的单命令**=「对指定卡住工单发一句用户指示并继续」。有→回传命令名+入参形状;没有→包一个薄命令(复用现有 follow-up/继续机器,**人闸语义不变**:该确认的照确认),不造新执行路径。

## 红线

全部只读或薄包装;安全闸/解封/S1/path-lock/复核实证闸/存储模式语义零碰;不 commit;测试 temp;回传 10 项模板(第 7 项 gate 三数,仓根)。

## 验收

A/B 命令 temp 端到端(造数据→命令→形状断言);C 勘察结论+缺口实现的案发测试;全量基线只增不减(964/45 起);gate 14/5/5(注意:前端并行施工可能动 tsx——你只对 Rust 面负责,tsx 面 diff 不是你的,分账写清)。
