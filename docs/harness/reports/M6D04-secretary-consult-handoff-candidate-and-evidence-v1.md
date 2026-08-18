# M6D04 Secretary consult Handoff candidate and evidence v1

状态：`SUPERVISOR_SELF_REVIEW_PASS / CP2_PENDING / DOMAIN_RUNTIME_ONLY / NOT_RELEASED`

## Harness

- stage：`stage-15`；leaf：`M6D04 Secretary consult Handoff（ORG-004，域层）`；执行来源 receipt：`u-bf5cbe4117b9c087c7f1`。
- 内容候选为 `ec1ba997af6c8b2418c5f1b7051f1015a5307996` / tree `685e458b3670fbc99dd57aa7f55c624d1307f271`；parent 为 `e7fd08f32478b4053d22e76e045232cd06caf93b` / tree `ac6b9851bbfc99b241b3a30315981e1bd225e3e3`。
- Grok 是优先实现者，但本叶产品派发因外部代码出站策略被系统拒绝，未产生 Grok 修改。同一长驻 Codex 按用户明确的无人值守保底授权接管；全程只有一个产品源码写者，内容提交按实际贡献只署 Codex。
- 候选精确包含 11 个允许产品路径：新增 `m6_org_consult_handoff.rs`；修改 `m6_org_dto.rs`、`m6_org_schema.rs`、`m6_org_store.rs`、`m6_org_cross_project_advisory.rs`、`m6_org_global_role_session.rs`、`m4_secretary_domain.rs`、`secretary_agent.rs`、`commands.rs`、`command_registry.rs`、`lib.rs`。
- 本叶已按七项判据自复核放行并归档。authorization 在 CP2 前回到精确 closed，`docs/harness/leaves/` 留空；M6D03+M6D04 仍须取得独立 Cursor Opus CP2 PASS，PASS 前不进入 M6D05。

## 产品

- `M6OrgConsultHandoffAdapter` 只把 M6 咨询特化接到 M3 既有 Handoff repository。M3 继续唯一拥有状态、revision、transition receipt、CAS、期限与幂等；M6 表只保存重放所需请求、advisory refs 和拒绝原因，不形成第二套 Handoff 真值。
- start 输入显式携带 question ref、source refs、至少两个版本化 ProjectSummary query、accept/return UTC 毫秒期限和 idempotency key；缺失、重复、越界、非规范期限或敏感/路径式 ref 在 store 访问前拒绝。
- 输出 `ConsultHandoff` 精确使用 M6D01 冻结字段 `handoff_revision`、`status_ref`、`from_role_session_id` 等，`consult_kind=SECRETARY_TO_GLOBAL_SUPERVISOR`，并带 to、scope、question、object refs、当前 M3 receipt；`project_write_capability=false` 恒定。
- source authority 来自普通 M4 Primary Secretary 的已验证 M3 RoleSession/binding。Global Supervisor 决策端是同一 M3 repository 中固定 global actor/role 的个人 scope Handoff endpoint；其本地 endpoint binding 走 M3 已有 create/effect claim/receipt/readback/bind 生命周期，不调用外部 provider、模型、消息或 connector，也不授予项目写能力。
- accept 先取得精确 M3 `ACCEPTED` receipt，再把该 receipt/revision 作为 M6D03 advisory 的 join；advisory 返回时依次走 M3 `RETURN_PENDING` 与 `RETURNED`。拒绝走 M3 `REJECTED`，并在 M6 投影保存封闭原因枚举，不用静默失败或超时替代。
- 同一 start/accept 重放复用同一 M3 Handoff、transition receipts、M6 projection 和 advisory；测试确认 M3 handoff/command-receipt/transition-receipt 行数及 M6 binding/advisory 行数不增加，不重复产生 advisory 或成本。
- Secretary 展示/回执读取经既有 `M4SecretaryApplicationService` 与 `M4SecretaryHandoffPort`；returned advisory 保留每个 summary 的 object ref、scrubbed summary ref 与 deep-link metadata ref。读取/回源不把原文加入 Global RoleSession。
- 项目零写：咨询模块没有项目 command、project store、projection、outbox 或 writeback port。接受/返回测试固定 product index、tasks 与 workflow-state 文件 hash，前后相同；响应计数为 `project_command_attempts=0`、`provider_invocations=0`。
- 普通生产调用链：Tauri `generate_handler!` → `commands.rs` 三个真实 command → `secretary_agent.rs` 的普通 Secretary entry / `M4SecretaryApplicationService` → `M6OrgConsultHandoffAdapter` → M3 Handoff + M6D03 advisory + M6 store。三个 command 分别为 start、Global Supervisor accept/reject、Secretary receipt/read；均非 `#[cfg(test)]` 或 fixture-only。
- 相邻路径说明：`m4_secretary_domain.rs` 只把既有 `permission_descriptor` 提升为 `pub(crate)`；`m6_org_cross_project_advisory.rs` 只暴露复用既有请求形状校验的 consult helper；`m6_org_global_role_session.rs` 只暴露同一 repository/binding/session 的内部 authority seed；`lib.rs` 只新增 module 声明；M3 Handoff 与 M4 service 语义正文零改动。

## 证据

原始证据根：`/home/synadmin/workspace/.syn-gates/evidence/M6D04-ec1ba99/`。全部命令在绑定候选 `ec1ba99` 的 detached disposable checkout 上执行，只使用 fake Secretary provider、隔离 app-data 与合成 ProjectSummary；每份 `script` 日志末尾记录 `COMMAND_EXIT_CODE="0"`。

- `cargo test --lib m6d04_ --offline -- --nocapture`：exit 0；4 passed / 0 failed / 0 ignored / 2138 filtered。覆盖 start→accept→return→read、冻结 DTO 形状、精确重放、显式拒绝、缺字段零写、项目文件 hash、deep-link refs 和真实 command/M4 调用链。
- `cargo test --lib m6d03_ --offline -- --nocapture`：exit 0；13 passed / 0 failed / 0 ignored / 2129 filtered。
- `cargo test --lib m6d02_ --offline -- --nocapture`：exit 0；15 passed / 0 failed / 0 ignored / 2127 filtered。
- `cargo test --lib m4c05_ --offline -- --nocapture`：exit 0；9 passed / 0 failed / 0 ignored / 2133 filtered。
- `cargo test --lib m3c05_ --offline -- --nocapture`：exit 0；43 passed / 0 failed / 0 ignored / 2099 filtered，确认既有 Handoff 状态、receipt、return validation、permission continuation 与 source-application 边界保持通过。
- `cargo check --lib --offline`：exit 0；rustc 汇总 888 个既有 warnings，日志 `warning:` 文本行 889；不声称 warning-free，也没有借本叶扩大清理。
- `git diff --check HEAD^ HEAD`：exit 0；`git-name-status.log` 精确列出上述 11 个产品路径；`frozen-contract-diff.log` 为空，M1–M5 与 M6D01 冻结合同正文零改动。
- detached checkout 验证后唯一 delta 是构建生成的 untracked `gen/schemas/linux-schema.json`；其 SHA-256 为 `7e51a7ed92547e6c96f8d37d0ff7de836e9ee5b6102b1c6ba06ae075207c2a15`，与主工作树受保护载体相同。临时 worktree 已用 `git worktree remove --force` 清理。
- `protected-wip-sha256.log` 固定 6 个受保护 `m6_*.rs`（含 `.bak`）与 `linux-schema.json` 的 7 个 SHA-256，逐项等于 M6D03 基线；它们未暂存、提交、修改、清理、恢复或用作实现输入。后来出现的无关 tracked WIP 也没有进入候选。

主管七项判据：

1. 写域：候选只有 11 个 current leaf 明示路径；三处必要相邻接线分别限于权限 descriptor 可见性、M6D03 shape validator 和 Global RoleSession authority seed，没有前端、manifest、用户载体、后续 M6 叶或禁止域写入。
2. 冻结物：冻结合同 diff 为空；M3 Handoff 状态/receipt/permission 语义与 M4 已接受 service 语义零改动，M3C05 43/43、M4C05 9/9 回归通过；ExecutionGrant、WorkerReport、receipt/audit/quarantine 与 guarded legacy 未放宽。
3. WIP 保全：7 个受保护载体 hash 与 M6D03 基线一致且未入候选；主工作树其他无关 dirty/untracked 文件均未暂存、清理或归责。
4. 独立重跑：M6D04 4、M6D03 13、M6D02 15、M4C05 9、M3C05 43 个测试和 cargo check/diff-check 均在 SHA 绑定的 disposable checkout 退出 0，原始日志留在证据根。
5. 实质：三个真实 Tauri command 已进入普通 handler；Secretary start/read 真实穿过 M4 service/port，Global Supervisor decision 真实穿过 M3 Handoff owner，accepted receipt 才能驱动 M6D03 advisory，不是测试专用空转。
6. 不越级：证据只证明 WSL local/offline/synthetic 的 M6D04 域层与普通 Tauri composition；没有证明 GUI、新壳、真实 provider/模型/消息/账号/项目写、部署、发布、CP2 或 M6 完成。
7. 欠账：本叶标准内没有未满足项。888/889 warnings 作为既有 ENG-01 债保留；renderer/new-shell consumption 仍归 M6S01，ordinary startup failure coupling 与阶段整合归 M6D08。CP2 PASS 前不进入 M6D05。

## 载体

- 产品载体是候选 `ec1ba99` 的 Rust 域层、M3/M4/M6 ordinary composition 与三个 Tauri command，不是正在运行的 GUI、新壳、真实 provider 集成或发布产物。
- 本报告、归档 leaf、stage/plan/current-state、audit 与 authorization close 属独立 Harness 记账；它们不改变候选 tree，也不代替 M6D03+M6D04 的 CP2 独立 verdict。
- 当前结论为 `M6D04 SUPERVISOR SELF-REVIEW PASS / CP2 PENDING / NOT RELEASED`。
