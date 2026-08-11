# 阶段7 M4 独立修正与再验收

总计划：product-line 唯一基线与 Harness Lite 切换
目标：保留 M4 已进入主线的底座，补齐独立总线复核发现的五项普通产品 P1，并用普通产品 composition、隔离基础端口和可携带证据重新完成 M4 产品验收。

干完的标准：

- 五项 P1 各有普通产品生产调用图、正向证据和失败反例；fixture 不直调 adapter/repository 冒充产品闭环。
- 至少一个真实存在的内部 source owner 从普通产品 command/event 入口进入 M4；PersonalAction、Reminder、Notification 与 typed Decision 有正式产品组合入口。
- 服务端 scheduler 驱动 snoozed OpenLoop 与 Reminder 到期恢复，重复/并发 tick、CAS 冲突、强退和重启不漏不重。
- 注册 owner resolver 产生有限 typed 导航并由目标页消费 focus；unknown/stale/revision mismatch/missing 明确失败。
- Secretary 首页复用 M3 RoleSession/Turn/ConversationTransport 完成两轮以上对话、失败显示与跨重启历史恢复；空事件仍零模型调用。
- 五类旧读面由实际 server-owned reader 形成 PARITY/EMPTY/UNJOINABLE/QUARANTINED 和受守卫 fallback，至少一类产生真实 PARITY。
- 全新隔离 root 使用普通 AppState、command registry、source dispatcher、scheduler、route resolver、conversation transport 和 legacy readers 完成分层验收与新鲜全量回归。
- M1/M3/M4 冻结合同正文和历史 hash exact；stage-06、M4C01-M4C10 与旧报告/receipt 不改写。
- M4R07 后只关闭本阶段并等待总线独立复核，不自动激活 M5-M10。

允许动：

- docs/contracts/m4-independent-remediation-addendum-v1.md
- docs/current-state.md
- docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md
- docs/plans/2026-08-01-syn-stage-4-secretary-attention-and-daily-rhythm-plan-v1.md
- docs/plans/2026-08-11-syn-m4-independent-remediation-and-reacceptance-plan-v1.md
- docs/plans/README.md
- docs/task-queue.md
- docs/harness/
- handoffs/2026-08-10-syn-m4-to-m5-m6-m7-handoff-v1.md
- prototypes/productized-desktop-shell/src-tauri/src/
- prototypes/productized-desktop-shell/src/
- prototypes/productized-desktop-shell/tests/
- prototypes/productized-desktop-shell/scripts/
- prototypes/productized-desktop-shell/package.json
- prototypes/productized-desktop-shell/dist/
- prototypes/productized-desktop-shell/src-tauri/target/
- refs/heads/main
- /private/tmp/product-line-syn-m4r-
- /private/tmp/syn-m4r-acceptance-

只读：

- docs/contracts/identity-scope-v1.md
- docs/contracts/role-session-v1.md
- docs/contracts/m3-role-session-turn-handoff-resolution-v1.md
- docs/contracts/m4-secretary-attention-daily-resolution-v1.md
- docs/harness/done/2026-08/stage-06.md
- docs/harness/done/2026-08/M4C01-contract-and-current-fact-correction.md
- docs/harness/done/2026-08/M4C02-product-secretary-role-session-personal-scope.md
- docs/harness/done/2026-08/M4C03-persistent-inbox-attention-projection.md
- docs/harness/done/2026-08/M4C04-attention-lifecycle-todo-source-writeback.md
- docs/harness/done/2026-08/M4C05-secretary-application-service.md
- docs/harness/done/2026-08/M4C06-home-context-continuous-conversation.md
- docs/harness/done/2026-08/M4C07-daily-scheduler-zero-model.md
- docs/harness/done/2026-08/M4C08-legacy-read-compatibility-migration.md
- docs/harness/done/2026-08/M4C09-isolated-product-app-acceptance.md
- docs/harness/done/2026-08/M4C10-integration-regression-closeout.md
- /Users/yoyi/workspace/product-line-syn-fnd-002
- /Users/yoyi/workspace/product-line-syn-m2-closeout

不许动：

- M1/M3 冻结合同正文、M4 v1 冻结合同正文、stage-06、M4C01-M4C10 归档及原 C09/C10 报告和 receipt
- 两个保全工作树的 index、tracked/untracked 内容和分支头
- 真实资料、真实用户项目写入、真实模型/provider、真实消息、真实账号、凭据和 connector
- 网络外部写入、远端、push、merge、rebase、部署和发布
- reset、clean、stash、破坏性删除、覆盖既有工作和 leaf 写域外顺手修改
- M5-M10 产品实现

## 叶子

- [x] M4R01 修正合同、生产调用图与红灯验收
- [x] M4R02 普通产品来源与个人对象组合
- [x] M4R03 服务端到期时钟与恢复
- [x] M4R04 注册 owner 的精确回源
- [x] M4R05 持续 Secretary 对话
- [x] M4R06 五类旧读面的实际 shadow/parity/fallback
- [ ] M4R07 普通产品隔离验收、全量回归与收口
