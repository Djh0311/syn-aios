# M4R01 修正合同、生产调用图与红灯验收报告

日期：2026-08-11

阶段：`stage-07`

任务包：`M4R01`

## 1. 结论

M4R01 已把独立总线复核的五项 P1 分别固定到普通产品入口、owner、单写者、禁止旁路和验收层级。新增合同是独立增补，不改写 M1/M3/M4 冻结正文。

红灯探针在计划基线 `7f9c6da717f0ec49c22fcd76327431fcfff0cb4e` 对应源码上稳定得到：`source=RED`、`clock=RED`、`route=RED`、`conversation=RED`、`legacy=RED`，同时六份冻结合同 SHA-256 全部 exact（精确一致）。探针状态只表示显式 PRESENT / ABSENT marker（存在/不存在标记）是否匹配；它不证明 reachability（可达性）、真实调用边或产品行为。

## 2. 点名 owners 与普通入口

| 用途 | 真实 owner | 普通产品入口 | 精确 revision / watermark |
|---|---|---|---|
| 主来源、R06 parity | M2 workflow-state WorkItem | `update_work_item_state`；前置造数仍走 `initialize_workflow_state`、`bootstrap_project_workflow`、`create_task_draft` | `receipt.committed_revision == event.source_revision`；`current_snapshot.source_watermark == event.event_id` |
| typed Decision | project consultation proposal | `create_project_consultation_proposal`、`record_project_consultation_proposal_decision` | 同次 owner 输出的 `store_revision`；event id 为 `audit_event_id` |

WorkItem 已有同事务 receipt/event/audit/domain/snapshot；R02 必须加持久 consumer checkpoint 和普通 production dispatcher。proposal audit 尚缺同次 store revision，因此 R02 必须在 owner 原子持久化边界内补脱敏 outbox envelope。两者都禁止在 command wrapper 返回后 best-effort 直投 M4。

## 3. 五项调用图和旧断点

| P1 | 冻结普通产品调用图 | 本次复现的旧断点 |
|---|---|---|
| P1-A 来源 | owner ordinary command -> durable event/publication -> AppState dispatcher -> registered adapter -> M4 | 只有 owner UoW；没有 WorkItem M4 checkpoint、proposal typed Decision outbox 或 production dispatcher |
| P1-B 时钟 | ordinary scheduler -> startup/tick -> one captured server-now -> due batch -> atomic transition | scheduler 只跑 daily cycle；没有 due batch caller；Reminder server fire reason 未精确接入 |
| P1-C 回源 | sealed route ref -> server resolver -> registered owner -> finite target -> owner page exact focus | renderer 猜 Projects；无 resolver、registry client 或 owner focus consumer |
| P1-D 对话 | composer -> ordinary send command -> M3 RoleSession/Turn/transport -> provider transcript read | 只有 RoleSession status；无 load/send command、transport client或可用 composer |
| P1-E 旧读面 | five server-owned readers -> exact candidates -> canonical reread -> parity/quarantine/fallback | ordinary command 固定生成五个全空 inventory-only candidates |

详细单写者、失败反例、恢复规则、敏感边界与证据分层见 `docs/contracts/m4-independent-remediation-addendum-v1.md`。

## 4. 红灯探针

探针入口：

```text
node prototypes/productized-desktop-shell/scripts/run-m4-remediation-probes.mjs --expect=red
node prototypes/productized-desktop-shell/scripts/run-m4-remediation-probes.mjs --only=<source|clock|route|conversation|legacy> --expect=red
```

两类命令均以退出码 `0` 证明预期红灯被复现。脚本是 opt-in（显式运行）静态 marker 门，不进入默认失败套件，也没有 ignored test（忽略测试）。基线结构化 receipt 保存于 `docs/harness/reports/M4R01-red-baseline-receipt.json`，并固定探针脚本与增补合同自身 SHA-256。

R02–R06 必须先对各自 `--only` 重放 red，然后实现行为与组合测试，最后以 `--expect=green` 复核 marker。字符串 marker 变绿本身不算包完成。

## 5. 本包验证

已运行并通过：

- `node --check prototypes/productized-desktop-shell/scripts/run-m4-remediation-probes.mjs`
- all-probes `--expect=red`
- 五个逐项 `--only=... --expect=red`
- receipt JSON 解析，且保存文件与所记命令 stdout 逐字 `cmp` 一致
- 六份冻结合同 SHA-256 exact（由探针直接重算）
- 独立只读审查后补齐 owner native provenance / M4 seal 分层、Reminder 审计反例、关键生产接缝与旁路反向 marker
- `git diff --check`

## 6. 证据上限与遗留

本包没有修改产品代码，也没有运行 App、provider、connector、网络或外部写入。五项 P1 仍是红灯，分别交由 M4R02–M4R06 修正；M4R07 才承担普通隔离 App、强退/重启、可见 UI 和全量回归。M5–M10 保持 `NOT_ACTIVE`。
