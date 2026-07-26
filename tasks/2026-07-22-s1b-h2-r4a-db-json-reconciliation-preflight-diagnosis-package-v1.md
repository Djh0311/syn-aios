# 任务包：S1B-H2-R4A DB/JSON 全语义预检诊断 v1

- 日期：2026-07-22
- 状态：待用户另行授权
- 类型：只读生产状态与源码诊断；不含修复、重 seed、App 或对话验收
- 前置证据：`evidence/2026-07-22-s1b-h2-r4-real-app-diagnostic-and-pending-card-verification-v1.md`

## 0. 唯一目标

解释 R4-R2 中“计数级 DB/JSON 对平，但当前 binary 启动期 full reconciliation 在 project-proposals 投影不绿”的最早、可证实边界。输出 natural-key/hash 层的脱敏差异摘要与最小修复建议；不把方案当作授权修复。

## 1. 已知事实与未知项

- 已知：R4-R2 未发送 H2 两句；R/I/S/D、B/P/C、固定测试项目均无业务增量。启动仅产生正常 storage initialized 审计。
- 已知：private `reconcile_db_vs_json` 做 natural-key/hash 对账；现有 production surface 没有可直接调用、保证无写的等价命令。
- 未知：project-proposals 的差异是历史 import/reseed 遗留、JSON projection、DB row hash、normalization 还是观察到的启动顺序；计数无法裁决。
- 现场还有一个未由本包终止的裸 binary 残留。其存在时停止，不自行 kill。

## 2. 严格范围

允许：

- 只读确认 App/holder/registry 全空后，读取真实 JSON/SQLite 的安全元数据、计数、自然键 hash 摘要；
- 只读阅读相关 source 与历史 evidence；
- 必要时在不含用户正文、原始错误、auth/token、私有家内容的离线副本上运行既有 reconciliation 逻辑，前提是该副本操作得到新的精确授权。

禁止：

- 不启动 App、不 build、不发送消息、不生成 client/message identity；
- 不直接写/重 seed/迁移/恢复真实 store，不修改 storage mode、DB、JSON、.codex 或固定测试项目；
- 不改 H2 单工具预批准、approval/sandbox/read-only/path-lock/watchdog/进程组清理；
- 不新增 command、sidecar、MCP server、消息运输路；不 stage、commit、push；
- 不把 record_json、proposal 文本、用户正文、原始错误或私有路径正文写进 evidence。

## 3. Gate 0：现场卫生与冻结

1. 只读确认 Workbench/dev/Codex/MCP、registry、JSON/DB/WAL/SHM holder 均为空。残留 bare binary 或任何 holder 即 `BLOCKED_LIVE_HOLDER`，等待进程所有者。
2. 重新冻结 HEAD、staged、相关 dirty 项、R4 八源码 hash、storage-mode safe fields、JSON/DB/sidecar hash、SQLite integrity 与固定测试项目 manifest。
3. 普通 read-only SQLite 打不开时，不得写恢复；immutable 仅在 WAL/SHM 静止且 holder 为空时作为明确标注的只读替代。

## 4. 诊断要求

1. 先证明 count-level parity 与 full reconcile 结论的差异，不能用 `LIKE` 或用户文本猜测。
2. 使用 project proposal natural key 和 canonical hash 的安全摘要（数量、只读 hash/tail、表名、方向）定位最早 mismatch；不得输出 proposal title、goal、user snapshot 或完整 id。
3. 同时核对 DB `record_hash` 的长度/格式、JSON 序列化/normalization 规则、DB import metadata 与 startup 调用顺序。
4. 对每个判断至少给两类证据：源码调用/规范与只读现场或离线副本结果。证据不足则裁决 `NEEDS_SAFE_OFFLINE_RECONCILE_PROBE`，不得猜。
5. 单独记录 R4-R2 normal Quit 后残留是否已由用户正常清理；不要把它与数据根因混为同一结论。

## 5. 成功出口与后续

- 成功不是修复或 H2 live 通过，而是完整、脱敏的 mismatch 矩阵、最早边界和 A/B/C/D 根因裁决。
- 若需要代码或数据修复，另出最小修复包；若需 real-store apply/reseed，另出用户在场的恢复包；若对账全绿且进程卫生绿，仍需另包重新执行 R4 Gate 0/1，不能复用 R4-R2 binary 或基线。
- 最小回写 evidence、`CURRENT.md`；`docs/harness-catch-log.md` 仅发现新拦截时 EOF 追加；不 stage/commit。
