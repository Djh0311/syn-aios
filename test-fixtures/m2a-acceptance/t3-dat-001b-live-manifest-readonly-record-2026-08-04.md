# M2a · T3 DAT-001B 真实 Workbench Live Manifest 只读记录 · 2026-08-04

任务：`tasks/2026-08-04-syn-m2a-remaining-one-shot-package-v1.md`

证据等级：**LIVE-MANIFEST / READ-ONLY / HOLD**。

## 结论

**HOLD，不宣称 DAT-001B live manifest PASS。**

本记录确认了真实 CodexGovernanceWorkbench 数据面、DB-primary 配置和无值读法；同时按冻结合同的敏感字段名规则发现受限键形态，并确认 legacy `execution_attempts` 仍存在。两者都必须在普通存储、切换或迁移之前停下并人工重新分类。未读取或输出任何 JSON/SQLite 原始业务值、凭据、prompt、transcript，也未读取或写入 `/Users/yoyi/.codex`。

## 真实根与只读方法

| 项 | 真实绝对路径 | 方法 | 结果 |
| --- | --- | --- | --- |
| Workbench root | `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench` | `stat` | directory `0755` |
| JSON sidecar | `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json` | `stat` + SHA-256 + 无值 JSON key/count scan | 7,654,539 bytes; SHA-256 `4137836236f20fbc8390d7f1b73d2dcb19eb5e592c6de72ab633d4c70f52e89c` |
| storage mode | `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/runtime-artifacts/storage-mode.v1.json` | `stat` + SHA-256 + 仅 schema/path configuration fields | 606 bytes; SHA-256 `b35188a133852dc260f248c4af61e0cf186348698e5fb64742737691ef25c155`; `db_primary_json_projection` |
| DB primary | `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/production-db/workbench-state.v1.sqlite` | `sqlite3 -readonly`; `PRAGMA query_only=ON`; `integrity_check`; aggregate counts | 32,235,520 bytes; SHA-256 `1c333ccdd98d852ad858e6fc1fb3f04a87e5625b8c90556a6a4c0238b6f32357`; integrity `ok` |
| legacy lock path | `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.lock` | metadata existence only | absent; no lock file was created |

storage mode 的 `workflow_state_path`/`confirmed_workflow_state_path` 与 JSON 路径一致；`db_path`/`confirmed_db_path` 与 DB primary 路径一致。旧静态合同中的 `$HOME/.syn/**` 与 `$HOME/.syn/workbench.sqlite` 不属于本次 live evidence，不能作为真实路径使用。

## 无值 shape/count 结果

| Slice | JSON count | DB count | 结论 |
| --- | ---: | ---: | --- |
| projects | 5 | 5 | count match |
| workflows | 8 | 8 | count match |
| work_items | 58 | 58 | count match |
| workflow_audit_events | 1,819 | 1,819 | count match |
| execution_attempts | 164 | 164 | **restricted legacy runtime state; HOLD/no cutover** |

JSON 只执行了 top-level key、对象/数组数量和递归字段名扫描：4,007 objects、4,726 arrays。按 `token|secret|password|credential|api_key|auth` 冻结字段名规则，扫描命中 9 个受限键路径（归于 artifact/authorization/dispatch metadata 形态）。本记录不保存这些字段的值，也不由键名推断是否真的含有凭据；但该规则触发时必须走 `SCRUB_AND_STOP_BEFORE_ORDINARY_STORE`，因此不能升级为 PASS。

## 零 mutation 边界

- 未运行 migration、seed、SQLite 写入、VACUUM、复制、删除或修复命令；故不需要创建备份。
- SQLite 用 `-readonly` 打开并立即设置 `PRAGMA query_only=ON`；唯一查询为 `integrity_check` 和固定表计数。
- 所有本地项目输出仅为本 Markdown 证据记录；真实 Workbench 文件的路径、bytes、mtime 与 SHA-256 应在总验收末尾复核。

## 后续解除 HOLD 的最小条件

1. 提供并运行正式 DAT-001B 无值 manifest 入口：仅接受 Workbench root、拒绝 `.codex`/越界/符号链接，交叉校验 storage mode、JSON 与 DB；
2. 对受限键形态完成不保存原值的人工分类/审批，明确哪些可作为 hash/reference 保留；
3. 为 `execution_attempts` 建立精确 owner、migration disposition、rollback/export 边界，或由后续阶段接受其 `HOLD/no cutover`；
4. 在上述前置完成前，不对真实 Workbench 数据写入、迁移或宣称 DAT-001B PASS。
