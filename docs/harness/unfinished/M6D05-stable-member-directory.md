# M6D05 稳定成员目录（ORG-005，域层）

阶段：stage-15 M6 全局主管与内部组织（域层先行，UI 验收载体为新壳）

状态：`PLANNED` / `NOT_STARTED`。stage-15 检查点 CP3 的第一叶。前置：CP2 获总指导 PASS。

来源收据：stage-6 计划第 4 节 SYN-ORG-005、第 3 节 `StableMember` 与 `Availability` 不变量、第 7 节关键验收（更换伪服务提供方或 runtime 后身份 / 记忆引用 / 权限不漂移；stale availability 不参与能力判定）；判据以 M6D01 冻结合同为准。

目标：建立稳定成员目录——身份、生命周期、scope / role 分配、能力与权限的只读引用、availability、会话历史与直接联系入口。目录是后台能力与查找面，永远不是授权真源。

做完的标准：

1. 新增 `m6_org_member_directory.rs`（**注意不要用被保全占用的 `m6_member_directory.rs` 名字**），实现 stable `MemberId` 身份；身份不等于模型、服务提供方、线程或进程；
2. 严禁 heuristic 推断成员：不得从 session 数量、名称、provider thread 或 parent/child 关系推出成员身份；只有满足 M6D01 冻结的 explicit identity contract 的记录才进目录，其余进 quarantine（反例测试必须覆盖"看起来像但不满足合同"的记录被拒）；
3. membership lifecycle（建立 / 更新 / 停用）与 scope / role assignments 成立；停用保留历史 refs，不物理删除；
4. capability / permission 只存 `source + revision + observed_at` 的只读 ref / projection；**目录改动不改变任何授权判定**（定向测试证明：改目录后同一授权判定结果不变）；
5. availability 带 source / observed_at / TTL；陈旧即 unknown，且 unknown 不参与能力判定、不被当 permission（反例测试）；
6. 会话历史与直接联系入口：contact 只建立会话 / Handoff，不自动授予任何项目能力；
7. 目录可导出 / 重建：从既有 refs 重建后身份不变（定向测试）；
8. 替换伪 provider / runtime 后，stable member 的身份、记忆引用与权限不漂移（定向测试，用 fake provider）；
9. **真实生产消费者**：真实 Tauri command 在 `commands.rs` 注册、在 `lib.rs` 接入 `AppState`，普通启动路径可达；报告须给出完整调用链，禁止只有测试能触发；
10. `cargo check --lib --offline` 与本叶定向测试在 disposable checkout 上通过，记录真实数字与退出码，证据绑定候选 SHA；
11. 独立内容提交，写域精确，`git diff --check` 通过；本叶做完不停，直接进 M6D06（同属 CP3）。

证据：只在 disposable checkout 上产出定向证据，绑定候选 SHA。只用 fake roles / provider / runtime 与隔离 app-data。本叶不做 GUI、不接真实账号、真实个人资料或真实联系动作。

允许动：

- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_member_directory.rs`（新建）
- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_schema.rs`、`m6_org_store.rs`、`m6_org_dto.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_global_role_session.rs`、`m6_org_consult_handoff.rs`（仅本叶所需接线）
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（仅 `mod` 声明、`AppState` 接线与 command 注册）
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`（仅本叶 command 接线）
- `m3_role_session.rs`、`m3_role_session_repository.rs`：**仅**可见性调整与新增 trait 实现，不改既有语义；每处改动在报告里逐条说明
- `docs/contracts/`（仅新增增补合同）
- `tasks/2026-08-*`、`tasks/2026-08-19-*`
- `docs/harness/stages/stage-15.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/done/2026-08/`、`docs/harness/authorization.json`、`docs/harness/audit/2026-08.jsonl`、`docs/harness/reports/M6D05-*`、`docs/harness/plan.md`、`docs/current-state.md`

不许动：

- 把目录当授权真源；让 contact 扩权；让 availability 当 permission
- 从 session / provider thread / 名称推断成员身份；把既有 Agent Center session 自动迁成 StableMember
- 组织薪酬、HR、通用组织图、多租户权限系统（明确不在 M6）
- 直读项目 store / projection / project root；写项目事实
- M1–M5 冻结合同正文与旧 hash；M5 执行语义不放宽
- 临时 agent 历史与会诊（分属后续叶）
- 6 个未跟踪 `m6_*.rs`（含 `.bak`，其中 `m6_member_directory.rs` 与 `.bak` 尤其易被同名覆盖）与 `gen/schemas/linux-schema.json`：只读保全，不暂存、不清理、不恢复、不作实现输入，不得被同名新文件覆盖
- 前端源码、页面布局、旧壳 UI、`syn-shell` 仓库、F2/F3/F5、壳采纳
- stage-12、`unfinished/D0C04`、`unfinished/D0C05`、`OSS-01`、用户自有载体
- 真实凭据 / provider / 模型 / 账号 / 个人资料 / 真实联系动作 / 外部网络业务写
- 自行关闭 stage-15、宣布 M6 完成、跳过检查点
- 伪造 receipt、authorization、stage/leaf、测试或 App 证据
- push、merge、rebase、部署、发布
