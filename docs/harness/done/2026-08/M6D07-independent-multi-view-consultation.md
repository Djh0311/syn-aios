# M6D07 独立多视角会诊（ORG-006A，域层）

阶段：stage-15 M6 全局主管与内部组织（域层先行，UI 验收载体为新壳）

状态：`DONE` / `SUPERVISOR_SELF_REVIEW_PASS`。内容候选 `15bd053cccbaee1302d244afc84eb05578e7fa1c` / tree `1efd6cfd5cd02e2b3510acae9fb200f102b8b5a7` 已在 detached checkout 通过本叶 8/8、相邻回归 99/99 与 `cargo check --lib --offline`；原始证据在 `.syn-gates/evidence/M6D07-15bd053/`。本叶已归档并按同段规则继续 M6D08；阶段交包前不得宣布 M6 域层或 stage-15 完成。

来源收据：stage-6 计划第 4 节 SYN-ORG-006A、第 3 节 `MultiViewConsultation` 不变量、第 7 节关键验收（多视角输入在汇总前保持独立；runtime final answer 不自动生成决定）；判据以 M6D01 冻结合同为准。

目标：对同一重大问题让两个以上相互独立的咨询角色各自形成意见，并排给出共识、分歧与证据索引。结果只进用户待决定链，不产生任何项目命令或授权。

做完的标准：

1. 新增 `m6_org_multi_view_consultation.rs`，对同一问题向 ≥2 个相互独立的咨询角色派发**相同、最小、有来源**的问题包；
2. 独立性可被测试证明（不是文档声明）：各咨询方使用独立 RoleSession / Workcell / context packet，提交前互不读取对方结论；须有反例测试证明"串台"路径不存在或被拒；
3. 并排输出共识、分歧与证据索引，每条都能回源到具体 summary / ref；
4. 成本上限与超时是显式状态（超限 / 超时可判别，不静默截断、不用部分结果冒充完整会诊）；
5. runtime final answer 只形成咨询结果候选；会诊结果只形成意见与用户待决定项，**不直接生成项目命令、授权或正式事实**（write-spy 或 hash baseline 证明项目侧零变化）；
6. 普通问题仍走单角色：默认不升级为会诊，升级条件显式；
7. 只用 fake provider / runtime / 咨询角色，不接真实模型；
8. **真实生产消费者**：真实 Tauri command 在 `commands.rs` 注册、在 `lib.rs` 接入 `AppState`，普通启动路径可达；报告须给出完整调用链，禁止只有测试能触发；
9. `cargo check --lib --offline` 与本叶定向测试在 disposable checkout 上通过，记录真实数字与退出码，证据绑定候选 SHA；
10. 独立内容提交，写域精确，`git diff --check` 通过；本叶做完不停，直接进 M6D08。

证据：只在 disposable checkout 上产出定向证据，绑定候选 SHA。本叶不做 GUI、不接真实模型 / provider / 账号，不产生真实成本。

允许动：

- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_multi_view_consultation.rs`（新建）
- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_global_role_session.rs`、`m6_org_consult_handoff.rs`、`m6_org_cross_project_advisory.rs`、`m6_org_member_directory.rs`、`m6_org_schema.rs`、`m6_org_store.rs`、`m6_org_dto.rs`（仅本叶所需接线）
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（仅 `mod` 声明、`AppState` 接线与 command 注册）
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`（仅本叶 command 接线）
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`（只允许把 `start_global_supervisor_multi_view_consultation`、`submit_global_supervisor_consultation_view`、`assemble_global_supervisor_multi_view_consultation` 这 3 个精确 command 加入普通 `generate_handler!`；不得改其他 registry、gate 或 command）
- `m3_role_session.rs`、`m5_agent_runtime.rs`：**仅**可见性调整与新增 trait 实现，不改既有语义、不放宽 runtime admission；每处改动在报告里逐条说明
- `docs/contracts/`（仅新增增补合同）
- `tasks/2026-08-*`、`tasks/2026-08-19-*`
- `docs/harness/stages/stage-15.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/done/2026-08/`、`docs/harness/authorization.json`、`docs/harness/audit/2026-08.jsonl`、`docs/harness/reports/M6D07-*`、`docs/harness/plan.md`、`docs/current-state.md`

不许动：

- 让会诊结果自动变成决定、项目命令、授权或正式事实
- 用部分结果或单角色输出冒充多视角会诊；把"文档声明独立"当独立性证据
- 直读项目 store / projection / project root；写项目事实
- M1–M5 冻结合同正文与旧 hash；M5 执行语义与 runtime admission 不放宽
- 6 个未跟踪 `m6_*.rs`（含 `.bak`）与 `gen/schemas/linux-schema.json`：只读保全，不暂存、不清理、不恢复、不作实现输入，不得被同名新文件覆盖
- 前端源码、页面布局、旧壳 UI、`syn-shell` 仓库、F2/F3/F5、壳采纳
- stage-12、`unfinished/D0C04`、`unfinished/D0C05`、`OSS-01`、用户自有载体
- 真实模型 / provider / 凭据 / 账号 / 个人资料 / 真实成本 / 外部网络业务写
- 自行关闭 stage-15、宣布 M6 完成、跳过检查点
- 伪造 receipt、authorization、stage/leaf、测试或 App 证据
- push、merge、rebase、部署、发布
