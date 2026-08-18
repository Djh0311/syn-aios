# M6D02 Global Supervisor RoleSession candidate and evidence v1

状态：`SUPERVISOR_SELF_REVIEW_PASS / CP1_PENDING / DOMAIN_RUNTIME_ONLY / NOT_RELEASED`

## Harness

- stage：`stage-15`；leaf：`M6D02 顶层 Global Supervisor 持久 RoleSession（ORG-003，域层）`；执行来源 receipt：`u-bf5cbe4117b9c087c7f1`。
- 内容候选：`651a8fb9329d2ff07b4befe14fb37a1811942766`；tree：`8be2ac175f0aeb4027441f53883d9e7f9d5f67aa`；parent：`b36ab625433fd178f47888eb5d8d5afe3bee433b` / tree `2c1df4e604ba3c6857e97c91c9649e926c8ea046`。
- Grok 写出初版实现与测试；Codex 独立审查后移除越界格式化噪声、补齐歧义/权限错配反例、把 quarantine 吞错改为显式失败，并把只读状态 DTO 机械绑定到真实 fail-closed guard。内容提交按事实同时署名。
- 内容提交实际写域为 8 路径：新增 `m6_org_global_role_session.rs` 与本叶任务包；修改 `lib.rs`、`commands.rs`、`command_registry.rs`；另 3 个既有测试装配文件只补新 `AppState` slot 的 `Default::default()`。必要相邻路径已分别由 `709cef2` 与 `b36ab62` 先行记入 current leaf 和 audit。
- `m3_role_session.rs`、`m3_role_session_repository.rs`、`m3_role_session_schema.rs` 零改动；既有 M1–M5 合同正文与旧 hash 零改动。没有新 M6 平行 RoleSession schema/store。
- 本叶收口后归档，`authorization.json` 回精确 closed，`docs/harness/leaves/` 为空并停在 CP1；这里是主管逐叶自复核，不代替覆盖 M6D01+M6D02 的 Cursor Opus 独立 verdict，不关闭 stage-15。

## 产品

- 普通产品启动从 M4 已安装组合中克隆同一个 `M3RoleSessionSqliteRepository`，以 server-fixed actor、role、`GLOBAL` scope、current object、read channel、permission refs 建立或恢复唯一 Global Supervisor RoleSession；slot 只持 repository handle、精确 binding 与 session id，不形成第二真相源。
- 身份材料不读取 cwd、项目 path、display name、provider/model/thread/process、renderer input、环境变量或 fixture。创建使用固定 session id 与 M3 幂等 receipt；重启扫描完整 server-authorized candidate set，不以目录顺序或最新时间选身份。
- 唯一 active/current-permission candidate 才可恢复。多个 live candidate 或 permission mismatch 先经 M3 正常 command ledger quarantine，再 fail-closed；quarantined、closed、created、suspended、缺失已建立记录、损坏仓库或精确 binding 不符均不返回 ready，不自动导入、猜测或重建。
- 默认能力是 global read-only。状态 DTO 的 `project_write_capability=false` 来自实际 `authorize_attempted_project_write` 拒绝路径；provider handle 明确不授权。上下文 DTO 只有 `summary_refs`、`source_refs` 两个容器，不含原始文件、transcript、secret、未裁剪 memory、provider response、prompt、stdout/stderr 或 tool output。
- Project Supervisor 与 Secretary 由 actor/role/scope/object/channel/owner fingerprint 的精确 binding 判别，名称不参与授权。反例证明 Project Supervisor session 不会被选成 global session，Secretary/Project binding 不能通过 global validator。
- 普通调用链真实可达：`index_host_app_entrypoints.rs:770-793` 的 Tauri builder/setup → `AppState::try_new_with_tauri_app_data_root` → ordinary product ports → `lib.rs:428-431` 安装本叶 runtime → `commands.rs:4801-4810` 的零身份输入 status command → `command_registry.rs:59-84` 的 `tauri::generate_handler!`。isolated-uninstalled 与 historical legacy 组合都保持 unavailable。
- 本叶没有实现跨项目 query、advisory、Secretary consult Handoff、前端页面或 Global Supervisor 模型/消息；这些分别属于后续 M6D03/M6D04、新壳载体或明确未进入范围。

## 证据

原始证据根：`/home/synadmin/workspace/.syn-gates/evidence/M6D02-651a8fb/`。候选在 detached worktree `/tmp/syn-m6d02-verify.gM4ILq` 上运行，默认 target 目录位于该 disposable checkout 内；parent warning 基线在同一 disposable worktree 临时切换后运行并切回候选。

- `cargo test --lib m6d02_ --offline`：exit 0；15 passed / 0 failed / 0 ignored / 2110 filtered。覆盖 M3 持久往返、drop/reopen 同一身份、非 path 派生、Project/Secretary 隔离、Project session 不误选、只读零 mutation、歧义 live candidates quarantine、permission mismatch quarantine、最小 context、缺失/损坏 fail-closed、普通/isolated/legacy AppState、真实 command 与 registry。
- 直接相邻回归 `cargo test --lib m4c02_ --offline`：exit 0；14 passed / 0 failed / 0 ignored / 2111 filtered，证明共享 M3 repository 上既有 Secretary bootstrap/restart/ambiguity/quarantine 语义未被本叶破坏。
- `cargo check --lib --offline`：candidate exit 0；parent exit 0。两边均为 898 个既有 warning；候选日志中没有 `m6_org_global_role_session.rs` warning，因此本叶 warning delta 为 0。
- `git diff --check b36ab62 651a8fb`：exit 0。fresh detached checkout 验证前 `git status --porcelain=v1 -uall` 为 0 bytes；验证后唯一生成物是 untracked `gen/schemas/linux-schema.json`，SHA-256 `7e51a7ed92547e6c96f8d37d0ff7de836e9ee5b6102b1c6ba06ae075207c2a15`，与主工作树受保护载体及 M5R08 记录相同，未进入候选。
- `git-name-status.log` 精确列出 8 个候选路径；`m3-core-diff.log` 与 `frozen-contract-diff.log` 均为 0 bytes；`git-show-commit.log` 固定候选 SHA、作者与 Codex/Grok trailers。

主管七项判据：

1. 写域：`git show --stat 651a8fb` 只有 8 个允许路径；3 个必要相邻测试文件各自只有新 slot 默认值，registry 只有一个新 handler entry，无前端、manifest、M3 核心或其他产品面。
2. 冻结物：既有 M1–M5 合同与 M3 aggregate/repository/schema 零 diff；ExecutionGrant、WorkerReport、receipt/audit/quarantine 与 guarded legacy 边界未改判。
3. WIP 保全：主工作树 6 个受保护 `m6_*.rs`（含 `.bak`）与 `linux-schema.json` 仍为未跟踪，7 个 SHA-256 逐项等于 M5R08；未暂存、提交、reset、stash、clean 或作为实现输入。
4. 独立重跑：本叶要求的 15 个定向测试、14 个直接相邻 M4C02 回归、candidate/parent cargo check 与 diff check 全部在 SHA 绑定的 disposable checkout 运行，退出码和完整日志均在证据根。
5. 实质：实现安装在普通 Tauri setup 的真实 AppState 调用链并注册真实 command，不是 `#[cfg(test)]`、env gate 或 fixture-only；M3 receipt/session 持久化、完整候选解析、精确 binding、quarantine 与缺失/损坏读回路径均是运行时代码且 fail-closed。
6. 不越级：证据只证明 local/offline/synthetic scratch data 下的 M6D02 域层与普通 Tauri composition；没有声称 GUI/新壳、真实资料/provider/账号、外部写、部署、发布、CP1 或 M6 完成。
7. 欠账：本叶标准内没有未满足项。898 个 warning 是 candidate/parent 相同的既存 ENG-01 基线；M6D03 必须按 M6P00 独立 verdict 先修 canonical workflow owner-binding prerequisite，再接只读跨项目 query/advisory；M6D04、M6S01 与后续叶继续承担 Handoff、UI/隔离 App 等既定范围。

## 载体

- 产品载体是候选 `651a8fb` 的 M6D02 Rust 域层、普通 AppState/Tauri command 接线与离线测试；不是运行中的 GUI、新壳、真实 provider 集成或发布产物。
- 本报告、归档 leaf、stage/plan/current-state、audit 与 authorization closed 属独立 Harness 记账；CP1 PASS 前不会拉入 M6D03 或重签下一叶 authorization。
- 当前结论为 `M6D02 SUPERVISOR SELF-REVIEW PASS / CP1 PENDING / NOT_RELEASED`。
