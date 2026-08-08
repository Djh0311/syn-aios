# M2a · T2 独立证据审计 · 2026-08-04

任务：`tasks/2026-08-04-syn-m2a-remaining-one-shot-package-v1.md`

审计对象：`tasks/2026-08-04-syn-m2a-t2-isolated-crash-recovery-package-v1.md` 所留的执行者记录与其当前仍可读取的隔离现场。

结论等级：**INDEPENDENT-ARTIFACT-AUDIT / HOLD**。本记录不把执行者的 `ISOLATED-RUNTIME` 叙述自动提升为独立验收，也不把本次只读重读当成新的隔离 App 重跑。

## 冻结输入与本次只读核验

- 工作树：`/Users/yoyi/workspace/product-line-syn-fnd-002`，`syn-fnd-002-dev`，HEAD `2a7229bde7f0b5bb6701f4a7aa21944973f1881f`。
- 执行者记录：`test-fixtures/m2a-acceptance/t2-isolated-crash-recovery-record-2026-08-04.md`。
- 当前仍存在的 R4 root：`/private/var/folders/nj/y6s1fvl936xgfwg20w08sk6r0000gn/T/syn-r4-acceptance-m2a-t2`。
- 只读重读（2026-08-04）：profile `856` bytes；JSON `23,737` bytes、SHA-256 `f14462aefeb1543c2c6aa58222abb43bf77d6dc243d4d9c6b8f1a48fad1ba173`；SQLite `909,312` bytes、SHA-256 `508eae7e573929d10389b219ee7b20c5849b796c39e93dfb5409e38d77843fdb`。
- `sqlite3 -readonly` 当前计数：`command_receipts=4`、`events=4`、`audit_records=4`、`current_snapshots=1`、`work_items=1`。
- 可重读日志确证：pre-commit gate 行在 `/tmp/m2a-t2-dev3.log`、`/tmp/m2a-t2-dev4.log`；post-commit gate 行在 `/tmp/m2a-t2-dev5.log`、`/tmp/m2a-t2-dev7.log`；SIGKILL 记录为 S2 PID `86254`（`14:03:32`）与 S3 PID `57143`（`15:19:33`）；`/tmp/m2a-t2-dev8.log` 与 `dev11.log` 分别保留 DB-leading / JSON-leading 的 fail-closed 诊断。

这些只证明现有文件仍在、且与记录的若干锚点一致。它们不能从一个已推进至 S6 的末态反推出每一个场景当时的 SQLite/JSON 前后状态。

## 六场景独立结论

| 场景 | 执行者记录的主张 | 本次独立可核实部分 | 独立结论 / 缺口 |
| --- | --- | --- | --- |
| S1 冷启动、SIGTERM 重启 | 合法命令写 DB 与 JSON，重启可读 | 当前 root、末态 DB 计数和日志文件仍存在 | **HOLD**：没有按 S1 保留并可独立关联的命令返回、重启 PID/退出记录、SQLite 与 JSON 前后查询/哈希。 |
| S2 commit 前 SIGKILL | 四表与业务状态零半提交 | pre-commit gate 行与 PID `86254` 的 SIGKILL 哨兵日志仍可读 | **HOLD**：没有可独立复算的 kill 前/后 SQLite 四表和 JSON 快照；当前 S6 末态不能证明 S2 的零增量。 |
| S3 commit 后 SIGKILL | DB 已提交、JSON 陈旧，重启 fail-closed 且数据不丢 | post-commit gate 行、PID `57143` 哨兵日志、`dev8.log` 的 fail-closed 诊断仍可读 | **HOLD**：未保留与该 PID 精确配对的 kill 后双侧查询、DB/JSON 哈希和重启读回；记录中的计数未能在当前末态逐项还原。 |
| S4 投影失败、重启 replay | 命令返回真实错误，DB-leading 后 replay 转绿 | 记录叙述该注入点；当前 source 的 focused tests 覆盖相关 fail-closed 语义 | **HOLD**：保留的 `/tmp/m2a-t2-dev*.log` 未提供可独立关联的 `acceptance_injected_failure:projection-fail` 命令错误、前后 DB/JSON 查询和 replay-green 原始输出。单元/源码检查不能代替隔离 App 证据。 |
| S5 重复命令 | 同 receipt_id，四表零新增 | 当前末态的四表计数可读取 | **HOLD**：没有两次实际命令返回及每次前后计数/receipt_id 的原始对照，不能从当前总数推导幂等差分。 |
| S6 JSON-leading | 启动 fail-closed、DB hash 不变、JSON 不回写，降级写仅落 JSON | `dev11.log` 仍显示 JSON-leading fail-closed；当前 DB SHA 与执行者记录的最终 SHA 一致 | **HOLD**：未保留输入 JSON、构造前/后 JSON canonical hash 和降级写后的精确 DB/JSON 查询；当前末态不足以独立判定所有四个断言。 |

## 额外边界

- 执行者记录所述门只应位于 debug/验收 profile；本次未启动 App，也没有用单元测试或日志片段声称普通路径的真实运行验收。
- 本次没有读取或写入 `/Users/yoyi/.codex`，没有修改 R4 root、SQLite、JSON、日志或项目文件。
- 因此 T2 的正确状态是“执行者交付记录保留，**独立验收 NOT_ACCEPTED / HOLD**”，而不是失败地否认执行者曾完成工作。

## 新鲜机械重跑的现有能力核定

本次也只读核过是否可在不接触真实 HOME 的前提下机械重跑。结论是：**当前仓库没有受支持的无 GUI S1-S6 runner，不能以手工 console/手工双侧 seed 冒充独立复验。**

- `scripts/harness-v2/run-r4-isolated-app-preflight.mjs` 能生成 canonical temp R4 root、启动 bundle 并等待外部 UI inspection；它不是 `update_work_item_state` 的 IPC/invoke driver。
- 被测命令仍是窗口内 Tauri command；当前没有受支持的 non-GUI invoke client。
- fresh R4 fixture 只带 workflow；它没有记录要求的 nodes/edges/work item。`bootstrap_project_workflow` 遇到已有 workflow 为 no-op；现有执行记录本身说明使用过停机手工 JSON + SQLite 双侧注入。可复用的 db_primary seed helper 只在 `#[cfg(test)]` 私有测试模块中。
- 现有 root 已带 `.r4-initialized` 并推进至 S6，且旧 launcher TTL/记录 TTL 不一致；它不是新的独立样本。

所以 blanket authorization 不足以让本包把人工操作拼成新的物理验收。需要先有一个窄、受支持的 M2 验收基础设施：non-GUI invoke driver、只适用于 fresh R4 的 db_primary mirror seed/前后取证 runner、以及 PID/gate/signal/restart artifact capture。该能力未在本包实现，避免把新的测试基础设施误报为已经发生的 App 验收。

## 最小解除条件

先以独立授权提供上述受支持验收基础设施；随后在新的、独立的合法 R4 root 中重跑 S1-S6，并为每场景保留：绝对 root、启动/重启 PID 与 SIGTERM/SIGKILL 时间线、原始命令返回、门 armed 行、只读 SQLite 四表/业务行查询、JSON canonical hash/状态、DB hash、及场景前后差分。该新证据必须由非执行者按同一清单复核；在此之前不得将 T2 标为 accepted。
