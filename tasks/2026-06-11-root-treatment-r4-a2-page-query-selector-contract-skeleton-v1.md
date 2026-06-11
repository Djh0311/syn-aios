# Root Treatment / R4-A2 Page Query Selector Contract Skeleton v1

日期：2026-06-11

状态：待执行 / pending R4-A1 review gate。本文是 Root Treatment / Stage R 的 R4-A2 任务包草案；只有在 R4-A1 只读复核线返回 `CLEAR` 或 `CLEAR_WITH_P2` 后才能进入实现。如果 R4-A1 复核返回 `BLOCKED`，本任务保持待执行，主管线必须先修复 R4-A1 阻断项。

规划基线 commit：`03fd247`

## 0. 全局主管理解

已知事实：

- R4-A1 已实现 `WorkbenchSnapshot.page_read_model_inventory`，用于冻结页面数据需求矩阵和合同 skeleton。
- R4-A1 不新增 Tauri command，不切任何页面真实数据来源，不废弃 `WorkbenchSnapshot`。
- 当前 authority 入口指向下一步 R4-A2：后端按页查询 skeleton / selector contract，或先做 Projects / Agents 首批页面 selector 分域。
- R4 目标是读模型和前端瘦身，不是 UI 视觉重做。

当前未知：

- R4-A1 复核线最终 STATUS 尚未回收。
- R4-A2 是否先落后端 command skeleton，还是先落前端 selector 分域。主管线默认选择“后端 command skeleton + frontend wrapper/types + 小测试”，因为它能建立后续页面迁移的稳定边界。

核心判断：

```text
R4-A2 只建立 page query selector contract：输入 page_id，返回该页面合同、selector plan、migration guard 和 source boundary。它仍不返回完整页面业务数据，不迁移 UI 消费路径，不废弃 WorkbenchSnapshot。
```

## 1. Execution Mode

Execution Mode：Supervisor-led implementation after review gate。

Multi-Agent Policy：

- 复核线继续复用既有只读线程，只回收 STATUS 和 findings。
- 主管线负责实现、验证、证据、checkpoint 和再次交复核。
- 不新增更多开发线程，避免上下文维护成本超过 R4-A2 本身。

Review Gate：

- 允许进入实现：R4-A1 复核 STATUS 为 `CLEAR` 或 `CLEAR_WITH_P2`。
- 不允许进入实现：R4-A1 复核 STATUS 为 `BLOCKED` 或仍未返回。

## 2. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `tasks/2026-06-11-root-treatment-r4-a1-page-read-model-inventory-and-query-contract-v1.md`
- R4-A1 evidence / handoff。

## 3. 允许范围

允许实现：

- 后端新增 `PageReadModelQueryInput` / `PageReadModelQueryResult` 或等价小类型，优先放在 `page_read_model.rs` 小模块，避免增长 `types.rs`。
- 新增只读 Tauri command skeleton，例如 `query_workbench_page_read_model`。
- command 输入只接受 R4-A1 inventory 内的 page id。
- command 输出只包含：
  - requested page id / label；
  - matching R4-A1 contract；
  - selector plan；
  - current source boundary；
  - migration status；
  - warnings / blocked reasons。
- 前端新增 `queryWorkbenchPageReadModel` wrapper 和独立 TS 类型模块，避免继续撑大 `types.ts`。
- 新增一个小离线 / TS 测试或后端单元测试验证：
  - known page 可查询；
  - unknown page 被拒绝；
  - 输出仍标明 `workbench_snapshot_still_active`；
  - 输出不声称 page data migrated。

允许但不要求：

- Settings 开发者区可以增加只读说明“R4-A2 skeleton available”，但不得改普通用户页面布局。

## 4. 禁止范围

禁止实现：

- 不把 Projects / Agents / Home / Running / Memory / Knowledge / Skill / Harness 页面真实数据来源迁移到新 command。
- 不让 UI 页面开始依赖 R4-A2 command。
- 不废弃、删除或弱化 `load_workbench_snapshot`。
- 不新增 sidecar / DB migration / production read-cut / stop-write。
- 不改真实 Codex runner，不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`，不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 不启动 Tauri / Browser / Chrome / Vite / screenshot。
- 不改视觉风格，不重做布局，不实现 Xuanji/Mobbin/inkwash 视觉变化。

## 5. 文件落点

预期允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`
- `prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- 小测试文件，优先新增 `tests/r4-page-read-model-query-contract.test.tsx` 或后端 `page_read_model` 单元测试。
- 本任务包、evidence、handoff。
- checkpoint 时同步当前入口文档和正式计划。

尽量避免修改：

- `types.rs`
- `types.ts`
- `offline-permission-dialog.test.tsx`
- `ProjectsView.tsx`
- `AgentView.tsx`
- `styles.css`

如必须修改 ratchet 文件，必须先运行 shape gate 并说明原因。

## 6. 验收

必须通过：

- `node scripts/harness/workbench-shape-gate.js --mode check`
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib page_read_model`
- `cargo test --lib`
- `cargo fmt -- --check`
- `git diff --check`

扫描：

- 不得出现“R4 已完成 / 页面已按页查询完成 / WorkbenchSnapshot 已废弃 / UI 已重做”冒领。
- 不得新增真实 `codex exec` / `codex exec resume` 执行路径。
- 不得新增 `.codex` / secret / token / credential / full transcript 真实读取路径。

## 7. 完成后必须写

预期 evidence：

- `evidence/2026-06-11-root-treatment-r4-a2-page-query-selector-contract-skeleton-v1.md`

预期 handoff：

- `handoffs/2026-06-11-root-treatment-r4-a2-page-query-selector-contract-skeleton-v1-result.md`

handoff 必须包含：

- R4-A1 复核 STATUS。
- implementation commit。
- 新增 command / wrapper / type / test。
- 明确未迁移任何真实页面数据来源。
- 明确下一步建议：Projects / Agents 首批 selector 分域或 TS 类型分域。

## 8. 禁止声明

R4-A2 禁止声明：

- R4 完成。
- 所有页面已完成按页查询。
- 任一真实页面已迁移到新 command，除非另有后续任务包完成。
- `WorkbenchSnapshot` 已废弃。
- `ProjectsView` / `AgentView` 已拆分完成。
- UI 已重做或视觉已验收。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- 多 agent 并行真实执行已解锁。
