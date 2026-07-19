# Gate 0 基线冻结（2026-07-19 23:55–00:20 +0800 现场重取）

## 进程 / 端口 / 持有者

- Workbench App、tauri-capability-probe、Vite、vitest、cargo-tauri、相关 codex exec：`ps` 扫描为零。
- 既有无关进程（不持有本任务文件，不动）：PID 14847 `__probe_run.mjs`（6/30 孤儿 probe）、PID 98835 `node apps/api/...main.js` 监听 127.0.0.1:3000（7/15 起，与本任务无关）。禁止事项#4：不 kill 未点名进程。
- 端口 5173/1420/4202/5174/9229 无 LISTEN；仅 3000（上述无关服务）。
- `lsof` DB 三件（sqlite/wal/shm）：无持有者（exit=1）。
- `lsof` workflow-state.v0.json / project-proposals.v1.json / exec-process-registry.v1.json / storage-mode.v1.json：无持有者（exit=1）。
- exec-process-registry.v1.json：revision 1119，`entries=[]`（warnings 为历史 PID 87693 核验失败噪音，非本轮）。
- SHM mtime 2026-07-19 21:37:37（出包后有人开过 DB 连接；用户在场确认继续，当前无持有者，备份按现场实况归档）。

## Git

- HEAD = `97fca19bc8d3effd4959dec8cc4827e27cac31e6`（与包基线一致）。
- staged set 唯一项：`prototypes/design-mockups/ui-vision-mockup/index.html`（开工前既有 mockup rename，R 状态）。
- `git status --short` 其余为当前主线既有 dirty（M5/H2/S1C 相关源文件与文档、证据、任务包），无来源不明改动。

## 五个关键源码 SHA-256（与包 §二表逐一相等）

| 文件 | SHA-256 |
|---|---|
| src-tauri/src/workbench_sqlite_storage_mode_m5f1.rs | 5d248c34e6332666d4d4ae7405cbf1c12ba84e039285a61bac47c6960b18a092 |
| src-tauri/src/workflow_db_primary_wiring.rs | c61ab2b93fd32d1b6e4c9780e6055dbf3dca7e5dcacdba02a1b306ff04cfc70a |
| src-tauri/src/supervisor_resident_oneshot_session.rs | 86bae55ccc9cd9e1499eae9396b987ea9ef18a31c43f872ad97c0e5e79db2da3 |
| src-tauri/src/mcp/supervisor_orchestrator_submit_proposal.rs | 6130ee77e3b6ce4a3730fd049adc2b9bc18718ae49d2401af8d2c035d351962b |
| src/views/projects/jiaoban/useJiaobanConversationState.ts | 47ac7053f55403c55d0a467703937b865c01fe001413bec81dc9776e46558bd2 |

## 真实 JSON 状态

- workflow-state.v0.json：SHA-256 `7d1153f341716ccd7b915027a48b625b9aae96b965866c7daabaca1303798dea`，size 7620285，mtime 2026-07-19 14:13:02。
  - revision 274；audit_events 1771；workflows 8；work_items 58；nodes 66；edges 50；
    workflow_chain_runs **40**；execution_attempts 164；workflow_node_dispatches 404；
    artifacts 27；projects 5；reviews 11；permission_requests 1。
  - storage_mode_initialized 累计 **25**；storage_mode_degraded_json_only 累计 **11**（末次 1784441581893 ≈ 14:13 案发 fallback）。
  - supervisor_resident_user_message_recorded 7、_injected 3、supervisor_message_recorded 3。
- project-proposals.v1.json：SHA-256 `3d7d965e02fb12761d5f7e9d85218fd154050131edf77e92951f90540238f631`，size 614212，mtime 2026-07-18 20:39:15。
  - revision 131；总数 74；pending_user_confirmation 17；user_confirmed 56；rejected 1。
- supervisor-orchestrator.v1.json：SHA-256 `699043d7ed8b06b988d41fe358ca90658710457ea4985ab9c23dd60ea780c1ac`。
  - sessions[24]（最新 resident）：resident_project_id=project:users-yoyi-codex-workflow-mario-test，resident_generation=**6**，resident_thread_id=**019f7857-0630-7d50-910d-855fa3e0d87a**。

## DB / WAL / SHM

| 文件 | size | mtime | SHA-256 |
|---|---|---|---|
| workbench-state.v1.sqlite | 28549120 | 2026-07-17 15:54:25 | 5cbf8e2c60ec68560e9a60f498017efb7abbcd6ef3d2215f25d011924bfec69a |
| workbench-state.v1.sqlite-wal | 0 | 2026-07-17 15:54:27 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| workbench-state.v1.sqlite-shm | 32768 | 2026-07-19 21:37:37 | fd4c9fda9cd3f9ae7c962b0ddf37232294d55580e1aa165aa06129b8549389eb |

## storage-mode 配置

- runtime-artifacts/storage-mode.v1.json：SHA-256 `b35188a133852dc260f248c4af61e0cf186348698e5fb64742737691ef25c155`。
- mode=`db_primary_json_projection`；workflow_state_path 与 db_path 均指向既有真实路径，confirmed_* 一致。本轮不改。

## 测试项目 /Users/yoyi/codex-workflow-mario-test

- HEAD = `caa02ded684d9e1d92d00c367949fab6f83430d1`。
- git status：`M README.md`、`M index.html` + 12 个未跟踪 proof 文件（既有状态）。
- 16 个文件全量 SHA-256 已冻结（见本目录 gate0-mario-test-hashes.txt）。

## Gate 0 结论

进程/端口/lsof/registry 全空 ✓；HEAD 与五 hash 与包冻结一致、无 BLOCKED_DIRTY_OVERLAP ✓；基线全部现场重取 ✓。**Gate 0 绿，进入 Gate 1 构建。**
