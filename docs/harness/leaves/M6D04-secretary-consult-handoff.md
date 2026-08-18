# M6D04 Secretary consult Handoff（ORG-004，域层）

阶段：stage-15 M6 全局主管与内部组织（域层先行，UI 验收载体为新壳）

状态：`CURRENT` / `NOT_STARTED`。stage-15 检查点 CP2 的第二叶；M6D03 已由主管自复核放行并归档。本叶做完即到 CP2，必须收口交包并由同一长驻 Codex 前台阻塞启动独立验收官。

来源收据：stage-6 计划第 4 节 SYN-ORG-004、第 3 节 `ConsultHandoff` 不变量（from/to、scope、refs、question、receipt；无项目写权限）；M3 Handoff 为唯一 owner；判据以 M6D01 冻结合同为准。

目标：让秘书能显式把跨项目问题咨询给全局主管，全过程留痕、可拒绝、可回执、可回源，且咨询意见不触发任何项目命令。

做完的标准：

1. 新增 `m6_org_consult_handoff.rs`，用 M3 既有 Handoff 机制实现 Secretary 发起 → Global Supervisor 接受 / 拒绝 → 返回 advisory → Secretary 展示 / 回源的完整状态机；**不得新建平行 handoff 结构**；
2. Handoff 携带 from / to / scope / refs / question / receipt 全字段，缺字段即拒；无项目写权限；
3. 重复 Handoff 幂等：同一 handoff 身份重复投递不产生第二份 advisory、不重复消耗成本、不产生第二条 receipt（定向测试）；
4. 拒绝是显式状态并带原因，不是静默失败或超时兜底；
5. 咨询意见不触发任何项目 command，也不改变任何项目事实（write-spy 或 hash baseline 证明）；
6. 回源：秘书侧能从 advisory 回到具体 summary source ref，deep-link metadata 完整；用户点击回源不等于把原文加入 global session；
7. **真实生产消费者**：真实 Tauri command 在 `commands.rs` 注册、在 `lib.rs` 接入 `AppState`，并与 M4 秘书侧既有入口接线；报告须给出从普通 entrypoint 起的完整调用链，禁止只有测试能触发；
8. 定向测试覆盖：发起 / 接受 / 拒绝、幂等、缺字段拒绝、零项目写、回源 refs 完整；
9. `cargo check --lib --offline` 与本叶定向测试在 disposable checkout 上通过，记录真实数字与退出码，证据绑定候选 SHA；
10. 独立内容提交，写域精确，`git diff --check` 通过；
11. 本叶做完即到 **CP2 检查点**：主管自复核放行并收口后，authorization 打回精确 closed，在 `/home/synadmin/workspace/.syn-gates/open/` 写 `stage-15-cp2-<YYYYMMDD-HHMM>.md` 交包（含 M6D03 与 M6D04 两叶），由同一长驻 Codex 前台阻塞启动零上下文 Cursor Opus 验收官并每两分钟心跳。PASS 才处理交包并进入 M6D05；FAIL 只按点名范围返修；无有效 verdict 或连续两次 FAIL 按协议 halt。不得在 PASS 前自行进入 M6D05。

证据：只在 disposable checkout 上产出定向证据，绑定候选 SHA。只用 fake roles / provider 与合成 summary。本叶不做 GUI、不接真实 provider / 消息 / 账号。

允许动：

- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_consult_handoff.rs`（新建）
- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_cross_project_advisory.rs`、`m6_org_global_role_session.rs`、`m6_org_schema.rs`、`m6_org_store.rs`、`m6_org_dto.rs`（仅本叶所需接线）
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（仅 `mod` 声明、`AppState` 接线与 command 注册）
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`（仅本叶 command 接线）
- `m3_handoff.rs`、`m4_secretary_service.rs`、`m4_secretary_domain.rs`：**仅**可见性调整、新增 trait 实现与本叶咨询入口接线，不改 M3 Handoff 语义、不改 M4 已接受的秘书语义；每处改动在报告里逐条说明
- `docs/contracts/`（仅新增增补合同）
- `tasks/2026-08-*`、`tasks/2026-08-19-*`
- `docs/harness/stages/stage-15.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/done/2026-08/`、`docs/harness/authorization.json`、`docs/harness/audit/2026-08.jsonl`、`docs/harness/reports/M6D04-*`、`docs/harness/plan.md`、`docs/current-state.md`

不许动：

- 新建平行 Handoff / RoleSession 结构；绕过 M3 owner
- 让咨询意见触发项目 command、写项目事实或自动批准 grant
- 直读项目 store / projection / project root
- M1–M4 已接受语义与 M1–M5 冻结合同正文、旧 hash；M5 执行语义不放宽
- 成员目录、临时 agent、会诊（分属后续叶）
- 6 个未跟踪 `m6_*.rs`（含 `.bak`）与 `gen/schemas/linux-schema.json`：只读保全，不暂存、不清理、不恢复、不作实现输入，不得被同名新文件覆盖
- 前端源码、页面布局、旧壳 UI、`syn-shell` 仓库、F2/F3/F5、壳采纳
- stage-12、`unfinished/D0C04`、`unfinished/D0C05`、`OSS-01`、用户自有载体
- 真实凭据 / provider / 模型 / 账号 / 个人资料 / 真实消息 / 外部网络业务写
- 自行关闭 stage-15、宣布 M6 完成、跳过 CP2、越过检查点继续下一叶
- 伪造 receipt、authorization、stage/leaf、测试或 App 证据
- push、merge、rebase、部署、发布
