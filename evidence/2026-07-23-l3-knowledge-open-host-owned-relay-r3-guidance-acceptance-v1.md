# L3 `knowledge_open` host-owned relay R3 指导验收 v1

- 日期：2026-07-23
- 对应任务包：`tasks/2026-07-23-l3-knowledge-open-host-owned-relay-and-real-app-acceptance-package-v1.md`
- 执行证据：`evidence/2026-07-23-l3-knowledge-open-host-owned-relay-offline-verification-v2.md`
- 结论：**R3 GUIDANCE ACCEPTED；允许按既有精确授权进入 R4 fresh Gate 0。**

## 1. 指导线独立复核

- `index_host_app_entrypoints.rs` 当前 SHA-256 为
  `5ae1f355bc5f0c3f07c24bfa91be7479728affd275dc5259d328b5bf68182a8e`；
  `lib.rs` 为
  `828667f3f8631d3d6a9932f3abb93b9101e79c94817ff9b3e24f7167a9abf3ff`。
- 当前 Git diff 与 §4.3 冻结结果一致：前者只新增授权的 2 个 rustfmt
  diff block，后者只新增授权的 6 个 block；relay 注册、start/shutdown
  与 `knowledge_open_relay: None` 保持。
- 指导线独立重跑：
  - `cargo test knowledge_open_relay --lib --quiet`：11 passed；
  - `cargo test knowledge_ --lib --quiet`：54 passed；
  - `cargo test shared_supervisor --lib --quiet`：13 passed，1 ignored；
  - `cargo check --lib`：通过，仍有 598 条既有 warnings；
  - `npm run typecheck`：通过；
  - `node scripts/run-offline-interaction-test.mjs`：15 项通过；
  - 任务包 13 个目标 Rust 文件的受限 `rustfmt --check`：通过；
  - `git diff --check`：通过；暂存区为空。

## 2. 验收边界

- 本结论只验收 R1-R3 离线实现、失败闭锁和格式门，不等于真实 App
  或 `knowledge_open` 实际聚焦已经通过。
- shape 仍为执行线实测的 `17 errors / 5 warnings / 5 info`，此前最后已知为
  `16/5/5`；没有开工前同源快照，不能宣称绝对零净增或全仓绿色。
- R4 仍须从 fresh Gate 0 开始。主管首句、Active binding、自然回复和
  `tools/list` 精确五工具面任一失败，立即停止，不进入十二项。
- 对话底座三句重验继续 HOLD；不得与知识 R4 共享真实 App、store、sidecar、
  SQLite、MCP、进程或 build lock。

## 3. R4 许可

沿用任务包第 5 节既有授权：只启动当前工作树 Syn 和本包需要的一次
主管 Codex CLI/MCP 会话，只访问 Syn 自管固定 vault，只创建/清理唯一前缀的
验收命名空间，并保存脱敏截图、日志和 manifest。

不授权 Obsidian、其他 vault、真实项目文件、登录/付费、辅助功能权限、
`submit_proposal` 调用、卡/chain/worker、任意 shell/filesystem 能力或任何
Git 写入。
