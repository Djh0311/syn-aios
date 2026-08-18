# M6D03 read-only cross-project advisory candidate and evidence v1

状态：`SUPERVISOR_SELF_REVIEW_PASS / CP2_SEGMENT_CONTINUES / DOMAIN_RUNTIME_ONLY / NOT_RELEASED`

## Harness

- stage：`stage-15`；leaf：`M6D03 只读跨项目 query 与 advisory（ORG-002，域层）`；执行来源 receipt：`u-bf5cbe4117b9c087c7f1`。
- 本叶先完成 M6P00 verdict 点名的 canonical workflow owner exact-join 前置：内容 `977770f115f6a416a9466c59728ab9ecfc04b669` / tree `97e026fc9bb6fba97859cb7d99b2dc5307c40136`。随后交付 advisory 内容候选 `60a8e198f7319c8d175754079d08c61ddb88614c` / tree `e4539f211f1c160906b4c05f41f75041a6e5134b`；其 parent 为 `70fc36e5340065210bcb392585a9712c03249ef3` / tree `29814acc800a404203b8f143b6a6fa3f1dfa78ed`。
- Grok 是优先实现者，但本叶派发因外部代码出站策略被阻断且没有产生任何 Grok 修改；同一长驻 Codex 按用户明确的保底授权接管，期间只有一个产品源码写者。两个内容提交均只按实际贡献署 Codex。
- `977770f` 只改 `c4_c6_workflow_governance_entrypoints.rs` 与 `project_workflow_automation.rs`，关闭 canonical process-fact 与 Phase A workflow/node/work-item 的 foreign-owner exact-join 缺口，并封死 path-derived `default_workflow_id` 兜底；没有改 observation / execution 合同的其他语义。
- `60a8e19` 精确修改 8 个允许路径：新增 `m6_org_cross_project_advisory.rs`、`m6_org_dto.rs`、`m6_org_schema.rs`、`m6_org_store.rs`；修改 `m6_org_global_role_session.rs`、`lib.rs`、`commands.rs`、`command_registry.rs`。`m5_project_summary.rs` 零改动，因此没有可见性调整或 trait 语义变更需要说明；`workflow_run_dispatch_entrypoints.rs` 也零改动。
- 本叶主管自复核通过后归档并直接进入同段 M6D04；CP2 尚未进行。本报告不代替覆盖 M6D03+M6D04 的 Cursor Opus 独立 verdict，不关闭 stage-15。

## 产品

- 跨项目核心只依赖 M5 `ProjectSummaryQueryPort`，一次要求至少两个 summary。普通 AppState 通过既有 M5 持久 ProjectSummary 端口供给输入；M6 核心没有 project root、项目 store、projection、sidecar 或 guarded-legacy 读取入口。
- 每份 summary 都精确校验 owner、schema/version、watermark、canonical hash、policy 与 handoff binding。fresh、missing、denied、degraded 和 stale 明确分流；缺 watermark、缺 owner、foreign owner、scope 越权、过期或旧版本均 fail-closed。幂等重放仍重新查询当前 summary，不能用旧 fresh advisory 代替新输入。
- 风险、依赖、冲突与优先级结论是确定性输出，每条结论携带 summary id、version、watermark 与 source link。普通成功路径把非空且裁剪过的 `summary_refs` / `source_refs` 写入既有 Global RoleSession context；不写 raw summary、项目原文、transcript、provider response 或 tool output。
- 用户采纳只在 M6 自有数据库创建完整 pending `DecisionRequest`，冻结 required source owner/object/revision、requesting/required actor、scope、schemas、command type、idempotency 与期限；不创建 workflow、grant、action 或项目事实。
- `AdvisoryApplicationProjection` 只在先只读验证真实 M5 authoritative receipt/grant 的 exact join 后追加 `applied / failed / rolled_back / unknown` observation。partial apply 与 compensation refs 保留历史，advisory lifecycle 不被投影改写；伪造 receipt 在任何 M6 写入前拒绝。
- summary version、watermark 或 hash 变化会把既有 issued advisory 标为 stale，同时保留历史；不会静默覆盖或把旧 advisory 当新鲜结果。
- Global RoleSession 的直接项目写尝试走真实 ordinary application guard，首个动作恒拒绝；项目 domain store、event/audit/outbox、sidecar/compatibility projection 与相关文件均无写入路径。
- 普通生产调用链：`index_host_app_entrypoints.rs` Tauri setup → `AppState::try_new_with_tauri_app_data_root` → M5 ProjectSummary port 与 M6 advisory store/RoleSession slot → `commands.rs` 四个 Tauri wrapper → `command_registry.rs` 的 `tauri::generate_handler!`。四个 command 分别为运行 advisory、采纳为 DecisionRequest、观察 authoritative receipt、恒拒绝 project write；均不是 `#[cfg(test)]`、env gate 或 fixture-only。
- 固定测试项目封条下的 `execute_project_workflow_node` guarded legacy 不进入 M6 query 输入面，也没有在本叶解封。

## 证据

原始证据根：`/home/synadmin/workspace/.syn-gates/evidence/M6D03-60a8e19/`。候选在 detached worktree `/tmp/syn-m6d03-verify.G1qUvV` 上运行；所有 summary、receipt 与项目对象均来自隔离 app-data 和 scratch projects 的合成 fixture。

- `cargo test --lib m6d03_ --offline`：exit 0；13 passed / 0 failed / 0 ignored / 2125 filtered。其中 8 个 M6D03 测试覆盖两项目回源、四态与拒绝反例、stale/幂等、DecisionRequest、append-only receipt projection、普通 AppState/真实 M5 port、零写 guard、command reachability 与 guarded-legacy 不可达；另 5 个 owner exact-join 测试同前缀随本命令通过。
- `cargo test --lib m6p00_ --offline`：exit 0；21 passed / 0 failed / 0 ignored / 2117 filtered，覆盖 owner exact-join 前置与 foreign-owner 零写回归。
- `cargo test --lib m6d02_ --offline`：exit 0；15 passed / 0 failed / 0 ignored / 2123 filtered；`cargo test --lib m5_project_summary --offline`：exit 0；3 passed / 0 failed / 0 ignored / 2135 filtered。
- `cargo check --lib --offline`：candidate exit 0；parent exit 0。两边 rustc 均汇总 897 warnings、文本 `warning:` 行均为 898；候选没有指向新 M6 文件的 warning，因此本叶 warning delta 为 0。
- `git diff --check 70fc36e 60a8e19`：exit 0。`frozen-contract-diff.log`、`m5-frozen-semantics-diff.log` 与 diff-check 日志均为 0 bytes；后者覆盖 `m5_project_summary.rs`、ExecutionGrant、runtime receipt 与 WorkerReport 语义载体。
- `git-name-status.log` 精确列出 `60a8e19` 的 8 个产品路径；`git-show-commit.log` 固定候选 SHA、tree、parent 与署名。detached 验证后唯一生成物是 untracked `gen/schemas/linux-schema.json`，其 SHA-256 与主工作树受保护载体相同。
- `protected-wip-sha256.log` 固定 6 个受保护 `m6_*.rs`（含 `.bak`）与 `linux-schema.json` 的 7 个 SHA-256；它们未暂存、提交、修改、清理、恢复或用作实现输入。

主管七项判据：

1. 写域：owner 前置提交只有 2 个获准路径；advisory 内容提交只有 8 个获准路径。无前端、manifest、用户载体、M1–M5 冻结合同或后续 M6 叶写面。
2. 冻结物：合同 diff 与 M5 执行/summary 语义 diff 为空；stale、foreign、watermark、hash、ExecutionGrant、WorkerReport、receipt/audit/quarantine 和 guarded-legacy 边界未放宽。
3. WIP 保全：7 个受保护未跟踪载体的 hash 与状态保持不变，未被暂存、提交、reset、stash、clean 或作为实现输入。
4. 独立重跑：13 个本叶测试、21 个 M6P00 回归、15 个 M6D02 回归、3 个 M5 ProjectSummary 回归、candidate/parent cargo check 与 diff check 均在 SHA 绑定的 disposable checkout 退出 0，完整原始日志留在证据根。
5. 实质：普通 AppState 安装真实 M5 port、M6 store 和 Global RoleSession consumer，四个 Tauri command 注册在普通 handler；advisory、DecisionRequest、receipt observation 与项目写拒绝均走运行时代码，不是测试专用空转。
6. 不越级：证据只证明 local/offline/synthetic scratch data 下的 M6D03 域层与普通 Tauri composition；没有声称 GUI、新壳、真实项目/provider/账号、外部写、部署、发布、CP2 或 M6 完成。
7. 欠账：本叶标准内没有未满足项。897/898 warning 是 candidate/parent 相同的既存 ENG-01 基线；Secretary consult Handoff 属 M6D04，普通启动失败耦合、renderer consumption 与 UI/隔离 App 分别仍由 M6D08、M6S01 承担。

## 载体

- 产品载体是候选 `60a8e19` 所含 M6D03 Rust 域层、普通 AppState/Tauri command 接线，以及其 tree 中已包含的 owner 前置 `977770f`；不是运行中的 GUI、新壳、真实 provider 集成或发布产物。
- 本报告、归档 leaf、stage/plan/current-state、audit 与 leaf 切换属于独立 Harness 记账；M6D04 完成前不会写 CP2 交包或启动验收官。
- 当前结论为 `M6D03 SUPERVISOR SELF-REVIEW PASS / CP2 SEGMENT CONTINUES / NOT RELEASED`。
